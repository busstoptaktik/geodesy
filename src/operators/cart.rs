use crate::OperatorArgs;
use crate::OperatorCore;
use crate::OperatorWorkSpace;

// ----------------- CART -------------------------------------------------
pub struct Cart {
    dx: f64,
    dy: f64,
    dz: f64,
}

impl Cart {
    pub fn new(args: &mut OperatorArgs) -> Cart {
        let dx = args.numeric_value("dx", 0.0);
        let dy = args.numeric_value("dy", 0.0);
        let dz = args.numeric_value("dz", 0.0);
        let cart = Cart {
            dx: dx,
            dy: dy,
            dz: dz,
        };
        return cart;
    }
}

impl OperatorCore for Cart {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> bool {
        ws.coord.0 += self.dx;
        ws.coord.1 += self.dy;
        ws.coord.2 += self.dz;
        return true;
    }
    fn inv(&self, ws: &mut OperatorWorkSpace) -> bool {
        ws.coord.0 -= self.dx;
        ws.coord.1 -= self.dy;
        ws.coord.2 -= self.dz;
        return true;
    }
    fn name(&self) -> &'static str {
        return "CART";
    }
}
