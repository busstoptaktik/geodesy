use std::collections::HashMap;

use crate::CoordinateTuple;
use crate::Operator;
use crate::OperatorCore;

#[derive(Debug, Default)]
struct Resource {
    bbox: CoordinateTuple
}

impl Resource {
    #[must_use]
    pub fn _new() -> Resource {
        Resource {
            bbox: CoordinateTuple(0., 0., 0., 0.)
        }
    }
}

#[derive(Debug, Default)]
pub struct Shuttle {
    pub coord: CoordinateTuple,
    pub stack: Vec<f64>,
    pub coordinate_stack: Vec<CoordinateTuple>,
    resources: HashMap<String, Resource>,
    pub last_failing_operation: &'static str,
    pub cause: &'static str,
}

impl Shuttle {
    #[must_use]
    pub fn new() -> Shuttle {
        Shuttle {
            coord: CoordinateTuple(0., 0., 0., 0.),
            stack: vec![],
            coordinate_stack: vec![],
            resources: HashMap::new(),
            last_failing_operation: "",
            cause: "",
        }
    }

    pub fn operate(&mut self, operator: &Operator, forward: bool) -> bool {
        operator.operate(self, forward)
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operand() {
        use crate::Shuttle;
        let ond = Shuttle::new();
        assert_eq!(ond.stack.len(), 0);
        assert_eq!(ond.coordinate_stack.len(), 0);
        assert_eq!(ond.coord.0, 0.);
        assert_eq!(ond.coord.1, 0.);
        assert_eq!(ond.coord.2, 0.);
        assert_eq!(ond.coord.3, 0.);
    }

    #[test]
    fn operate() {
        use crate::Operator;
        use crate::Shuttle;
        use crate::{fwd, inv};
        let pipeline = "ed50_etrs89: {
            steps: [
                cart: {ellps: intl},
                helmert: {dx: -87, dy: -96, dz: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";
        let op = Operator::new(pipeline).unwrap();
        let mut ond = Shuttle::new();
        ond.coord = crate::CoordinateTuple::deg(12., 55., 100., 0.);
        ond.operate(&op, fwd);
        assert!((ond.coord.to_degrees().0 - 11.998815342385206861).abs() < 1e-12);
        assert!((ond.coord.to_degrees().1 - 54.999382648950991381).abs() < 1e-12);
        println!("{:?}", ond.coord.to_degrees());
        ond.operate(&op, inv);
        let e = ond.coord.to_degrees();
        println!("{:?}", e);
        assert!((ond.coord.to_degrees().0 - 12.).abs() < 1e-12);
        assert!((ond.coord.to_degrees().1 - 55.).abs() < 1e-12);
    }

}
