use super::OperatorArgs;
use super::OperatorCore;
use super::OperatorWorkSpace;

pub struct Noop {
}

impl Noop {
    pub fn new(_args: &mut OperatorArgs) -> Noop {
        Noop{}
    }
}

impl OperatorCore for Noop {
    fn fwd(&self, _ws: &mut OperatorWorkSpace) -> bool {
        true
    }

    fn inv(&self, _ws: &mut OperatorWorkSpace) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "noop"
    }

    fn is_inverted(&self) -> bool {
        false
    }

    fn is_noop(&self) -> bool {
        true
    }
}


#[cfg(test)]
mod tests {
    use crate::operators::operator_factory;

    #[test]
    fn noop() {
        use super::*;
        let mut o = OperatorWorkSpace::new();
        let mut args = OperatorArgs::new();
        let c = operator_factory("noop", &mut args);

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
        assert!(c.is_noop());
        assert!(c.name() == "noop");
    }
}
