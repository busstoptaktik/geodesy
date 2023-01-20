//! Lambert azimuthal equal area
use super::*;

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

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 3] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Real { key: "x", default: Some(0f64) },
    OpParameter::Text { key: "convention", default: Some("") },
];

pub fn new(parameters: &RawParameters, Context: &dyn Context) -> Result<Op, Error> {
    Op::plain(parameters, InnerOp(fwd), InnerOp(inv), &GAMUT, Context)
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
