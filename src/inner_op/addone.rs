use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(_op: &Op, _provider: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let mut n = 0;
    for o in operands {
        o[0] += 1.;
        n += 1;
    }
    Ok(n)
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(_op: &Op, _provider: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let mut n = 0;
    for o in operands {
        o[0] -= 1.;
        n += 1;
    }
    Ok(n)
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
    Op::plain(parameters, InnerOp(fwd), InnerOp(inv), &GAMUT, provider)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn addone() -> Result<(), Error> {
        let provider = Minimal::default();
        let op = Op::new("addone", &provider)?;
        let mut data = crate::some_basic_coordinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        op.apply(&provider, &mut data, Direction::Fwd)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        op.apply(&provider, &mut data, Direction::Inv)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        Ok(())
    }
}
