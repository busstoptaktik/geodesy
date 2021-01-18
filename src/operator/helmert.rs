//! For now, Helmert only supports the basic 3-parameter version

use super::Operand;
use super::OperatorArgs;
use super::OperatorCore;

pub struct Helmert {
    dx: f64,
    dy: f64,
    dz: f64,
    inverted: bool,
    args: OperatorArgs,
}

impl Helmert {
    pub fn new(args: &mut OperatorArgs) -> Result<Helmert, String> {
        Ok(Helmert {
            dx: args.numeric_value("Helmert", "dx", 0.0)?,
            dy: args.numeric_value("Helmert", "dy", 0.0)?,
            dz: args.numeric_value("Helmert", "dz", 0.0)?,
            inverted: args.flag("inv"),
            args: args.clone(),
        })
    }
}

impl OperatorCore for Helmert {
    fn fwd(&self, ws: &mut Operand) -> bool {
        ws.coord.0 += self.dx;
        ws.coord.1 += self.dy;
        ws.coord.2 += self.dz;
        true
    }

    fn inv(&self, ws: &mut Operand) -> bool {
        ws.coord.0 -= self.dx;
        ws.coord.1 -= self.dy;
        ws.coord.2 -= self.dz;
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
        let mut o = Operand::new();
        let mut args = OperatorArgs::new();

        // Check that non-numeric value, for key expecting numeric, errs properly.
        args.name("helmert");
        args.insert("dx", "foo"); // Bad value here.
        args.insert("dy", "-96");
        args.insert("dz", "-120");

        let h = operator_factory(&mut args);
        assert!(h.is_err());

        // EPSG:1134 - 3 parameter, ED50/WGS84, s = sqrt(27) m
        args.insert("dx", "-87");
        assert_eq!(args.value("dx", ""), "-87");
        assert_eq!(args.value("dy", ""), "-96");
        assert_eq!(args.value("dz", ""), "-120");

        let h = operator_factory(&mut args).unwrap();

        h.fwd(&mut o);
        assert_eq!(o.coord.first(), -87.);
        assert_eq!(o.coord.second(), -96.);
        assert_eq!(o.coord.third(), -120.);

        h.inv(&mut o);
        assert_eq!(o.coord.first(), 0.);
        assert_eq!(o.coord.second(), 0.);
        assert_eq!(o.coord.third(), 0.);
    }
}
