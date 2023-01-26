//! Lambert Conformal Conic
use super::*;
use std::f64::consts::FRAC_PI_2;

const EPS10: f64 = 1e-10;

// ----- F O R W A R D -----------------------------------------------------------------

// Forward Lambert conformal conic, following the PROJ implementation,
// cf.  https://proj.org/operations/projections/lcc.html
fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> usize {
    let a = op.params.ellps[0].semimajor_axis();
    let e = op.params.ellps[0].eccentricity();
    let lon_0 = op.params.lon[0];
    let k_0 = op.params.k[0];
    let x_0 = op.params.x[0];
    let y_0 = op.params.y[0];
    let Ok(n) = op.params.real("n") else { return 0 };
    let Ok(c) = op.params.real("c") else { return 0 };
    let Ok(rho0) = op.params.real("rho0") else { return 0 };
    let mut successes = 0_usize;

    for coord in operands {
        let lam = coord[0] - lon_0;
        let phi = coord[1];
        let mut rho = 0.;

        // Close to one of the poles?
        if (phi.abs() - FRAC_PI_2).abs() < EPS10 {
            if phi * n <= 0. {
                *coord = Coord::nan();
                continue;
            }
        } else {
            rho = c * crate::math::ts(phi.sin_cos(), e).powf(n);
        }
        let sc = (lam * n).sin_cos();
        coord[0] = a * k_0 * rho * sc.0 + x_0;
        coord[1] = a * k_0 * (rho0 - rho * sc.1) + y_0;
        successes += 1;
    }
    successes
}

// ----- I N V E R S E -----------------------------------------------------------------
fn inv(op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> usize {
    let a = op.params.ellps[0].semimajor_axis();
    let e = op.params.ellps[0].eccentricity();
    let lon_0 = op.params.lon[0];
    let k_0 = op.params.k[0];
    let x_0 = op.params.x[0];
    let y_0 = op.params.y[0];
    let Ok(n) = op.params.real("n") else { return 0 };
    let Ok(c) = op.params.real("c") else { return 0 };
    let Ok(rho0) = op.params.real("rho0") else { return 0 };
    let mut successes = 0_usize;

    for coord in operands {
        let mut x = (coord[0] - x_0) / (a * k_0);
        let mut y = rho0 - (coord[1] - y_0) / (a * k_0);

        let mut rho = x.hypot(y);

        // On one of the poles
        if rho == 0. {
            coord[0] = 0.;
            coord[1] = FRAC_PI_2.copysign(n);
            successes += 1;
            continue;
        }

        // Standard parallel on the southern hemisphere?
        if n < 0. {
            rho = -rho;
            x = -x;
            y = -y;
        }

        let ts0 = (rho / c).powf(1. / n);
        let phi = crate::math::pj_phi2(ts0, e);
        if phi.is_infinite() || phi.is_nan() {
            *coord = Coord::nan();
            continue;
        }
        coord[0] = x.atan2(y) / n + lon_0;
        coord[1] = phi;
        successes += 1;
    }
    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 9] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") },

    OpParameter::Real { key: "lat_1", default: Some(0_f64) },
    OpParameter::Real { key: "lat_2", default: Some(f64::NAN) },
    OpParameter::Real { key: "lat_0", default: Some(f64::NAN) },
    OpParameter::Real { key: "lon_0", default: Some(0_f64) },

    OpParameter::Real { key: "k_0",   default: Some(1_f64) },
    OpParameter::Real { key: "x_0",   default: Some(0_f64) },
    OpParameter::Real { key: "y_0",   default: Some(0_f64) },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;
    if params.lat[2].is_nan() {
        params.lat[2] = params.lat[1];
    }

    let phi1 = params.lat[1];
    let mut phi2 = params.lat[2];
    if phi2.is_nan() {
        phi2 = phi1;
    }
    params.lat[2] = phi2;

    let mut lat_0 = params.lat[0];
    if lat_0.is_nan() {
        lat_0 = 0.;
        if (phi1 - phi2).abs() < EPS10 {
            lat_0 = phi1;
        }
    }

    let sc = phi1.sin_cos();
    let mut n = sc.0;
    let e = params.ellps[0].eccentricity();
    let es = params.ellps[0].eccentricity_squared();

    if (phi1 + phi2).abs() < EPS10 {
        return Err(Error::General(
            "Lcc: Invalid value for lat_1 and lat_2: |lat_1 + lat_2| should be > 0",
        ));
    }
    if sc.1.abs() < EPS10 || phi1.abs() >= FRAC_PI_2 {
        return Err(Error::General(
            "Lcc: Invalid value for lat_1: |lat_1| should be < 90°",
        ));
    }
    if phi2.cos().abs() < EPS10 || phi2.abs() >= FRAC_PI_2 {
        return Err(Error::General(
            "Lcc: Invalid value for lat_2: |lat_2| should be < 90°",
        ));
    }

    // Snyder (1982) eq. 12-15
    let m1 = crate::math::pj_msfn(sc, es);

    // Snyder (1982) eq. 7-10: exp(-𝜓)
    let ml1 = crate::math::ts(sc, e);

    // Secant case?
    if (phi1 - phi2).abs() >= EPS10 {
        let sc = phi2.sin_cos();
        n = (m1 / crate::math::pj_msfn(sc, es)).ln();
        if n == 0. {
            return Err(Error::General("Lcc: Invalid value for eccentricity"));
        }
        let ml2 = crate::math::ts(sc, e);
        let denom = (ml1 / ml2).ln();
        if denom == 0. {
            return Err(Error::General("Lcc: Invalid value for eccentricity"));
        }
        n /= denom;
    }

    let c = m1 * ml1.powf(-n) / n;
    let mut rho0 = 0.;
    if (lat_0.abs() - FRAC_PI_2).abs() > EPS10 {
        rho0 = c * crate::math::ts(lat_0.sin_cos(), e).powf(n);
    }

    params.real.insert("c", c);
    params.real.insert("n", n);
    params.real.insert("rho0", rho0);
    params.lat[0] = lat_0;

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
    #[test]
    fn one_standard_parallel() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "lcc lat_1=57 lon_0=12";
        let op = ctx.op(definition)?;

        // Validation values from PROJ:
        //     echo 12 55 0 0 | cct -d18 proj=lcc lat_1=57 lon_0=12  -- | clip
        //     echo 10 55 0 0 | cct -d18 proj=lcc lat_1=57 lon_0=12  -- | clip
        //     echo 14 59 0 0 | cct -d18 proj=lcc lat_1=57 lon_0=12  -- | clip

        let geo = [
            Coord::geo(55., 12., 0., 0.),
            Coord::geo(55., 10., 0., 0.),
            Coord::geo(59., 14., 0., 0.),
        ];

        let projected = [
            Coord::raw(-0.000000000101829246, -222728.122307816054672003, 0., 0.),
            Coord::raw(-128046.4724386522429995, -220853.7001605064142495, 0., 0.),
            Coord::raw(115005.41456620067765471, 224484.5143763388914522, 0., 0.),
        ];

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 2e-9);
        }

        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 1e-9);
        }
        Ok(())
    }

    #[test]
    fn two_standard_parallels() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "lcc lat_1=33 lat_2=45 lon_0=10";
        let op = ctx.op(definition)?;

        // Validation value from PROJ:
        // echo 12 40 0 0 | cct -d12 proj=lcc lat_1=33 lat_2=45 lon_0=10 -- | clip
        let geo = [Coord::geo(40., 12., 0., 0.)];
        let projected = [Coord::raw(
            169863.026093938301,
            4735925.219292450696,
            0.,
            0.,
        )];

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 9e-9);
        }

        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 1e-9);
        }
        Ok(())
    }

    #[test]
    fn one_standard_parallel_and_latitudinal_offset() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "lcc lat_1=39 lat_0=35 lon_0=10";
        let op = ctx.op(definition)?;

        // Validation value from PROJ:
        // echo 12 40 0 0 | cct -d12 proj=lcc lat_1=39 lat_0=35 lon_0=10 -- | clip
        let geo = [Coord::geo(40., 12., 0., 0.)];
        let projected = [Coord::raw(170800.011728740647, 557172.361112929415, 0., 0.)];

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 2e-9);
        }

        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 2e-9);
        }
        Ok(())
    }

    #[test]
    fn two_standard_parallels_and_latitudinal_offset() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "lcc lat_1=33 lat_2=45 lat_0=35 lon_0=10";
        let op = ctx.op(definition)?;

        // Validation value from PROJ:
        // echo 12 40 0 0 | cct -d12 proj=lcc lat_1=33 lat_2=45 lat_0=35 lon_0=10 -- | clip
        let geo = [Coord::geo(40., 12., 0., 0.)];
        let projected = [Coord::raw(169863.026093938359, 554155.440793916583, 0., 0.)];

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 2e-9);
        }

        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 1e-9);
        }
        Ok(())
    }

    #[test]
    fn two_sp_lat_offset_xy_offset() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "lcc lat_1=33 lat_2=45 lat_0=35 lon_0=10 x_0=12345 y_0=67890";
        let op = ctx.op(definition)?;

        // Validation value from PROJ:
        // echo 12 40 0 0 | cct -d12 proj=lcc lat_1=33 lat_2=45 lat_0=35 lon_0=10  x_0=12345 y_0=67890 -- | clip
        let geo = [Coord::geo(40., 12., 0., 0.)];
        let projected = [Coord::raw(182208.026093938301, 622045.440793916583, 0., 0.)];

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 2e-9);
        }

        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 1e-9);
        }
        Ok(())
    }

    #[test]
    fn two_sp_lat_offset_xy_offset_scaling() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "lcc lat_1=33 lat_2=45 lat_0=35 lon_0=10 x_0=12345 y_0=67890 k_0=0.99";
        let op = ctx.op(definition)?;

        // Validation value from PROJ:
        // echo 12 40 0 0 | cct -d12 proj=lcc lat_1=33 lat_2=45 lat_0=35 lon_0=10  x_0=12345 y_0=67890 k_0=0.99 -- | clip
        let geo = [Coord::geo(40., 12., 0., 0.)];
        let projected = [Coord::raw(180509.395832998911, 616503.886385977501, 0., 0.)];

        let mut operands = geo.clone();
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 2e-9);
        }

        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 1e-9);
        }
        Ok(())
    }
}
