use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn pipeline_fwd(op: &Op, provider: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let mut n = usize::MAX;
    for step in &op.steps[..] {
        n = n.min(step.apply(provider, operands, Direction::Fwd)?);
    }
    Ok(n)
}

// ----- I N V E R S E -----------------------------------------------------------------

fn pipeline_inv(op: &Op, provider: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let mut n = usize::MAX;
    for step in op.steps[..].iter().rev() {
        n = n.min(step.apply(provider, operands, Direction::Inv)?);
    }
    Ok(n)
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
    let definition = &parameters.definition;
    let thesteps = split_into_steps(definition).0;
    let mut steps = Vec::new();

    for step in thesteps {
        let step_parameters = parameters.next(&step);
        steps.push(Op::op(step_parameters, provider)?);
    }

    let params = ParsedParameters::new(parameters, &GAMUT)?;
    let fwd = InnerOp(pipeline_fwd);
    let inv = InnerOp(pipeline_inv);
    let descriptor = OpDescriptor::new(definition, fwd, Some(inv));
    let id = OpHandle::default();
    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

pub fn split_into_steps(definition: &str) -> (Vec<String>, String) {
    let all = definition.replace('\r', "\n").trim().to_string();

    // Collect docstrings and remove plain comments
    let mut trimmed = Vec::<String>::new();
    let mut docstring = Vec::<String>::new();
    for line in all.lines() {
        let line = line.trim();

        // Collect docstrings
        if line.starts_with("##") {
            docstring.push((line.to_string() + "    ")[3..].trim_end().to_string());
            continue;
        }

        // Remove comments
        let line: Vec<&str> = line.trim().split('#').collect();
        if line[0].starts_with('#') {
            continue;
        }
        trimmed.push(line[0].trim().to_string());
    }

    // Finalize the docstring
    let docstring = docstring.join("\n").trim().to_string();

    // Remove superfluous newlines in the comment-trimmed text
    let trimmed = trimmed.join(" ").replace('\n', " ");

    // Generate trimmed steps with elements spearated by a single space,
    // and key-value pairs glued by '=' as in 'key=value'
    let steps: Vec<_> = trimmed.split('|').collect();
    let mut trimmed_steps = Vec::<String>::new();
    for mut step in steps {
        step = step.trim();
        let elements: Vec<_> = step.split_whitespace().collect();
        let joined = elements.join(" ").replace("= ", "=");
        trimmed_steps.push(joined);
    }
    let trimmed_steps = trimmed_steps;
    (trimmed_steps, docstring)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipeline() -> Result<(), Error> {
        let mut prv = Minimal::default();
        let op = prv.op("addone|addone|addone")?;
        let mut data = some_basic_coordinates();

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        let op = prv.op("addone|addone inv|addone")?;
        let mut data = some_basic_coordinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Try to invoke garbage as a pipeline step
        assert!(matches!(
            prv.op("addone|addone|_garbage"),
            Err(Error::NotFound(_, _))
        ));

        Ok(())
    }
}
