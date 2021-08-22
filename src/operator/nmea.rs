use super::Context;
use super::OperatorArgs;
use super::OperatorCore;
use crate::operator_construction::*;
use crate::CoordinateTuple;

pub struct Nmea {
    args: OperatorArgs,
    inverted: bool
}

impl Nmea {
    pub fn new(args: &mut OperatorArgs) -> Result<Nmea, &'static str> {
        let inverted = args.flag("inv");
        Ok(Nmea { args: args.clone(), inverted })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, &'static str> {
        let op = crate::operator::nmea::Nmea::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for Nmea {
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for o in operands {
            *o = CoordinateTuple::nmea(o[0], o[1], o[2], o[3]);
        }
        true
    }

    fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for o in operands {
            let longitude = CoordinateTuple::dd_to_nmea(o[0].to_degrees());
            let latitude = CoordinateTuple::dd_to_nmea(o[1].to_degrees());
            *o = CoordinateTuple::raw(latitude, longitude, o[2], o[3]);
        }
        true
    }

    fn name(&self) -> &'static str {
        "nmea"
    }

    fn is_noop(&self) -> bool {
        false
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }

    fn args(&self, _step: usize) -> &OperatorArgs {
        &self.args
    }
}

#[cfg(test)]
mod tests {
    use crate::Context;
    use crate::CoordinateTuple;
    #[test]
    fn nmea() {
        let mut ctx = Context::new();
        let nmea = ctx.operation("nmea").unwrap();
        let aemn = ctx.operation("nmea inv").unwrap();

        let coord_nmea = CoordinateTuple::raw(5530.15, -1245.15, 0., 0.);
        let coord_internal = CoordinateTuple::geo(55.5025, -12.7525, 0., 0.);

        let mut operands = [coord_nmea];
        ctx.fwd(nmea, &mut operands);
        assert!(operands[0].default_ellps_dist(&coord_internal) < 1e-10);
        assert!((operands[0][0].to_degrees() + 12.7525).abs() < 1e-10);
        assert!((operands[0][1].to_degrees() - 55.5025).abs() < 1e-10);

        ctx.inv(nmea, &mut operands);
        assert!((operands[0][0] - 5530.15).abs() < 1e-10);
        assert!((operands[0][1] + 1245.15).abs() < 1e-10);

        let mut operands = [coord_internal];
        ctx.fwd(aemn, &mut operands);
        assert!((operands[0][0] - 5530.15).abs() < 1e-10);
        assert!((operands[0][1] + 1245.15).abs() < 1e-10);
        ctx.inv(aemn, &mut operands);
        assert!(operands[0].default_ellps_dist(&coord_internal) < 1e-10);
    }
}
