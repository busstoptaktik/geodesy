/// Deprecated version of the stack functionality for pipelines
/// DO NOT USE THIS. Use "stack push=...", "stack pop=..." etc.
use crate::authoring::*;
use std::collections::BTreeSet;

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

pub fn push(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.instantiated_as;
    let params = ParsedParameters::new(parameters, &PUSH_POP_GAMUT)?;

    let descriptor = OpDescriptor::new(def, InnerOp::default(), Some(InnerOp::default()));

    Ok(Op {
        descriptor,
        params,
        steps: None,
    })
}

pub fn pop(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.instantiated_as;
    let params = ParsedParameters::new(parameters, &PUSH_POP_GAMUT)?;

    let descriptor = OpDescriptor::new(def, InnerOp::default(), Some(InnerOp::default()));

    Ok(Op {
        descriptor,
        params,
        steps: None,
    })
}

pub(super) fn do_the_push(
    stack: &mut Vec<Vec<f64>>,
    operands: &mut dyn CoordinateSet,
    flags: &BTreeSet<&'static str>,
) -> usize {
    let n = operands.len();
    const ELEMENTS: [&str; 4] = ["v_1", "v_2", "v_3", "v_4"];
    for j in [0, 1, 2, 3] {
        if !flags.contains(ELEMENTS[j]) {
            continue;
        }

        let mut all = Vec::with_capacity(n);
        for i in 0..n {
            all.push(operands.get_coord(i)[j]);
        }
        stack.push(all);
    }
    operands.len()
}

pub(super) fn do_the_pop(
    stack: &mut Vec<Vec<f64>>,
    operands: &mut dyn CoordinateSet,
    flags: &BTreeSet<&'static str>,
) -> usize {
    let n = operands.len();
    const ELEMENTS: [&str; 4] = ["v_4", "v_3", "v_2", "v_1"];
    for j in [0, 1, 2, 3] {
        if !flags.contains(ELEMENTS[j]) {
            continue;
        }

        // Stack underflow?
        if stack.is_empty() {
            for i in 0..n {
                let mut op = operands.get_coord(i);
                op[3 - j] = f64::NAN;
                operands.set_coord(i, &op);
            }
            warn!("Stack underflow in pipeline");
            return 0;
        }

        // Insert the top-of-stack elements into the j'th coordinate of all operands
        let v = stack.pop().unwrap();
        for (i, value) in v.iter().enumerate() {
            let mut op = operands.get_coord(i);
            op[3 - j] = *value;
            operands.set_coord(i, &op);
        }
    }
    operands.len()
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let mut data = crate::test_data::coor3d();

        // First we swap lat, lon by doing two independent pops
        let op = ctx.op("push v_2 v_1|addone|pop v_1|pop v_2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 55.);

        // While popping both at once does not make any difference: In
        // case of more than one push/pop argument, push happens in
        // 1234-order, while pop happens in 4321-order, so a
        // "push all, pop all" pair is a noop: The order of operator
        // options is insignificant, so the 1234/4321 order is, in principle
        // arbitrary, but selected with the noop-characteristics in mind.
        let op = ctx.op("push v_1 v_2|pop v_1 v_2")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 55.);

        // Underflow the stack - get 0 successes
        let op = ctx.op("push v_1 v_2|pop v_2 v_1 v_3")?;
        assert_eq!(0, ctx.apply(op, Fwd, &mut data)?);
        assert!(data[0][0].is_nan());
        assert_eq!(data[0][2], 55.);

        // Check inversion
        let op = ctx.op("push v_1 v_2|pop v_2 v_1 v_3")?;
        let mut data = crate::test_data::coor3d();
        assert_eq!(2, ctx.apply(op, Inv, &mut data)?);
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 0.);

        // Check omit_fwd
        let op = ctx.op("push v_1 v_2|pop v_2 v_1 v_3 omit_fwd")?;
        let mut data = crate::test_data::coor3d();
        assert_eq!(2, ctx.apply(op, Fwd, &mut data)?);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[0][1], 12.);
        assert_eq!(2, ctx.apply(op, Inv, &mut data)?);
        assert_eq!(data[0][0], 12.);
        assert_eq!(data[0][1], 0.);

        // Check omit_inv
        let op = ctx.op("push v_1 v_2 v_3 omit_inv|pop v_1 v_2")?;
        let mut data = crate::test_data::coor3d();
        assert_eq!(2, ctx.apply(op, Inv, &mut data)?);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[0][1], 12.);

        Ok(())
    }
}
