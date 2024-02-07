use super::pushpop::{do_the_pop, do_the_push};
use super::stack::{stack_fwd, stack_inv};
use crate::authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn pipeline_fwd(op: &Op, ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut stack = Vec::new();
    let mut n = usize::MAX;
    for step in &op.steps {
        if step.params.boolean("omit_fwd") {
            continue;
        }
        let m = match step.params.name.as_str() {
            "push" => do_the_push(&mut stack, operands, &step.params.boolean),
            "pop" => do_the_pop(&mut stack, operands, &step.params.boolean),
            "stack" => stack_fwd(&mut stack, operands, &step.params),
            _ => step.apply(ctx, operands, Fwd),
        };
        n = n.min(m);
    }

    // In case every step has been marked as `omit_fwd`
    if n == usize::MAX {
        n = operands.len();
    }
    n
}

// ----- I N V E R S E -----------------------------------------------------------------

fn pipeline_inv(op: &Op, ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
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
            "stack" => stack_inv(&mut stack, operands, &step.params),
            _ => step.apply(ctx, operands, Inv),
        };
        n = n.min(m);
    }

    // In case every step has been marked as `omit_inv`
    if n == usize::MAX {
        n = operands.len();
    }
    n
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let definition = &parameters.definition;
    let thesteps = definition.split_into_steps().0;
    let mut steps = Vec::new();

    for step in thesteps {
        let step_parameters = parameters.next(&step);
        steps.push(Op::op(step_parameters, ctx)?);
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

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipeline() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("addone|addone|addone")?;
        let mut data = some_basic_coor2dinates();

        assert_eq!(2, ctx.apply(op, Fwd, &mut data)?);
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        let op = ctx.op("addone|addone inv|addone")?;
        let mut data = some_basic_coor2dinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Try to invoke garbage as a pipeline step
        assert!(matches!(
            ctx.op("addone|addone|_garbage"),
            Err(Error::NotFound(_, _))
        ));

        Ok(())
    }
}
