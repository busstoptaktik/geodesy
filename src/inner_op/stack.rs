//! Stack functionality for pipelines (push/pop/swap)
use crate::authoring::*;

// NOTE: roll and drop are not implemented yet
#[rustfmt::skip]
pub const STACK_GAMUT: [OpParameter; 7] = [
    OpParameter::Series  { key: "push", default: Some("") },
    OpParameter::Series  { key: "pop",  default: Some("") },
    OpParameter::Series  { key: "roll", default: Some("") },
    OpParameter::Series  { key: "unroll", default: Some("") },
    OpParameter::Series  { key: "flip", default: Some("") },
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

    if let Ok(flip_args) = params.series("flip") {
        subcommands_given += 1;
        for i in flip_args.iter() {
            if !valid_indices.contains(i) {
                return Err(Error::BadParam("flip".to_string(), i.to_string()));
            }
        }
        params.text.insert("action", "flip".to_string());
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
            || roll_args[0] <= roll_args[1].abs()
        {
            return Err(Error::MissingParam(
                "roll takes exactly two integer parameters, ´(m,n): |n|<=m´".to_string(),
            ));
        }
        params.text.insert("action", "roll".to_string());
    }

    if let Ok(roll_args) = params.series("unroll") {
        subcommands_given += 1;
        if roll_args.len() != 2
            || roll_args[0].fract() != 0.
            || roll_args[1].fract() != 0.
            || roll_args[0] <= roll_args[1].abs()
        {
            return Err(Error::MissingParam(
                "unroll takes exactly two integer parameters, ´(m,n): |n|<=m´".to_string(),
            ));
        }
        params.text.insert("action", "unroll".to_string());
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
            "stack: must specify exactly one of push/pop/roll/swap/unroll/drop".to_string(),
        ));
    }

    // The true action is handled by 'pipeline', so the `InnerOp`s are placeholders
    let descriptor = OpDescriptor::new(def, InnerOp::default(), Some(InnerOp::default()));

    Ok(Op {
        descriptor,
        params,
        steps: None,
        id: OpHandle::new(),
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

    match action.as_str() {
        "push" => {
            let args = params.series_as_usize("push").unwrap();
            stack_push(stack, operands, &args)
        }

        "pop" => {
            let args = params.series_as_usize("pop").unwrap();
            stack_pop(stack, operands, &args)
        }

        "roll" => {
            let args = params.series_as_i64("roll").unwrap();
            stack_roll(stack, operands, &args)
        }

        "unroll" => {
            let mut args = params.series_as_i64("unroll").unwrap();
            args[1] = args[0] - args[1];
            stack_roll(stack, operands, &args)
        }

        "flip" => {
            let args = params.series_as_usize("flip").unwrap();
            stack_flip(stack, operands, &args)
        }

        "swap" => {
            let n = stack.len();
            if n > 1 {
                stack.swap(n - 1, n - 2)
            }
            if n == 0 { 0 } else { stack[0].len() }
        }

        _ => 0,
    }
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

    match action.as_str() {
        // An inverse push is a pop with reversed args
        "push" => {
            let mut args = params.series_as_usize("push").unwrap();
            args.reverse();
            stack_pop(stack, operands, &args)
        }

        // And an inverse pop is a push with reversed args
        "pop" => {
            let mut args = params.series_as_usize("pop").unwrap();
            args.reverse();
            stack_push(stack, operands, &args)
        }

        "roll" => {
            let mut args = params.series_as_i64("roll").unwrap();
            args[1] = args[0] - args[1];
            stack_roll(stack, operands, &args)
        }

        "unroll" => {
            let args = params.series_as_i64("roll").unwrap();
            stack_roll(stack, operands, &args)
        }

        "flip" => {
            let args = params.series_as_usize("flip").unwrap();
            stack_flip(stack, operands, &args)
        }

        // Swap TOS and 2OS
        "swap" => {
            let n = stack.len();
            if n > 1 {
                stack.swap(n - 1, n - 2)
            }
            if n == 0 { 0 } else { stack[0].len() }
        }

        _ => 0,
    }
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
            // args are 1 based so we adjust
            ext[j][i] = coord[args[j] - 1];
        }
    }

    // And push them onto the existing stack
    stack.extend(ext);
    number_of_operands
}

/// Flip the operator and the TOS
fn stack_flip(stack: &mut [Vec<f64>], operands: &mut dyn CoordinateSet, args: &[usize]) -> usize {
    let number_of_flips = args.len();
    let number_of_operands = operands.len();
    let stack_depth = stack.len();

    // In case of underflow, we stomp on all input coordinates
    if stack_depth < number_of_flips {
        warn!("Stack flip underflow in pipeline");
        operands.stomp();
        return 0;
    }

    // Swap the stack elements and their corresponding coordinate elements
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        for j in 0..number_of_flips {
            // args are 1 based so we adjust
            let flip = coord[args[j] - 1];
            let depth = stack_depth - 1 - j;
            coord[args[j] - 1] = stack[depth][i];
            stack[depth][i] = flip;
        }
        operands.set_coord(i, &coord);
    }

    number_of_operands
}

/// roll m,n: On the sub-stack consisting of the m upper elements,
/// roll n elements from the top, to the bottom of the sub-stack.
/// Hence, roll is a "big swap", essentially swapping the n upper
/// elements with the m - n lower.
fn stack_roll(stack: &mut Vec<Vec<f64>>, operands: &mut dyn CoordinateSet, args: &[i64]) -> usize {
    let m = args[0].abs();
    let mut n = args[1];
    let depth = stack.len();

    // Negative n: count the number of rolled elements from the bottom,
    // i.e. roll 3,-2 = roll 3,1
    n = if n < 0 { m + n } else { n };

    // The remaining becomes simpler if m, n and depth are all usize
    let m = m as usize;
    let n = n as usize;

    if m > depth {
        warn!("Roll too deep");
        operands.stomp();
        return 0;
    }

    for _ in 0..n {
        let e = stack.pop().unwrap();
        stack.insert(depth - m, e);
    }

    operands.len()
}

/// Pop elements from the stack into elements of a CoordinateSet
fn stack_pop(stack: &mut Vec<Vec<f64>>, operands: &mut dyn CoordinateSet, args: &[usize]) -> usize {
    let number_of_pops = args.len();
    let number_of_operands = operands.len();
    let stack_depth = stack.len();

    // In case of underflow, we stomp on all input coordinates
    if stack_depth < number_of_pops {
        warn!("Stack underflow in pipeline");
        operands.stomp();
        return 0;
    }

    // Remove the correct number of elements and obtain a reversed version.
    let mut ext = Vec::with_capacity(number_of_pops);
    for _ in args {
        ext.push(stack.pop().unwrap());
    }

    // Inject the required stack elements into the proper
    // positions of the coordinate elements
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        for j in 0..number_of_pops {
            // args are 1 based so we adjust
            coord[args[j] - 1] = ext[j][i];
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

        // ----- Four tests of the actual functionality -----

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

        // 4: Test the `roll` subcommand
        let op = ctx.op("stack push=1,1,1,2,1,3,1,4 | stack roll=8,2 | stack pop=1,2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 13.);
        assert_eq!(data[0][1], 11.);

        // Then we do the inverse. We must, however, redo, since the push-pop asymmetry
        // would otherwise wreak havoc:

        // Just calling apply in the inverse direction leads to underflow:
        assert_eq!(0, ctx.apply(op, Inv, &mut data)?);

        // Instead, we must substitute (m,n) with (m,m-n)
        let mut data = master_data.clone();
        let op = ctx.op("stack push=1,2,3,4,1,2,3,4 | stack roll=8,6 | stack pop=1,2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 11.);

        let mut data = master_data.clone();
        let op = ctx.op("stack push=1,2,3,4,1,2,3,4 | stack roll=3,2 | stack pop=1,2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 14.);

        let mut data = master_data.clone();
        let op = ctx.op("stack push=1,2,3,4 | stack roll=3,-2 | stack pop=2,1")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 13.);

        // Roundrip roll
        let mut data = master_data.clone();
        let op = ctx.op("stack push=1,2,3,4 | stack roll=3,2 | stack roll=3,1 | stack pop=1,2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 14.);
        assert_eq!(data[0][1], 13.);

        // Roundrip roll using the unroll syntactic sugar
        let mut data = master_data.clone();
        let op =
            ctx.op("stack push=1,2,3,4 | stack roll=3,2 | stack unroll=3,2 | stack pop=1,2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 14.);
        assert_eq!(data[0][1], 13.);

        Ok(())
    }

    #[test]
    fn stack_examples_from_rumination_002() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let master_data = vec![Coor4D([1., 2., 3., 4.])];

        // Roll
        let op = ctx.op("stack push=1,2,3,4 | stack roll=3,2 | stack pop=4,3,2,1")?;
        let mut data = master_data.clone();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 3., 4., 2.]);

        let op = ctx.op("stack push=1,2,3,4 | stack roll=3,-2 | stack pop=4,3,2,1")?;
        let mut data = master_data.clone();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 4., 2., 3.]);

        let op = ctx.op("stack push=1,2,3,4 | stack roll=3,2 | stack pop=4,3,2,1")?;
        let mut data = master_data.clone();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 3., 4., 2.]);
        let op = ctx.op("stack push=1,2,3,4 | stack roll=3,1 | stack pop=4,3,2,1")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 2., 3., 4.]);

        // Unroll
        let op = ctx.op("stack push=1,2,3,4 | stack unroll=3,2 | stack pop=4,3,2,1")?;
        let mut data = master_data.clone();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 4., 2., 3.]);

        let op = ctx.op("stack push=1,2,3,4 | stack unroll=3,-2 | stack pop=4,3,2,1")?;
        let mut data = master_data.clone();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 3., 4., 2.]);

        let op = ctx.op("stack push=1,2,3,4 | stack unroll=3,2 | stack pop=4,3,2,1")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 2., 3., 4.]);

        let op = ctx.op("stack push=1,2,3,4 | stack roll=3,2 | stack pop=4,3,2,1")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 3., 4., 2.]);

        let op = ctx.op("stack push=1,2,3,4 | stack unroll=3,2 | stack pop=4,3,2,1")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [1., 2., 3., 4.]);

        let op = ctx.op("stack push=1,2,3,4 | helmert x=4 y=4 z=4 | stack flip=1,2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [4., 3., 7., 4.]);

        let mut data = master_data.clone();
        let op = ctx.op(
            "stack push=1,2,3,4 | helmert translation=4,4,4 | stack flip=1,2 | stack flip=1,2",
        )?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0].0, [5., 6., 7., 4.]);

        Ok(())
    }
}
