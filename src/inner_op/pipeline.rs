use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn pipeline_fwd(op: &Op, provider: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = usize::MAX;
    for step in &op.steps[..] {
        n = n.min(step.apply(provider, operands, Direction::Fwd));
    }
    n
}

// ----- I N V E R S E -----------------------------------------------------------------

fn pipeline_inv(op: &Op, provider: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = usize::MAX;
    for step in op.steps[..].iter().rev() {
        n = n.min(step.apply(provider, operands, Direction::Inv));
    }
    n
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
    let definition = &parameters.definition;
    let thesteps = etc::split_into_steps(definition).0;
    let mut steps = Vec::new();

    for step in thesteps {
        let step_parameters = parameters.next(&step);
        steps.push(Op::op(step_parameters, provider)?);
    }

    let params = ParsedParameters::new(parameters, &GAMUT)?;
    let fwd = InnerOp(pipeline_fwd);
    let inv = InnerOp(pipeline_inv);
    let descriptor = OpDescriptor::new(definition, fwd, Some(inv));
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
    fn pipeline() -> Result<(), Error> {
        let provider = Minimal::default();
        let op = Op::new("addone|addone|addone", &provider)?;
        let mut data = etc::some_basic_coordinates();
        op.apply(&provider, &mut data, Direction::Fwd);
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);
        op.apply(&provider, &mut data, Direction::Inv);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        let op = Op::new("addone|addone inv|addone", &provider)?;
        dbg!(&op);
        let mut data = etc::some_basic_coordinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        op.apply(&provider, &mut data, Direction::Fwd);
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        op.apply(&provider, &mut data, Direction::Inv);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Try to invoke garbage as a pipeline step
        assert!(matches!(
            Op::new("addone|addone|_garbage", &provider),
            Err(Error::NotFound(_, _))
        ));

        Ok(())
    }
}
