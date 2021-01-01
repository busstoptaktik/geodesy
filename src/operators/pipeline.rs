use super::OperatorArgs;
use super::OperatorCore;
use super::Operand;
use super::Operator;
use super::operator_factory;

pub struct Pipeline {
    name: String,
    steps: Vec<Operator>,
    inverted: bool,
}

impl Pipeline {
    pub fn new(args: &mut OperatorArgs) -> Pipeline {
        let inverted = args.flag("inv");
        let mut steps: Vec<Operator> = Vec::new();
        let n = args.numeric_value("_nsteps", 0.0) as usize;

        for i in 0..n {
            let step_name = format!("_step_{}", i);
            let step_args = &args.args[&step_name];

            // We need a copy of "all recursive globals so far"
            let mut oa = OperatorArgs::new();
            for (arg, val) in args.args.iter() {
                oa.insert(arg, val);
            }
            oa.populate(step_args, "");
            steps.push(operator_factory(&mut oa));
            println!("****** STEPS: {:?}", steps[i].name());
        }
        Pipeline {
            name: args.name.clone(),
            inverted: inverted,
            steps: steps
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
        use crate::Ellipsoid;
        let mut o = Operand::new();
        let mut args = OperatorArgs::new();
        args.insert("ellps", "intl");
        args.name("cart");

        let c = operator_factory("cart", &mut args);

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
}
