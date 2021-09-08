//! Mercator

use super::OperatorArgs;
use super::OperatorCore;
use crate::operator_construction::*;
use crate::Context;
use crate::CoordinateTuple;
use crate::Ellipsoid;

#[derive(Debug)]
pub struct Merc {
    ellps: Ellipsoid,
    inverted: bool,
    k_0: f64,
    lon_0: f64,
    lat_0: f64,
    x_0: f64,
    y_0: f64,
    args: OperatorArgs,
}

impl Merc {
    pub fn new(args: &mut OperatorArgs) -> Result<Merc, &'static str> {
        let ellps = Ellipsoid::named(&args.value("ellps", "GRS80"));
        let inverted = args.flag("inv");
        let lat_ts = args.numeric_value("lat_ts", f64::NAN)?;
        let k_0 = if lat_ts.is_nan() {
            args.numeric_value("k_0", 1.)?
        } else {
            if lat_ts.abs() > 90. {
                return Err("Invalid value for lat_ts: |lat_ts| should be <= 90°");
            }
            let sc = lat_ts.to_radians().sin_cos();
            sc.1 / (1. - ellps.eccentricity_squared() * sc.0 * sc.0).sqrt()
        };
        let lon_0 = args.numeric_value("lon_0", 0.)?.to_radians();
        let lat_0 = args.numeric_value("lat_0", 0.)?.to_radians();
        let x_0 = args.numeric_value("x_0", 0.)?;
        let y_0 = args.numeric_value("y_0", 0.)?;
        let args = args.clone();
        Ok(Merc {
            ellps,
            inverted,
            k_0,
            lon_0,
            lat_0,
            x_0,
            y_0,
            args,
        })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, &'static str> {
        let op = crate::operator::merc::Merc::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

// #[allow(non_snake_case)]
impl OperatorCore for Merc {
    // Forward mercator, following the PROJ implementation,
    // cf.  https://proj.org/operations/projections/merc.html
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        let a = self.ellps.semimajor_axis();
        let e = self.ellps.eccentricity();
        for coord in operands {
            // Easting
            coord[0] = (coord[0] - self.lon_0) * self.k_0 * a - self.x_0;
            // Northing - basically the isometric latitude multiplied by a
            let lat = coord[1] + self.lat_0;
            let sc = lat.sin_cos();
            coord[1] = a * self.k_0 * ((sc.0 / sc.1).asinh() - e * (e * sc.0).atanh()) - self.y_0;
        }
        true
    }

    fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        let a = self.ellps.semimajor_axis();
        let e = self.ellps.eccentricity();

        for coord in operands {
            // Longitude
            let x = coord[0] + self.x_0;
            coord[0] = x / (a * self.k_0) - self.lon_0;

            // Latitude
            let y = coord[1] + self.y_0;
            // The isometric latitude
            let psi = y / (a * self.k_0);
            coord[1] = sinhpsi_to_tanphi(psi.sinh(), e).atan() - self.lat_0;
        }
        true
    }

    fn name(&self) -> &'static str {
        "merc"
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }

    fn args(&self, _step: usize) -> &OperatorArgs {
        &self.args
    }
}

// This follows Karney, 2011, and the PROJ implementation at
// https://github.com/OSGeo/PROJ/blob/e3d7e18f988230973ced5163fa2581b6671c8755/src/phi2.cpp#L10-L108
// TODO: Should be the inverse mode of ellipsoid.isometric_latitude()
fn sinhpsi_to_tanphi(taup: f64, e: f64) -> f64 {
    // min iterations = 1, max iterations = 2; mean = 1.954
    const NUMIT: usize = 5;
    // Currently, Rust cannot const-evaluate powers and roots.
    // Could use lazy_static or wait for evolution
    let /*const*/ rooteps: f64 = f64::EPSILON.sqrt();
    let /*const*/ tol: f64 = rooteps / 10.; // the criterion for Newton's method
    let /*const*/ tmax: f64 = 2. / rooteps; // threshold for large arg limit exact
    let e2m = 1. - e * e;
    let stol = tol * taup.abs().max(1.0);

    // The initial guess.  70 corresponds to chi = 89.18 deg
    let mut tau = if taup.abs() > 70. {
        taup * (e * e.atanh()).exp()
    } else {
        taup / e2m
    };

    // Handle +/-inf, nan, and e = 1
    if !(tau.abs() < tmax) {
        return tau;
    }

    // If we need to deal with e > 1, then we could include:
    // if (e2m < 0) return std::numeric_limits<double>::quiet_NaN();
    let mut i = NUMIT;
    while i > 0 {
        let tau1 = (1. + tau * tau).sqrt();
        let sig = (e * (e * tau / tau1).atanh()).sinh();
        let taupa = (1. + sig * sig).sqrt() * tau - sig * tau1;
        let dtau =
            (taup - taupa) * (1. + e2m * (tau * tau)) / (e2m * tau1 * (1. + taupa * taupa).sqrt());
        tau += dtau;
        i -= 1;

        // backwards test to allow nans to succeed.
        if !(dtau.abs() >= stol) {
            return tau;
        }
    }
    return f64::NAN;
}

#[cfg(test)]
mod tests {
    #[test]
    fn merc() {
        use crate::operator_construction::*;
        use crate::{Context, CoordinateTuple};

        let mut ctx = Context::new();

        // Test the Mercator implementation
        let op = Operator::new("merc", &mut ctx).unwrap();

        let geo = CoordinateTuple::geo(55., 12., 100., 0.);
        let mut operands = [geo];

        // Validation value from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=merc
        let merc_proj = CoordinateTuple::raw(
            1335833.889519282849505544,
            7326837.714873877353966236,
            100.,
            0.,
        );
        assert!(op.fwd(&mut ctx, operands.as_mut()));
        assert!(operands[0].hypot2(&merc_proj) < 1e-5);

        // Roundtrip...
        assert!(op.inv(&mut ctx, operands.as_mut()));
        println!("{:?}", operands[0].to_geo());
        assert!(geo.default_ellps_dist(&operands[0]) < 1e-10);
    }
}
