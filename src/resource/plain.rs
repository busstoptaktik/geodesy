use log::info;
/// Plain resource provider. Support for user defined operators
/// and macros using a text file library
use std::collections::BTreeMap;
use std::fs::File;
use uuid::Uuid;

use super::GysResource;
use crate::CoordinateTuple;
use crate::GeodesyError;
use crate::Provider;
use crate::{Operator, OperatorConstructor, OperatorCore};

use enum_iterator::IntoEnumIterator;
#[derive(Debug, IntoEnumIterator, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum SearchLevel {
    LocalPatches,
    Locals,
    GlobalPatches,
    Globals,
    Builtins,
}

pub struct PlainResourceProvider {
    searchlevel: SearchLevel,
    persistent_builtins: bool,
    pile: memmap::Mmap,
    user_defined_operators: BTreeMap<String, OperatorConstructor>,
    user_defined_macros: BTreeMap<String, String>,
    operations: BTreeMap<Uuid, Operator>,
    globals: Vec<(String, String)>,
}

impl Default for PlainResourceProvider {
    fn default() -> PlainResourceProvider {
        PlainResourceProvider::new(SearchLevel::LocalPatches, false)
    }
}

// use std::time::Instant;

impl PlainResourceProvider {
    pub fn new(searchlevel: SearchLevel, persistent_builtins: bool) -> PlainResourceProvider {
        let user_defined_operators = BTreeMap::new();
        let user_defined_macros = BTreeMap::new();
        let operations = BTreeMap::new();
        let globals = Vec::from([("ellps".to_string(), "GRS80".to_string())]);

        info!(
            "Creating new PlainResourceProvider - level: {:#?}, builtin persistence; {}",
            searchlevel, persistent_builtins
        );

        // Memory map the 'pile' of grid files.
        // let start = Instant::now();
        let f = File::open("geodesy/pile/pile.bin").expect("Error: 'pile.bin' not found");
        let pile = unsafe { memmap::Mmap::map(&f).expect("Error mapping 'pile.bin'")};
        // let duration = start.elapsed();
        // **350 us** println!("Time elapsed in file mapping is: {:?}", duration);

        dbg!(pile[0]);
        dbg!(pile[1]);
        dbg!(pile[2]);
        dbg!(pile[3]);
        dbg!(&pile);

        // let start = Instant::now();
        let pop: &[f32] = unsafe { std::slice::from_raw_parts(pile.as_ptr() as *const f32, pile.len()/4) };
        // let duration = start.elapsed();
        // **500 ns** println!("Time elapsed in slice building is: {:?}", duration);

        dbg!(pop[0]);
        dbg!(pop[1]);

        PlainResourceProvider {
            searchlevel,
            persistent_builtins,
            pile,
            user_defined_operators,
            user_defined_macros,
            operations,
            globals,
        }
    }

    pub fn searchlevel(&self) -> SearchLevel {
        self.searchlevel
    }

    pub fn persistent_builtins(&self) -> bool {
        self.persistent_builtins
    }

    // Workhorse for gys_definition
    fn get_gys_definition_from_level(
        &self,
        level: SearchLevel,
        branch: &str,
        name: &str,
    ) -> Option<String> {
        use std::io::BufRead;
        let filename: std::path::PathBuf = match level {
            SearchLevel::GlobalPatches => {
                // $HOME/share/geodesy/branch/name.gys
                let mut d = dirs::data_local_dir().unwrap_or_default();
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", name));
                d
            }
            SearchLevel::Globals => {
                // $HOME/share/geodesy/branch/branch.gys
                let mut d = dirs::data_local_dir().unwrap_or_default();
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", branch));
                d
            }
            SearchLevel::LocalPatches => {
                // ./geodesy/branch/name.gys
                let mut d = std::path::PathBuf::from(".");
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", name));
                d
            }
            SearchLevel::Locals => {
                // ./geodesy/branch/branch.gys
                let mut d = std::path::PathBuf::from(".");
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", branch));
                d
            }
            _ => return None,
        };

        let file = std::fs::File::open(filename);
        if file.is_err() {
            return None;
        }
        let mut file = file.unwrap();

        // Patches
        if level == SearchLevel::LocalPatches || level == SearchLevel::GlobalPatches {
            use std::io::Read;
            let mut definition = String::new();
            if file.read_to_string(&mut definition).is_ok() {
                return Some(definition);
            }
            return None;
        }

        // For non-patches, we search for the *last* occurence of a section with
        // the name we want, since updates are *appended* to the existing file
        let mut definition = String::new();
        let target = format!("<{}>", name);
        let mut skipping = true;

        let lines = std::io::BufReader::new(file).lines();
        for line in lines {
            if line.is_err() {
                continue;
            }
            let line = line.ok()?;
            if skipping && line.trim() == target {
                skipping = false;
                definition.clear();
                continue;
            }
            if skipping {
                continue;
            }
            if line.trim().starts_with('<') {
                if line.trim() != target {
                    skipping = true;
                    continue;
                }
                // Another instance of the same target
                definition.clear();
                continue;
            }
            definition += &line;
            definition += "\n";
        }

        if definition.is_empty() {
            return None;
        }
        Some(definition)
    }

    pub fn expand_experiment(&self, definition: &str) {
        let first = GysResource::new(definition, &self.globals);
        dbg!(first);
    }
}

impl Provider for PlainResourceProvider {
    fn get_user_defined_macro(&self, name: &str) -> Option<&String> {
        self.user_defined_macros.get(name)
    }

    fn get_user_defined_operator(&self, name: &str) -> Option<&OperatorConstructor> {
        self.user_defined_operators.get(name)
    }

    fn get_resource_definition(&self, branch: &str, name: &str) -> Result<String, GeodesyError> {
        if branch == "macros" {
            if let Some(m) = self.get_user_defined_macro(name) {
                return Ok(String::from(m));
            }
        }

        for i in SearchLevel::into_enum_iter() {
            if i < self.searchlevel() && i != SearchLevel::Builtins {
                continue;
            }
            if let Some(definition) = Self::get_gys_definition_from_level(self, i, branch, name) {
                return Ok(definition.trim().to_string());
            }
        }
        Err(GeodesyError::NotFound(format!("{}({})", branch, name)))
    }

    fn gys_resource(
        &self,
        branch: &str,
        name: &str,
        globals: Vec<(String, String)>,
    ) -> Result<GysResource, GeodesyError> {
        if branch == "macros" && self.user_defined_macros.contains_key(name) {
            return Ok(GysResource::new(&self.user_defined_macros[name], &globals));
        }
        let definition = self.get_resource_definition(branch, name)?;
        Ok(GysResource::new(&definition, &globals))
    }

    fn apply_operation(
        &self,
        operation: Uuid,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        if !self.operations.contains_key(&operation) {
            info!("Bad key: {}", operation);
            return false;
        }
        let op = &self.operations[&operation];
        op.operate(self, operands, forward)
    }

    fn define_operation(&mut self, definition: &str) -> Result<Uuid, GeodesyError> {
        let op = Operator::new(definition, self)?;
        let id = Uuid::new_v4();
        let name = op.name();
        self.operations.insert(id, op);
        assert_eq!(name, self.operations[&id].name());
        Ok(id)
    }

    fn get_operation(&mut self, id: Uuid) -> Result<&Operator, GeodesyError> {
        if let Some(op) = self.operations.get(&id) {
            return Ok(op);
        }
        Err(GeodesyError::General("Unknown operation"))
    }

    fn register_macro(&mut self, name: &str, definition: &str) -> Result<bool, GeodesyError> {
        self.user_defined_macros
            .insert(String::from(name), String::from(definition));
        Ok(true)
    }

    fn register_operator(
        &mut self,
        name: &str,
        constructor: OperatorConstructor,
    ) -> Result<bool, GeodesyError> {
        self.user_defined_operators
            .insert(String::from(name), constructor);
        dbg!(self.user_defined_operators.keys());
        Ok(true)
    }

    fn globals(&self) -> &[(String, String)] {
        &self.globals
    }

    /// Forward operation.
    fn fwd(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.apply_operation(operation, operands, true)
    }

    /// Inverse operation.
    fn inv(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.apply_operation(operation, operands, false)
    }
}


#[cfg(test)]
mod resourceprovidertests {
    use super::*;
    use crate::GysResource;
    #[test]
    fn plain() -> Result<(), GeodesyError> {
        let rp = PlainResourceProvider::new(SearchLevel::LocalPatches, true);
        let foo = rp
            .get_gys_definition_from_level(SearchLevel::LocalPatches, "macros", "foo")
            .unwrap();
        assert_eq!(foo.trim(), "bar");

        let rp = PlainResourceProvider::new(SearchLevel::LocalPatches, true);
        let foo = rp
            .get_gys_definition_from_level(SearchLevel::Locals, "macros", "foo")
            .unwrap();
        assert_eq!(foo.trim(), "baz");

        let rp_patch = PlainResourceProvider::new(SearchLevel::LocalPatches, false);
        let foo = rp_patch.get_resource_definition("macros", "foo")?;
        assert_eq!(foo, "bar");
        let rp_local = PlainResourceProvider::new(SearchLevel::Locals, false);
        let foo = rp_local.get_resource_definition("macros", "foo")?;
        assert_eq!(foo, "baz");

        let pop = rp_local.get_resource_definition("pile", "geoid")?;
        println!("POP: {}", pop);
        let gys = GysResource::new(&pop, &[]);
        println!("GYS: {:#?}", gys);
        let mut args = gys.to_args(0)?;
        println!("ARGS: {:?}", args);

        let left = args.numeric("Left", f64::NAN)?;
        let right = args.numeric("Right", f64::NAN)?;

        let top = args.numeric("Top", f64::NAN)?;
        let bottom = args.numeric("Bottom", f64::NAN)?;

        let cols = args.numeric("Columns", f64::NAN)?;
        let rows = args.numeric("Rows", f64::NAN)?;

        assert!(cols > 1.);
        assert!(rows > 1.);
        println!("size: [{} x {}]", cols, rows);

        // from first to last
        println!("e interval: [{}; {}]", left, right);
        println!("n interval: [{}; {}]", top, bottom);

        // last minus first
        let de = (right - left) / (cols - 1.);
        let dn = (bottom - top) / (rows - 1.);
        println!("step: [{} x {}]", de, dn);

        Ok(())
    }
}
