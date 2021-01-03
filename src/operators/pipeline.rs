use super::OperatorArgs;
use super::OperatorCore;
use super::Operand;
use super::Operator;
use super::operator_factory;


pub struct Pipeline {
    args: OperatorArgs,
    steps: Vec<Operator>,
    inverted: bool,
}


impl Pipeline {
    pub fn new(args: &mut OperatorArgs) -> Pipeline {
        let inverted = args.flag("inv");
        let mut steps = Vec::new();
        let n = args.numeric_value("_nsteps", 0.0) as usize;

        for i in 0..n {
            // Each step is represented as args[_step_0] = YAML step definition.
            // (see OperatorArgs::populate())
            let step_name = format!("_step_{}", i);
            let step_args = &args.args[&step_name];

            // We need a recursive copy of "all globals so far"
            let mut oa = OperatorArgs::with_globals_from(args, step_args, "");
            steps.push(operator_factory(&mut oa));
        }

        // if args.name == "badvalue" ... returner en fejl med cause som besked
        Pipeline {
            inverted: inverted,
            steps: steps,
            args: args.clone()
        }
    }
}

impl OperatorCore for Pipeline {
    fn fwd(&self, operand: &mut Operand) -> bool {
        for step in &self.steps {
            if !step.operate(operand, true) {
                return false
            }
        }
        true
    }

    fn inv(&self, operand: &mut Operand) -> bool {
        for step in self.steps.iter().rev() {
            if !step.operate(operand, false) {
                return false
            }
        }
        true
    }

    fn steps(&self) -> usize {
        self.steps.len()
    }

    fn args(&self, step: usize) -> &OperatorArgs {
        if step >= self.steps() {
            return &self.args;
        }
        self.steps[step].args(0 as usize)
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
        let pipeline = "ed50_etrs89: {steps: [cart: {ellps: intl}, helmert: {dx: -87, dy: -96, dz: -120}, cart: {inv: true, ellps: GRS80}]}";
        let mut args = OperatorArgs::global_defaults();
        args.populate(&pipeline, "");
        let op = Pipeline::new(&mut args);

        // Check step-by-step that the pipeline was formed as expected
        assert_eq!(op.steps(), 3);
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
        let mut operand = Operand::new();
        operand.coord = crate::CoordinateTuple(12f64.to_radians(), 55f64.to_radians(), 100., 0.);

        /* DRUM ROLL... */ op.operate(&mut operand, true); // TA-DAA!

        // For comparison: the point (12, 55, 100, 0) transformed by the cct
        // application of the PROJ package yields:
        // 11.998815342385206861  54.999382648950991381  131.202401081100106239  0.0000
        // cct -d18 proj=pipeline step proj=cart ellps=intl step proj=helmert x=-87 y=-96 z=-120 step proj=cart inv --
        operand.coord.0 = operand.coord.0.to_degrees();
        operand.coord.1 = operand.coord.1.to_degrees();
        assert!((operand.coord.0 - 11.998815342385206861).abs() < 1e-12);
        assert!((operand.coord.1 - 54.999382648950991381).abs() < 1e-12);
        // We use an improved height expression, so this value differs slightly
        // (is better) than the one from PROJ.
        assert!((operand.coord.2 - 131.202401081100106239).abs() < 1e-8);

        // And the other way round
        operand.coord.0 = operand.coord.0.to_radians();
        operand.coord.1 = operand.coord.1.to_radians();
        /* DRUM ROLL... */ op.operate(&mut operand, false); // TA-DAA!
        operand.coord.0 = operand.coord.0.to_degrees();
        operand.coord.1 = operand.coord.1.to_degrees();
        assert!((operand.coord.0 - 12.).abs() < 1e-14);
        assert!((operand.coord.1 - 55.).abs() < 1e-14);
        assert!((operand.coord.2 - 100.).abs() < 1e-8);

        // -------------------------------------------------------------------------
    }
}
