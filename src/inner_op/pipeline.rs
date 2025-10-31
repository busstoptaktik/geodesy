use super::pushpop::{do_the_pop, do_the_push};
use super::stack::{stack_fwd, stack_inv};
use crate::authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn pipeline_fwd(op: &Op, ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut stack = Vec::new();
    let mut n = usize::MAX;
    let steps = op.steps.as_ref().unwrap();
    for step in steps {
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
    let steps = op.steps.as_ref().unwrap();
    for step in steps.iter().rev() {
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
    let definition = &parameters.instantiated_as;
    let thesteps = definition.split_into_steps();
    let mut steps = Vec::new();

    for step in thesteps {
        let step_parameters = RawParameters::new(&step, &parameters.globals);
        steps.push(Op::op(step_parameters, ctx)?);
    }

    let params = ParsedParameters::new(parameters, &GAMUT)?;
    let fwd = InnerOp(pipeline_fwd);
    let inv = InnerOp(pipeline_inv);
    let descriptor = OpDescriptor::new(definition, fwd, Some(inv));
    Ok(Op {
        descriptor,
        params,
        steps: Some(steps),
    })
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipeline() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        // Plain pipeline
        let op = ctx.op("addone|addone|addone")?;
        let mut data = crate::test_data::coor2d();

        assert_eq!(2, ctx.apply(op, Fwd, &mut data)?);
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Pipeline with one inverted step
        let op = ctx.op("addone|addone inv|addone")?;
        let mut data = crate::test_data::coor2d();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Same, but with prefix modifier syntax
        let op = ctx.op("addone|inv addone|addone")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Registered resource
        ctx.register_resource("why:eggs", "helmert x=$eggs | helmert y=$why(1)");
        let op = ctx.op("why:eggs eggs=4")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 59.);
        assert_eq!(data[1][0], 63.);
        assert_eq!(data[0][1], 13.);
        assert_eq!(data[1][1], 19.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Invoke it as inverted
        let op = ctx.op("inv why:eggs eggs=4")?;
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 59.);
        assert_eq!(data[1][0], 63.);
        assert_eq!(data[0][1], 13.);
        assert_eq!(data[1][1], 19.);

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // And with inversion in postfix position
        let op = ctx.op("why:eggs eggs=4 inv")?;
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 59.);
        assert_eq!(data[1][0], 63.);
        assert_eq!(data[0][1], 13.);
        assert_eq!(data[1][1], 19.);

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Pipeline with registered resource as one step
        let op = ctx.op("helmert x=-2 | why:eggs eggs=1 why=0")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 54.);
        assert_eq!(data[1][0], 58.);
        assert_eq!(data[0][1], 12.);
        assert_eq!(data[1][1], 18.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Pipeline with registered resource as one INVERTED step
        let op = ctx.op("helmert x=-2 | inv why:eggs eggs=1 why=0")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 52.);
        assert_eq!(data[1][0], 56.);
        assert_eq!(data[0][1], 12.);
        assert_eq!(data[1][1], 18.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Try to invoke the registered resource, without a required parameter
        assert!(matches!(ctx.op("why:eggs why=10"), Err(Error::Syntax(_))));

        // Try to invoke garbage as a pipeline step
        assert!(matches!(
            ctx.op("addone|addone|_garbage"),
            Err(Error::NotFound(_, _))
        ));

        Ok(())
    }
}
