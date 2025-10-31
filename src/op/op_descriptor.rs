use super::*;

/// The fundamental elements of an operator (i.e. everything but steps and args)
#[derive(Debug, Default)]
pub struct OpDescriptor {
    pub invoked_as: String, // e.g. geo:helmert ellps_0=GRS80 x=1 y=2 z=3 ellps_1=intl
    pub instantiated_as: String, // e.g. cart ellps=$ellps_0 | helmert | cart inv ellps=$ellps_1
    pub invertible: bool,
    pub inverted: bool,
    pub fwd: InnerOp,
    pub inv: InnerOp,
    // pub id: OpHandle,
}

impl OpDescriptor {
    pub fn new(definition: &str, fwd: InnerOp, inv: Option<InnerOp>) -> OpDescriptor {
        let definition = definition.to_string();
        let invertible = inv.is_some();
        let inverted = false; // Handled higher up in the call hierarchy
        let invocation = "".to_string(); // Handled higher up in the call hierarchy
        let inv = inv.unwrap_or_default();
        // let id = OpHandle::new();
        OpDescriptor {
            invoked_as: invocation,
            instantiated_as: definition,
            invertible,
            inverted,
            fwd,
            inv,
            // id,
        }
    }
}
