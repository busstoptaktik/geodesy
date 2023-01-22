//! Lambert azimuthal equal area: EPSG coordinate operation method 9820, implemented
//! following the IOGP Geomatics Guidance Note Number 7, part 2, pp.78--80
use super::*;
use std::f64::consts::FRAC_PI_2;
const EPS10: f64 = 1e-10;

// ----- C O M M O N -------------------------------------------------------------------

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let Ok(xi_0) = op.params.real("xi_0") else { return Ok(0) };
    let Ok(qp)   = op.params.real("qp")   else { return Ok(0) };
    let Ok(rq)   = op.params.real("rq")   else { return Ok(0) };
    let Ok(d)    = op.params.real("d")    else { return Ok(0) };

    let lon_0 = op.params.lon(0);
    let x_0 = op.params.x(0);
    let y_0 = op.params.y(0);
    let ellps = op.params.ellps(0);
    let e = ellps.eccentricity();

    let (sin_xi_0, cos_xi_0) = xi_0.sin_cos();

    let mut successes = 0_usize;
    for coord in operands {
        let lon = coord[0];
        let lat = coord[1];
        let (sin_lon, cos_lon) = (lon - lon_0).sin_cos();

        // Authalic latitude, 𝜉
        let xi = (qs(lat.sin(), e) / qp).asin();

        let (sin_xi, cos_xi) = xi.sin_cos();
        let factor = 1.0 + sin_xi_0 * sin_xi + (cos_xi_0 * cos_xi * cos_lon);
        let b = rq * (2.0 / factor).sqrt();

        // Easting
        coord[0] = x_0 + (b * d) * (cos_xi * sin_lon);

        // Northing
        coord[1] = y_0 + (b / d) * (cos_xi_0 * sin_xi - sin_xi_0 * cos_xi * cos_lon);

        successes += 1;
    }

    Ok(successes)
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let mut successes = 0_usize;
    for _coord in operands {
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

    let lat_0 = params.lat[0];
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

    // --- Precompute some latitude invariant factors ---

    let a = params.ellps[0].semimajor_axis();
    let es = params.ellps[0].eccentricity_squared();
    let e = es.sqrt();
    let (sin_phi_0, cos_phi_0) = lat_0.sin_cos();

    // qs for the central parallel
    let q0 = qs(sin_phi_0, e);
    // qs for the North Pole
    let qp = qs(1.0, e);
    // Authalic latitude of the central parallel - 𝛽₀ in the IOGP text
    let xi_0 = (q0 / qp).asin();
    // Rq in the IOGP text
    let rq = a * (0.5 * qp).sqrt();
    // D in the IOGP text
    let d = a * (cos_phi_0 / (1.0 - es * sin_phi_0 * sin_phi_0).sqrt()) / (rq * xi_0.cos());

    params.real.insert("xi_0", xi_0);
    params.real.insert("q0", q0);
    params.real.insert("qp", qp);
    params.real.insert("rq", rq);
    params.real.insert("d", d);

    /*
    PROJ snippet:
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

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laea_oblique() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("laea ellps=GRS80 lat_0=52 lon_0=10  x_0=4321000 y_0=3210000")?;

        let mut operands = [Coord::origin(), Coord::geo(50.0, 5.0, 0.0, 0.0)];

        // Forward
        ctx.apply(op, Fwd, &mut operands)?;
        println!("{:#?}", operands);
        assert!(1 == 2);

        // assert_eq!(operands[0].first(), -87.);
        // assert_eq!(operands[0].second(), -96.);
        // assert_eq!(operands[0].third(), -120.);
        // // Inverse + roundtrip
        // ctx.apply(op, Inv, &mut operands)?;
        // assert_eq!(operands[0].first(), 0.);
        // assert_eq!(operands[0].second(), 0.);
        // assert_eq!(operands[0].third(), 0.);
        Ok(())
    }
}
