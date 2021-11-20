#![allow(dead_code)]

use crate::CoordinateTuple;
use crate::Ellipsoid;
use crate::GeodesyError;
use crate::GysResource;
use crate::OperatorConstructor;
use crate::operator::OperatorCore;
use crate::{FWD, INV};
use enum_iterator::IntoEnumIterator;
use uuid::Uuid;
pub mod gys;
pub mod plain;

#[derive(Debug, IntoEnumIterator, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum SearchLevel {
    LocalPatches,
    Locals,
    GlobalPatches,
    Globals,
    Builtins,
}


pub trait Provider {
    fn searchlevel(&self) -> SearchLevel {
        SearchLevel::Builtins
    }
    fn persistent_builtins(&self) -> bool {
        true
    }

    fn globals(&self) -> &[(String, String)];

    fn gys_resource(
        &self,
        branch: &str,
        name: &str,
        globals: Vec<(String, String)>,
    ) -> Result<GysResource, GeodesyError> {
        let definition = self.gys_definition(branch, name)?;
        Ok(GysResource::new(&definition, &globals))
    }

    fn gys_definition(&self, branch: &str, name: &str) -> Result<String, GeodesyError> {
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

    /// workhorse for gys_definition (perhaps implement as generic instead, since we don't actually want this to be user visible)
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

    fn operate(&self, operation: Uuid, operands: &mut [CoordinateTuple], forward: bool) -> bool;

    fn operation(&mut self, definition: &str) -> Result<Uuid, GeodesyError>;
    fn operator(&mut self, id: Uuid) -> Result<&Operator, GeodesyError> {
        Err(GeodesyError::General("Operator extraction not supported"))
    }

    #[allow(unused_variables)]
    fn register_macro(&mut self, name: &str, definition: &str) -> Result<bool, GeodesyError> {
        Err(GeodesyError::General("Macro registration not supported"))
    }

    #[allow(unused_variables)]
    fn register_operator(
        &mut self,
        name: &str,
        definition: OperatorConstructor,
    ) -> Result<bool, GeodesyError> {
        Err(GeodesyError::General("Operator registration not supported"))
    }

    /// Operate in forward direction.
    fn fwd(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, true)
    }

    /// Operate in inverse direction.
    fn inv(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }

    fn ellipsoid(&self, name: &str) -> Result<Ellipsoid, GeodesyError> {
        if name == "GRS80" {
            return Ok(Ellipsoid::default());
        }
        Err(GeodesyError::NotFound(String::from(name)))
    }

    // fn grid_descriptor(&self, name: &str) -> Result<GridDescriptor, GeodesyError> {
    //     Err(GeodesyError::NotFound(String::from(name)))
    // }
}

use crate::GysArgs;
use crate::Operator;

#[derive(Debug)]
pub struct Popeline {
    args: Vec<(String, String)>,
    pub steps: Vec<Operator>,
    inverted: bool,
}

impl Popeline {
    pub fn new(
        args: &GysResource,
        rp: &dyn Provider,
        recursion_level: usize,
    ) -> Result<Operator, GeodesyError> {
        if recursion_level > 100 {
            return Err(GeodesyError::Recursion(format!("{:#?}", args)));
        }
        let mut margs = args.clone();
        let mut globals = GysArgs::new(&args.globals, "");

        // Is the pipeline itself inverted?
        let inverted = globals.flag("inv");

        // How many steps?
        let n = args.steps.len();

        // Redact the globals to eliminate the chaos-inducing "inv" and "name":
        // These are related to the pipeline itself, not its constituents.
        let globals: Vec<_> = args
            .globals
            .iter()
            .filter(|x| x.0 != "inv" && x.0 != "name")
            .cloned()
            .collect();
        let nextglobals = globals.clone();
        let mut steps = Vec::<Operator>::new();
        for step in &args.steps {
            // An embedded pipeline? (should not happen - elaborate!)
            if step.find('|').is_some() {
                continue;
            }

            let mut args = GysArgs::new(&nextglobals, step);

            let nextname = &args.value("name")?.unwrap_or_default();

            // A macro? - args are now globals!
            if let Ok(mac) = rp.gys_definition("macros", nextname) {
                for arg in &args.locals {
                    let a = arg.clone();
                    args.globals.push(a);
                }
                let nextargs = GysResource::new(&mac, &globals);
                let next = Popeline::new(&nextargs, rp, recursion_level + 1)?;
                if n == 1 {
                    return Ok(next);
                }
                steps.push(next);
                continue;
            }

            // If we did not find nextname among the resources - it's probably a builtin
            let op = crate::operator::builtins::builtin(nextname)?;
            let args = GysResource::new(step, &nextglobals);
            let next = op(&args, rp)?;
            if n == 1 {
                return Ok(next);
            }
            steps.push(next);
            continue;
        }

        // makeshift clear text description
        margs.globals.clear();
        for step in margs.steps {
            margs.globals.push((String::from("step"), step));
        }

        let result = Popeline {
            args: margs.globals,
            steps,
            inverted: inverted,
        };

        Ok(Operator(Box::new(result)))
    }
}


impl OperatorCore for Popeline {
    fn fwd(&self, ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        for step in &self.steps {
            if step.is_noop() {
                continue;
            }
            if !step.operate(ctx, operands, FWD) {
                return false;
            }
        }
        true
    }

    fn inv(&self, ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        for step in self.steps.iter().rev() {
            if step.is_noop() {
                continue;
            }
            if !step.operate(ctx, operands, INV) {
                return false;
            }
        }
        true
    }

    fn len(&self) -> usize {
        self.steps.len()
    }

    fn args(&self, step: usize) -> &[(String, String)] {
        if step >= self.len() {
            return &self.args;
        }
        self.steps[step].args(0_usize)
    }

    fn name(&self) -> &'static str {
        "pipeline"
    }

    fn debug(&self) -> String {
        let mut repr = String::new();
        for step in &self.steps {
            repr += "\n";
            repr += &format!("{:#?}", step);
        }
        repr
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }
}

#[cfg(test)]
mod popelinetests {
    use super::*;
    use crate::PlainResourceProvider;

    #[test]
    fn gys() -> Result<(), GeodesyError> {
        let rp = PlainResourceProvider::new(SearchLevel::LocalPatches, true);
        let foo = rp
            .get_gys_definition_from_level(SearchLevel::LocalPatches, "macros", "foo")
            .unwrap();
        assert_eq!(foo.trim(), "bar");

        // This should be OK, since noop is a builtin
        let res = GysResource::from("noop pip");
        let p = Popeline::new(&res, &rp, 0);
        assert!(p.is_ok());

        // This should be OK, due to "ignore" resolving to noop
        let res = GysResource::from("ignore pip");
        let p = Popeline::new(&res, &rp, 0);
        assert!(p.is_ok());

        // This should fail, due to "baz" being undefined
        let res = GysResource::from("ignore pip|baz pop");
        let p = Popeline::new(&res, &rp, 0);
        assert!(p.is_err());
        Ok(())
    }
}
