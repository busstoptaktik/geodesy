#![allow(non_snake_case)]
/// The full and abridged Molodensky transformations for 2D and 3D data.
///
/// Partially based on the PROJ implementation by Kristian Evers,
/// partially on OGP Publication 373-7-2 – Geomatics Guidance Note
/// number 7, part 2, and partially on Dea04,
/// R.E.Deakin, 2004: The Standard
/// and Abridged Molodensky Coordinate Transformation Formulae.
/// URL http://www.mygeodesy.id.au/documents/Molodensky%20V2.pdf
use super::OperatorArgs;
use super::OperatorCore;
use crate::operator_construction::*;
use crate::Context;
use crate::CoordinateTuple;
use crate::Ellipsoid;

#[derive(Debug)]
pub struct Molodensky {
    ellps: Ellipsoid,
    inverted: bool,
    abridged: bool,
    dx: f64,
    dy: f64,
    dz: f64,
    da: f64,
    df: f64,
    adffda: f64,
    es: f64,
    args: OperatorArgs,
}

impl Molodensky {
    pub fn new(args: &mut OperatorArgs) -> Result<Molodensky, &'static str> {
        let inverted = args.flag("inv");
        let abridged = args.flag("abridged");
        let dx = args.numeric_value("dx", 0.)?;
        let dy = args.numeric_value("dy", 0.)?;
        let dz = args.numeric_value("dz", 0.)?;

        let mut da = args.numeric_value("da", 0.)?;
        let mut df = args.numeric_value("df", 0.)?;

        // We may use `ellps, da, df`, to parameterize the operator,
        // but `left_ellps, right_ellps` is a more likely set of
        // parameters to come across in real life.
        let mut left_ellps = Ellipsoid::named(&args.value("ellps", "GRS80"));
        if !args.value("left_ellps", "").is_empty() {
            left_ellps = Ellipsoid::named(&args.value("left_ellps", "GRS80"));
        }
        if !args.value("right_ellps", "").is_empty() {
            let right_ellps = Ellipsoid::named(&args.value("right_ellps", "GRS80"));
            da = right_ellps.semimajor_axis() - left_ellps.semimajor_axis();
            df = right_ellps.flattening() - left_ellps.flattening();
        }

        let es = left_ellps.eccentricity_squared();

        // Precompute what little we can
        let adffda = left_ellps.semimajor_axis() * df + left_ellps.flattening() * da;

        let args = args.clone();
        Ok(Molodensky {
            ellps: left_ellps,
            inverted,
            abridged,
            dx,
            dy,
            dz,
            da,
            df,
            adffda,
            es,
            args,
        })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, &'static str> {
        let op = crate::operator::molodensky::Molodensky::new(args)?;
        Ok(Operator(Box::new(op)))
    }

    fn calc_molodensky_params(&self, coord: &CoordinateTuple) -> CoordinateTuple {
        // First abbreviate some much used parameters from `self.par` to just `par`
        let a = self.ellps.semimajor_axis();
        let f = self.ellps.flattening();
        let es = self.es;

        let dx = self.dx;
        let dy = self.dy;
        let dz = self.dz;
        let da = self.da;
        let df = self.df;
        let adffda = self.adffda;

        // Then compute the needed trigonometrical factors
        let lam = coord[0];
        let phi = coord[1];
        let h = coord[2];
        let (slam, clam) = lam.sin_cos();
        let (sphi, cphi) = phi.sin_cos();

        // We also need the radii of curvature
        let N = self.ellps.prime_vertical_radius_of_curvature(phi);
        let M = self.ellps.meridian_radius_of_curvature(phi);

        // Now compute the offsets in the ellipsoidal space
        let fac = dx * clam + dy * slam;

        if self.abridged {
            // delta phi
            let dphi = (-fac * sphi + dz * cphi + adffda * (2.0 * phi).sin()) / M;

            // delta lambda
            let dlam_denom = N * cphi;
            if dlam_denom == 0.0 {
                return CoordinateTuple::nan();
            }
            let dlam = (dy * clam - dx * slam) / dlam_denom;

            // delta h
            let dh = fac * cphi + (dz + adffda * sphi) * sphi - da;
            return CoordinateTuple::raw(dlam, dphi, dh, 0.0);
        }

        // delta phi
        let mut dphi = (dz + ((N * es * sphi * da) / a)) * cphi - fac * sphi
            + (M / (1.0 - f) + N * (1.0 - f)) * df * sphi * cphi;
        let dphi_denom = M + h;
        if dphi_denom == 0.0 {
            return CoordinateTuple::nan();
        }
        dphi /= dphi_denom;

        // delta lambda
        let dlam_denom = (N + h) * cphi;
        if dlam_denom == 0.0 {
            return CoordinateTuple::nan();
        }
        let dlam = (dy * clam - dx * slam) / dlam_denom;

        // delta h
        let dh = fac * cphi + dz * sphi - (a / N) * da + N * (1.0 - f) * df * sphi * sphi;

        CoordinateTuple::raw(dlam, dphi, dh, 0.)
    }
}

impl OperatorCore for Molodensky {
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            let par = self.calc_molodensky_params(coord);
            coord[0] += par[0];
            coord[1] += par[1];
            coord[2] += par[2];
        }
        true
    }

    // Inverse Molodensky
    fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            let par = self.calc_molodensky_params(coord);
            coord[0] -= par[0];
            coord[1] -= par[1];
            coord[2] -= par[2];
        }
        true
    }

    fn name(&self) -> &'static str {
        "Molodensky"
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
    fn molodensky() {
        use super::*;
        use crate::CoordinateTuple as C;
        let mut ctx = Context::new();
        // ---------------------------------------------------------------------------
        // Test case from OGP Publication 373-7-2: Geomatics Guidance Note number 7,
        // part 2: Transformation from WGS84 to ED50.
        // ---------------------------------------------------------------------------

        let definition = "molodensky: {
            left_ellps: WGS84, right_ellps: intl,
            dx: 84.87, dy: 96.49, dz: 116.95, abridged: false
        }";
        let op = ctx.operation(definition).unwrap();

        // Test point (53.80939444444444, 2.12955, 73 m)
        let lat = C::dms_to_dd(53, 48, 33.82);
        let lon = C::dms_to_dd(2, 7, 46.38);
        let WGS84 = CoordinateTuple::geo(lat, lon, 73., 0.0);

        // Commented out test coordinates from EPSG are not of terribly high
        // resolution: 3 decimals on the seconds, corresponding to 30 mm.
        // let lat = C::dms_to_dd(53, 48, 36.563);
        // let lon = C::dms_to_dd(2, 7, 51.477);
        // The values actually used are taken from a direct 3 parameter
        // Helmert calculation with the same constants:
        // echo 53.80939444444444 2.12955 73 | kp ^
        //        "geo | cart WGS84 | helmert  x:84.87 y:96.49 z:116.95 | cart inv ellps:intl | geo inv"
        let lat = 53.8101570592;
        let lon = 2.1309658097;
        let ED50 = CoordinateTuple::geo(lat, lon, 28.02470, 0.0);

        // In the unabridged case, Molodensky replicates Helmert to
        // within 5 mm in the plane and the elevation.
        let mut operands = [WGS84];
        ctx.fwd(op, &mut operands);
        assert!(ED50.default_ellps_dist(&operands[0]) < 0.005);
        assert!((ED50[2] - operands[0][2]).abs() < 0.005);

        // The same holds in the reverse unabridged case, where
        // additionally the elevation is even better
        let mut operands = [ED50];
        ctx.inv(op, &mut operands);
        assert!(WGS84.default_ellps_3d_dist(&operands[0]) < 0.005);
        assert!((WGS84[2] - operands[0][2]).abs() < 0.001);

        // The abridged case. Same test point. Both plane coordinates and
        // elevations are *much* worse, but still better-than-decimeter.
        let definition = "molodensky: {
            left_ellps: WGS84, right_ellps: intl,
            dx: 84.87, dy: 96.49, dz: 116.95, abridged: true
        }";
        let op = ctx.operation(definition).unwrap();

        let mut operands = [WGS84];
        ctx.fwd(op, &mut operands);
        assert!(ED50.default_ellps_dist(&operands[0]) < 0.1);
        // Heights are worse in the abridged case
        assert!((ED50[2] - operands[0][2]).abs() < 0.075);

        let mut operands = [ED50];
        ctx.inv(op, &mut operands);
        assert!(WGS84.default_ellps_dist(&operands[0]) < 0.1);
        // Heights are worse in the abridged case
        assert!((WGS84[2] - operands[0][2]).abs() < 0.075);
    }
}
