use crate::internal::*;

/// The fundamental elements of an operator (i.e. everything but steps and args)
#[derive(Debug, Default)]
pub struct Base {
    pub invocation: String, // e.g. geohelmert ellps_0=GRS80 x=1 y=2 z=3 ellps_1=intl
    pub definition: String, // e.g. cart ellps=^ellps_0 | helmert | cart inv ellps=^ellps_1
    pub invertible: bool,
    pub inverted: bool,
    pub fwd: InnerOp,
    pub inv: InnerOp,
    pub uuid: uuid::Uuid,
}

impl Base {
    pub fn new(definition: &str, fwd: InnerOp, inv: Option<InnerOp>) -> Base {
        let definition = definition.to_string();
        let invertible = inv.is_some();
        let inverted = false; // TODO
        let invocation = "".to_string(); // TODO
        let inv = inv.unwrap_or_default();
        let uuid = uuid::Uuid::new_v4();
        Base {
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
