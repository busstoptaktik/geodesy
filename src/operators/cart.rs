use crate::OperatorArgs;
use crate::OperatorCore;
use crate::OperatorWorkSpace;

pub struct Cart {
    a: f64,
    b: f64,
    f: f64,
    eps: f64,  // Second eccentricity squared
    es: f64,   // Eccentricity squared

    dx: f64,
    dy: f64,
    dz: f64,
    inverted: bool,
}

impl Cart {
    pub fn new(args: &mut OperatorArgs) -> Cart {
        Cart {
            a: 0.0,
            b: 0.0,
            f: 0.0,
            eps: 0.0,
            es: 0.0,
            dx: args.numeric_value("dx", 0.0),
            dy: args.numeric_value("dy", 0.0),
            dz: args.numeric_value("dz", 0.0),
            inverted: args.boolean_value("inv"),
        }
    }
}

impl OperatorCore for Cart {
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
        "Cartesian"
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }

}
