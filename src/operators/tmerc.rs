//! Transverse Mercator

use super::OperatorArgs;
use super::OperatorCore;
use super::Operand;
use crate::Ellipsoid;

#[derive(Debug)]
pub struct Tmerc {
    ellps: Ellipsoid,
    inverted: bool,
    eps: f64,
    k_0: f64,
    lon_0: f64,
    lat_0: f64,
    x_0: f64,
    y_0: f64,
    args: OperatorArgs,
}

impl Tmerc {
    pub fn new(args: &mut OperatorArgs) -> Result <Tmerc, String> {
        let ellps = Ellipsoid::named(&args.value("ellps", "GRS80"));
        Ok(Tmerc {
            ellps: ellps,
            inverted: args.flag("inv"),
            args: args.clone(),
            k_0: args.numeric_value("Tmerc", "k_0", 1.)?,
            lon_0: args.numeric_value("Tmerc", "lon_0", 0.)?.to_radians(),
            lat_0: args.numeric_value("Tmerc", "lat_0", 0.)?.to_radians(),
            x_0: args.numeric_value("Tmerc", "x_0", 0.)?,
            y_0: args.numeric_value("Tmerc", "y_0", 0.)?,
            eps: ellps.second_eccentricity_squared(),
        })
    }

    pub fn utm(args: &mut OperatorArgs) ->  Result <Tmerc, String> {
        let ellps = Ellipsoid::named(&args.value("ellps", "GRS80"));
        let zone = args.numeric_value("Utm", "zone", f64::NAN)?;
        Ok(Tmerc {
            ellps: ellps,
            inverted: args.flag("inv"),
            args: args.clone(),
            k_0: 0.9996,
            lon_0: (-183. + 6. * zone).to_radians(),
            lat_0: 0.,
            x_0: 500000.,
            y_0: 0.,
            eps: ellps.second_eccentricity_squared(),
        })
    }
}

#[allow(non_snake_case)]
impl OperatorCore for Tmerc {
    // Forward transverse mercator, following Bowring
    fn fwd(&self, operand: &mut Operand) -> bool {
        let lat = operand.coord.1;
        let c = lat.cos();
        let s = lat.sin();
        let cc = c*c;
        let ss = s*s;

        let dlon = operand.coord.0 - self.lon_0;
        let oo = dlon * dlon;

        let N = self.ellps.prime_vertical_radius_of_curvature(lat);
        let z = self.eps * dlon.powi(3) * c.powi(5) / 6.;
        let sd2 = (dlon / 2.).sin();

        let theta_2 = (2. * s * c * sd2 * sd2).atan2(ss + cc * dlon.cos());

        // Easting
        let sd = dlon.sin();
        operand.coord.0 = self.x_0 + self.k_0 * N * (
            (c * sd).atanh() + z * (1. + oo * (36. * cc - 29.) / 10.)
        );

        // Northing
        let m = self.ellps.meridional_distance(lat, true);
        let znos4 = z * N * dlon * s / 4.;
        let ecc = 4.*self.eps*cc;
        operand.coord.1 = self.y_0 + self.k_0 * (
            m + N * theta_2 + znos4 * (9. + ecc + oo * (20. * cc - 11.))
        );

        true
    }

    // Forward transverse mercator, following Bowring (1989)
    fn inv(&self, operand: &mut Operand) -> bool {
        // Footpoint latitude, i.e. the latitude of a point on the central meridian
        // having the same northing as the point of interest
        let lat = self.ellps.meridional_distance((operand.coord.1 - self.y_0)/self.k_0, false);
        let t = lat.tan();
        let c = lat.cos();
        let cc = c * c;
        let N = self.ellps.prime_vertical_radius_of_curvature(lat);
        let x = (operand.coord.0 - self.x_0) / (self.k_0 * N);
        let xx = x * x;
        let theta_4 = x.sinh().atan2(c);
        let theta_5 = (t * theta_4.cos()).atan();

        // Latitude
        let xet = xx*xx * self.eps * t / 24.;
        operand.coord.1 = self.lat_0 + (1. + cc * self.eps)
                        * (theta_5 - xet * (9. - 10. * cc))
                        - self.eps * cc * lat;

        // Longitude
        let approx = self.lon_0 + theta_4;
        let coef = self.eps/60. * xx*x * c;
        operand.coord.0 = approx - coef * (10. - 4.*xx/cc + xx*cc);
        true
    }

    fn name(&self) -> &'static str {
        "tmerc"
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
    fn utm() {
        use super::*;

        // Test the UTM implementation
        let utm = "utm: {zone: 32}";
        let mut args = OperatorArgs::global_defaults();
        args.populate(&utm, "");
        let op = Tmerc::utm(&mut args).unwrap();

        let mut operand = Operand::new();
        operand.coord = crate::CoordinateTuple(12f64.to_radians(), 55f64.to_radians(), 100., 0.);
        op.fwd(&mut operand);

        // Validation value from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32
        assert!((operand.coord.0 - 691875.6321396606508642440).abs() < 1e-5);
        assert!((operand.coord.1 - 6098907.825005011633038521).abs() < 1e-5);

        // Roundtrip...
        op.inv(&mut operand);

        // The latitude roundtrips beautifully, at better than 0.1 mm
        assert!((operand.coord.1.to_degrees() - 55.0).abs()*111_000_000. < 0.05);
        // And the longitude even trumps that by a factor of 10.
        assert!((operand.coord.0.to_degrees() - 12.0).abs()*56_000_000. < 0.005);
    }

    #[test]
    fn tmerc() {
        use super::*;

        // Test the plain tmerc, by reimplementing the UTM above manually
        let tmerc = "tmerc: {k_0: 0.9996, lon_0: 9, x_0: 500000}";
        let mut args = OperatorArgs::global_defaults();
        args.populate(&tmerc, "");
        let op = Tmerc::new(&mut args).unwrap();

        let mut operand = Operand::new();
        operand.coord = crate::CoordinateTuple(12f64.to_radians(), 55f64.to_radians(), 100., 0.);
        op.fwd(&mut operand);

        // Validation value from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32
        assert!((operand.coord.0 - 691875.6321396606508642440).abs() < 1e-5);
        assert!((operand.coord.1 - 6098907.825005011633038521).abs() < 1e-5);

    }
}
