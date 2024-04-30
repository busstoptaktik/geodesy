use super::*;

/// The fundamental elements of an operator (i.e. everything but steps and args)
#[derive(Debug, Default)]
pub struct OpDescriptor {
    pub invocation: String, // e.g. geo:helmert ellps_0=GRS80 x=1 y=2 z=3 ellps_1=intl
    pub definition: String, // e.g. cart ellps=$ellps_0 | helmert | cart inv ellps=$ellps_1
    pub steps: Vec<String>,
    pub invertible: bool,
    pub inverted: bool,
    pub fwd: InnerOp,
    pub inv: InnerOp,
    pub id: OpHandle,
}

impl OpDescriptor {
    pub fn new(definition: &str, fwd: InnerOp, inv: Option<InnerOp>) -> OpDescriptor {
        let steps = definition.split_into_steps();
        let definition = definition.to_string();
        let invertible = inv.is_some();
        let inverted = false; // Handled higher up in the call hierarchy
        let invocation = "".to_string(); // Handled higher up in the call hierarchy
        let inv = inv.unwrap_or_default();
        let id = OpHandle::new();
        OpDescriptor {
            invocation,
            definition,
            steps,
            invertible,
            inverted,
            fwd,
            inv,
            id,
        }
    }
}
