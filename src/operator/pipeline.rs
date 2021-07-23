use super::operator_factory;
use super::Context;
use super::Operator;
use super::OperatorArgs;
use super::OperatorCore;
use crate::{fwd, inv};

pub struct Pipeline {
    args: OperatorArgs,
    steps: Vec<Operator>,
    inverted: bool,
}

impl Pipeline {
    pub fn new(args: &mut OperatorArgs, ctx: &mut Context) -> Result<Pipeline, String> {
        let inverted = args.flag("inv");
        let mut steps = Vec::new();
        let n = args.numeric_value("Pipeline", "_nsteps", 0.0)? as usize;

        for i in 0..n {
            // Each step is represented as args[_step_0] = YAML step definition.
            // (see OperatorArgs::populate())
            let step_name = format!("_step_{}", i);
            let step_args = &args.args[&step_name];

            // We need a recursive copy of "all globals so far"
            let mut oa = args.spawn(step_args);
            let op = operator_factory(&mut oa, ctx, 0)?;
            steps.push(op);
        }

        let args = args.clone();

        Ok(Pipeline {
            args,
            steps,
            inverted,
        })
    }
}

impl OperatorCore for Pipeline {
    fn fwd(&self, ctx: &mut Context) -> bool {
        for step in &self.steps {
            if !step.operate(ctx, fwd) {
                return false;
            }
        }
        true
    }

    fn inv(&self, ctx: &mut Context) -> bool {
        for step in self.steps.iter().rev() {
            if !step.operate(ctx, inv) {
                return false;
            }
        }
        true
    }

    fn len(&self) -> usize {
        self.steps.len()
    }

    fn args(&self, step: usize) -> &OperatorArgs {
        if step >= self.len() {
            return &self.args;
        }
        self.steps[step].args(0_usize)
    }

    fn name(&self) -> &'static str {
        "pipeline"
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn pipeline() {
        use super::*;

        // Setup a 3 step pipeline
        let pipeline = "ed50_etrs89: {
            globals: [
                foo: bar,
                baz: bonk
            ],
            steps: [
                cart: {ellps: intl},
                helmert: {dx: -87, dy: -96, dz: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";

        // We cannot use Operator::new here, because we want to access internal
        // elements of the Pipeline struct below. These are inaccesible after
        // boxing.
        let mut ctx = Context::new();
        let mut args = OperatorArgs::new();
        args.populate(&pipeline, "");
        let op = Pipeline::new(&mut args, &mut ctx).unwrap();

        // Check step-by-step that the pipeline was formed as expected
        assert_eq!(op.len(), 3);
        assert_eq!(op.steps[0].name(), "cart");
        assert_eq!(op.steps[0].is_inverted(), false);

        assert_eq!(op.steps[1].name(), "helmert");
        assert_eq!(op.steps[1].is_inverted(), false);

        assert_eq!(op.steps[2].name(), "cart");
        assert_eq!(op.steps[2].is_inverted(), true);

        // Check that definition argument introspection works
        assert_eq!(op.args(0).used["ellps"], "intl");

        assert_eq!(op.args(1).used["dx"], "-87");
        assert_eq!(op.args(1).used["dy"], "-96");
        assert_eq!(op.args(1).used["dz"], "-120");

        // Note: It's superfluous to give the arg "ellps: GRS80", so it is not registered as "used"
        assert_eq!(op.args(2).used["inv"], "true");
        assert!(op.args(2).used.get("ellps").is_none());

        // -------------------------------------------------------------------------
        // This is the first example of a running pipeline in Rust Geodesy. Awesome!
        // -------------------------------------------------------------------------
        ctx.coord = crate::CoordinateTuple::deg(12., 55., 100., 0.);

        /* DRUM ROLL... */
        op.operate(&mut ctx, fwd); // TA-DAA!

        // For comparison: the point (12, 55, 100, 0) transformed by the cct
        // application of the PROJ package yields:
        // 11.998815342385206861  54.999382648950991381  131.202401081100106239  0.0000
        // cct -d18 proj=pipeline step proj=cart ellps=intl step proj=helmert x=-87 y=-96 z=-120 step proj=cart inv --
        ctx.coord.0 = ctx.coord.0.to_degrees();
        ctx.coord.1 = ctx.coord.1.to_degrees();
        assert!((ctx.coord.0 - 11.998815342385206861).abs() < 1e-12);
        assert!((ctx.coord.1 - 54.999382648950991381).abs() < 1e-12);
        // We use an improved height expression, so this value differs slightly
        // (is better) than the one from PROJ.
        assert!((ctx.coord.2 - 131.202401081100106239).abs() < 1e-8);

        // And the other way round
        ctx.coord.0 = ctx.coord.0.to_radians();
        ctx.coord.1 = ctx.coord.1.to_radians();
        /* DRUM ROLL... */
        op.operate(&mut ctx, false); // TA-DAA!
        ctx.coord.0 = ctx.coord.0.to_degrees();
        ctx.coord.1 = ctx.coord.1.to_degrees();
        assert!((ctx.coord.0 - 12.).abs() < 1e-14);
        assert!((ctx.coord.1 - 55.).abs() < 1e-14);
        assert!((ctx.coord.2 - 100.).abs() < 1e-8);

        // -------------------------------------------------------------------------
    }
}
