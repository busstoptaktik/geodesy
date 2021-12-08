use log::info;
/// Minimal resource provider. No support for user defined operators and macros
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::CoordinateTuple;
use crate::GeodesyError;
use crate::Provider;
use crate::{Operator, OperatorCore};

pub struct MinimalResourceProvider {
    operations: BTreeMap<Uuid, Operator>,
    globals: Vec<(String, String)>,
}

impl Default for MinimalResourceProvider {
    fn default() -> MinimalResourceProvider {
        MinimalResourceProvider::new()
    }
}

impl MinimalResourceProvider {
    pub fn new() -> MinimalResourceProvider {
        let operations = BTreeMap::new();
        let globals = Vec::from([("ellps".to_string(), "GRS80".to_string())]);
        info!("Creating new MinimalResourceProvider");
        MinimalResourceProvider {
            operations,
            globals,
        }
    }
}

impl Provider for MinimalResourceProvider {
    fn apply_operation(
        &self,
        operation: Uuid,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        if !self.operations.contains_key(&operation) {
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
        Err(GeodesyError::General("Unknown operator"))
    }

    fn globals(&self) -> &[(String, String)] {
        &self.globals
    }
}

#[cfg(test)]
mod resourceprovidertests {
    use super::*;
    #[test]
    fn minimal() -> Result<(), GeodesyError> {
        let rp_patch = MinimalResourceProvider::new();
        let foo = rp_patch.get_resource_definition("macros", "foo");
        assert!(foo.is_err());
        let rp_local = MinimalResourceProvider::new();
        let foo = rp_local.get_resource_definition("macros", "foo");
        assert!(foo.is_err());

        Ok(())
    }
}
