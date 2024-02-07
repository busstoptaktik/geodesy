/// Stack functionality for pipelines (push/pop/swap)
use crate::authoring::*;

// NOTE: roll and drop are not implemented yet
#[rustfmt::skip]
pub const STACK_GAMUT: [OpParameter; 5] = [
    OpParameter::Series  { key: "push", default: Some("") },
    OpParameter::Series  { key: "pop",  default: Some("") },
    OpParameter::Series  { key: "roll", default: Some("") },
    OpParameter::Flag    { key: "swap" },
    OpParameter::Flag    { key: "drop" },
];

/// Construct a new stack operator. Check the syntax and semantics
pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &STACK_GAMUT)?;

    // The subcommands (push, pop, roll, swap, drop) are mutually exclusive,
    // so we count them and err if more than one is given
    let mut subcommands_given: usize = 0;

    // The arguments to push and pop are specified as a series, but Geodesy
    // series are represented internally as a Vec<f64>, so the valid
    // coordinate indices (1..4, i.e. the max coordinate dimensionality)
    // are also stored as f64, to simplify comparison
    let valid_indices = [1., 2., 3., 4.];

    // Now do a sanity check for all subcommands

    if let Ok(push_args) = params.series("push") {
        subcommands_given += 1;
        for i in push_args.iter() {
            if !valid_indices.contains(i) {
                return Err(Error::BadParam("push".to_string(), i.to_string()));
            }
        }
        params.text.insert("action", "push".to_string());
    }

    if let Ok(pop_args) = params.series("pop") {
        subcommands_given += 1;
        for i in pop_args.iter() {
            if !valid_indices.contains(i) {
                return Err(Error::BadParam("pop".to_string(), i.to_string()));
            }
        }
        params.text.insert("action", "pop".to_string());
    }

    if let Ok(roll_args) = params.series("roll") {
        subcommands_given += 1;
        if roll_args.len() != 2
            || roll_args[0].fract() != 0.
            || roll_args[1].fract() != 0.
            || roll_args[0] < roll_args[1].abs()
        {
            return Err(Error::MissingParam(
                "roll takes exactly two integer parameters".to_string(),
            ));
        }
        params.text.insert("action", "roll".to_string());
    }

    if params.boolean("swap") {
        subcommands_given += 1;
        params.text.insert("action", "swap".to_string());
    }

    if params.boolean("drop") {
        subcommands_given += 1;
        params.text.insert("action", "drop".to_string());
    }

    if subcommands_given != 1 {
        return Err(Error::MissingParam(
            "stack: must specify exactly one of push/pop/roll/swap/drop".to_string(),
        ));
    }

    // The true action is handled by 'pipeline', so the `InnerOp`s are placeholders
    let descriptor = OpDescriptor::new(def, InnerOp::default(), Some(InnerOp::default()));
    let steps = Vec::new();
    let id = OpHandle::new();

    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

/// Called by `pipeline_fwd` to execute stack operations in forward mode
pub(super) fn stack_fwd(
    stack: &mut Vec<Vec<f64>>,
    operands: &mut dyn CoordinateSet,
    params: &ParsedParameters,
) -> usize {
    let Some(action) = params.text.get("action") else {
        return 0;
    };

    let successes = match action.as_str() {
        "push" => {
            // Turn f64 dimensions 1-4 into usize indices 0-3
            let args: Vec<usize> = params
                .series("push")
                .unwrap()
                .iter()
                .map(|i| *i as usize - 1)
                .collect();
            stack_push(stack, operands, &args)
        }

        "pop" => {
            // Turn f64 dimensions 1-4 into usize indices 0-3
            let args: Vec<usize> = params
                .series("pop")
                .unwrap()
                .iter()
                .map(|i| *i as usize - 1)
                .collect();
            stack_pop(stack, operands, &args)
        }

        "swap" => {
            let n = stack.len();
            if n > 1 {
                stack.swap(n - 1, n - 2)
            }
            if n == 0 {
                0
            } else {
                stack[0].len()
            }
        }

        _ => 0,
    };

    successes
}

/// Called by `pipeline_inv` to execute stack operations in inverse mode.
/// Inverse mode has two major differences from forward: push and pop switches
/// functionality, and their argument order swaps direction
pub(super) fn stack_inv(
    stack: &mut Vec<Vec<f64>>,
    operands: &mut dyn CoordinateSet,
    params: &ParsedParameters,
) -> usize {
    let Some(action) = params.text.get("action") else {
        return 0;
    };

    let successes = match action.as_str() {
        "push" => {
            // Turn f64 dimensions 1-4 into **reversed** usize indices 0-3  ******
            let args: Vec<usize> = params
                .series("push")
                .unwrap()
                .iter()
                .rev()
                .map(|i| *i as usize - 1)
                .collect();
            stack_pop(stack, operands, &args)
        }

        "pop" => {
            // Turn f64 dimensions 1-4 into **reversed** usize indices 0-3
            let args: Vec<usize> = params
                .series("pop")
                .unwrap()
                .iter()
                .rev()
                .map(|i| *i as usize - 1)
                .collect();
            return stack_push(stack, operands, &args);
        }

        "swap" => {
            let n = stack.len();
            if n > 1 {
                stack.swap(n - 1, n - 2)
            }
            if n == 0 {
                0
            } else {
                stack[0].len()
            }
        }

        _ => 0,
    };

    successes
}

/// Push elements from a CoordinateSet onto the stack
fn stack_push(
    stack: &mut Vec<Vec<f64>>,
    operands: &mut dyn CoordinateSet,
    args: &[usize],
) -> usize {
    let number_of_pushes = args.len();
    let number_of_operands = operands.len();

    // Make room for the new sets of elements to be pushed on to the stack
    let mut ext = vec![vec![0f64; number_of_operands]; number_of_pushes];

    // Extract the coordinate elements into the new stack elements
    for i in 0..number_of_operands {
        let coord = operands.get_coord(i);
        for j in 0..number_of_pushes {
            ext[j][i] = coord[args[j]];
        }
    }

    // And push them onto the existing stack
    stack.extend(ext);
    number_of_operands
}

/// Pop elements from the stack into elements of a CoordinateSet
fn stack_pop(stack: &mut Vec<Vec<f64>>, operands: &mut dyn CoordinateSet, args: &[usize]) -> usize {
    let number_of_pops = args.len();
    let number_of_operands = operands.len();
    let stack_depth = stack.len();

    // In case of underflow, we stomp on all input coordinates
    if stack_depth < number_of_pops {
        warn!("Stack underflow in pipeline");
        let nanny = Coor4D::nan();
        for i in 0..number_of_operands {
            operands.set_coord(i, &nanny);
        }
        return 0;
    }

    // Remove the correct number of elements and obtain a reversed version.
    // Incidentally, this is both the easiest way to obtain the popped
    // subset, and the easiest way to make the top-of-stack (i.e. the
    // element first popped) have the index 0, which makes the 'for j...'
    // loop below slightly more straightforward
    let mut ext = Vec::with_capacity(number_of_pops);
    for _ in args {
        ext.push(stack.pop().unwrap());
    }

    // Extract the required stack elements into the proper
    // positions of the coordinate elements
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        for j in 0..number_of_pops {
            coord[args[j]] = ext[j][i];
        }
        operands.set_coord(i, &coord);
    }

    number_of_operands
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let master_data = vec![Coor4D([11., 12., 13., 14.]), Coor4D([21., 22., 23., 24.])];

        // ----- Three initial sanity checks -----

        // Yes, we may push any number of elements onto the stack. And
        // no, I do not see any immediate actual uses for this, but
        // disallowing it would require more code, more complicated code,
        // all for no gain other than stomping on a potential future use
        // case
        assert!(ctx.op("stack push=2,2,1,1,3,3,4,4,4,4,4,4,4").is_ok());

        // But we must not have more than one subcommand for each
        // stack operator
        assert!(ctx.op("stack push=2,2,1,1 pop=1,1,2").is_err());

        // ...while in two consecutive steps it works as it should
        // (the push/pop-imbalance is not an error. Again a potential
        // use case, that would require code complication to disallow)
        assert!(ctx.op("stack push=2,2,1,1 | stack pop=1,1,2").is_ok());

        // ----- Three tests of the actual functionality -----

        let mut data = master_data.clone();

        // 1: Swap the first and second coordinate dimensions by a push/pop dance

        // Explanation:
        // The first step pushes the first and second coordinate of
        // the operand onto the stack **in that order**.
        // Hence,
        // - The second coordinate becomes top-of-stack (TOS), as
        //   it is the last one pushed, and
        // - The first coordinate becomes second-of-stack (2OS)
        //
        // The second step pops the TOS into the first coordinate of
        // the operand, and the 2OS into the first coordinate of the
        // operand
        let op = ctx.op("stack push=1,2|stack pop=1,2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[1][1], 21.);

        // Then we do the inverse
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0], master_data[0]);
        assert_eq!(data[1], master_data[1]);

        // 2: An exercise in reverse thinking - doing the inverse call first

        let op = ctx.op("stack push=2,1 | stack pop=2,1")?;
        // The inverse call should in effect execute "stack push=1,2 | stack pop=1,2"
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[1][1], 21.);

        // Then we do the inverse
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0], master_data[0]);
        assert_eq!(data[1], master_data[1]);

        // 3: Test the `swap` subcommand

        let op = ctx.op("stack push=2,1 | stack swap | stack pop=1,2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[1][1], 21.);

        // Then we do the inverse
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0], master_data[0]);
        assert_eq!(data[1], master_data[1]);

        Ok(())
    }
}
