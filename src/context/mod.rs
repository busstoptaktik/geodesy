use log::info;
use std::collections::HashMap;

use crate::operator_construction::*;
use crate::CoordinateTuple;

/// The central administration of the transformation functionality
// #[derive(Default)]
pub struct Context {
    user_defined_operators: HashMap<String, OperatorConstructor>,
    user_defined_macros: HashMap<String, String>,
    operations: Vec<Operator>,
}

mod gys;
mod test;
mod user_defined;

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Context {
        info!("Creating new Context");
        Context {
            user_defined_operators: HashMap::new(),
            user_defined_macros: HashMap::new(),
            operations: Vec::new(),
        }
    }

    fn _grid_provider() {
        // let mut pile_path = dirs::data_local_dir().unwrap_or_default();
        // pile_path.push("geodesy");
        // pile_path.push("assets.pile");
        // let pile_name = pile_path.clone();
        // let thepile = File::open(pile_path);
        // if thepile.is_err() {
        //     info!("Could not find asset pile {:?}", pile_name);
        // } else {
        //     info!("Found asset pile {:?}", pile_name);
        // }
    }

    // Parallel execution helper for `operate`, below
    fn _operate(
        &mut self,
        operator: &Operator,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        operator.operate(self, operands, forward)
    }

    pub fn operate(
        &self,
        operation: usize,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        if operation >= self.operations.len() {
            // self.last_failing_operation = String::from("Invalid");
            // self.cause = String::from("Attempt to access an invalid operator from context");
            return false;
        }
        let op = &self.operations[operation];
        op.operate(self, operands, forward)
    }

    /// Forward operation.
    pub fn fwd(&self, operation: usize, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, true)
    }

    /// Inverse operation.
    pub fn inv(&self, operation: usize, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operate() {
        use crate::Context;
        use crate::CoordinateTuple;

        let pipeline = "ed50_etrs89: {
            steps: [
                cart: {ellps: intl},
                helmert: {x: -87, y: -96, z: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";

        let mut ctx = Context::new();
        let op = ctx.operation(pipeline);
        assert!(op.is_ok());
        let op = op.unwrap();
        let geo = CoordinateTuple::gis(12., 55., 100., 0.);
        let mut operands = [geo];

        ctx.fwd(op, &mut operands);
        let result = operands[0].to_degrees();
        assert!((result[0] - 11.998815342385206861).abs() < 1e-10);
        assert!((result[1] - 54.999382648950991381).abs() < 1e-10);

        ctx.inv(op, &mut operands);
        let result = operands[0].to_degrees();
        assert!((result[0] - 12.).abs() < 1e-12);
        assert!((result[1] - 55.).abs() < 1e-12);
    }
}
