use super::Context;
use super::OperatorArgs;
use super::OperatorCore;
use crate::CoordinateTuple;

pub struct Noop {
    args: OperatorArgs,
}

impl Noop {
    pub fn new(args: &mut OperatorArgs) -> Result<Noop, String> {
        Ok(Noop { args: args.clone() })
    }
}

impl OperatorCore for Noop {
    fn fwd(&self, _ctx: &mut Context, _operands: &mut [CoordinateTuple]) -> bool {
        true
    }

    fn inv(&self, _ctx: &mut Context, _operands: &mut [CoordinateTuple]) -> bool {
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
        use crate::CoordinateTuple;
        use crate::operator::OperatorCore;
        use crate::Context;
        use crate::Operator;
        let mut o = Context::new();
        let c = Operator::new("noop: {}", &mut o).unwrap();

        let mut operands = [CoordinateTuple::origin()];

        // Make sure we do not do anything
        c.fwd(&mut o, operands.as_mut());
        assert_eq!(operands[0][0], 0.0);
        assert_eq!(operands[0][1], 0.0);
        assert_eq!(operands[0][2], 0.0);
        assert_eq!(operands[0][3], 0.0);
        c.inv(&mut o, operands.as_mut());
        assert_eq!(operands[0][0], 0.0);
        assert_eq!(operands[0][1], 0.0);
        assert_eq!(operands[0][2], 0.0);
        assert_eq!(operands[0][3], 0.0);

        // Make sure we say what we are
        assert!(c.name() == "noop");
    }
}
