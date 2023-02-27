use crate::operator_authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(_op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let n = operands.len();
    let mut successes = 0;
    for i in 0..n {
        let mut o = operands.get_coord(i);
        o[0] += 1.;
        successes += 1;
        operands.set_coord(i, &o);
    }
    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(_op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let n = operands.len();
    let mut successes = 0;
    for i in 0..n {
        let mut o = operands.get_coord(i);
        o[0] -= 1.;
        successes += 1;
        operands.set_coord(i, &o);
    }
    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    Op::plain(parameters, InnerOp(fwd), InnerOp(inv), &GAMUT, ctx)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn addone() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("addone")?;
        let mut data = some_basic_coordinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        Ok(())
    }
}
