//! Transverse Mercator, following Engsager & Poder (2007)
use crate::authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

// Forward transverse mercator, following Engsager & Poder(2007)
fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    // Make all precomputed parameters directly accessible
    let ellps = op.params.ellps(0);
    let lon_0 = op.params.lon(0).to_radians();
    let x_0 = op.params.x(0);
    let Some(conformal) = op.params.fourier_coefficients.get("conformal") else {
        warn!("Missing Fourier coefficients for conformal mapping!");
        return 0;
    };
    let Some(tm) = op.params.fourier_coefficients.get("tm") else {
        warn!("Missing Fourier coefficients for TM!");
        return 0;
    };
    let Some(qs) = op.params.real.get("scaled_radius") else {
        warn!("Missing a scaled radius!");
        return 0;
    };
    let Some(zb) = op.params.real.get("zb") else {
        warn!("Missing a zombie parameter!");
        return 0;
    };

    let range = 0..operands.len();
    let mut successes = 0_usize;
    for i in range {
        let mut coord = operands.get_coord(i);

        // --- 1. Geographical -> Conformal latitude, rotated longitude

        // The conformal latitude
        let lat = ellps.latitude_geographic_to_conformal(coord[1], conformal);
        // The longitude as reckoned from the central meridian
        let lon = coord[0] - lon_0;

        // --- 2. Conformal LAT, LNG -> complex spherical LAT

        let (sin_lat, cos_lat) = lat.sin_cos();
        let (sin_lon, cos_lon) = lon.sin_cos();
        let cos_lat_lon = cos_lat * cos_lon;
        let mut lat = sin_lat.atan2(cos_lat_lon);

        // --- 3. Complex spherical N, E -> ellipsoidal normalized N, E

        // Some numerical optimizations from PROJ modifications by Even Rouault,
        let inv_denom_tan_lon = 1. / sin_lat.hypot(cos_lat_lon);
        let tan_lon = sin_lon * cos_lat * inv_denom_tan_lon;
        // Inverse Gudermannian, using the precomputed tan(lon)
        let mut lon = tan_lon.asinh();

        // Trigonometric terms for Clenshaw summation
        // Non-optimized version:  `let trig = (2.*lat).sin_cos()`
        let two_inv_denom_tan_lon = 2.0 * inv_denom_tan_lon;
        let two_inv_denom_tan_lon_square = two_inv_denom_tan_lon * inv_denom_tan_lon;
        let tmp_r = cos_lat_lon * two_inv_denom_tan_lon_square;
        let trig = [sin_lat * tmp_r, cos_lat_lon * tmp_r - 1.0];

        // Hyperbolic terms for Clenshaw summation
        // Non-optimized version:  `let hyp = [(2.*lon).sinh(), (2.*lon).sinh()]`
        let hyp = [
            tan_lon * two_inv_denom_tan_lon,
            two_inv_denom_tan_lon_square - 1.0,
        ];

        // Evaluate and apply the differential term
        let dc = fourier::complex_sin_optimized_for_tmerc(trig, hyp, &tm.fwd);
        lat += dc[0];
        lon += dc[1];

        // Don't wanna play if we're too far from the center meridian
        if lon.abs() > 2.623395162778 {
            coord[0] = f64::NAN;
            coord[1] = f64::NAN;
            continue;
        }

        // --- 4. ellipsoidal normalized N, E -> metric N, E

        coord[0] = qs * lon + x_0; // Easting
        coord[1] = qs * lat + zb; // Northing
        successes += 1;
        operands.set_coord(i, &coord);
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

// Inverse Transverse Mercator, following Engsager & Poder (2007)
fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    // Make all precomputed parameters directly accessible
    let ellps = op.params.ellps(0);
    let lon_0 = op.params.lon(0).to_radians();
    let x_0 = op.params.x(0);
    let Some(conformal) = op.params.fourier_coefficients.get("conformal") else {
        warn!("Missing Fourier coefficients for conformal mapping!");
        return 0;
    };
    let Some(tm) = op.params.fourier_coefficients.get("tm") else {
        warn!("Missing Fourier coefficients for TM!");
        return 0;
    };
    let Some(qs) = op.params.real.get("scaled_radius") else {
        warn!("Missing a scaled radius!");
        return 0;
    };
    let Some(zb) = op.params.real.get("zb") else {
        warn!("Missing a zombie parameter!");
        return 0;
    };

    let range = 0..operands.len();
    let mut successes = 0_usize;
    for i in range {
        let mut coord = operands.get_coord(i);

        // --- 1. Normalize N, E

        let mut lon = (coord[0] - x_0) / qs;
        let mut lat = (coord[1] - zb) / qs;

        // Don't wanna play if we're too far from the center meridian
        if lon.abs() > 2.623395162778 {
            coord[0] = f64::NAN;
            coord[1] = f64::NAN;
            continue;
        }

        // --- 2. Normalized N, E -> complex spherical LAT, LNG

        let dc = fourier::complex_sin([2. * lat, 2. * lon], &tm.inv);
        lat += dc[0];
        lon += dc[1];
        lon = gudermannian::fwd(lon);

        // --- 3. Complex spherical LAT -> Gaussian LAT, LNG

        let (sin_lat, cos_lat) = lat.sin_cos();
        let (sin_lon, cos_lon) = lon.sin_cos();
        let cos_lat_lon = cos_lat * cos_lon;
        lon = sin_lon.atan2(cos_lat_lon);
        lat = (sin_lat * cos_lon).atan2(sin_lon.hypot(cos_lat_lon));

        // --- 4. Gaussian LAT, LNG -> ellipsoidal LAT, LNG

        let lon = angular::normalize_symmetric(lon + lon_0);
        let lat = ellps.latitude_conformal_to_geographic(lat, conformal);
        (coord[0], coord[1]) = (lon, lat);

        successes += 1;
        operands.set_coord(i, &coord);
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 7] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") },

    OpParameter::Real { key: "lat_0", default: Some(0_f64) },
    OpParameter::Real { key: "lon_0", default: Some(0_f64) },
    OpParameter::Real { key: "x_0",   default: Some(0_f64) },
    OpParameter::Real { key: "y_0",   default: Some(0_f64) },

    OpParameter::Real { key: "k_0",   default: Some(1_f64) },
];

#[rustfmt::skip]
pub const UTM_GAMUT: [OpParameter; 4] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Flag { key: "south" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") },
    OpParameter::Natural { key: "zone", default: None },
];

// ----- C O N S T R U C T O R,   U T M ------------------------------------------------

pub fn utm(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &UTM_GAMUT)?;

    // The UTM zone should be an integer between 1 and 60
    let zone = params.natural("zone")?;
    if !(1..61).contains(&zone) {
        error!("UTM: {zone}. Must be an integer in the interval 1..60");
        return Err(Error::General(
            "UTM: 'zone' must be an integer in the interval 1..60",
        ));
    }

    // The scaling factor is 0.9996 by definition of UTM
    params.real.insert("k_0", 0.9996);

    // The center meridian is determined by the zone
    params.real.insert("lon_0", -183. + 6. * zone as f64);

    // The base parallel is by definition the equator
    params.real.insert("lat_0", 0.);

    // The false easting is 500000 m by definition of UTM
    params.real.insert("x_0", 500_000.);

    // The false northing is 0 m by definition of UTM
    params.real.insert("y_0", 0.);
    // or 10_000_000 m if using the southern aspect
    if params.boolean("south") {
        params.real.insert("y_0", 10_000_000.0);
    }

    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));
    let steps = Vec::<Op>::new();
    let id = OpHandle::new();

    let mut op = Op {
        descriptor,
        params,
        steps,
        id,
    };

    precompute(&mut op);
    Ok(op)
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

#[rustfmt::skip]
const TRANSVERSE_MERCATOR: PolynomialCoefficients = PolynomialCoefficients {
    // Geodetic to TM. [Engsager & Poder, 2007](crate::Bibliography::Eng07)
    fwd: [
        [1./2.,   -2./3.,   5./16.,   41./180.,   -127./288.0 ,   7891./37800.],
        [0., 13./48.,   -3./5.,   557./1440.,   281./630.,   -1983433./1935360.],
        [0., 0., 61./240.,  -103./140.,   15061./26880.,   167603./181440.],
        [0., 0., 0., 49561./161280.,   -179./168.,   6601661./7257600.],
        [0., 0., 0., 0., 34729./80640.,   -3418889./1995840.],
        [0., 0., 0., 0., 0., 212378941./319334400.]
    ],

    // TM to Geodetic. [Engsager & Poder, 2007](crate::Bibliography::Eng07)
    inv: [
        [-1./2.,   2./3.,   -37./96.,   1./360.,   81./512.,   -96199./604800.],
        [0., -1./48.,   -1./15.,   437./1440.,   -46./105.,   1118711./3870720.],
        [0., 0., -17./480.,   37./840.,   209./4480.,   -5569./90720.],
        [0., 0., 0., -4397./161280.,   11./504.,   830251./7257600.],
        [0., 0., 0., 0., -4583./161280.,   108847./3991680.],
        [0., 0., 0., 0., 0., -20648693./638668800.]
    ]
};

// Common setup workhorse between utm and the plain tmerc:
// Pre-compute some of the computationally heavy prerequisites,
// to get better amortization over the full operator lifetime.
fn precompute(op: &mut Op) {
    let ellps = op.params.ellps(0);
    let n = ellps.third_flattening();
    let lat_0 = op.params.lat(0).to_radians();
    let y_0 = op.params.y(0);

    // The scaled spherical Earth radius - Qn in Engsager's implementation
    let qs = op.params.k(0) * ellps.semimajor_axis() * ellps.normalized_meridian_arc_unit();
    op.params.real.insert("scaled_radius", qs);

    // The Fourier series for the conformal latitude
    let conformal = ellps.coefficients_for_conformal_latitude_computations();
    op.params
        .fourier_coefficients
        .insert("conformal", conformal);

    // The Fourier series for the transverse mercator coordinates,
    // from [Engsager & Poder, 2007](crate::Bibliography::Eng07),
    // with extensions to 6th order by [Karney, 2011](crate::Bibliography::Kar11).
    let tm = fourier_coefficients(n, &TRANSVERSE_MERCATOR);
    op.params.fourier_coefficients.insert("tm", tm);

    // Conformal latitude value of the latitude-of-origin - Z in Engsager's notation
    let z = ellps.latitude_geographic_to_conformal(lat_0, &conformal);
    // Origin northing minus true northing at the origin latitude
    // i.e. true northing = N - zb
    let zb = y_0 - qs * (z + fourier::sin(2. * z, &tm.fwd));
    op.params.real.insert("zb", zb);
}

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let mut op = Op::plain(parameters, InnerOp(fwd), Some(InnerOp(inv)), &GAMUT, ctx)?;
    precompute(&mut op);
    Ok(op)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn tmerc() -> Result<(), Error> {
        // Validation values from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32 | clip
        #[rustfmt::skip]
        let geo = [
            Coor2D::geo( 55.,  12.),
            Coor2D::geo(-55.,  12.),
            Coor2D::geo( 55., -6.),
            Coor2D::geo(-55., -6.),
        ];

        #[rustfmt::skip]
        let projected = [
            Coor2D::raw( 691_875.632_139_661, 6_098_907.825_005_012),
            Coor2D::raw( 691_875.632_139_661,-6_098_907.825_005_012),
            Coor2D::raw(-455_673.814_189_040, 6_198_246.671_090_279),
            Coor2D::raw(-455_673.814_189_040,-6_198_246.671_090_279)
        ];

        let mut ctx = Minimal::default();
        let definition = "tmerc k_0=0.9996 lon_0=9 x_0=500000";
        let op = ctx.op(definition)?;

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;

        for i in 0..operands.len() {
            assert_float_eq!(operands[i].0, projected[i].0, abs_all <= 1e-8);
        }

        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 5e-6);
        }

        // Test involving scale and all offsets, inspired by a bug reported by Sean Rennie
        // over at https://github.com/busstoptaktik/geodesy/issues/59 - revealing the
        // double correction for lat_0 in the forward case
        let definition =
            "tmerc lat_0=49 lon_0=-2 k_0=0.9996012717 x_0=400000 y_0=-100000 ellps=airy";
        let op = ctx.op(definition)?;

        let geo = [Coor2D::geo(52., 1.)];

        // Expected value from PROJ:
        // echo 1 52 0 0 | cct -d 15 proj=tmerc lat_0=49 lon_0=-2 k_0=0.9996012717 x_0=400000 y_0=-100000 ellps=airy  --
        let projected = [Coor2D::raw(605909.130344302393496, 237803.365171569399536)];

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;

        for i in 0..operands.len() {
            assert_float_eq!(operands[i].0, projected[i].0, abs_all <= 1e-8);
        }

        Ok(())
    }

    #[test]
    fn utm() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "utm zone=32";
        let op = ctx.op(definition)?;

        // Validation values from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32 | clip
        #[rustfmt::skip]
        let geo = [
            Coor2D::geo( 55.,  12.),
            Coor2D::geo(-55.,  12.),
            Coor2D::geo( 55., -6.),
            Coor2D::geo(-55., -6.)
        ];

        #[rustfmt::skip]
        let projected = [
            Coor2D::raw( 691_875.632_139_661, 6_098_907.825_005_012),
            Coor2D::raw( 691_875.632_139_661,-6_098_907.825_005_012),
            Coor2D::raw(-455_673.814_189_040, 6_198_246.671_090_279),
            Coor2D::raw(-455_673.814_189_040,-6_198_246.671_090_279)
        ];

        let mut operands = geo.clone();
        assert_eq!(ctx.apply(op, Fwd, &mut operands)?, 4);
        for i in 0..operands.len() {
            assert_float_eq!(operands[i].0, projected[i].0, abs_all <= 1e-8);
        }

        assert_eq!(ctx.apply(op, Inv, &mut operands)?, 4);
        for i in 0..operands.len() {
            assert_float_eq!(operands[i].0, geo[i].0, abs_all <= 1e-12);
        }

        Ok(())
    }

    #[test]
    fn utm_south() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("utm zone=32 south")?;

        #[rustfmt::skip]
        let geo = [
            Coor4D::geo( 55.,  12., 0., 0.),
            Coor4D::geo(-55.,  12., 0., 0.),
            Coor4D::geo( 55., -6., 0., 0.),
            Coor4D::geo(-55., -6., 0., 0.)
        ];

        #[rustfmt::skip]
        let projected = [
            Coor4D::raw( 691_875.632_139_661, 1e7+6_098_907.825_005_012, 0., 0.),
            Coor4D::raw( 691_875.632_139_661, 1e7-6_098_907.825_005_012, 0., 0.),
            Coor4D::raw(-455_673.814_189_040, 1e7+6_198_246.671_090_279, 0., 0.),
            Coor4D::raw(-455_673.814_189_040, 1e7-6_198_246.671_090_279, 0., 0.)
        ];

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 5e-3);
        }

        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 10e-8);
        }

        Ok(())
    }
}
