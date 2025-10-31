//! Oblique Mercator
//! Following IOGP Publication 373-7-2 – Geomatics Guidance Note number 7, part 2 – September 2019
//!
use crate::authoring::*;
use std::f64::consts::FRAC_PI_2;
use std::f64::consts::FRAC_PI_4;

// ----- F O R W A R D -----------------------------------------------------------------

#[allow(non_snake_case)]
fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);
    let es = ellps.eccentricity_squared();
    let e = es.sqrt();

    let kc = op.params.k(0);

    let FE = op.params.x(0);
    let FN = op.params.y(0);
    let Ec = FE;
    let Nc = FN;

    let latc = op.params.real["latc"].to_radians();
    let lonc = op.params.real["lonc"].to_radians();

    let alpha = op.params.real["alpha"];
    let ninety = alpha == 90_f64;
    let alpha = alpha.to_radians();

    // Detect the Laborde case by a missing gamma_c
    let mut gamma_c = op.params.real["gamma_c"];
    let laborde = gamma_c.is_nan();
    gamma_c = gamma_c.to_radians();

    // Discern between Hotine variant A and B cases, and the Laborde
    // case, which we currently approximate by Hotine with gamma_c = alpha
    let mut variant = op.params.boolean("variant");
    if laborde {
        variant = true;
        gamma_c = alpha;
    }

    // Remove mutability
    let gamma_c = gamma_c;

    // A horrible mess of constants. But by-and-large, just a transcription of
    // the material from Guidance Note 7-2.
    let (s, c) = latc.sin_cos();
    let B = (1_f64 + c.powi(4) * ellps.second_eccentricity_squared()).sqrt();
    let A = ellps.semimajor_axis() * B * kc * (1_f64 - es).sqrt() / (1.0 - es * s * s);
    let t0 = (FRAC_PI_4 - latc / 2.0).tan() / ((1.0 - e * s) / (1.0 + e * s)).powf(e / 2.0);
    let D = B * (1.0 - es).sqrt() / (c * (1.0 - es * s * s).sqrt());
    let DD = if D < 1.0 { 0.0 } else { (D * D - 1.0).sqrt() };
    let F = D + DD * latc.signum();
    let H = F * t0.powf(B);
    let G = (F - 1.0 / F) / 2.0;
    let gamma_0 = (alpha.sin() / D).asin();
    let lambda_0 = lonc - (G * gamma_0.tan()).asin() / B;

    // (uc, vc): Intermediate coordinates of the projection center
    // let vc = 0.0;
    let uc = if ninety {
        A * (lonc - lambda_0)
    } else {
        (A / B) * DD.atan2(alpha.cos()) * latc.signum()
    };

    let (s0, c0) = gamma_0.sin_cos();
    let (sc, cc) = gamma_c.sin_cos();

    let mut successes = 0_usize;

    for i in 0..operands.len() {
        let (lon, lat) = operands.xy(i);
        let slat = lat.sin();

        let t = (FRAC_PI_4 - lat / 2.0).tan() / ((1.0 - e * slat) / (1.0 + e * slat)).powf(e / 2.0);
        let Q = H / t.powf(B);
        let S = (Q - 1.0 / Q) / 2.0;
        let T = (Q + 1.0 / Q) / 2.0;
        let V = (B * (lon - lambda_0)).sin();
        let U = (S * s0 - V * c0) / T;
        let v = A * ((1.0 - U) / (1.0 + U)).ln() / (2.0 * B);

        let cblon = (B * (lon - lambda_0)).cos();

        // Variant A
        if !variant {
            let u = A * (S * c0 + V * s0).atan2(cblon) / B;
            let x = v * cc + u * sc + FE;
            let y = u * cc - v * sc + FN;
            operands.set_xy(i, x, y);
            successes += 1;
            continue;
        }

        // Variant B and/or Laborde

        // The special case
        if ninety {
            let u = if lon == lambda_0 {
                0.0
            } else {
                A * (S * c0 + V * s0).atan2(cblon) / B - uc.copysign(latc) * (lonc - lon).signum()
            };
            let x = v * cc + u * sc + Ec;
            let y = u * cc - v * sc + Nc;
            operands.set_xy(i, x, y);
            successes += 1;
            continue;
        }

        // The general case
        let u = A * (S * c0 + V * s0).atan2(cblon) / B - uc.copysign(latc);
        let x = v * cc + u * sc + Ec;
        let y = u * cc - v * sc + Nc;
        operands.set_xy(i, x, y);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

#[allow(non_snake_case)]
fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);
    let es = ellps.eccentricity_squared();
    let e = es.sqrt();

    let kc = op.params.k(0);

    let FE = op.params.x(0);
    let FN = op.params.y(0);

    let latc = op.params.real["latc"].to_radians();
    let lonc = op.params.real["lonc"].to_radians();

    let alpha = op.params.real["alpha"];
    let ninety = alpha == 90_f64;
    let alpha = alpha.to_radians();

    // Detect the Laborde case by a missing gamma_c
    let gamma_c = op.params.real["gamma_c"];
    let laborde = gamma_c.is_nan();

    // Discern between Hotine variant A and B cases, and the Laborde
    // case, which we currently approximate by Hotine with gamma_c = alpha
    let gamma_c = if laborde { alpha } else { gamma_c.to_radians() };
    let variant = op.params.boolean("variant") || laborde;

    // A horrible mess of constants. But by-and-large, just a transcription of
    // the material from Guidance Note 7-2.
    let (s, c) = latc.sin_cos();
    let B = (1_f64 + c.powi(4) * ellps.second_eccentricity_squared()).sqrt();
    let A = ellps.semimajor_axis() * B * kc * (1_f64 - es).sqrt() / (1.0 - es * s * s);
    let t0 = (FRAC_PI_4 - latc / 2.0).tan() / ((1.0 - e * s) / (1.0 + e * s)).powf(e / 2.0);
    let D = B * (1.0 - es).sqrt() / (c * (1.0 - es * s * s).sqrt());
    let DD = if D < 1.0 { 0.0 } else { (D * D - 1.0).sqrt() };
    let F = D + DD * latc.signum();
    let H = F * t0.powf(B);
    let G = (F - 1.0 / F) / 2.0;
    let gamma_0 = (alpha.sin() / D).asin();
    let lambda_0 = lonc - (G * gamma_0.tan()).asin() / B;

    // (uc, vc): Intermediate coordinates of the projection center
    // let vc = 0.0;
    let uc = if ninety {
        A * (lonc - lambda_0)
    } else {
        (A / B) * DD.atan2(alpha.cos()) * latc.signum()
    };

    let (s0, c0) = gamma_0.sin_cos();
    let (sc, cc) = gamma_c.sin_cos();
    let offset = if variant { uc.copysign(latc) } else { 0.0 };

    let mut successes = 0_usize;
    for i in 0..operands.len() {
        let (E, N) = operands.xy(i);

        let v = (E - FE) * cc - (N - FN) * sc;
        let u = (N - FN) * cc + (E - FE) * sc + offset;

        let Q = (-B * v / A).exp();
        let S = (Q - 1.0 / Q) / 2.0;
        let T = (Q + 1.0 / Q) / 2.0;
        let V = (B * u / A).sin();
        let U = (V * c0 + S * s0) / T;
        let t = (H / ((1.0 + U) / (1.0 - U)).sqrt()).powf(1.0 / B);

        let chi = FRAC_PI_2 - 2.0 * t.atan();

        // Fourier coefficients (the outer factor of *es* moved to the summation step)
        let f = [
            (1.0 / 2.0 + es * (5.0 / 24.0 + es * (1.0 / 12.0 + es * 13.0 / 360.0))),
            es * (7.0 / 48.0 + es * (29.0 / 240.0 + es * 811.0 / 11520.0)),
            es * es * (7.0 / 120.0 + es * 81.0 / 1120.0),
            es * es * es * 4279.0 / 161280.0,
        ];

        // Fourier sine components
        let s = [
            (2.0 * chi).sin(),
            (4.0 * chi).sin(),
            (6.0 * chi).sin(),
            (8.0 * chi).sin(),
        ];

        let lat = chi + es * (f[0] * s[0] + f[1] * s[1] + f[2] * s[2] + f[3] * s[3]);
        let lon = lambda_0 - (S * c0 - V * s0).atan2((B * u / A).cos()) / B;
        operands.set_xy(i, lon, lat);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 10] = [
    OpParameter::Flag { key: "inv" },

    // Default to Hotine Variant A
    OpParameter::Flag { key: "variant" }, // Set if Hotine variant B

    OpParameter::Text { key: "ellps",  default: Some("GRS80") },

    // Projection center. Note: PROJ uses (lat_0, lonc).
    OpParameter::Real { key: "latc",  default: Some(0_f64) },
    OpParameter::Real { key: "lonc",  default: Some(0_f64) },

    // Azimuth of the initial line
    OpParameter::Real { key: "alpha",  default: Some(f64::NAN) },

    // Angle from the rectified grid to the oblique grid (Hotine only)
    OpParameter::Real { key: "gamma_c",  default: Some(f64::NAN) },

    // False nothing/easting - at natural origin (Hotine variant A)
    // or projection center (Hotine variant B)
    OpParameter::Real { key: "x_0",    default: Some(0_f64) },
    OpParameter::Real { key: "y_0",    default: Some(0_f64) },

    // Scale factor on the initial line
    OpParameter::Real { key: "k_0",    default: Some(1_f64) },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.instantiated_as;
    let params = ParsedParameters::new(parameters, &GAMUT)?;
    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));

    Ok(Op {
        descriptor,
        params,
        steps: None,
        id: OpHandle::new(),
    })
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn omerc() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "
            omerc ellps=evrstSS variant
            x_0=590476.87 y_0=442857.65
            latc=4 lonc=115
            k_0=0.99984 alpha=53:18:56.9537 gamma_c=53:07:48.3685
        ";
        // k_0=0.99984 alpha=53.3158204722 gamma_c=53.1301023611
        let op = ctx.op(definition)?;

        // Validation value from EPSG
        let geo = [Coor2D::geo(5.3872535833, 115.8055054444)];
        let projected = [Coor2D::raw(679245.7281740266, 596562.7774687681)];

        // Forward
        let mut operands = geo;

        assert_eq!(1, ctx.apply(op, Fwd, &mut operands)?);
        for i in 0..operands.len() {
            assert_float_eq!(operands[i].0, projected[i].0, abs_all <= 1e-9);
        }

        // Roundtrip
        assert_eq!(1, ctx.apply(op, Inv, &mut operands)?);
        for i in 0..operands.len() {
            assert_float_eq!(operands[i].0, geo[i].0, abs_all <= 1e-9);
        }

        // Forward
        let mut operands = geo;

        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 1e-9);
        }

        // Roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 1e-9);
        }

        Ok(())
    }
}
