//! Transverse Mercator

// Renovering af Poder/Engsager tmerc i B:\2019\Projects\FIRE\tramp\tramp\tramp.c
// Detaljer i C:\Users\B004330\Downloads\2.1.2 A HIGHLY ACCURATE WORLD WIDE ALGORITHM FOR THE TRANSVE (1).doc

use crate::CoordinateTuple;
use crate::Ellipsoid;
use crate::GeodesyError;
use crate::GysResource;
use crate::Operator;
use crate::OperatorCore;
use crate::Provider;

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
    args: Vec<(String, String)>,
}

impl Tmerc {
    pub fn new(res: &GysResource) -> Result<Tmerc, GeodesyError> {
        let mut args = res.to_args(0)?;
        let inverted = args.flag("inv");
        let ellpsname = args.value("ellps")?.unwrap_or_default();
        let ellps = Ellipsoid::named(&ellpsname)?;

        let k_0 = args.numeric("k_0", 1.)?;
        let lon_0 = args.numeric("lon_0", 0.)?.to_radians();
        let lat_0 = args.numeric("lat_0", 0.)?.to_radians();
        let x_0 = args.numeric("x_0", 0.)?;
        let y_0 = args.numeric("y_0", 0.)?;
        let eps = ellps.second_eccentricity_squared();
        let args = args.used;
        Ok(Tmerc {
            ellps,
            inverted,
            eps,
            k_0,
            lon_0,
            lat_0,
            x_0,
            y_0,
            args,
        })
    }

    pub(crate) fn operator(
        args: &GysResource,
        _rp: &dyn Provider,
    ) -> Result<Operator, GeodesyError> {
        let op = crate::operator::tmerc::Tmerc::new(args)?;
        Ok(Operator(Box::new(op)))
    }

    pub(crate) fn utmoperator(
        args: &GysResource,
        _rp: &dyn Provider,
    ) -> Result<Operator, GeodesyError> {
        let op = crate::operator::tmerc::Tmerc::utm(args)?;
        Ok(Operator(Box::new(op)))
    }

    pub fn utm(res: &GysResource) -> Result<Tmerc, GeodesyError> {
        let mut args = res.to_args(0)?;
        let inverted = args.flag("inv");
        let ellpsname = args.string("ellps", "");

        let ellps = Ellipsoid::named(&ellpsname)?;
        let zone = args.numeric("zone", f64::NAN)?;
        if zone.is_nan() {
            return Err(GeodesyError::General("UTM: Bad or missing 'zone'"));
        }
        let izone = zone as i64;
        if zone != izone as f64 {
            return Err(GeodesyError::General(
                "UTM: 'zone' must be an integer in the interval 1..60",
            ));
        }
        if izone < 1 || izone > 60 {
            return Err(GeodesyError::General(
                "UTM: 'zone' must be in the interval 1..60",
            ));
        }

        let k_0 = 0.9996;
        let lon_0 = (-183. + 6. * zone).to_radians();
        let lat_0 = 0.;
        let x_0 = 500_000.;
        let y_0 = 0.;
        let eps = ellps.second_eccentricity_squared();
        let args = args.used;

        Ok(Tmerc {
            ellps,
            inverted,
            eps,
            k_0,
            lon_0,
            lat_0,
            x_0,
            y_0,
            args,
        })
    }
}

#[allow(non_snake_case)]
impl OperatorCore for Tmerc {
    // Forward transverse mercator, following Bowring (1989)
    fn fwd(&self, _ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            let lat = coord[1] + self.lat_0;
            let c = lat.cos();
            let s = lat.sin();
            let cc = c * c;
            let ss = s * s;

            let dlon = coord[0] - self.lon_0;
            let oo = dlon * dlon;

            let N = self.ellps.prime_vertical_radius_of_curvature(lat);
            let z = self.eps * dlon.powi(3) * c.powi(5) / 6.;
            let sd2 = (dlon / 2.).sin();

            let theta_2 = (2. * s * c * sd2 * sd2).atan2(ss + cc * dlon.cos());

            // Easting
            let sd = dlon.sin();
            coord[0] = self.x_0
                + self.k_0 * N * ((c * sd).atanh() + z * (1. + oo * (36. * cc - 29.) / 10.));

            // Northing
            let m = self.ellps.meridional_distance(lat, true);
            let znos4 = z * N * dlon * s / 4.;
            let ecc = 4. * self.eps * cc;
            coord[1] = self.y_0
                + self.k_0 * (m + N * theta_2 + znos4 * (9. + ecc + oo * (20. * cc - 11.)));
        }
        true
    }

    // Inverse transverse mercator, following Bowring (1989)
    fn inv(&self, _ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        // Footpoint latitude, i.e. the latitude of a point on the central meridian
        // having the same northing as the point of interest
        for coord in operands {
            let lat = self
                .ellps
                .meridional_distance((coord[1] - self.y_0) / self.k_0, false);
            let t = lat.tan();
            let c = lat.cos();
            let cc = c * c;
            let N = self.ellps.prime_vertical_radius_of_curvature(lat);
            let x = (coord[0] - self.x_0) / (self.k_0 * N);
            let xx = x * x;
            let theta_4 = x.sinh().atan2(c);
            let theta_5 = (t * theta_4.cos()).atan();

            // Latitude
            let xet = xx * xx * self.eps * t / 24.;
            coord[1] = self.lat_0 + (1. + cc * self.eps) * (theta_5 - xet * (9. - 10. * cc))
                - self.eps * cc * lat;

            // Longitude
            let approx = self.lon_0 + theta_4;
            let coef = self.eps / 60. * xx * x * c;
            coord[0] = approx - coef * (10. - 4. * xx / cc + xx * cc);
        }
        true
    }

    fn name(&self) -> &'static str {
        "tmerc"
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
    use super::*;

    #[test]
    fn utm() -> Result<(), GeodesyError> {
        let mut ctx = crate::Plain::default();

        // Test the UTM implementation
        let op = Operator::new("utm zone: 32", &mut ctx)?;

        let geo = CoordinateTuple::geo(55., 12., 100., 0.);
        let mut operands = [geo];

        // Validation value from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32
        let utm_proj = CoordinateTuple::raw(691_875.632_139_661, 6_098_907.825_005_012, 100., 0.);
        assert!(op.fwd(&mut ctx, operands.as_mut()));
        assert!(operands[0].hypot2(&utm_proj) < 1e-5);

        // Roundtrip...
        assert!(op.inv(&mut ctx, operands.as_mut()));

        // The latitude roundtrips beautifully, at better than 0.1 mm
        assert!((operands[0][1].to_degrees() - 55.0).abs() * 111_000_000. < 0.05);
        // And the longitude even trumps that by a factor of 10.
        assert!((operands[0][0].to_degrees() - 12.0).abs() * 56_000_000. < 0.005);

        // So also the geodesic distance is smaller than 0.1 mm
        let ellps = Ellipsoid::default();
        assert!(ellps.distance(&operands[0], &geo) < 1e-4);

        // Test a Greenland extreme value (a zone 19 point projected in zone 24)
        let op = Operator::new("utm zone: 24", &mut ctx).unwrap();
        let geo = CoordinateTuple::geo(80., -72., 100., 0.);
        let mut operands = [geo];

        // Roundtrip...
        op.fwd(&mut ctx, operands.as_mut());
        op.inv(&mut ctx, operands.as_mut());
        assert!(ellps.distance(&operands[0], &geo) < 1.05);

        let result = operands[0].to_degrees();
        assert!((result[1] - 80.0).abs() * 111_000. < 1.02);
        assert!((result[0] + 72.0).abs() * 20_000. < 0.04);

        // i.e. Bowring's verion is **much** better than Snyder's:
        // echo -72 80 0 0 | cct +proj=utm +approx +zone=24 +ellps=GRS80 | cct -I +proj=utm +approx +zone=24 +ellps=GRS80
        // -71.9066920547   80.0022281660        0.0000        0.0000
        //
        // But obviously much worse than Poder/Engsager's:
        // echo -72 80 0 0 | cct +proj=utm +zone=24 +ellps=GRS80 | cct -I +proj=utm +zone=24 +ellps=GRS80
        // -72.0000000022   80.0000000001        0.0000        0.0000
        Ok(())
    }

    #[test]
    fn tmerc() -> Result<(), GeodesyError> {
        let mut ctx = crate::Plain::default();

        // Test the plain tmerc, by reimplementing the UTM above manually
        let op = Operator::new("tmerc k_0: 0.9996 lon_0: 9 x_0: 500000", &mut ctx)?;

        let mut operands = [CoordinateTuple::gis(12., 55., 100., 0.)];
        assert!(op.fwd(&mut ctx, operands.as_mut()));

        // Validation value from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32
        let utm_proj = CoordinateTuple::raw(691_875.632_139_661, 6_098_907.825_005_012, 100., 0.);
        assert!(operands[0].hypot2(&utm_proj) < 1e-5);
        Ok(())
    }
}
