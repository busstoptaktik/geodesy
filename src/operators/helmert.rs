use super::OperatorArgs;
use super::OperatorCore;
use super::OperatorWorkSpace;

pub struct Helmert {
    dx: f64,
    dy: f64,
    dz: f64,
    inverted: bool,
}

impl Helmert {
    pub fn new(args: &mut OperatorArgs) -> Helmert {
        Helmert {
            dx: args.numeric_value("dx", 0.0),
            dy: args.numeric_value("dy", 0.0),
            dz: args.numeric_value("dz", 0.0),
            inverted: args.flag("inv"),
        }
    }
}

impl OperatorCore for Helmert {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> bool {
        ws.coord.0 += self.dx;
        ws.coord.1 += self.dy;
        ws.coord.2 += self.dz;
        true
    }

    fn inv(&self, ws: &mut OperatorWorkSpace) -> bool {
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
}

#[cfg(test)]
mod tests {
    use crate::operators::operator_factory;

    #[test]
    fn helmert() {
        use super::*;
        let mut o = OperatorWorkSpace::new();
        let mut args = OperatorArgs::new();
        // EPSG:1134 - 3 parameter, ED50/WGS84, s = sqrt(27) m
        args.insert("dx", "-87");
        args.insert("dy", "-96");
        args.insert("dz", "-120");
        println!("\nargs: {:?}\n", args);
        let h = operator_factory("helmert", &mut args);
        h.fwd(&mut o);
        assert_eq!(o.coord.first(), -87.);

        h.inv(&mut o);
        assert_eq!(o.coord.first(), 0.);
        assert_eq!(o.coord.second(), 0.);
        assert_eq!(o.coord.third(), 0.);
    }
}
