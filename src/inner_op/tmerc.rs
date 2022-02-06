//! Transverse Mercator, according to Bowring
use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

// Forward transverse mercator, following Bowring (1989)
fn fwd(op: &Op, _prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let ellps = op.params.ellps[0];
    let eps = ellps.second_eccentricity_squared();
    let lat_0 = op.params.lat[0];
    let lon_0 = op.params.lon[0];
    let x_0 = op.params.x[0];
    let y_0 = op.params.y[0];
    let k_0 = op.params.k[0];

    let mut successes = 0_usize;
    for coord in operands {
        let lat = coord[1] + lat_0;
        let c = lat.cos();
        let s = lat.sin();
        let cc = c * c;
        let ss = s * s;

        let dlon = coord[0] - lon_0;
        let oo = dlon * dlon;

        #[allow(non_snake_case)]
        let N = ellps.prime_vertical_radius_of_curvature(lat);
        let z = eps * dlon.powi(3) * c.powi(5) / 6.;
        let sd2 = (dlon / 2.).sin();

        let theta_2 = (2. * s * c * sd2 * sd2).atan2(ss + cc * dlon.cos());

        // Easting
        let sd = dlon.sin();
        coord[0] = x_0
            + k_0 * N * ((c * sd).atanh() + z * (1. + oo * (36. * cc - 29.) / 10.));

        // Northing
        let m = ellps.meridional_distance(lat, Fwd);
        let znos4 = z * N * dlon * s / 4.;
        let ecc = 4. * eps * cc;
        coord[1] = y_0
            + k_0 * (m + N * theta_2 + znos4 * (9. + ecc + oo * (20. * cc - 11.)));
        successes += 1;
    }

    Ok(successes)
}

// ----- I N V E R S E -----------------------------------------------------------------

// Inverse transverse mercator, following Bowring (1989)
fn inv(op: &Op, _prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let ellps = op.params.ellps[0];
    let eps = ellps.second_eccentricity_squared();
    let lat_0 = op.params.lat[0];
    let lon_0 = op.params.lon[0];
    let x_0 = op.params.x[0];
    let y_0 = op.params.y[0];
    let k_0 = op.params.k[0];

    let mut successes = 0_usize;
    for coord in operands {
        // Footpoint latitude, i.e. the latitude of a point on the central meridian
        // having the same northing as the point of interest
        let lat = ellps.meridional_distance((coord[1] - y_0) / k_0, Inv);
        let t = lat.tan();
        let c = lat.cos();
        let cc = c * c;
        #[allow(non_snake_case)]
        let N = ellps.prime_vertical_radius_of_curvature(lat);
        let x = (coord[0] - x_0) / (k_0 * N);
        let xx = x * x;
        let theta_4 = x.sinh().atan2(c);
        let theta_5 = (t * theta_4.cos()).atan();

        // Latitude
        let xet = xx * xx * eps * t / 24.;
        coord[1] = lat_0 + (1. + cc * eps) * (theta_5 - xet * (9. - 10. * cc))
            - eps * cc * lat;

        // Longitude
        let approx = lon_0 + theta_4;
        let coef = eps / 60. * xx * x * c;
        coord[0] = approx - coef * (10. - 4. * xx / cc + xx * cc);

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


pub fn new(parameters: &RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
    Op::plain(parameters, InnerOp(fwd), InnerOp(inv), &GAMUT, provider)
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------


// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmerc() -> Result<(), Error> {
        let prv = Minimal::default();
        let definition = "tmerc k_0=0.9996 lon_0=9 x_0=500000";
        let op = Op::new(definition, &prv)?;

        // Validation value from PROJ:
        // echo 12 55 0 0 | cct -d18 +proj=utm +zone=32 | clip
        let geo = [Coord::geo(55., 12., 0., 0.)];
        let projected = [Coord::raw(691_875.632_139_661, 6_098_907.825_005_012, 0., 0.)];

        let mut operands = geo.clone();
        op.apply(&prv, &mut operands, Fwd)?;
        for i in 0..operands.len() {
            assert!(dbg!(operands[i].hypot2(&projected[i])) < 4e-6);
        }

        op.apply(&prv, &mut operands, Inv)?;
        for i in 0..operands.len() {
            assert!(dbg!(operands[i].hypot2(&geo[i])) < 10e-12);
        }
        Ok(())
    }
}
