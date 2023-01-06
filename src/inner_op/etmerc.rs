//! Transverse Mercator, following Engsager & Poder (2007)
use crate::math::*;

use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

// Forward transverse mercator, following Engsager & Poder(2007)
fn fwd(op: &Op, _prv: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let ellps = op.params.ellps[0];
    let lat_0 = op.params.lat[0];
    let lon_0 = op.params.lon[0];
    let x_0 = op.params.x[0];
    let Some(conformal) = op.params.fourier_coefficients.get("conformal") else {
        return Ok(0);
    };
    let Some(tm) = op.params.fourier_coefficients.get("tm") else {
        return Ok(0);
    };
    let Some(qs) = op.params.real.get("scaled_radius") else {
        return Ok(0);
    };
    let Some(zb) = op.params.real.get("zb") else {
        return Ok(0);
    };

    let mut successes = 0_usize;
    for coord in operands {
        // --- 1. Geographical -> Conformal latitude, rotated longitude

        // The conformal latitude
        let lat = ellps.latitude_geographic_to_conformal(coord[1] + lat_0, *conformal);
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
        let dc = clenshaw_complex_sin_optimized_for_tmerc(trig, hyp, &tm.fwd);
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
    }

    Ok(successes)
}

// ----- I N V E R S E -----------------------------------------------------------------

// Inverse Transverse Mercator, following Engsager & Poder (2007) (currently Bowring stands in!)
fn inv(op: &Op, _prv: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let ellps = op.params.ellps[0];
    let lon_0 = op.params.lon[0];
    let x_0 = op.params.x[0];

    let Some(conformal) = op.params.fourier_coefficients.get("conformal") else {
        return Ok(0);
    };
    let Some(tm) = op.params.fourier_coefficients.get("tm") else {
        return Ok(0);
    };
    let Some(qs) = op.params.real.get("scaled_radius") else {
        return Ok(0);
    };
    let Some(zb) = op.params.real.get("zb") else {
        return Ok(0);
    };

    let mut successes = 0_usize;
    for coord in operands {
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

        let dc = clenshaw_complex_sin([2. * lat, 2. * lon], &tm.inv);
        lat += dc[0];
        lon += dc[1];
        lon = gudermannian(lon);

        // --- 3. Complex spherical LAT -> Gaussian LAT, LNG

        let (sin_lat, cos_lat) = lat.sin_cos();
        let (sin_lon, cos_lon) = lon.sin_cos();
        let cos_lat_lon = cos_lat * cos_lon;
        lon = sin_lon.atan2(cos_lat_lon);
        lat = (sin_lat * cos_lon).atan2(sin_lon.hypot(cos_lat_lon));

        // --- 4. Gaussian LAT, LNG -> ellipsoidal LAT, LNG

        let lon = normalize_angle_symmetric(lon + lon_0);
        let lat = ellps.latitude_conformal_to_geographic(lat, *conformal);
        (coord[0], coord[1]) = (lon, lat);

        successes += 1;
    }

    Ok(successes)
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

pub fn new(parameters: &RawParameters, provider: &dyn Context) -> Result<Op, Error> {
    let mut op = Op::plain(parameters, InnerOp(fwd), InnerOp(inv), &GAMUT, provider)?;
    let ellps = op.params.ellps[0];
    let n = ellps.third_flattening();
    let lat_0 = op.params.lat[0];
    let y_0 = op.params.y[0];

    // Pre-compute some of the computationally heavy prerequisites,
    // to get better amortization over the full operator lifetime.

    // The scaled spherical Earth radius - Qn in Engsager's implementation
    let qs = op.params.k[0] * ellps.semimajor_axis() * ellps.normalized_meridian_arc_unit(); // meridian_quadrant();
    op.params.real.insert("scaled_radius", qs);

    // The Fourier series for the conformal latitude
    let conformal = ellps.coefficients_for_conformal_latitude_computations();
    op.params
        .fourier_coefficients
        .insert("conformal", conformal);

    // The Fourier series for the transverse mercator coordinates, from [Engsager & Poder, 2007](crate::Bibliography::Eng07),
    // with extensions to 6th order by [Karney, 2011](crate::Bibliography::Kar11).
    let tm = fourier_coefficients(n, &TRANSVERSE_MERCATOR);
    op.params.fourier_coefficients.insert("tm", tm);

    // Conformal latitude value of the latitude-of-origin - Z in Engsager's notation
    let z = ellps.latitude_geographic_to_conformal(lat_0, conformal);
    // op.params.real.insert("z", z);

    // Origin northing minus true northing at the origin latitude
    // i.e. true northing = N - zb
    let zb = y_0 - qs * (z + clenshaw_sin(2. * z, &tm.fwd));
    op.params.real.insert("zb", zb);

    Ok(op)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn etmerc() -> Result<(), Error> {
        // Validation values from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32 | clip
        #[rustfmt::skip]
        let geo = [
            Coord::geo( 55.,  12., 0., 0.),
            Coord::geo(-55.,  12., 0., 0.),
            Coord::geo( 55., -6., 0., 0.),
            Coord::geo(-55., -6., 0., 0.)
        ];

        #[rustfmt::skip]
        let projected = [
            Coord::raw( 691_875.632_139_661, 6_098_907.825_005_012, 0., 0.),
            Coord::raw( 691_875.632_139_661,-6_098_907.825_005_012, 0., 0.),
            Coord::raw(-455_673.814_189_040, 6_198_246.671_090_279, 0., 0.),
            Coord::raw(-455_673.814_189_040,-6_198_246.671_090_279, 0., 0.)
        ];

        let prv = Minimal::default();
        let definition = "etmerc k_0=0.9996 lon_0=9 x_0=500000";
        let op = Op::new(definition, &prv)?;

        let mut operands = geo.clone();
        op.apply(&prv, &mut operands, Fwd)?;

        for i in 0..operands.len() {
            dbg!(operands[i]);
            dbg!(projected[i]);
            assert!(operands[i].hypot2(&projected[i]) < 1e-6);
        }

        op.apply(&prv, &mut operands, Inv)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 5e-6);
        }

        Ok(())
    }

    #[test]
    fn utm() -> Result<(), Error> {
        let prv = Minimal::default();
        let definition = "utm zone=32";
        let op = Op::new(definition, &prv)?;

        // Validation values from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32 | clip
        #[rustfmt::skip]
        let geo = [
            Coord::geo( 55.,  12., 0., 0.),
            Coord::geo(-55.,  12., 0., 0.),
            Coord::geo( 55., -6., 0., 0.),
            Coord::geo(-55., -6., 0., 0.)
        ];

        #[rustfmt::skip]
        let projected = [
            Coord::raw( 691_875.632_139_661, 6_098_907.825_005_012, 0., 0.),
            Coord::raw( 691_875.632_139_661,-6_098_907.825_005_012, 0., 0.),
            Coord::raw(-455_673.814_189_040, 6_198_246.671_090_279, 0., 0.),
            Coord::raw(-455_673.814_189_040,-6_198_246.671_090_279, 0., 0.)
        ];

        let mut operands = geo.clone();
        op.apply(&prv, &mut operands, Fwd)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 5e-3);
        }

        op.apply(&prv, &mut operands, Inv)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 10e-8);
        }
        Ok(())
    }
}
