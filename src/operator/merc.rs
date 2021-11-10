//! Mercator

use super::OperatorArgs;
use super::OperatorCore;
use crate::operator_construction::*;
use crate::Context;
use crate::CoordinateTuple;
use crate::Ellipsoid;
use crate::GeodesyError;

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
    pub fn new(args: &mut OperatorArgs) -> Result<Merc, GeodesyError> {
        let ellps = Ellipsoid::named(&args.value("ellps", "GRS80"))?;
        let inverted = args.flag("inv");
        let lat_ts = args.numeric_value("lat_ts", f64::NAN)?;
        let k_0 = if lat_ts.is_nan() {
            args.numeric_value("k_0", 1.)?
        } else {
            if lat_ts.abs() > 90. {
                return Err(GeodesyError::General(
                    "Merc: Invalid value for lat_ts: |lat_ts| should be <= 90°",
                ));
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

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, GeodesyError> {
        let op = crate::operator::merc::Merc::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

// #[allow(non_snake_case)]
impl OperatorCore for Merc {
    // Forward mercator, following the PROJ implementation,
    // cf.  https://proj.org/operations/projections/merc.html
    fn fwd(&self, _ctx: &Context, operands: &mut [CoordinateTuple]) -> bool {
        let a = self.ellps.semimajor_axis();
        for coord in operands {
            // Easting
            coord[0] = (coord[0] - self.lon_0) * self.k_0 * a - self.x_0;
            // Northing - basically the isometric latitude multiplied by a
            let lat = coord[1] + self.lat_0;
            coord[1] = a * self.k_0 * self.ellps.isometric_latitude(lat, crate::FWD) - self.y_0;
        }
        true
    }

    fn inv(&self, _ctx: &Context, operands: &mut [CoordinateTuple]) -> bool {
        let a = self.ellps.semimajor_axis();
        for coord in operands {
            // Easting -> Longitude
            let x = coord[0] + self.x_0;
            coord[0] = x / (a * self.k_0) - self.lon_0;

            // Northing -> Latitude
            let y = coord[1] + self.y_0;
            // The isometric latitude
            let psi = y / (a * self.k_0);
            coord[1] = self.ellps.isometric_latitude(psi, crate::INV) - self.lat_0;
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

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::CoordinateTuple as C;

    /// Basic test of the Mercator implementation
    #[test]
    fn merc() {
        let op = "merc";

        // Validation value from PROJ: echo 12 55 0 0 | cct -d18 +proj=merc
        // followed by quadrant tests from PROJ builtins.gie
        let mut operands = [
            C::geo(55., 12., 0., 0.),
            C::geo(1., 2., 0., 0.),
            C::geo(-1., 2., 0., 0.),
            C::geo(1., -2., 0., 0.),
            C::geo(-1., -2., 0., 0.),
        ];

        let mut results = [
            C::raw(1335833.889519282850, 7326837.714873877354, 0., 0.),
            C::raw(222638.981586547, 110579.965218249, 0., 0.),
            C::raw(222638.981586547, -110579.965218249, 0., 0.),
            C::raw(-222638.981586547, 110579.965218249, 0., 0.),
            C::raw(-222638.981586547, -110579.965218249, 0., 0.),
        ];

        assert!(Context::test(
            op,
            0,
            20e-9,
            0,
            10e-9,
            &mut operands,
            &mut results
        ));
    }

    /// Test the "latitude of true scale" functionality
    #[test]
    fn lat_ts() {
        let op = "merc lat_ts:55";

        // Validation values from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=merc +lat_ts=55
        // echo 15 45 0 0 | cct -d18 +proj=merc +lat_ts=55
        let mut operands = [
            C::geo(55., 12., 0., 0.),
            C::geo(-55., 12., 0., 0.),
            C::geo(45., 15., 0., 0.),
        ];

        let mut results = [
            C::raw(767929.5515811865916, 4211972.1958214361221, 0., 0.),
            C::raw(767929.5515811865916, -4211972.1958214361221, 0., 0.),
            C::raw(959911.9394764832687, 3214262.9417223907076, 0., 0.),
        ];

        assert!(Context::test(
            op,
            0,
            20e-9,
            0,
            10e-9,
            &mut operands,
            &mut results
        ));
    }
}
