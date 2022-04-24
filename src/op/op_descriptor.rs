use super::*;

/// The fundamental elements of an operator (i.e. everything but steps and args)
#[derive(Debug, Default)]
pub struct OpDescriptor {
    pub invocation: String, // e.g. geohelmert ellps_0=GRS80 x=1 y=2 z=3 ellps_1=intl
    pub definition: String, // e.g. cart ellps=^ellps_0 | helmert | cart inv ellps=^ellps_1
    pub invertible: bool,
    pub inverted: bool,
    pub fwd: InnerOp,
    pub inv: InnerOp,
    pub uuid: OpHandle,
}

impl OpDescriptor {
    pub fn new(definition: &str, fwd: InnerOp, inv: Option<InnerOp>) -> OpDescriptor {
        let definition = definition.to_string();
        let invertible = inv.is_some();
        let inverted = false; // Handled higher up in the call hierarchy
        let invocation = "".to_string(); // Handled higher up in the call hierarchy
        let inv = inv.unwrap_or_default();
        let uuid = OpHandle(uuid::Uuid::new_v4());
        OpDescriptor {
            invocation,
            definition,
            invertible,
            inverted,
            fwd,
            inv,
            uuid,
        }
    }
}
