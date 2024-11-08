/// Permanent tide systems
use crate::authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;
    let n = operands.len();
    let ellps = op.params.ellps(0);
    let Ok(coefficient) = op.params.real("coefficient") else {
        return successes;
    };

    for i in 0..n {
        let mut coord = operands.get_coord(i);
        let phibar = ellps.latitude_geographic_to_geocentric(coord[1]);
        let s = phibar.sin();
        coord[2] += coefficient * (-0.198) * (1.5 * s * s - 0.5);
        operands.set_coord(i, &coord);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;
    let n = operands.len();
    let ellps = op.params.ellps(0);
    let Ok(coefficient) = op.params.real("coefficient") else {
        return successes;
    };

    for i in 0..n {
        let mut coord = operands.get_coord(i);
        let phibar = ellps.latitude_geographic_to_geocentric(coord[1]);
        let s = phibar.sin();
        coord[2] -= coefficient * (-0.198) * (1.5 * s * s - 0.5);
        operands.set_coord(i, &coord);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 5] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Real { key: "k",     default: Some(0.3) },
    OpParameter::Text { key: "ellps", default: Some("GRS80") },
    OpParameter::Text { key: "from",  default: None },
    OpParameter::Text { key: "to",    default: None }
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let mut op = Op::plain(parameters, InnerOp(fwd), Some(InnerOp(inv)), &GAMUT, ctx)?;
    let k = op.params.real("k")?;

    let Ok(to) = op.params.text("to") else {
        return Err(Error::MissingParam(
            "permtide: must specify 'to=' as exactly one of {'mean', 'zero', 'free'}".to_string(),
        ));
    };
    let Ok(from) = op.params.text("from") else {
        return Err(Error::MissingParam(
            "permtide: must specify 'from=' as exactly one of {'mean', 'zero', 'free'}".to_string(),
        ));
    };

    let coefficient = match (to.as_str(), from.as_str()) {
        ("mean", "mean") => 0.0,
        ("mean", "zero") => 1.0,
        ("mean", "free") => 1.0 + k,
        ("zero", "zero") => 0.0,
        ("zero", "mean") => -1.0,
        ("zero", "free") => k,
        ("free", "free") => 0.0,
        ("free", "mean") => -(1.0 + k),
        ("free", "zero") => -k,
        _ => f64::NAN,
    };

    if coefficient.is_nan() {
        return Err(Error::BadParam(
            "'to=' or 'from='".to_string(),
            "must be one of {'mean', 'zero', 'free'}".to_string(),
        ));
    }

    op.params.real.insert("coefficient", coefficient);
    Ok(op)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn permtide() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        // A test point near Copenhagen
        let pnt = Coor4D::geo(55., 12., 0., 0.);

        // Mean -> zero
        let op = ctx.op("permtide from=mean to=zero ellps=GRS80")?;
        let mut operands = [pnt];
        ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0][2], 0.099407199, abs_all <= 1e-8);
        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][2], pnt[2], abs_all <= 1e-12);

        // Mean -> free
        let op = ctx.op("permtide from=mean to=free ellps=GRS80")?;
        let mut operands = [pnt];
        ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0][2], 0.1292293587824579, abs_all <= 1e-8);
        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][2], pnt[2], abs_all <= 1e-12);

        // Inversion
        let fwd_op = ctx.op("permtide from=mean to=zero ellps=GRS80")?;
        let inv_op = ctx.op("permtide from=zero to=mean ellps=GRS80 inv")?;
        let mut operands = [pnt];
        ctx.apply(fwd_op, Fwd, &mut operands)?;
        let fwd_h = operands[0][2];

        let mut operands = [pnt];
        ctx.apply(inv_op, Fwd, &mut operands)?;
        let inv_h = operands[0][2];
        assert_float_eq!(fwd_h, inv_h, abs_all <= 1e-20);

        // Bad tide system names
        assert!(matches!(
            ctx.op("permtide from=cheese to=zero ellps=GRS80"),
            Err(Error::BadParam(_, _))
        ));

        // Missing tide system names
        assert!(matches!(ctx.op("permtide"), Err(Error::MissingParam(_))));
        assert!(matches!(
            ctx.op("permtide to=zero ellps=GRS80"),
            Err(Error::MissingParam(_))
        ));
        assert!(matches!(
            ctx.op("permtide from=mean ellps=GRS80"),
            Err(Error::MissingParam(_))
        ));
        Ok(())
    }
}
