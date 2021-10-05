use std::collections::HashMap;

use log::info;

use crate::operator_construction::*;
use crate::CoordinateTuple;

/// The central administration of the transformation functionality
#[derive(Default)]
pub struct Context {
    pub stack: Vec<Vec<CoordinateTuple>>,
    minions: Vec<Context>,
    user_defined_operators: HashMap<String, OperatorConstructor>,
    user_defined_macros: HashMap<String, String>,
    operations: Vec<Operator>,
    last_failing_operation_definition: String,
    last_failing_operation: String,
    cause: String,
}

mod gys;
mod test;
mod user_defined;

impl Context {
    /// Number of chunks to process in (principle in) parallel.
    const CHUNKS: usize = 3;

    /// Maximum size of each chunk.
    const CHUNK_SIZE: usize = 1000;

    pub fn new() -> Context {
        info!("Creating new Context");
        let mut ctx = Context::_new();
        for _ in 0..Self::CHUNKS {
            ctx.minions.push(Context::_new());
        }
        ctx
    }

    fn _new() -> Context {
        Context {
            stack: Vec::new(),
            minions: Vec::new(),
            last_failing_operation_definition: String::new(),
            last_failing_operation: String::new(),
            cause: String::new(),
            user_defined_operators: HashMap::new(),
            user_defined_macros: HashMap::new(),
            operations: Vec::new(),
        }
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
        &mut self,
        operation: usize,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        if operation >= self.operations.len() {
            self.last_failing_operation = String::from("Invalid");
            self.cause = String::from("Attempt to access an invalid operator from context");
            return false;
        }
        let mut i = 0_usize;
        let mut result = true;
        for chunk in operands.chunks_mut(Self::CHUNK_SIZE) {
            // Need a bit more std::thread-Rust-fu to do actual mutithreading.
            // For now, we just split the input data in chunks, process them
            // and verify that the parallel stack-functionality works.
            result &= self.minions[i]._operate(&self.operations[operation], chunk, forward);
            self.minions[i].stack.clear();
            i = (i + 1) % Self::CHUNKS;
        }
        result
    }

    /// Forward operation.
    pub fn fwd(&mut self, operation: usize, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, true)
    }

    /// Inverse operation.
    pub fn inv(&mut self, operation: usize, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }

    pub fn error(&mut self, which: &str, why: &str) {
        self.last_failing_operation = String::from(which);
        self.cause = String::from(why);
    }

    pub fn report(&mut self) -> String {
        format!(
            "Last failure in {}: {}\n{}",
            self.last_failing_operation, self.cause, self.last_failing_operation_definition
        )
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operand() {
        use crate::Context;
        let ctx = Context::new();
        assert_eq!(ctx.stack.len(), 0);
    }

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
        assert!(op.is_some());
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
