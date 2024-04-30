//! Lambert azimuthal equal area: EPSG coordinate operation method 9820, implemented
//! following [IOGP, 2019](crate::Bibliography::Iogp19), pp. 78-80
use crate::authoring::*;

use std::f64::consts::FRAC_PI_2;
const EPS10: f64 = 1e-10;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    // Oblique aspect: [IOGP, 2019](crate::Bibliography::Iogp19), pp. 78-80
    let Ok(xi_0) = op.params.real("xi_0") else {
        return 0;
    };
    let Ok(qp) = op.params.real("qp") else {
        return 0;
    };
    let Ok(rq) = op.params.real("rq") else {
        return 0;
    };
    let Ok(d) = op.params.real("d") else { return 0 };

    let oblique = op.params.boolean("oblique");
    let north_polar = op.params.boolean("north_polar");
    let south_polar = op.params.boolean("south_polar");

    let lon_0 = op.params.real("lon_0").unwrap_or(0.).to_radians();
    let x_0 = op.params.real("x_0").unwrap_or(0.);
    let y_0 = op.params.real("y_0").unwrap_or(0.);
    let ellps = op.params.ellps(0);
    let e = ellps.eccentricity();
    let a = ellps.semimajor_axis();

    let (sin_xi_0, cos_xi_0) = xi_0.sin_cos();

    let mut successes = 0_usize;
    let n = operands.len();

    // The polar aspects are fairly simple
    if north_polar || south_polar {
        let sign = if north_polar { -1.0 } else { 1.0 };
        for i in 0..n {
            let (lon, lat) = operands.xy(i);
            let (sin_lon, cos_lon) = (lon - lon_0).sin_cos();

            let q = ancillary::qs(lat.sin(), e);
            let rho = a * (qp + sign * q).sqrt();

            let easting = x_0 + rho * sin_lon;
            let northing = y_0 + sign * rho * cos_lon;
            operands.set_xy(i, easting, northing);
            successes += 1;
        }
        return successes;
    }

    // Either equatorial or oblique aspects
    for i in 0..n {
        let (lon, lat) = operands.xy(i);
        let (sin_lon, cos_lon) = (lon - lon_0).sin_cos();

        // Authalic latitude, 𝜉
        let xi = (ancillary::qs(lat.sin(), e) / qp).asin();
        let (sin_xi, cos_xi) = xi.sin_cos();

        let b = if oblique {
            let factor = 1.0 + sin_xi_0 * sin_xi + (cos_xi_0 * cos_xi * cos_lon);
            rq * (2.0 / factor).sqrt()
        } else {
            1.0
        };

        let easting = x_0 + (b * d) * (cos_xi * sin_lon);
        let northing = y_0 + (b / d) * (cos_xi_0 * sin_xi - sin_xi_0 * cos_xi * cos_lon);
        operands.set_xy(i, easting, northing);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    // Oblique aspect: [IOGP, 2019](crate::Bibliography::Iogp19), pp. 78-80
    let Ok(xi_0) = op.params.real("xi_0") else {
        return 0;
    };
    let Ok(rq) = op.params.real("rq") else {
        return 0;
    };
    let Ok(d) = op.params.real("d") else { return 0 };
    let Ok(authalic) = op.params.fourier_coefficients("authalic") else {
        return 0;
    };

    let north_polar = op.params.boolean("north_polar");
    let south_polar = op.params.boolean("south_polar");

    let lon_0 = op.params.real("lon_0").unwrap_or(0.).to_radians();
    let lat_0 = op.params.real("lat_0").unwrap_or(0.).to_radians();
    let x_0 = op.params.real("x_0").unwrap_or(0.);
    let y_0 = op.params.real("y_0").unwrap_or(0.);

    let ellps = op.params.ellps(0);
    let a = ellps.semimajor_axis();
    let es = ellps.eccentricity_squared();
    let e = es.sqrt();

    let (sin_xi_0, cos_xi_0) = xi_0.sin_cos();

    let mut successes = 0_usize;
    let n = operands.len();

    // The polar aspects are not quite as simple as in the forward case
    if north_polar || south_polar {
        let sign = if north_polar { -1.0 } else { 1.0 };
        for i in 0..n {
            let (x, y) = operands.xy(i);
            let rho = (x - x_0).hypot(y - y_0);

            // The authalic latitude is a bit convoluted
            let denom = a * a * (1.0 - ((1.0 - es) / (2.0 * e)) * ((1.0 - e) / (1.0 + e)).ln());
            let xi = (-sign) * (1.0 - rho * rho / denom);

            let lon = lon_0 + (x - x_0).atan2(sign * (y - y_0));
            let lat = ellps.latitude_authalic_to_geographic(xi, &authalic);
            operands.set_xy(i, lon, lat);
            successes += 1;
        }
        return successes;
    }

    // Either equatorial or oblique aspects
    for i in 0..n {
        let (x, y) = operands.xy(i);
        let rho = ((x - x_0) / d).hypot(d * (y - y_0));

        // A bit of reality hardening ported from the PROJ implementation
        if rho < EPS10 {
            operands.set_xy(i, lon_0, lat_0);
            successes += 1;
            continue;
        }

        // Another case of PROJ reality hardening
        let asin_argument = 0.5 * rho / rq;
        if asin_argument.abs() > 1.0 {
            debug!("LAEA: ({x}, {y}) outside domain");
            operands.set_xy(i, f64::NAN, f64::NAN);
            continue;
        }

        let c = 2.0 * asin_argument.asin();
        let (sin_c, cos_c) = c.sin_cos();

        // The authalic latitude, 𝜉
        let xi = (cos_c * sin_xi_0 + (d * (y - y_0) * sin_c * cos_xi_0) / rho).asin();

        let lat = ellps.latitude_authalic_to_geographic(xi, &authalic);

        let num = (x - x_0) * sin_c;
        let denom = d * rho * cos_xi_0 * cos_c - d * d * (y - y_0) * sin_xi_0 * sin_c;
        let lon = num.atan2(denom) + lon_0;
        operands.set_xy(i, lon, lat);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 6] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") },

    OpParameter::Real { key: "lat_0", default: Some(0_f64) },
    OpParameter::Real { key: "lon_0", default: Some(0_f64) },

    OpParameter::Real { key: "x_0",   default: Some(0_f64) },
    OpParameter::Real { key: "y_0",   default: Some(0_f64) },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    let lat_0 = params.real("lat_0").unwrap_or(0.).to_radians();

    if lat_0.is_nan() {
        warn!("LAEA: Bad central latitude!");
        return Err(Error::BadParam("lat_0".to_string(), def.clone()));
    }

    let t = lat_0.abs();
    if t > FRAC_PI_2 + EPS10 {
        warn!("LAEA: Bad central latitude!");
        return Err(Error::BadParam("lat_0".to_string(), def.clone()));
    }

    let polar = (t - FRAC_PI_2).abs() < EPS10;
    let north = polar && (t > 0.0);
    let equatorial = !polar && t < EPS10;
    let oblique = !polar && !equatorial;
    match (polar, equatorial, north) {
        (true, _, true) => params.boolean.insert("north_polar"),
        (true, _, false) => params.boolean.insert("south_polar"),
        (_, true, _) => params.boolean.insert("equatorial"),
        _ => params.boolean.insert("oblique"),
    };

    // --- Precompute some latitude invariant factors ---

    let ellps = params.ellps(0);
    let a = ellps.semimajor_axis();
    let es = ellps.eccentricity_squared();
    let e = es.sqrt();
    let (sin_phi_0, cos_phi_0) = lat_0.sin_cos();

    // qs for the central parallel
    let q0 = ancillary::qs(sin_phi_0, e);
    // qs for the North Pole
    let qp = ancillary::qs(1.0, e);
    // Authalic latitude of the central parallel - 𝛽₀ in the IOGP text
    let xi_0 = (q0 / qp).asin();
    // Rq in the IOGP text
    let rq = a * (0.5 * qp).sqrt();
    // D in the IOGP text
    let d = if oblique {
        a * (cos_phi_0 / (1.0 - es * sin_phi_0 * sin_phi_0).sqrt()) / (rq * xi_0.cos())
    } else if equatorial {
        rq.recip()
    } else {
        a
    };

    params.real.insert("xi_0", xi_0);
    params.real.insert("q0", q0);
    params.real.insert("qp", qp);
    params.real.insert("rq", rq);
    params.real.insert("d", d);

    let authalic = ellps.coefficients_for_authalic_latitude_computations();
    params.fourier_coefficients.insert("authalic", authalic);

    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));
    let steps = Vec::<Op>::new();
    let id = OpHandle::new();
    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn laea_oblique() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        // ETRS-LAEA grid definition
        let op = ctx.op("laea ellps=GRS80 lat_0=52 lon_0=10  x_0=4321000 y_0=3210000")?;

        // The test point from IOGP
        let p = Coor2D::geo(50.0, 5.0);
        let geo = [p];
        let p = Coor2D::raw(3962799.45, 2999718.85);
        let projected = [p];

        let mut operands = geo;

        // Forward
        ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0].0, projected[0].0, abs_all <= 0.01);
        assert!((operands[0][0] - 3962799.45).abs() < 0.01);
        assert!((operands[0][1] - 2999718.85).abs() < 0.01);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][0].to_degrees() - 5.0).abs() < 1e-12);
        assert!((operands[0][1].to_degrees() - 50.).abs() < 1e-12);

        let p = Coor4D::raw(1e30, 1e30, 0.0, 0.0);
        let mut operands = [p];
        ctx.apply(op, Inv, &mut operands)?;
        assert!(operands[0][0].is_nan());

        // Missing test points for the polar aspects

        Ok(())
    }

    // Test the "if rho < EPS10" branch in the inverse case.
    // From this issue: https://github.com/busstoptaktik/geodesy/issues/89
    // reported by @maximkaaa
    #[test]
    fn origin() {
        let mut ctx = Minimal::new();
        let op = ctx
            .op("laea lon_0=10 lat_0=52 x_0=4321000 y_0=3210000")
            .unwrap();
        let mut data = [Coor2D::geo(52.0, 10.0)];
        let clone = data;
        ctx.apply(op, Fwd, &mut data).unwrap();
        ctx.apply(op, Inv, &mut data).unwrap();
        assert_eq!(data, clone);
    }
}
