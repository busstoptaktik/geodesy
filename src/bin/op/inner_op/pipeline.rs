use super::{Error, Op};
use crate::etc;
use crate::provider::Provider;
use crate::rawparameters::RawParameters;
use geodesy::CoordinateTuple;

pub fn new(definition: &RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
    let steps = etc::split_into_steps(&definition.definition);
    todo!()
}

fn pipeline_fwd(op: &Op, provider: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = usize::MAX;
    for step in &op.steps[..] {
        n = n.min(step.operate(provider, operands, crate::Direction::Fwd));
    }
    n
}

fn pipeline_inv(op: &Op, provider: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = usize::MAX;
    for step in op.steps[..].iter().rev() {
        n = n.min(step.operate(provider, operands, crate::Direction::Inv));
    }
    n
}
