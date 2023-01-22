//! Lambert azimuthal equal area: EPSG method
use super::*;
use std::f64::consts::FRAC_PI_2;
const EPS10: f64 = 1e-10;

// ----- C O M M O N -------------------------------------------------------------------

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    todo!();
    let mut successes = 0_usize;
    for coord in operands {
        successes += 1;
    }

    Ok(successes)
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    todo!();
    let mut successes = 0_usize;
    for coord in operands {
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

    OpParameter::Real { key: "k_0",   default: Some(1_f64) },
    OpParameter::Real { key: "x_0",   default: Some(0_f64) },
    OpParameter::Real { key: "y_0",   default: Some(0_f64) },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    let mut lat_0 = params.lat[0];
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
    let equatoreal = !polar && t < EPS10;
    let oblique = !polar && !equatoreal;
    match (polar, equatoreal, north) {
        (true, _, true) => params.boolean.insert("north_polar"),
        (true, _, false) => params.boolean.insert("south_polar"),
        (_, true, _) => params.boolean.insert("equatoreal"),
        _ => params.boolean.insert("oblique"),
    };

    let sc = lat_0.sin_cos();
    let mut n = sc.0;
    let es = params.ellps[0].eccentricity_squared();
    let e = es.sqrt();
    let qp = qs(sc.0, e);

    /*
    Q->mmf = .5 / (1. - P->es);
    Q->apa = pj_authset(P->es);
    if (nullptr==Q->apa)
        return destructor(P, PROJ_ERR_OTHER /*ENOMEM*/);
    switch (Q->mode) {
    case N_POLE:
    case S_POLE:
        Q->dd = 1.;
        break;
    case EQUIT:
        Q->dd = 1. / (Q->rq = sqrt(.5 * Q->qp));
        Q->xmf = 1.;
        Q->ymf = .5 * Q->qp;
        break;
    case OBLIQ:
        Q->rq = sqrt(.5 * Q->qp);
        sinphi = sin(P->phi0);
        Q->sinb1 = pj_qsfn(sinphi, P->e, P->one_es) / Q->qp;
        Q->cosb1 = sqrt(1. - Q->sinb1 * Q->sinb1);
        Q->dd = cos(P->phi0) / (sqrt(1. - P->es * sinphi * sinphi) *
           Q->rq * Q->cosb1);
        Q->ymf = (Q->xmf = Q->rq) / Q->dd;
        Q->xmf *= Q->dd;
        break;
    }
    P->inv = laea_e_inverse;
    P->fwd = laea_e_forward;
    */

    //Ok(Op::default());
    Err(Error::General("Failure"))
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        // EPSG:1134 - 3 parameter, ED50/WGS84, s = sqrt(27) m
        let op = ctx.op("helmert x=-87 y=-96 z=-120")?;

        let mut operands = [Coord::origin()];

        // Forward
        ctx.apply(op, Fwd, &mut operands)?;
        assert_eq!(operands[0].first(), -87.);
        assert_eq!(operands[0].second(), -96.);
        assert_eq!(operands[0].third(), -120.);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_eq!(operands[0].first(), 0.);
        assert_eq!(operands[0].second(), 0.);
        assert_eq!(operands[0].third(), 0.);
        Ok(())
    }
}
