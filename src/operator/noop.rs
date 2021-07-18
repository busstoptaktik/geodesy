use super::Context;
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
    fn fwd(&self, _ws: &mut Context) -> bool {
        true
    }

    fn inv(&self, _ws: &mut Context) -> bool {
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
    #[test]
    fn noop() {
        use crate::*;
        let mut o = Context::new();
        let c = Operator::new("noop: {}", None).unwrap();

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
