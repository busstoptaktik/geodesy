use super::OperatorArgs;
use super::OperatorCore;
use super::Operand;
use crate::Ellipsoid;

// For now, we just use the shrinkwrapped Ellipsoid-methods, but we can
// potentially speed up by extending struct Cart with additional
// precomputed ellipsoidal parameters.
pub struct Cart {
    ellps: Ellipsoid,
    inverted: bool,
    args: OperatorArgs
}

impl Cart {
    pub fn new(args: &mut OperatorArgs) -> Cart {
        Cart {
            ellps: Ellipsoid::named(&args.value("ellps", "GRS80")),
            inverted: args.flag("inv"),
            args: args.clone(),
        }
    }
}

impl OperatorCore for Cart {
    fn fwd(&self, operand: &mut Operand) -> bool {
        operand.coord = self.ellps.cartesian(&operand.coord);
        true
    }

    fn inv(&self, operand: &mut Operand) -> bool {
        operand.coord = self.ellps.geographic(&operand.coord);
        true
    }

    fn name(&self) -> &'static str {
        "cart"
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
    use crate::operators::operator_factory;

    #[test]
    fn cart() {
        use super::*;
        let mut o = Operand::new();
        let mut args = OperatorArgs::new();
        args.name("cart");
        args.insert("ellps", "intl");

        let c = operator_factory(&mut args);

        // First check that (0,0,0) takes us to (a,0,0)
        c.fwd(&mut o);
        let a = Ellipsoid::named("intl").semimajor_axis();
        assert_eq!(o.coord.0, a);
        assert_eq!(o.coord.1, 0.0);
        assert_eq!(o.coord.1, 0.0);

        // Some arbitrary spot - southwest of Copenhagen
        o.coord.0 = 12f64.to_radians();
        o.coord.1 = 55f64.to_radians();
        o.coord.2 = 100.0;

        // Roundtrip
        c.fwd(&mut o);
        c.inv(&mut o);

        // And check that we're back
        assert!((o.coord.first().to_degrees() -  12.).abs() < 1.0e-10);
        assert!((o.coord.third() - 100.).abs() < 1.0e-10);
        assert!((o.coord.second().to_degrees() - 55.).abs() < 1.0e-10);
    }
}
