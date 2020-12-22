use super::OperatorArgs;
use super::OperatorCore;
use super::OperatorWorkSpace;

pub struct BadValue {
}

impl BadValue {
    pub fn new(_args: &mut OperatorArgs) -> BadValue {
        BadValue{}
    }
}

impl OperatorCore for BadValue {
    fn fwd(&self, _ws: &mut OperatorWorkSpace) -> bool {
        true
    }

    fn inv(&self, _ws: &mut OperatorWorkSpace) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "badvalue"
    }

    fn is_inverted(&self) -> bool {
        false
    }

    fn is_badvalue(&self) -> bool {
        true
    }
}


#[cfg(test)]
mod tests {
    use crate::operators::operator_factory;

    #[test]
    fn badvalue() {
        use super::*;
        let mut o = OperatorWorkSpace::new();
        let mut args = OperatorArgs::new();
        let c = operator_factory("badvalue", &mut args);

        // Make sure we do not do anything
        c.fwd(&mut o);
        assert_eq!(o.coord.0, 0.0);
        assert_eq!(o.coord.1, 0.0);
        assert_eq!(o.coord.2, 0.0);
        c.inv(&mut o);
        assert_eq!(o.coord.0, 0.0);
        assert_eq!(o.coord.1, 0.0);
        assert_eq!(o.coord.2, 0.0);

        // Make sure we say what we are
        assert!(c.is_badvalue());
        assert!(c.name() == "badvalue");
    }
}
