//! For now, Helmert only supports the basic 3-parameter version

use super::Context;
use super::OperatorArgs;
use super::OperatorCore;
use crate::CoordinateTuple;
use crate::Operator;

pub struct Helmert {
    dx: f64,
    dy: f64,
    dz: f64,
    inverted: bool,
    args: OperatorArgs,
}

impl Helmert {
    fn new(args: &mut OperatorArgs) -> Result<Helmert, String> {
        let dx = args.numeric_value("Helmert", "dx", 0.0)?;
        let dy = args.numeric_value("Helmert", "dy", 0.0)?;
        let dz = args.numeric_value("Helmert", "dz", 0.0)?;
        let inverted = args.flag("inv");
        let argsc = args.clone();
        Ok(Helmert {
            dx,
            dy,
            dz,
            inverted,
            args: argsc,
        })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, String> {
        let op = crate::operator::helmert::Helmert::new(args)?;
        Ok(Operator { 0: Box::new(op) })
    }
}

impl OperatorCore for Helmert {
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            coord[0] += self.dx;
            coord[1] += self.dy;
            coord[2] += self.dz;
        }
        true
    }

    fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            coord[0] -= self.dx;
            coord[1] -= self.dy;
            coord[2] -= self.dz;
        }
        true
    }

    fn name(&self) -> &'static str {
        "helmert"
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }

    fn args(&self, _step: usize) -> &OperatorArgs {
        &self.args
    }
}

#[cfg(test)]
mod tests {
    use crate::operator::operator_factory;

    #[test]
    fn helmert() {
        use super::*;
        let mut ctx = Context::new();
        let mut args = OperatorArgs::new();

        // Check that non-numeric value, for key expecting numeric, errs properly.
        args.name("helmert");
        args.insert("dx", "foo"); // Bad value here.
        args.insert("dy", "-96");
        args.insert("dz", "-120");

        let h = operator_factory(&mut args, &mut ctx, 0);
        assert!(h.is_err());

        // EPSG:1134 - 3 parameter, ED50/WGS84, s = sqrt(27) m
        args.insert("dx", "-87");
        assert_eq!(args.value("dx", ""), "-87");
        assert_eq!(args.value("dy", ""), "-96");
        assert_eq!(args.value("dz", ""), "-120");

        let h = operator_factory(&mut args, &mut ctx, 0).unwrap();

        let mut operands = [CoordinateTuple::origin()];
        h.fwd(&mut ctx, operands.as_mut());
        assert_eq!(operands[0].first(), -87.);
        assert_eq!(operands[0].second(), -96.);
        assert_eq!(operands[0].third(), -120.);

        h.inv(&mut ctx, operands.as_mut());
        assert_eq!(operands[0].first(), 0.);
        assert_eq!(operands[0].second(), 0.);
        assert_eq!(operands[0].third(), 0.);
    }
}
