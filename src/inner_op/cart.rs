#![allow(non_snake_case)]

/// Template for implementation of operators
use super::*;

// ----- F O R W A R D --------------------------------------------------------------

fn cart_fwd(op: &Op, _prv: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let mut n = 0_usize;
    for coord in operands {
        *coord = op.params.ellps[0].cartesian(coord);
        if !coord.0.iter().any(|c| c.is_nan()) {
            n += 1;
        }
    }
    Ok(n)
}

// ----- I N V E R S E --------------------------------------------------------------

fn cart_inv(op: &Op, _prv: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    // eccentricity squared, Fukushima's E, Claessens' c3 = 1-c2`
    let es = op.params.ellps[0].eccentricity_squared();
    // semiminor axis
    let b = op.params.ellps[0].semiminor_axis();
    // semimajor axis
    let a = op.params.ellps[0].semimajor_axis();
    // reciproque of a
    let ra = 1. / op.params.ellps[0].semimajor_axis();
    // aspect ratio, b/a: Fukushima's ec, Claessens' c4
    let ar = b * ra;
    // 1.5 times the fourth power of the eccentricity
    let ce4 = 1.5 * es * es;
    // if we're closer than this to the Z axis, we force latitude to one of the poles
    let cutoff = op.params.ellps[0].semimajor_axis() * 1e-16;

    let mut n = 0_usize;
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
        if p < cutoff {
            let phi = std::f64::consts::FRAC_PI_2.copysign(Z);
            let h = Z.abs() - b;
            *coord = Coord::raw(lam, phi, h, t);
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
        let h = (p * CC.abs() + Z.abs() * S1.abs() - a * CC.hypot(ar * S1)) / CC.hypot(S1);
        // Bowring's height formula works better close to the ellipsoid, but requires a (sin, cos)-pair
        *coord = Coord::raw(lam, phi, h, t);
        if ![lam, phi, h, t].iter().any(|c| c.is_nan()) {
            n += 1;
        }
    }
    Ok(n)
}

// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 2] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") },
];

pub fn new(parameters: &RawParameters, provider: &dyn Context) -> Result<Op, Error> {
    Op::plain(
        parameters,
        InnerOp(cart_fwd),
        InnerOp(cart_inv),
        &GAMUT,
        provider,
    )
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() -> Result<(), Error> {
        let provider = Minimal::default();
        let op = Op::new("cart", &provider)?;

        let geo = [
            Coord::geo(85., 0., 100000., 0.),
            Coord::geo(55., 10., -100000., 0.),
            Coord::geo(25., 20., 0., 0.),
            Coord::geo(0., -20., 0., 0.),
            Coord::geo(-25., 20., 10., 0.),
            Coord::geo(-25., -20., 10., 0.),
            Coord::geo(25., -20., 10., 0.),
        ];

        let cart = [
            Coord::raw(566462.633537476765923, 0.0, 6432020.33369012735784, 0.0),
            Coord::raw(
                3554403.47587193036451,
                626737.23312017065473,
                5119468.31865925621241,
                0.,
            ),
            Coord::raw(
                5435195.38214521575719,
                1978249.33652197546325,
                2679074.46287727775052,
                0.,
            ),
            Coord::raw(5993488.27326157130301, -2181451.33089075051248, 0., 0.),
            Coord::raw(
                5435203.89865261223167,
                1978252.43627716740593,
                -2679078.68905989499763,
                0.,
            ),
            Coord::raw(
                5435203.89865261223167,
                -1978252.43627716740593,
                -2679078.68905989499763,
                0.,
            ),
            Coord::raw(
                5435203.89865261223167,
                -1978252.43627716740593,
                2679078.68905989499763,
                0.,
            ),
        ];

        // Forward
        let mut operands = geo.clone();
        op.apply(&provider, &mut operands, Fwd)?;
        for i in 0..4 {
            assert!(operands[i].hypot3(&cart[i]) < 20e-9);
        }

        // Inverse
        op.apply(&provider, &mut operands, Inv)?;
        for i in 0..5 {
            assert!(operands[i].default_ellps_3d_dist(&geo[i]) < 10e-9);
        }

        Ok(())
    }
}
