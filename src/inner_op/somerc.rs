#![allow(non_snake_case)] // So we can use the mathematical notation from the original text

// Swiss Oblique Mercator Projection
//
// Implementation based on https://download.osgeo.org/proj/swiss.pdf
// with inspirations taken from
//     - [proj4rs](https://github.com/3liz/proj4rs/blob/main/src/projections/somerc.rs)
//     - [proj4js](https://github.com/proj4js/proj4js/blob/5995fa62fc7f4fdbbafb23d89b260bd863b0ca03/lib/projections/somerc.js)
//     - [PROJ](https://proj.org/operations/projections/somerc.html)
use crate::authoring::*;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

// ----- C O M M O N -------------------------------------------------------------------

const EPS_10: f64 = 1.0e-10;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;
    let n = operands.len();

    let el = op.params.ellps(0);
    let e = el.eccentricity();
    let hlf_e = e * 0.5;

    // Grab pre-computed values
    let y_0 = op.params.real["y_0"];
    let x_0 = op.params.real["x_0"];
    let lam_0 = op.params.real["lon_0"].to_radians();

    let c = op.params.real["c"];
    let K = op.params.real["K"];
    let R = op.params.real["R"];

    let sin_phi_0_p = op.params.real["sin_phi_0_p"];
    let cos_phi_0_p = op.params.real["cos_phi_0_p"];

    for i in 0..n {
        let (lam, phi) = operands.xy(i);
        let sp = e * phi.sin();
        let phi_p = 2.
            * ((c * ((FRAC_PI_4 + 0.5 * phi).tan().ln() - hlf_e * ((1. + sp) / (1. - sp)).ln())
                + K)
                .exp())
            .atan()
            - FRAC_PI_2;

        let lam_p = c * (lam - lam_0);
        let (sin_lam_p, cos_lam_p) = lam_p.sin_cos();
        let (sin_phi_p, cos_phi_p) = phi_p.sin_cos();

        let phi_pp = (cos_phi_0_p * sin_phi_p - sin_phi_0_p * cos_phi_p * cos_lam_p).asin();
        let lam_pp = (cos_phi_p * sin_lam_p / phi_pp.cos()).asin();

        let x = R * lam_pp + x_0;
        let y = R * (FRAC_PI_4 + 0.5 * phi_pp).tan().ln() + y_0;

        operands.set_xy(i, x, y);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;
    let n = operands.len();
    const MAX_ITERATIONS: isize = 20;

    let el = op.params.ellps(0);
    let e = el.eccentricity();

    // Grab pre-computed values
    let c = op.params.real["c"];
    let K = op.params.real["K"];
    let R = op.params.real["R"];

    let lam_0 = op.params.real["lon_0"].to_radians();
    let sin_phi_0_p = op.params.real["sin_phi_0_p"];
    let cos_phi_0_p = op.params.real["cos_phi_0_p"];
    let y_0 = op.params.real["y_0"];
    let x_0 = op.params.real["x_0"];

    for i in 0..n {
        let (x, y) = operands.xy(i);
        let X = x - x_0;
        let Y = y - y_0;

        let phi_pp = 2.0 * (((Y / R).exp()).atan() - FRAC_PI_4);
        let lam_pp = X / R;

        let sin_phi_p = cos_phi_0_p * phi_pp.sin() + sin_phi_0_p * phi_pp.cos() * lam_pp.cos();
        let phi_p = sin_phi_p.asin();
        let sin_lam_p = (phi_pp.cos() * lam_pp.sin()) / phi_p.cos();
        let lam_p = sin_lam_p.asin();

        let C = (K - (FRAC_PI_4 + 0.5 * phi_p).tan().ln()) / c;

        let lam = (lam_p / c) + lam_0;
        let mut phi = phi_p;

        let mut prev_phi = phi_p;
        let mut j = MAX_ITERATIONS;
        while j > 0 {
            if (phi - prev_phi).abs() < EPS_10 {
                break;
            }

            let S = C + e * ((FRAC_PI_4 + (e * phi.sin()).asin() / 2.0).tan().ln());

            prev_phi = phi;
            phi = 2.0 * (S.exp()).atan() - FRAC_PI_2;
            j -= 1;
        }
        if j <= 0 {
            operands.set_xy(i, f64::NAN, f64::NAN);
            continue;
        } else {
            operands.set_xy(i, lam, phi);
            successes += 1;
        }
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 7] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "ellps",  default: Some("GRS80") },
    // TODO: Handle case when R is used.
    // If R is present it takes precedence over ellps
    // OpParameter::Real{key: "R", default: None},

    OpParameter::Real { key: "lon_0",  default: Some(0_f64) },
    OpParameter::Real { key: "lat_0",  default: Some(0_f64) },
    OpParameter::Real { key: "x_0",    default: Some(0_f64) },
    OpParameter::Real { key: "y_0",    default: Some(0_f64) },

    OpParameter::Real { key: "k_0",    default: Some(1_f64) },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.instantiated_as;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    let el = params.ellps(0);
    let e = el.eccentricity();
    let hlf_e = e * 0.5;
    let es = el.eccentricity_squared();
    let a = el.semimajor_axis();

    let k_0 = params.real["k_0"];
    let phi_0 = params.real["lat_0"].to_radians();

    let (sin_phi_0, cos_phi_0) = phi_0.sin_cos();

    let c = (1.0 + (es * cos_phi_0.powi(4) / (1.0 - es))).sqrt();
    let sin_phi_0_p = sin_phi_0 / c;
    let phi_0_p = sin_phi_0_p.asin();
    let cos_phi_0_p = phi_0_p.cos();

    let R = k_0 * a * (1.0 - es).sqrt() / (1.0 - es * sin_phi_0.powi(2));

    let k1 = (FRAC_PI_4 + 0.5 * (sin_phi_0 / c).asin()).tan().ln();
    let k2 = (FRAC_PI_4 + 0.5 * phi_0).tan().ln();
    let k3 = ((1.0 + e * sin_phi_0) / (1.0 - e * sin_phi_0)).ln();
    let K = k1 - c * k2 + c * hlf_e * k3;

    params.real.insert("K", K);
    params.real.insert("R", R);
    params.real.insert("c", c);
    params.real.insert("sin_phi_0_p", sin_phi_0_p);
    params.real.insert("cos_phi_0_p", cos_phi_0_p);

    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));

    Ok(Op {
        descriptor,
        params,
        steps: None,
        id: OpHandle::new(),
    })
}

// ----- Ancillary functions -----------------------------------------------------------

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn somerc_inv() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("somerc", OpConstructor(new));
        let op = ctx.op("somerc lat_0=46.9524055555556 lon_0=7.43958333333333 k_0=1 x_0=2600000 y_0=1200000 ellps=bessel")?;

        let input = [Coor4D::raw(2531098.0, 1167363.0, 452.0, 0.0)];
        let mut operands = input;

        let expected = [Coor4D::raw(
            0.11413236074541264,
            0.814287372550452,
            452.0,
            0.0,
        )];

        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][0], expected[0][0], abs_all <= 1e-9);

        Ok(())
    }

    #[test]
    fn somerc_fwd_and_round_trip() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("somerc", OpConstructor(new));
        let op = ctx.op("somerc lat_0=46.9524055555556 lon_0=7.43958333333333 k_0=1 x_0=2600000 y_0=1200000 ellps=bessel")?;

        let input = [Coor4D::raw(
            0.11413236074541264,
            0.814287372550452,
            452.0,
            0.0,
        )];
        let mut operands = input;
        let expected = [Coor4D::raw(2531098.0, 1167363.0, 452.0, 0.0)];

        ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0][0], expected[0][0], abs_all <= 1e-9);

        // Inv + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][0], input[0][0], abs_all <= 1e-9);

        Ok(())
    }

    #[test]
    fn somerc_el() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("somerc", OpConstructor(new));
        let op = ctx.op("somerc ellps=GRS80")?;

        let input = [
            Coor4D::gis(2., 1., 0., 0.0),
            Coor4D::gis(2., -1., 0., 0.0),
            Coor4D::gis(-2., 1., 0., 0.0),
            Coor4D::gis(-2., -1., 0., 0.0),
        ];

        let mut operands = input;

        let expected = [
            Coor4D::raw(222638.98158654713, 110579.96521824898, 0., 0.0),
            Coor4D::raw(222638.98158654713, -110579.96521825089, 0., 0.0),
            Coor4D::raw(-222638.98158654713, 110579.96521824898, 0., 0.0),
            Coor4D::raw(-222638.98158654713, -110579.96521825089, 0., 0.0),
        ];

        // Forward
        let successes = ctx.apply(op, Fwd, &mut operands)?;

        for i in 0..successes {
            assert_float_eq!(operands[i][0], expected[i][0], abs_all <= 1e-8);
            assert_float_eq!(operands[i][1], expected[i][1], abs_all <= 1e-8);
            assert_float_eq!(operands[i][2], expected[i][2], abs_all <= 1e-8);
            assert_float_eq!(operands[i][3], expected[i][3], abs_all <= 1e-8);
        }

        // Inverse + roundtrip
        let inverse_successes = ctx.apply(op, Inv, &mut operands)?;
        for i in 0..inverse_successes {
            assert_float_eq!(operands[i][0], input[i][0], abs_all <= 1e-4);
            assert_float_eq!(operands[i][1], input[i][1], abs_all <= 1e-4);
            assert_float_eq!(operands[i][2], input[i][2], abs_all <= 1e-4);
            assert_float_eq!(operands[i][3], input[i][3], abs_all <= 1e-4);
        }

        Ok(())
    }
}
