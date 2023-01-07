/// The no-operation. Does nothing, and is good at it
use super::*;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(_op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    Ok(operands.len())
}

// ----- I N V E R S E --------------------------------------------------------------

fn inv(_op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    Ok(operands.len())
}

// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 0] = [
];

pub fn new(parameters: &RawParameters, provider: &dyn Context) -> Result<Op, Error> {
    Op::plain(parameters, InnerOp(fwd), InnerOp(inv), &GAMUT, provider)
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    const GDA94: Coord = Coord([-4052051.7643, 4212836.2017, -2545106.0245, 0.0]);

    #[test]
    fn no_change() -> Result<(), Error> {
        let provider = Minimal::default();
        let op = Op::new("noop", &provider)?;

        // EPSG:1134 - 3 parameter, ED50/WGS84, s = sqrt(27) m
        let mut operands = [GDA94];

        // Forward
        op.apply(&provider, &mut operands, Fwd)?;
        assert_eq!(operands[0], GDA94);

        // Inverse + roundtrip
        op.apply(&provider, &mut operands, Inv)?;
        assert_eq!(operands[0], GDA94);
        Ok(())
    }
}
