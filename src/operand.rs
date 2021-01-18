use crate::CoordinateTuple;

#[derive(Debug, Default)]
pub struct Operand {
    pub coord: CoordinateTuple,
    pub stack: Vec<f64>,
    pub coordinate_stack: Vec<CoordinateTuple>,
    pub last_failing_operation: &'static str,
    pub cause: &'static str,
}

impl Operand {
    #[must_use]
    pub fn new() -> Operand {
        Operand {
            coord: CoordinateTuple(0., 0., 0., 0.),
            stack: vec![],
            coordinate_stack: vec![],
            last_failing_operation: "",
            cause: "",
        }
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operand() {
        use super::*;
        let ond = Operand::new();
        assert_eq!(ond.stack.len(), 0);
        assert_eq!(ond.coordinate_stack.len(), 0);
        assert_eq!(ond.coord.0, 0.);
        assert_eq!(ond.coord.1, 0.);
        assert_eq!(ond.coord.2, 0.);
        assert_eq!(ond.coord.3, 0.);
    }
}
