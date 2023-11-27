use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let n = operands.len();

    // We default to order=1,2,3,4, so if order is not given, we are done already
    let Ok(order) = op.params.series("order") else {
        return n;
    };

    let dimensionality = order.len();

    let mut pos = [0_usize, 1, 2, 3];
    let mut sgn = [1., 1., 1., 1.];
    for (index, value) in order.iter().enumerate() {
        pos[index] = (value.abs() - 1.) as usize;
        sgn[index] = 1_f64.copysign(*value);
    }

    let mut successes = 0_usize;
    for i in 0..n {
        let inp = operands.get_coord(i);
        let mut out = inp;
        for index in 0..dimensionality {
            out[index] = inp[pos[index]] * sgn[index];
        }
        operands.set_coord(i, &out);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let n = operands.len();

    // We default to order=1,2,3,4, so if order is not given, we are done already
    let Ok(order) = op.params.series("order") else {
        return n;
    };

    let dimensionality = order.len();

    let mut pos = [0_usize, 1, 2, 3];
    let mut sgn = [1., 1., 1., 1.];
    for (index, value) in order.iter().enumerate() {
        pos[index] = (value.abs() - 1.) as usize;
        sgn[index] = 1_f64.copysign(*value);
    }

    let mut successes = 0_usize;
    for i in 0..n {
        let inp = operands.get_coord(i);
        let mut out = inp;
        for index in 0..dimensionality {
            out[pos[index]] = inp[index] * sgn[index];
        }
        operands.set_coord(i, &out);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 2] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Series { key: "order", default: Some("1,2,3,4") },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let op = Op::plain(parameters, InnerOp(fwd), Some(InnerOp(inv)), &GAMUT, ctx)?;

    // We default to order=1,2,3,4, so if order is not given, all is OK
    let Ok(order) = op.params.series("order") else {
        return Ok(op);
    };

    if order.len() > 4 {
        return Err(Error::BadParam(
            "order".to_string(),
            "More than 4 indices given".to_string(),
        ));
    }

    // While the Series type returns a Vec<f64>, the elements must be convertible to i64
    // and further to (x.abs() as usize) for use as array indices
    for &o in order {
        let i = o as i64;
        if (i as f64) != o || i == 0 || (i.unsigned_abs() as usize) > order.len() {
            return Err(Error::BadParam("order".to_string(), o.to_string()));
        }
    }

    // PROJ does not allow duplicate axes, presumably for a well considered reason,
    // so neither do we
    for o in 1_u64..5 {
        if order.iter().filter(|x| (x.abs() as u64) == o).count() > 1 {
            return Err(Error::BadParam(
                "order".to_string(),
                "duplicate axis specified".to_string(),
            ));
        }
    }

    Ok(op)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_dim() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("axisswap order=2,1,-3,-4")?;

        let mut operands = [Coor4D([1., 2., 3., 4.])];

        // Forward
        ctx.apply(op, Fwd, &mut operands)?;
        assert_eq!(operands[0][0], 2.);
        assert_eq!(operands[0][1], 1.);
        assert_eq!(operands[0][2], -3.);
        assert_eq!(operands[0][3], -4.);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_eq!(operands[0][0], 1.);
        assert_eq!(operands[0][1], 2.);
        assert_eq!(operands[0][2], 3.);
        assert_eq!(operands[0][3], 4.);

        Ok(())
    }

    #[test]
    fn default_order() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("axisswap")?;

        let mut operands = [Coor4D([1., 2., 3., 4.])];

        // Forward
        ctx.apply(op, Fwd, &mut operands)?;
        assert_eq!(operands[0][0], 1.);
        assert_eq!(operands[0][1], 2.);
        assert_eq!(operands[0][2], 3.);
        assert_eq!(operands[0][3], 4.);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_eq!(operands[0][0], 1.);
        assert_eq!(operands[0][1], 2.);
        assert_eq!(operands[0][2], 3.);
        assert_eq!(operands[0][3], 4.);

        Ok(())
    }

    #[test]
    fn two_dim() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        let op = ctx.op("axisswap order=2,-1")?;
        let mut operands = [Coor4D([1., 2., 3., 4.])];

        // Forward
        ctx.apply(op, Fwd, &mut operands)?;
        assert_eq!(operands[0][0], 2.);
        assert_eq!(operands[0][1], -1.);
        assert_eq!(operands[0][2], 3.);
        assert_eq!(operands[0][3], 4.);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_eq!(operands[0][0], 1.);
        assert_eq!(operands[0][1], 2.);
        assert_eq!(operands[0][2], 3.);
        assert_eq!(operands[0][3], 4.);
        Ok(())
    }

    #[test]
    fn bad_parameters() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        // Too many indices
        let op = ctx.op("axisswap order=4,4,4,2,-1");
        assert!(matches!(op, Err(Error::BadParam(_, _))));

        // Repeated indices
        let op = ctx.op("axisswap order=4,-4,2,-1");
        assert!(matches!(op, Err(Error::BadParam(_, _))));

        // Index exceeding dimensionality
        let op = ctx.op("axisswap order=2,3");
        assert!(matches!(op, Err(Error::BadParam(_, _))));

        // Missing indices ('order' becomes a flag)
        let op = ctx.op("axisswap order");
        assert!(matches!(op, Err(Error::BadParam(_, _))));

        // Missing all args: axisswap succeeds and becomes a no-op
        let op = ctx.op("axisswap");
        assert!(op.is_ok());

        Ok(())
    }
}
