use super::Operand;
use super::OperatorArgs;
use super::OperatorCore;

pub struct Noop {
    args: OperatorArgs,
}

impl Noop {
    pub fn new(args: &mut OperatorArgs) -> Result<Noop, String> {
        Ok(Noop { args: args.clone() })
    }
}

impl OperatorCore for Noop {
    fn fwd(&self, _ws: &mut Operand) -> bool {
        true
    }

    fn inv(&self, _ws: &mut Operand) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "noop"
    }

    fn is_inverted(&self) -> bool {
        false
    }

    fn args(&self, _step: usize) -> &OperatorArgs {
        &self.args
    }
}

#[cfg(test)]
mod tests {
    use crate::operators::operator_factory;

    #[test]
    fn noop() {
        use super::*;
        let mut o = Operand::new();
        let mut args = OperatorArgs::new();
        args.name("noop");
        let c = operator_factory(&mut args).unwrap();

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
        assert!(c.name() == "noop");
    }
}
