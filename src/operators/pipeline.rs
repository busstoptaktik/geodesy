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
    fn fwd(&self, ws: &mut Operand) -> bool {
        // ws.coord = self.ellps.cartesian(&ws.coord);
        true
    }

    fn inv(&self, ws: &mut Operand) -> bool {
        // ws.coord = self.ellps.geographic(&ws.coord);
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
    fn pipeline_manual_setup() {
        use super::*;
        use crate::Ellipsoid;
        let mut o = Operand::new();
        let mut args = OperatorArgs::new();
        args.insert("ellps", "intl");
        args.name("cart");

        let c = operator_factory(&mut args);

        // First check that (0,0,0) takes us to (a,0,0)
        c.fwd(&mut o);
        let a = Ellipsoid::named("intl").semimajor_axis();
        assert_eq!(o.coord.0, a);
        assert_eq!(o.coord.1, 0.0);
        assert_eq!(o.coord.1, 0.0);

        // Some arbitrary spot - southwest of Copenhagen
        o.coord.0 = 12f64.to_radians();
        o.coord.1 = 55f64.to_radians();
        o.coord.2 = 100.0;

        // Roundtrip
        c.fwd(&mut o);
        c.inv(&mut o);

        // And check that we're back
        assert!((o.coord.first().to_degrees() -  12.).abs() < 1.0e-10);
        assert!((o.coord.third() - 100.).abs() < 1.0e-10);
        assert!((o.coord.second().to_degrees() - 55.).abs() < 1.0e-10);
    }


    #[test]
    fn pipeline_automatic_setup() {
        use super::*;

        // Setup a 3 step pipeline
        let pipeline = "ed50_etrs89: {steps: [cart: {ellps: intl}, helmert: {dx: -87, dy: -96, dz: -120}, cart: {inv: true, ellps: GRS80}]}";
        let mut args = OperatorArgs::global_defaults();
        args.populate(&pipeline, "");
        let op = Pipeline::new(&mut args);

        // Check that the steps are as expected
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
        assert_eq!(op.args(2).used["inv"], "true");
        // Note: Superfluous to give the arg "ellps: GRS80", so it is not registered as "used"
        assert!(op.args(2).used.get("ellps").is_none());


        /*
        // First check that (0,0,0) takes us to (a,0,0)
        c.fwd(&mut o);
        let a = Ellipsoid::named("intl").semimajor_axis();
        assert_eq!(o.coord.0, a);
        assert_eq!(o.coord.1, 0.0);
        assert_eq!(o.coord.1, 0.0);

        // Some arbitrary spot - southwest of Copenhagen
        o.coord.0 = 12f64.to_radians();
        o.coord.1 = 55f64.to_radians();
        o.coord.2 = 100.0;

        // Roundtrip
        c.fwd(&mut o);
        c.inv(&mut o);

        // And check that we're back
        assert!((o.coord.first().to_degrees() -  12.).abs() < 1.0e-10);
        assert!((o.coord.third() - 100.).abs() < 1.0e-10);
        assert!((o.coord.second().to_degrees() - 55.).abs() < 1.0e-10);
  */  }



}
