use crate::CoordinateTuple;
use crate::GeodesyError;
use crate::GysResource;
use crate::Operator;
use crate::OperatorCore;
use crate::Provider;

pub struct Noop {
    args: Vec<(String, String)>,
}

impl Noop {
    pub fn new(res: &GysResource) -> Result<Noop, GeodesyError> {
        let args = res.to_args(0)?;
        Ok(Noop { args: args.used })
    }

    pub(crate) fn operator(
        args: &GysResource,
        _rp: &dyn Provider,
    ) -> Result<Operator, GeodesyError> {
        let op = crate::operator::noop::Noop::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for Noop {
    fn fwd(&self, _ctx: &dyn Provider, _operands: &mut [CoordinateTuple]) -> bool {
        true
    }

    fn inv(&self, _ctx: &dyn Provider, _operands: &mut [CoordinateTuple]) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "noop"
    }

    fn is_noop(&self) -> bool {
        true
    }

    fn is_inverted(&self) -> bool {
        false
    }

    fn args(&self, _step: usize) -> &[(String, String)] {
        &self.args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noop() {
        let mut o = crate::resource::plain::PlainResourceProvider::default();
        let c = Operator::new("noop irrelevant: option", &mut o).unwrap();

        let mut operands = [CoordinateTuple::origin()];

        // Make sure we do not do anything
        c.fwd(&mut o, operands.as_mut());
        assert_eq!(operands[0][0], 0.0);
        assert_eq!(operands[0][1], 0.0);
        assert_eq!(operands[0][2], 0.0);
        assert_eq!(operands[0][3], 0.0);
        c.inv(&mut o, operands.as_mut());
        assert_eq!(operands[0][0], 0.0);
        assert_eq!(operands[0][1], 0.0);
        assert_eq!(operands[0][2], 0.0);
        assert_eq!(operands[0][3], 0.0);

        // Make sure we say what we are
        assert!(c.name() == "noop");
    }
}
