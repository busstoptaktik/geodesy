#![allow(dead_code)]
use log::info;

use crate::context::gys::GysArgs;
use crate::context::gys::GysResource;
use crate::operator_construction as oa;
use crate::operator_construction::Operator;
use crate::operator_construction::OperatorArgs;
use crate::CoordinateTuple;
use crate::GeodesyError;
use std::collections::HashMap;

use super::Provider;
use super::SearchLevel;

//---------------------------------------------------------------------------------
// Enter the land of the ResourceProviders
//---------------------------------------------------------------------------------

use uuid::Uuid;

// Preliminary scaffolding
pub trait Opera {
    fn operator_operate(&self, operands: &mut [CoordinateTuple], forward: bool) -> bool;
}
impl Opera for Operator {
    fn operator_operate(&self, _operands: &mut [CoordinateTuple], _forward: bool) -> bool {
        false
    }
}

pub struct PlainResourceProvider {
    searchlevel: SearchLevel,
    persistent_builtins: bool,
    // pile: memmap::Mmap
    user_defined_operators: HashMap<String, oa::OperatorConstructor>,
    user_defined_macros: HashMap<String, String>,
    operations: HashMap<Uuid, oa::Operator>,
    globals: Vec<(String, String)>,
}

impl Default for PlainResourceProvider {
    fn default() -> PlainResourceProvider {
        PlainResourceProvider::new(SearchLevel::Builtins, true)
    }
}

impl PlainResourceProvider {
    pub fn new(searchlevel: SearchLevel, persistent_builtins: bool) -> PlainResourceProvider {
        let user_defined_operators = HashMap::new();
        let user_defined_macros = HashMap::new();
        let operations = HashMap::new();
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

    pub fn expand_experiment(&self, definition: &str) {
        let first = GysResource::new(definition, &self.globals);
        dbg!(first);
    }
}

impl Provider for PlainResourceProvider {
    fn searchlevel(&self) -> SearchLevel {
        self.searchlevel
    }

    fn persistent_builtins(&self) -> bool {
        self.persistent_builtins
    }

    fn operate(&self, operation: Uuid, operands: &mut [CoordinateTuple], forward: bool) -> bool {
        if !self.operations.contains_key(&operation) {
            return false;
        }
        let op = &self.operations[&operation];
        op.operator_operate(operands, forward);
        false
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
    fn gys() -> Result<(), GeodesyError> {
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

#[derive(Debug)]
pub struct Popeline {
    args: GysResource,
    steps: Vec<Operator>,
    inverted: bool,
}

impl Popeline {
    pub fn new(
        args: &GysResource,
        rp: &dyn Provider,
        level: usize,
    ) -> Result<Popeline, GeodesyError> {
        if level > 100 {
            return Err(GeodesyError::Recursion(format!("{:#?}", args)));
        }
        let margs = args.clone();
        let mut globals = GysArgs::new(&args.globals, "");

        // Is the pipeline itself inverted?
        let inverted = globals.flag("inv");

        // How many steps?
        let _n = args.steps.len();

        // Redact the globals to eliminate the chaos-inducing "inv" and "name":
        // These are related to the pipeline itself, not its constituents.
        let globals: Vec<_> = args
            .globals
            .iter()
            .filter(|x| x.0 != "inv" && x.0 != "name")
            .cloned()
            .collect();
        // While testing, we just accumulate popelines, not Operators
        // let mut steps = Vec::<Operator>::new();
        let mut steps = Vec::<Operator>::new();
        for step in &args.steps {
            // An embedded pipeline? (should not happen - elaborate!)
            if step.find('|').is_some() {
                continue;
            }

            let mut args = GysArgs::new(&globals, step);
            dbg!(&args);

            let nextname = &args.value("name")?.unwrap_or_default();

            // A macro? - args are now globals!
            if let Ok(mac) = rp.gys_definition("macros", nextname) {
                for arg in &args.locals {
                    let a = arg.clone();
                    args.globals.push(a);
                }
                let nextargs = GysResource::new(&mac, &globals);
                dbg!(&nextargs);
                let next = Popeline::new(&nextargs, rp, level + 1)?;
                // TODO - do something with next
                continue;
            }

            // If we do not find nextname among the resources - it's probably a builtin
            let op = crate::operator::builtins::builtin(nextname)?;
            let next = op(&mut OperatorArgs::default())?; // TODO OperatorArgs => args
            steps.push(next);
            continue;
        }

        Ok(Popeline {
            args: margs,
            steps,
            inverted: inverted?,
        })
    }
}

#[cfg(test)]
mod popelinetests {
    use super::*;
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
        dbg!(p?);

        // This should be OK, due to "ignore" resolving to noop
        let res = GysResource::from("ignore pip");
        let p = Popeline::new(&res, &rp, 0);
        assert!(p.is_ok());
        dbg!(p?);

        // This should fail, due to "baz" being undefined
        let res = GysResource::from("ignore pip|baz pop");
        let p = Popeline::new(&res, &rp, 0);
        assert!(p.is_err());
        Ok(())
    }
}
