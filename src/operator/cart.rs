use super::Context;
use super::OperatorArgs;
use super::OperatorCore;
use crate::CoordinateTuple;
use crate::Ellipsoid;

pub struct Cart {
    // The usual suspects...
    args: OperatorArgs,
    inverted: bool,
    ellps: Ellipsoid,

    // We precompute a number of anciallary ellipsoidal parameters
    // to speed up the computations
    es: f64,  // eccentricity squared, Fukushima's E, Claessens' c3 = 1-c2
    b: f64,   // semiminor axis
    ra: f64,  // reciproque of a
    ar: f64,  // aspect ratio, b/a: Fukushima's ec, Claessens' c4
    ce4: f64, // 1.5 times the fourth power of the eccentricity

    cutoff: f64, // if we're closer than this to the Z axis, we force latitude to one of the poles
}

impl Cart {
    pub fn new(args: &mut OperatorArgs) -> Result<Cart, String> {
        let ellps = Ellipsoid::named(&args.value("ellps", "GRS80"));

        let es = ellps.eccentricity_squared();
        let b = ellps.semiminor_axis();
        let ra = 1. / ellps.semimajor_axis();
        let ar = b * ra;
        let ce4 = 1.5 * es * es;

        let cutoff = ellps.semimajor_axis() * 1e-16;

        // We must finish accessing flags before cloning - otherwise the
        // usage information in the cloned args will not be correct.
        let inverted = args.flag("inv");

        Ok(Cart {
            args: args.clone(),
            inverted: inverted,
            ellps: ellps,
            es: es,
            b: b,
            ra: ra,
            ar: ar,
            ce4: ce4,

            cutoff: cutoff,
        })
    }
}

impl OperatorCore for Cart {
    // For now, we just use the shrinkwrapped Ellipsoid-methods in
    // fwd() and inv(). But we can potentially speed up by
    // extending struct Cart with additional precomputed ellipsoidal
    // parameters, so we don't need to let Ellipsoid:: compute them
    // over and over on each invocation.
    fn fwd(&self, operand: &mut Context) -> bool {
        operand.coord = self.ellps.cartesian(&operand.coord);
        true
    }

    #[allow(non_snake_case)] // make it possible to mimic math notation from original paper
    #[allow(clippy::many_single_char_names)] // ditto
    fn inv(&self, operand: &mut Context) -> bool {
        let X = operand.coord.first();
        let Y = operand.coord.second();
        let Z = operand.coord.third();
        let t = operand.coord.fourth();

        let a = self.ellps.semimajor_axis();
        let es = self.es;
        let b = self.b;
        let ra = self.ra;
        let ar = self.ar;
        let ce4 = self.ce4;

        // The longitude is straightforward
        let lam = Y.atan2(X);

        // The perpendicular distance from the point coordinate to the Z-axis (HM eq. 5-28)
        let p = X.hypot(Y);

        // If we're close to the Z-axis, the full algorithm breaks down. But if
        // we're close to the Z-axis, we also know that the latitude must be close
        // to one of the poles. So we force the latitude to the relevant pole and
        // compute the height as |Z| - b
        if p < self.cutoff {
            let phi = std::f64::consts::FRAC_PI_2.copysign(Z);
            let h = Z.abs() - b;
            operand.coord = CoordinateTuple::new(lam, phi, h, t);
            return true;
        }

        let P = ra * p;
        let S0 = ra * Z;
        let C0 = ar * P;

        // There's a lot of common subexpressions in the following which,
        // in Fukushima's and Claessens' Fortranesque implementations,
        // were explicitly eliminated (by introducing s02 = S0*S0, etc.).
        // For clarity, we keep the full expressions here, and leave the
        // elimination task to the Rust optimizer.
        let A = S0.hypot(C0);
        let F = P * A * A * A - es * C0 * C0 * C0;
        let B = ce4 * S0 * S0 * C0 * C0 * P * (A - ar);

        let S1 = (ar * S0 * A * A * A + es * S0 * S0 * S0) * F - B * S0;
        let C1 = F * F - B * C0;
        let CC = ar * C1;

        let phi = S1.atan2(CC);
        let h = (p * CC + Z.abs() * S1 - a * CC.hypot(ar * S1)) / CC.hypot(S1);
        // Bowring's height formula works better close to the ellipsoid, but requires a (sin, cos)-pair
        operand.coord = CoordinateTuple::new(lam, phi, h, t);
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
    #[test]
    fn cart() {
        use crate::*;
        let c = Operator::new("cart: {ellps: intl}").unwrap();
        let mut o = Context::new();

        // First check that (0,0,0) takes us to (a,0,0)
        c.fwd(&mut o);
        let a = Ellipsoid::named("intl").semimajor_axis();
        assert_eq!(o.coord.0, a);
        assert_eq!(o.coord.1, 0.0);
        assert_eq!(o.coord.1, 0.0);

        // Some arbitrary spot - southwest of Copenhagen
        o.coord = CoordinateTuple::deg(12., 55., 100., 0.);

        // Roundtrip
        c.fwd(&mut o);
        c.inv(&mut o);
        let result = o.coord.to_degrees();

        // And check that we're back
        assert!((result.0 - 12.).abs() < 1.0e-10);
        assert!((result.1 - 55.).abs() < 1.0e-10);
        assert!((result.2 - 100.).abs() < 1.0e-8);
    }
}
