use crate::inner_op_authoring::*;

/// Blueprint for the overall instantiation of an operator.
/// OpConstructor needs to be a newtype, rather than a type definition,
/// since we must implement the Debug-trait for OpConstructor (to make
/// auto derivin the Debug-trait work for any derived type).

pub struct OpConstructor(pub fn(args: &RawParameters, ctx: &dyn Provider) -> Result<Op, Error>);

// Cannot autoderive the Debug trait
impl core::fmt::Debug for OpConstructor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OpConstructor")
    }
}

// ----------------------------------------------------------------------------

// Install new builtin operators by adding them in the pub(super) and
// BUILTIN_OPERATORS blocks below

pub(super) mod addone;
pub(super) mod pipeline;

#[rustfmt::skip]
const BUILTIN_OPERATORS: [(&str, OpConstructor); 2] = [
    ("pipeline", OpConstructor(crate::inner_op::pipeline::new)),
    ("addone", OpConstructor(crate::inner_op::addone::new)),
];
// A BTreeMap would have been a better choice for BUILTIN_OPERATORS, except
// for the annoying fact that it cannot be compile-time const-constructed.

/// Handle instantiation of built-in operators, as defined in
/// `BUILTIN_OPERATORS` above.
pub fn builtin(name: &str) -> Result<OpConstructor, Error> {
    for p in BUILTIN_OPERATORS {
        if p.0 == name {
            return Ok(p.1);
        }
    }
    Err(Error::NotFound(name.to_string(), String::default()))
}

// ----------------------------------------------------------------------------

/// Blueprint for the functions doing the actual transformation work.
/// InnerOp needs to be a newtype, rather than a type definition, since
/// we must implement the Debug-trait for InnerOp (to make auto deriving
/// the Debug-trait work for any derived type).
pub struct InnerOp(pub fn(op: &Op, ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize);

// Cannot autoderive the Debug trait
impl core::fmt::Debug for InnerOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InnerOp")
    }
}

// Defaults to no_op
impl Default for InnerOp {
    fn default() -> InnerOp {
        InnerOp(noop_placeholder)
    }
}

fn noop_placeholder(
    _params: &Op,
    _provider: &dyn Provider,
    _operands: &mut [CoordinateTuple],
) -> usize {
    0
}
