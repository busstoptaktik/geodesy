use super::*;
use std::collections::BTreeSet;

// ----- F O R W A R D -----------------------------------------------------------------

fn pipeline_fwd(op: &Op, provider: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let mut stack = Vec::new();
    let mut n = usize::MAX;
    for step in &op.steps {
        if step.params.boolean("omit_fwd") {
            continue;
        }
        let m = match step.params.name.as_str() {
            "push" => do_the_push(&mut stack, operands, &step.params.boolean),
            "pop" => do_the_pop(&mut stack, operands, &step.params.boolean),
            _ => step.apply(provider, operands, Direction::Fwd)?,
        };
        n = n.min(m);
    }

    // In case every step has been marked as `omit_fwd`
    if n == usize::MAX {
        n = operands.len();
    }
    Ok(n)
}

// ----- I N V E R S E -----------------------------------------------------------------

fn pipeline_inv(op: &Op, provider: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let mut stack = Vec::new();
    let mut n = usize::MAX;
    for step in op.steps.iter().rev() {
        if step.params.boolean("omit_inv") {
            continue;
        }
        // Note: Under inverse invocation "push" calls pop and vice versa
        let m = match step.params.name.as_str() {
            "push" => do_the_pop(&mut stack, operands, &step.params.boolean),
            "pop" => do_the_push(&mut stack, operands, &step.params.boolean),
            _ => step.apply(provider, operands, Direction::Inv)?,
        };
        n = n.min(m);
    }

    // In case every step has been marked as `omit_inv`
    if n == usize::MAX {
        n = operands.len();
    }
    Ok(n)
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, provider: &dyn Context) -> Result<Op, Error> {
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
    let id = OpHandle::new();
    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// The push and pop constructors are extremely simple, since the pipeline operator
// does all the hard work. Essentially, they are just flags telling pipeline
// what to do, given their provided options

// Yes - push and pop do not accept the inv flag although they are both invertible.
// If you want to invert a push, then use a pop (and vice versa).
#[rustfmt::skip]
pub const PUSH_POP_GAMUT: [OpParameter; 4] = [
    OpParameter::Flag { key: "v_1" },
    OpParameter::Flag { key: "v_2" },
    OpParameter::Flag { key: "v_3" },
    OpParameter::Flag { key: "v_4" },
];

pub fn push(parameters: &RawParameters, _prv: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let params = ParsedParameters::new(parameters, &PUSH_POP_GAMUT)?;

    let descriptor = OpDescriptor::new(
        def,
        InnerOp(noop_placeholder),
        Some(InnerOp(noop_placeholder)),
    );
    let steps = Vec::new();
    let id = OpHandle::new();

    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

pub fn pop(parameters: &RawParameters, _prv: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let params = ParsedParameters::new(parameters, &PUSH_POP_GAMUT)?;

    let descriptor = OpDescriptor::new(
        def,
        InnerOp(noop_placeholder),
        Some(InnerOp(noop_placeholder)),
    );
    let steps = Vec::new();
    let id = OpHandle::new();

    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// ----- H E L P E R S -----------------------------------------------------------------

fn do_the_push(
    stack: &mut Vec<Vec<f64>>,
    operands: &mut [Coord],
    flags: &BTreeSet<&'static str>,
) -> usize {
    const ELEMENTS: [&str; 4] = ["v_1", "v_2", "v_3", "v_4"];
    for i in [0, 1, 2, 3] {
        if !flags.contains(ELEMENTS[i]) {
            continue;
        }
        // Extract the i'th coordinate from all operands
        let all: Vec<f64> = operands.iter().map(|x| x[i]).collect();
        stack.push(all);
    }
    operands.len()
}

fn do_the_pop(
    stack: &mut Vec<Vec<f64>>,
    operands: &mut [Coord],
    flags: &BTreeSet<&'static str>,
) -> usize {
    const ELEMENTS: [&str; 4] = ["v_4", "v_3", "v_2", "v_1"];
    for i in [0, 1, 2, 3] {
        if !flags.contains(ELEMENTS[i]) {
            continue;
        }

        // Stack underflow?
        if stack.is_empty() {
            for op in operands {
                op[3 - i] = f64::NAN;
            }
            return 0;
        }

        // Insert the top-of-stack elements into the i'th coordinate of all operands
        let v = stack.pop().unwrap();
        for j in 0..operands.len() {
            operands[j][3 - i] = v[j]
        }
    }
    operands.len()
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
    // and key-value pairs glued by '=' as in
    //     key1=value1 key2=value2
    // as opposed to e.g.
    //     key1= value1            key2    =value2
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

    #[test]
    fn push_pop() -> Result<(), Error> {
        let mut prv = Minimal::default();
        let mut data = some_basic_coordinates();

        // First we swap lat, lon by doing two independent pops
        let op = prv.op("push v_2 v_1|addone|pop v_1|pop v_2")?;
        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 55.);

        // While popping both at once does not make any difference: In
        // case of more than one push/pop argument, push happens in
        // 1234-order, while pop happens in 4321-order, so a
        // "push all, pop all" pair is a noop: The order of operator
        // options is insignificant, so the 1234/4321 order is, in principle
        // arbitrary, but seleted with the noop-characteristicum in mind.
        let op = prv.op("push v_1 v_2|pop v_1 v_2")?;
        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 55.);

        // Underflow the stack - get 0 successes
        let op = prv.op("push v_1 v_2|pop v_2 v_1 v_3")?;
        assert_eq!(0, prv.apply(op, Fwd, &mut data)?);
        assert!(data[0][0].is_nan());
        assert_eq!(data[0][2], 55.);

        // Check inversion
        let op = prv.op("push v_1 v_2|pop v_2 v_1 v_3")?;
        let mut data = some_basic_coordinates();
        assert_eq!(2, prv.apply(op, Inv, &mut data)?);
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 0.);

        // Check omit_fwd
        let op = prv.op("push v_1 v_2|pop v_2 v_1 v_3 omit_fwd")?;
        let mut data = some_basic_coordinates();
        assert_eq!(2, prv.apply(op, Fwd, &mut data)?);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[0][1], 12.);
        assert_eq!(2, prv.apply(op, Inv, &mut data)?);
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 0.);

        // Check omit_inv
        let op = prv.op("push v_1 v_2 v_3 omit_inv|pop v_1 v_2")?;
        let mut data = some_basic_coordinates();
        assert_eq!(2, prv.apply(op, Inv, &mut data)?);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[0][1], 12.);

        Ok(())
    }
}
