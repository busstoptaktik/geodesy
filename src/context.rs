use std::collections::HashMap;

use crate::coordinates::CoordinateTuple;
use crate::Operator;
use crate::OperatorConstructor;
use crate::OperatorCore;

/// The central administration of the transformation functionality
#[derive(Default)]
pub struct Context {
    pub stack: Vec<Vec<CoordinateTuple>>,
    minions: Vec<Context>,
    user_defined_operators: HashMap<String, OperatorConstructor>,
    user_defined_macros: HashMap<String, String>,
    operators: Vec<Operator>,
    pub(crate) last_failing_operation: &'static str,
    pub(crate) cause: &'static str,
}

impl Context {
    pub fn new() -> Context {
        let mut ctx = Context::_new();
        ctx.minions.push(Context::_new());
        ctx.minions.push(Context::_new());
        ctx.minions.push(Context::_new());
        ctx
    }

    fn _new() -> Context {
        let mut thestack: Vec<Vec<CoordinateTuple>> = vec![];
        for _i in 0..1000 {
            thestack.push(vec![]);
        }
        Context {
            stack: thestack,
            minions: vec![],
            last_failing_operation: "",
            cause: "",
            user_defined_operators: HashMap::new(),
            user_defined_macros: HashMap::new(),
            operators: vec![],
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
        operator: usize,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        if operator >= self.operators.len() {
            self.last_failing_operation = "Invalid";
            self.cause = "Attempt to access an invalid operator from context";
            return false;
        }
        let mut i = 0_usize;
        let mut result = true;
        for chunk in operands.chunks_mut(2) {
            // Need a bit more std::thread-Rust-fu to do actual mutithreading.
            // For now, we just split the input data in chunks, process them
            // and verify that the parallel stack-functionality works.
            result &= self.minions[i]._operate(&self.operators[operator], chunk, forward);
            i = (i + 1) % 3;
        }
        result
    }

    pub fn fwd(&mut self, operator: usize, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operator, operands, true)
    }

    pub fn inv(&mut self, operator: usize, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operator, operands, false)
    }

    pub fn register_operator(&mut self, name: &str, constructor: OperatorConstructor) {
        self.user_defined_operators
            .insert(name.to_string(), constructor);
    }

    pub(crate) fn locate_operator(&mut self, name: &str) -> Option<&OperatorConstructor> {
        self.user_defined_operators.get(name)
    }

    #[must_use]
    pub fn register_macro(&mut self, name: &str, definition: &str) -> bool {
        // Registering a macro under the same name as its definition name
        // leads to infinite nesting - so we prohibit that
        let illegal_start = name.to_string() + ":";
        if definition.trim_start().starts_with(&illegal_start) {
            return false;
        }

        if self
            .user_defined_macros
            .insert(name.to_string(), definition.to_string())
            .is_some()
        {
            return false;
        }
        true
    }

    pub(crate) fn locate_macro(&mut self, name: &str) -> Option<&String> {
        self.user_defined_macros.get(name)
    }

    pub fn operator(&mut self, definition: &str) -> Result<usize, String> {
        let op = match Operator::new(definition, self) {
            Err(err) => return Err(err),
            Ok(ok) => ok,
        };

        let index = self.operators.len();
        self.operators.push(op);
        Ok(index)
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operand() {
        use crate::Context;
        let ctx = Context::new();
        assert_eq!(ctx.stack.len(), 1000);
    }

    #[test]
    fn operate() {
        use crate::Context;
        use crate::CoordinateTuple;

        let pipeline = "ed50_etrs89: {
            steps: [
                cart: {ellps: intl},
                helmert: {dx: -87, dy: -96, dz: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";

        let mut ctx = Context::new();
        let op = ctx.operator(pipeline);
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
