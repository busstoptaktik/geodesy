use super::*;

/// The fundamental elements of an operator (i.e. everything but steps and args)
#[derive(Debug, Default)]
pub struct OpDescriptor {
    pub instantiated_as: String, // e.g. cart ellps=$ellps_0 | helmert | cart inv ellps=$ellps_1
    pub inverted: bool,
    pub fwd: InnerOp,
    pub inv: Option<InnerOp>,
}

impl OpDescriptor {
    pub fn new(definition: &str, fwd: InnerOp, inv: Option<InnerOp>) -> OpDescriptor {
        let definition = definition.to_string();
        let inverted = false; // Handled higher up in the call hierarchy
        OpDescriptor {
            instantiated_as: definition,
            inverted,
            fwd,
            inv,
        }
    }
}
