use crate::inner_op_authoring::*;

// ----- F O R W A R D --------------------------------------------------------------

fn addone_fwd(op: &Op, provider: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = 0;
    for o in operands {
        o[0] += 1.;
        n += 1;
    }
    n
}

// ----- I N V E R S E --------------------------------------------------------------

fn addone_inv(op: &Op, provider: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = 0;
    for o in operands {
        o[0] -= 1.;
        n += 1;
    }
    n
}

// ----- C O N S T R U C T O R ------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
    let def = &parameters.definition;
    let params = ParsedParameters::new(parameters, &GAMUT)?;
    let fwd = InnerOp(addone_fwd);
    let inv = InnerOp(addone_inv);
    let base = Base::new(def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    Ok(Op {
        base,
        params,
        steps,
    })
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn addone() -> Result<(), Error> {
        let provider = provider::Minimal::default();
        let op = Op::new("addone", &provider)?;
        let copenhagen = C::raw(55., 12., 0., 0.);
        let stockholm = C::raw(59., 18., 0., 0.);
        let mut data = [copenhagen, stockholm];
        op.operate(&provider, &mut data, Direction::Fwd);
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        op.operate(&provider, &mut data, Direction::Inv);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        Ok(())
    }
}
