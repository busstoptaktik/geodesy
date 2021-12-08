/// Plain resource provider. Support for user defined operators
/// and macros using a text file library
use std::collections::BTreeMap;
use log::info;
use uuid::Uuid;

use super::GysResource;
use crate::Provider;
use crate::CoordinateTuple;
use crate::GeodesyError;
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
    // pile: memmap::Mmap
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

        PlainResourceProvider {
            searchlevel,
            persistent_builtins,
            user_defined_operators,
            user_defined_macros,
            operations,
            globals,
        }
    }

    fn searchlevel(&self) -> SearchLevel {
        self.searchlevel
    }

    fn persistent_builtins(&self) -> bool {
        self.persistent_builtins
    }

    /// workhorse for gys_definition. Move to plain, since we don't actually want this part
    /// to be user visible. Also, remove methods "searchlevel" and "persistent_builtins",
    /// since they are related to a specific resource provider (Plain).
    /// Then introduce the dynamically allocated grid accessor element. (TODO!)
    pub(crate) fn get_gys_definition_from_level(
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

    fn gys_definition(&self, branch: &str, name: &str) -> Result<String, GeodesyError> {
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
        let definition = self.gys_definition(branch, name)?;
        Ok(GysResource::new(&definition, &globals))
    }

    fn operate(&self, operation: Uuid, operands: &mut [CoordinateTuple], forward: bool) -> bool {
        if !self.operations.contains_key(&operation) {
            println!("Lortelort - forkert nøgle!!!");
            return false;
        }
        let op = &self.operations[&operation];
        op.operate(self, operands, forward)
    }

    fn operation(&mut self, definition: &str) -> Result<Uuid, GeodesyError> {
        let op = Operator::new(definition, self)?;
        let id = Uuid::new_v4();
        let name = op.name();
        self.operations.insert(id, op);
        assert_eq!(name, self.operations[&id].name());
        Ok(id)
    }

    fn operator(&mut self, id: Uuid) -> Result<&Operator, GeodesyError> {
        if let Some(op) = self.operations.get(&id) {
            return Ok(op);
        }
        Err(GeodesyError::General("Unknown operator"))
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
        self.operate(operation, operands, true)
    }

    /// Inverse operation.
    fn inv(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }
}

// let f = File::open("numbers.bin").expect("Error: 'numbers.bin' not found");
// Create the memory mapped buffer
// let mmap = unsafe { memmap::Mmap::map(&f).expect("Error mapping 'numbers.bin'") };

#[cfg(test)]
mod resourceprovidertests {
    use super::*;
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
        let foo = rp_patch.gys_definition("macros", "foo")?;
        assert_eq!(foo, "bar");
        let rp_local = PlainResourceProvider::new(SearchLevel::Locals, false);
        let foo = rp_local.gys_definition("macros", "foo")?;
        assert_eq!(foo, "baz");

        Ok(())
    }
}
