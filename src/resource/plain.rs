#![allow(dead_code)]
use log::info;

use crate::CoordinateTuple;
use crate::GeodesyError;
use crate::GysResource;
use crate::Operator;
use crate::OperatorConstructor;
use std::collections::BTreeMap;

use super::Provider;
use super::SearchLevel;
use crate::operator::OperatorCore;

//---------------------------------------------------------------------------------
// Enter the land of the ResourceProviders
//---------------------------------------------------------------------------------

use uuid::Uuid;

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
        PlainResourceProvider::new(SearchLevel::Builtins, true)
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

    fn get_user_defined_macro(&self, name: &str) -> Option<&String> {
        self.user_defined_macros.get(name)
    }

    fn get_user_defined_operator(&self, name: &str) -> Option<&OperatorConstructor> {
        self.user_defined_operators.get(name)
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
    use crate::GeodesyError;
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
