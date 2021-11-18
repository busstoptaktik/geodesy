#![allow(dead_code)]

use crate::context::gys::GysArgs;
use crate::context::gys::GysResource;
use crate::operator_construction as oa;
use crate::operator_construction::Operator;
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
    //use crate::GeodesyError;
    //use crate::context::nygys::*;
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

pub struct Popeline {
    args: GysResource,
    steps: Vec<Operator>,
    inverted: bool,
}

impl Popeline {
    pub fn new(args: &GysResource, rp: &dyn Provider) -> Result<Popeline, GeodesyError> {
        let margs = args.clone();
        let mut globals = GysArgs::new(&args.globals, "");
        let inverted = globals.flag("inv");
        let _n = args.steps.len();
        let globals: Vec<_> = args
            .globals
            .iter()
            .filter(|x| x.0 != "inv" && x.0 != "name")
            .collect();
        let steps = Vec::<Operator>::new();
        /*
                for i in 0..n {
                    // Each step is represented as args[_step_0] = YAML step definition.
                    // (see OperatorArgs::populate())
                    let step_name = format!("_step_{}", i);
                    let step_args = &args.args[&step_name];

                    // We need a recursive copy of "all globals so far"
                    let mut oa = args.spawn(step_args);
                    if let Ok(op) = operator_factory(&mut oa, ctx, 0) {
                        steps.push(op);
                    } else {
                        return Err(GeodesyError::General("Pipeline: Bad step"));
                    }
                }

                let args = args.clone();
        */
        Ok(Popeline {
            args: margs,
            steps: steps,
            inverted: inverted?,
        })
    }
}
