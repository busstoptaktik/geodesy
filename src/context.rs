use std::collections::HashMap;

use crate::CoordinateTuple;
use crate::CoordinatePrimitives;
use crate::Operator;
use crate::OperatorConstructor;
use crate::OperatorCore;

#[derive(Debug, Default)]
pub struct Resource {
    bbox: CoordinateTuple,
}

impl Resource {
    #[must_use]
    pub fn _new() -> Resource {
        Resource {
            bbox: CoordinateTuple::new(0., 0., 0., 0.),
        }
    }
}

#[derive(Default)]
pub struct Context {
    pub coord: CoordinateTuple,
    pub stack: Vec<CoordinateTuple>,
    minions: Vec<Context>,
    resources: HashMap<String, Resource>,
    user_defined_operators: HashMap<String, OperatorConstructor>,
    user_defined_macros: HashMap<String, String>,
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
        Context {
            coord: CoordinateTuple::new(0., 0., 0., 0.),
            stack: vec![],
            minions: vec![],
            resources: HashMap::new(),
            last_failing_operation: "",
            cause: "",
            user_defined_operators: HashMap::new(),
            user_defined_macros: HashMap::new(),
        }
    }

    pub fn operate(&mut self, operator: &Operator, forward: bool) -> bool {
        operator.operate(self, forward)
    }

    pub fn fwd(&mut self, operator: &Operator, agurk: &mut [CoordinateTuple]) -> bool {
        for _a in agurk {
            operator.operate(self, true);
        }
        true
    }

    pub fn inv(&mut self, operator: &Operator) -> bool {
        operator.operate(self, false)
    }

    pub fn register_operator(&mut self, name: &str, constructor: OperatorConstructor) {
        self.user_defined_operators
            .insert(name.to_string(), constructor);
    }

    pub fn locate_operator(&mut self, name: &str) -> Option<&OperatorConstructor> {
        self.user_defined_operators.get(name)
    }

    pub fn register_macro(&mut self, name: &str, definition: &str) {
        self.user_defined_macros
            .insert(name.to_string(), definition.to_string());
    }

    pub fn locate_macro(&mut self, name: &str) -> Option<&String> {
        self.user_defined_macros.get(name)
    }

    pub fn resource(&self, name: &str) -> Option<&Resource> {
        self.resources.get(name)
    }

    pub fn operator(&mut self, definition: &str) -> Result<Operator, String> {
        Operator::new(definition, self)
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operand() {
        use crate::Context;
        let ond = Context::new();
        assert_eq!(ond.stack.len(), 0);
        assert_eq!(ond.coord[0], 0.);
        assert_eq!(ond.coord[1], 0.);
        assert_eq!(ond.coord[2], 0.);
        assert_eq!(ond.coord[3], 0.);
    }

    #[test]
    fn operate() {
        use crate::Context;
        use crate::Operator;
        use crate::CoordinatePrimitives;
        use crate::{fwd, inv};
        let pipeline = "ed50_etrs89: {
            steps: [
                cart: {ellps: intl},
                helmert: {dx: -87, dy: -96, dz: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";
        let mut ond = Context::new();
        let op = Operator::new(pipeline, &mut ond).unwrap();
        ond.coord = crate::CoordinateTuple::deg(12., 55., 100., 0.);

        ond.operate(&op, fwd);
        assert!((ond.coord.to_degrees()[0] - 11.998815342385206861).abs() < 1e-12);
        assert!((ond.coord.to_degrees()[1] - 54.999382648950991381).abs() < 1e-12);

        ond.operate(&op, inv);
        assert!((ond.coord.to_degrees()[0] - 12.).abs() < 1e-12);
        assert!((ond.coord.to_degrees()[1] - 55.).abs() < 1e-12);
    }
}
