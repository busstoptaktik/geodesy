/// Geographical to cartesian (and v.v.) conversion
use crate::authoring::*;

// ----- F O R W A R D --------------------------------------------------------------

fn cart_fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let n = operands.len();
    let mut successes = 0;
    let ellps = op.params.ellps(0);
    for i in 0..n {
        let mut coord = operands.get_coord(i);
        coord = ellps.cartesian(&coord);
        if !coord.0.iter().any(|c| c.is_nan()) {
            successes += 1;
        }
        operands.set_coord(i, &coord);
    }
    successes
}

// ----- I N V E R S E --------------------------------------------------------------

fn cart_inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);

    // eccentricity squared, Fukushima's E, Claessens' c3 = 1-c2`
    let es = ellps.eccentricity_squared();

    let b = ellps.semiminor_axis();
    let a = ellps.semimajor_axis();
    let ra = 1. / ellps.semimajor_axis();

    // b/a: Fukushima's ec, Claessens' c4
    let ar = b * ra;
    // 1.5 times the fourth power of the eccentricity
    let ce4 = 1.5 * es * es;
    // if we're closer than this to the Z axis, we force latitude to one of the poles
    let cutoff = ellps.semimajor_axis() * 1e-16;

    let n = operands.len();
    let mut successes = 0;
    #[allow(non_snake_case)]
    for i in 0..n {
        let mut coord = operands.get_coord(i);
        let X = coord[0];
        let Y = coord[1];
        let Z = coord[2];
        let t = coord[3];

        // The longitude is straightforward
        let lam = Y.atan2(X);

        // The perpendicular distance from the point coordinate to the Z-axis (HM eq. 5-28)
        let p = X.hypot(Y);

        // If we're close to the Z-axis, the full algorithm breaks down. But if
        // we're close to the Z-axis, we also assert that the latitude is close
        // to one of the poles. So we force the latitude to the relevant pole and
        // compute the height as |Z| - b
        if p < cutoff {
            let phi = std::f64::consts::FRAC_PI_2.copysign(Z);
            let h = Z.abs() - b;
            coord = Coor4D::raw(lam, phi, h, t);
            operands.set_coord(i, &coord);
            continue;
        }

        let P = ra * p;
        let S0 = ra * Z;
        let C0 = ar * P;

        // There's a lot of common subexpressions in the following which,
        // in Fukushima's and Claessens' Fortranesque implementations,
        // were explicitly eliminated (by introducing s02 = S0*S0, etc.).
        // For clarity, we keep the full expressions here, and leave the
        // elimination task to the compiler's optimizer step.
        let A = S0.hypot(C0);
        let F = P * A * A * A - es * C0 * C0 * C0;
        let B = ce4 * S0 * S0 * C0 * C0 * P * (A - ar);

        let S1 = (ar * S0 * A * A * A + es * S0 * S0 * S0) * F - B * S0;
        let C1 = F * F - B * C0;
        let CC = ar * C1;

        let phi = S1.atan2(CC);
        let h = (p * CC.abs() + Z.abs() * S1.abs() - a * CC.hypot(ar * S1)) / CC.hypot(S1);
        // Bowring's height formula works better close to the ellipsoid, but requires a (sin, cos)-pair
        coord = Coor4D::raw(lam, phi, h, t);
        operands.set_coord(i, &coord);

        if ![lam, phi, h, t].iter().any(|c| c.is_nan()) {
            successes += 1;
        }
    }
    successes
}

// ----- C O N S T R U C T O R ------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 2] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    Op::basic(
        parameters,
        InnerOp(cart_fwd),
        Some(InnerOp(cart_inv)),
        &GAMUT,
    )
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("cart")?;

        let geo = [
            Coor4D::geo(85., 0., 100000., 0.),
            Coor4D::geo(55., 10., -100000., 0.),
            Coor4D::geo(25., 20., 0., 0.),
            Coor4D::geo(0., -20., 0., 0.),
            Coor4D::geo(-25., 20., 10., 0.),
            Coor4D::geo(-25., -20., 10., 0.),
            Coor4D::geo(25., -20., 10., 0.),
        ];

        let cart = [
            Coor4D::raw(566_462.633_537_476_8, 0.0, 6_432_020.333_690_127, 0.0),
            Coor4D::raw(
                3_554_403.475_871_930_4,
                626_737.233_120_170_7,
                5_119_468.318_659_256,
                0.,
            ),
            Coor4D::raw(
                5_435_195.382_145_216,
                1_978_249.336_521_975_5,
                2_679_074.462_877_277_8,
                0.,
            ),
            Coor4D::raw(5_993_488.273_261_571, -2_181_451.330_890_750_5, 0., 0.),
            Coor4D::raw(
                5_435_203.898_652_612,
                1_978_252.436_277_167_4,
                -2_679_078.689_059_895,
                0.,
            ),
            Coor4D::raw(
                5_435_203.898_652_612,
                -1_978_252.436_277_167_4,
                -2_679_078.689_059_895,
                0.,
            ),
            Coor4D::raw(
                5_435_203.898_652_612,
                -1_978_252.436_277_167_4,
                2_679_078.689_059_895,
                0.,
            ),
        ];

        let e = Ellipsoid::default();
        // Forward
        let mut operands = geo;
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..4 {
            assert!(operands[i].hypot3(&cart[i]) < 20e-9);
        }

        // Inverse
        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..5 {
            assert!(e.distance(&operands[i], &geo[i]) < 1e-8);
        }

        Ok(())
    }
}
