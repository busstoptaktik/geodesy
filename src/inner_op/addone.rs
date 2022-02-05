use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn addone_fwd(_op: &Op, _provider: &dyn Provider, operands: &mut [Coord]) -> usize {
    let mut n = 0;
    for o in operands {
        o[0] += 1.;
        n += 1;
    }
    n
}

// ----- I N V E R S E -----------------------------------------------------------------

fn addone_inv(_op: &Op, _provider: &dyn Provider, operands: &mut [Coord]) -> usize {
    let mut n = 0;
    for o in operands {
        o[0] -= 1.;
        n += 1;
    }
    n
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, _provider: &dyn Provider) -> Result<Op, Error> {
    let def = &parameters.definition;
    let params = ParsedParameters::new(parameters, &GAMUT)?;
    let fwd = InnerOp(addone_fwd);
    let inv = InnerOp(addone_inv);
    let descriptor = OpDescriptor::new(def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    Ok(Op {
        descriptor,
        params,
        steps,
    })
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
        op.apply(&provider, &mut data, Direction::Fwd);
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        op.apply(&provider, &mut data, Direction::Inv);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        Ok(())
    }
}
