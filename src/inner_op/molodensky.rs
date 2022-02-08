//! The full and abridged Molodensky transformations for 2D and 3D data.
//!
//! Partially based on the PROJ implementation by Kristian Evers,
//! partially on the following 3 publications:
//!
//! 1. OGP Publication 373-7-2 – Geomatics Guidance Note, number 7, part 2,
//!
//! 2. [Dea04] R.E.Deakin, 2004: The Standard and Abridged Molodensky
//!    Coordinate Transformation Formulae.
//!    URL http://www.mygeodesy.id.au/documents/Molodensky%20V2.pdf
//!
//! 3. [Ruf16] A. C. Ruffhead, 2016:  The SMITSWAM method of datum transformations
//!    consisting of Standard Molodensky in two stages with applied misclosures,
//!    Survey Review, 48:350, pp. 376-384,
//!    [DOI](https://doi.org/10.1080/00396265.2016.1191748)
//!
#![allow(non_snake_case)]
use super::*;

// ----- C O M M O N -------------------------------------------------------------------


fn common(op: &Op, _prv: &dyn Provider, operands: &mut [Coord], direction: Direction) -> Result<usize, Error> {
    let ellps = op.params.ellps[0];
    let a = ellps.semimajor_axis();
    let f = ellps.flattening();
    let es = ellps.eccentricity_squared();
    let abridged = op.params.boolean("abridged");
    let dx = op.params.real("dx")?;
    let dy = op.params.real("dy")?;
    let dz = op.params.real("dz")?;
    let da = op.params.real("da")?;
    let df = op.params.real("df")?;
    let adffda = ellps.semimajor_axis() * df + ellps.flattening() * da;
    let moped = Molodensky {
        a,
        f,
        es,
        dx,
        dy,
        dz,
        da,
        df,
        adffda,
        ellps,
        abridged,
    };

    let n = operands.len();

    for coord in operands {
        let par = calc_molodensky_params(&moped, coord);
        if direction == Fwd {
            coord[0] += par[0];
            coord[1] += par[1];
            coord[2] += par[2];
        } else {
            coord[0] -= par[0];
            coord[1] -= par[1];
            coord[2] -= par[2];
        }
    }

    Ok(n)
}


// ----- F O R W A R D -----------------------------------------------------------------
fn fwd(op: &Op, prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    common(op, prv, operands, Fwd)
}

// ----- I N V E R S E -----------------------------------------------------------------
fn inv(op: &Op, prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    common(op, prv, operands, Inv)
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 10] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Flag { key: "abridged" },
    OpParameter::Real { key: "dx", default: Some(0f64) },
    OpParameter::Real { key: "dy", default: Some(0f64) },
    OpParameter::Real { key: "dz", default: Some(0f64) },
    OpParameter::Real { key: "da", default: Some(0f64) },
    OpParameter::Real { key: "df", default: Some(0f64) },
    OpParameter::Text { key: "ellps",  default: Some("GRS80") },
    OpParameter::Text { key: "ellps_0",  default: Some("GRS80") },
    OpParameter::Text { key: "ellps_1",  default: Some("GRS80") },
];

pub fn new(parameters: &RawParameters, _provider: &dyn Provider) -> Result<Op, Error> {
    let def = &parameters.definition;
    dbg!(parameters);
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    let ellps_0 = params.ellps[0];
    let ellps_1 = params.ellps[1];

    // We may use `ellps, da, df`, to parameterize the op, but `ellps_0, ellps_1`
    // is a more likely set of parameters to come across in real life.
    if params.given.contains_key("ellps_0") && params.given.contains_key("ellps_1") {
        let da = ellps_1.semimajor_axis() - ellps_0.semimajor_axis();
        let df = ellps_1.flattening() - ellps_0.flattening();
        params.real.insert("da", da);
        params.real.insert("df", df);
    }

    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));
    let steps = Vec::<Op>::new();

    Ok(Op {
        descriptor,
        params,
        steps,
    })
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------
struct Molodensky {
    a: f64,
    f: f64,
    es: f64,
    dx: f64,
    dy: f64,
    dz: f64,
    da: f64,
    df: f64,
    adffda: f64,
    ellps: Ellipsoid,
    abridged: bool,
}

fn calc_molodensky_params(op: &Molodensky, coord: &Coord) -> Coord {
    let lam = coord[0];
    let phi = coord[1];
    let h = coord[2];
    let (slam, clam) = lam.sin_cos();
    let (sphi, cphi) = phi.sin_cos();

    // We also need the radii of curvature
    let N = op.ellps.prime_vertical_radius_of_curvature(phi);
    let M = op.ellps.meridian_radius_of_curvature(phi);

    // Now compute the offsets in the ellipsoidal space
    let fac = op.dx * clam + op.dy * slam;

    if op.abridged {
        // delta phi
        let dphi = (-fac * sphi + op.dz * cphi + op.adffda * (2.0 * phi).sin()) / M;

        // delta lambda
        let dlam_denom = N * cphi;
        if dlam_denom == 0.0 {
            return Coord::nan();
        }
        let dlam = (op.dy * clam - op.dx * slam) / dlam_denom;

        // delta h
        let dh = fac * cphi + (op.dz + op.adffda * sphi) * sphi - op.da;
        return Coord::raw(dlam, dphi, dh, 0.0);
    }

    // delta phi
    let mut dphi = (op.dz + ((N * op.es * sphi * op.da) / op.a)) * cphi - fac * sphi
        + (M / (1.0 - op.f) + N * (1.0 - op.f)) * op.df * sphi * cphi;
    let dphi_denom = M + h;
    if dphi_denom == 0.0 {
        return Coord::nan();
    }
    dphi /= dphi_denom;

    // delta lambda
    let dlam_denom = (N + h) * cphi;
    if dlam_denom == 0.0 {
        return Coord::nan();
    }
    let dlam = (op.dy * clam - op.dx * slam) / dlam_denom;

    // delta h
    let dh = fac * cphi + op.dz * sphi - (op.a / N) * op.da + N * (1.0 - op.f) * op.df * sphi * sphi;

    Coord::raw(dlam, dphi, dh, 0.)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn molodensky() -> Result<(), Error> {
        let prv = Minimal::default();
        // ---------------------------------------------------------------------------
        // Test case from OGP Publication 373-7-2: Geomatics Guidance Note number 7,
        // part 2: Transformation from WGS84 to ED50.
        // ---------------------------------------------------------------------------

        let definition = "
            molodensky ellps_0=WGS84 ellps_1=intl
            dx=84.87 dy=96.49 dz=116.95
        ";
        let op = Op::new(definition, &prv)?;

        // Test point (53.80939444444444, 2.12955, 73 m)
        let lat = Coord::dms_to_dd(53, 48, 33.82);
        let lon = Coord::dms_to_dd(2, 7, 46.38);
        let WGS84 = Coord::geo(lat, lon, 73., 0.0);

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
        let ED50 = Coord::geo(lat, lon, 28.02470, 0.0);

        // In the unabridged case, Molodensky replicates Helmert to
        // within 5 mm in the plane and the elevation.
        let mut operands = [WGS84];
        op.apply(&prv, &mut operands, Fwd)?;
        dbg!(ED50);
        dbg!(operands);
        assert!(dbg!(ED50.default_ellps_dist(&operands[0])) < 0.005);
        assert!((ED50[2] - operands[0][2]).abs() < 0.005);

        // The same holds in the reverse unabridged case, where
        // additionally the elevation is even better
        let mut operands = [ED50];
        op.apply(&prv, &mut operands, Inv)?;
        assert!(WGS84.default_ellps_3d_dist(&operands[0]) < 0.005);
        assert!((WGS84[2] - operands[0][2]).abs() < 0.001);

        // The abridged case. Same test point. Both plane coordinates and
        // elevations are *much* worse, but still better-than-decimeter.
        let definition = "
            molodensky ellps_0=WGS84 ellps_1=intl
            dx=84.87 dy=96.49 dz=116.95 abridged
        ";
        let op = Op::new(definition, &prv)?;

        let mut operands = [WGS84];
        op.apply(&prv, &mut operands, Fwd)?;
        assert!(ED50.default_ellps_dist(&operands[0]) < 0.1);
        // Heights are worse in the abridged case
        assert!((ED50[2] - operands[0][2]).abs() < 0.075);

        let mut operands = [ED50];
        op.apply(&prv, &mut operands, Inv)?;
        assert!(WGS84.default_ellps_dist(&operands[0]) < 0.1);
        // Heights are worse in the abridged case
        assert!((WGS84[2] - operands[0][2]).abs() < 0.075);
        Ok(())
    }
}
