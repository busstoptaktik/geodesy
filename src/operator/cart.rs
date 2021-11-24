use crate::CoordinateTuple;
use crate::Ellipsoid;
use crate::GeodesyError;
use crate::GysResource;
use crate::Operator;
use crate::OperatorCore;
use crate::Provider;
#[derive(Debug)]
pub struct Cart {
    // The usual suspects...
    args: Vec<(String, String)>,
    inverted: bool,
    ellps: Ellipsoid,

    // We precompute a number of ancillary ellipsoidal parameters
    // to speed up the computations
    es: f64,  // eccentricity squared, Fukushima's E, Claessens' c3 = 1-c2
    b: f64,   // semiminor axis
    ra: f64,  // reciproque of a
    ar: f64,  // aspect ratio, b/a: Fukushima's ec, Claessens' c4
    ce4: f64, // 1.5 times the fourth power of the eccentricity

    cutoff: f64, // if we're closer than this to the Z axis, we force latitude to one of the poles
}

impl Cart {
    pub fn new(res: &GysResource) -> Result<Cart, GeodesyError> {
        let mut args = res.to_args(0)?;
        let ellps = Ellipsoid::named(&args.string("ellps", "GRS80"))?;

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
            args: args.used,
            inverted,
            ellps,
            es,
            b,
            ra,
            ar,
            ce4,
            cutoff,
        })
    }

    pub(crate) fn operator(
        args: &GysResource,
        _rp: &dyn Provider,
    ) -> Result<Operator, GeodesyError> {
        let op = crate::operator::cart::Cart::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for Cart {
    // For now, we just use the shrinkwrapped Ellipsoid-method in
    // fwd() and an optimized version of Fukushima (2006) in inv().
    // We should, however, switch to Bowring (1985).
    fn fwd(&self, _ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            *coord = self.ellps.cartesian(coord);
        }
        true
    }

    #[allow(non_snake_case)] // make it possible to mimic math notation from original paper
    #[allow(clippy::many_single_char_names)] // ditto
    #[allow(clippy::suspicious_operation_groupings)]
    fn inv(&self, _ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        let a = self.ellps.semimajor_axis();
        let es = self.es;
        let b = self.b;
        let ra = self.ra;
        let ar = self.ar;
        let ce4 = self.ce4;

        for coord in operands {
            let X = coord[0];
            let Y = coord[1];
            let Z = coord[2];
            let t = coord[3];

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
                *coord = CoordinateTuple::raw(lam, phi, h, t);
                continue;
            }

            let P = ra * p;
            let S0 = ra * Z;
            let C0 = ar * P;

            // There's a lot of common subexpressions in the following which,
            // in Fukushima's and Claessens' Fortranesque implementations,
            // were explicitly eliminated (by introducing s02 = S0*S0, etc.).
            // For clarity, we keep the full expressions here, and leave the
            // elimination task to the optimizer.
            let A = S0.hypot(C0);
            let F = P * A * A * A - es * C0 * C0 * C0;
            let B = ce4 * S0 * S0 * C0 * C0 * P * (A - ar);

            let S1 = (ar * S0 * A * A * A + es * S0 * S0 * S0) * F - B * S0;
            let C1 = F * F - B * C0;
            let CC = ar * C1;

            let phi = S1.atan2(CC);
            let h = (p * CC + Z.abs() * S1 - a * CC.hypot(ar * S1)) / CC.hypot(S1);
            // Bowring's height formula works better close to the ellipsoid, but requires a (sin, cos)-pair
            *coord = CoordinateTuple::raw(lam, phi, h, t);
        }
        true
    }

    fn name(&self) -> &'static str {
        "cart"
    }

    fn debug(&self) -> String {
        format!("{:#?}", self)
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }

    fn args(&self, _step: usize) -> &[(String, String)] {
        &self.args
    }
}

// --------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::CoordinateTuple as C;
    #[test]
    fn cart() {
        let mut rp = crate::Plain::default();
        let op = "cart";
        let mut operands = [
            C::geo(85., 0., 100000., 0.),
            C::geo(55., 10., -100000., 0.),
            C::geo(25., 20., 0., 0.),
            C::geo(0., -20., 0., 0.),
            C::geo(-25., 20., 10., 0.),
        ];

        let mut results = [
            C::raw(566462.633537476765923, 0., 6432020.33369012735784, 0.),
            C::raw(
                3554403.47587193036451,
                626737.23312017065473,
                5119468.31865925621241,
                0.,
            ),
            C::raw(
                5435195.38214521575719,
                1978249.33652197546325,
                2679074.46287727775052,
                0.,
            ),
            C::raw(5993488.27326157130301, -2181451.33089075051248, 0., 0.),
            C::raw(
                5435203.89865261223167,
                1978252.43627716740593,
                -2679078.68905989499763,
                0.,
            ),
        ];

        assert!(crate::resource::test(
            &mut rp,
            op,
            3,
            20e-9,
            0,
            10e-9,
            &mut operands,
            &mut results
        ));
    }
}
