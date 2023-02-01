use crate::operator_authoring::*;

// ----- B U I L T I N   O P E R A T O R S ---------------------------------------------

// Install new builtin operators by adding them in the `mod` and
// `BUILTIN_OPERATORS` blocks below

mod adapt;
mod addone;
mod btmerc;
mod cart;
mod gridshift;
mod helmert;
mod laea;
mod latitude;
mod lcc;
mod merc;
mod molodensky;
mod nmea;
mod noop;
pub(crate) mod pipeline; // Needed by Op for instantiation
mod proj;
mod tmerc;

#[rustfmt::skip]
const BUILTIN_OPERATORS: [(&str, OpConstructor); 20] = [
    ("adapt",      OpConstructor(adapt::new)),
    ("addone",     OpConstructor(addone::new)),
    ("btmerc",     OpConstructor(btmerc::new)),
    ("butm",       OpConstructor(btmerc::utm)),
    ("cart",       OpConstructor(cart::new)),
    ("gridshift",  OpConstructor(gridshift::new)),
    ("helmert",    OpConstructor(helmert::new)),
    ("laea",       OpConstructor(laea::new)),
    ("latitude",   OpConstructor(latitude::new)),
    ("lcc",        OpConstructor(lcc::new)),
    ("merc",       OpConstructor(merc::new)),
    ("molodensky", OpConstructor(molodensky::new)),
    ("nmea",       OpConstructor(nmea::new)),
    ("noop",       OpConstructor(noop::new)),
    ("tmerc",      OpConstructor(tmerc::new)),
    ("utm",        OpConstructor(tmerc::utm)),
    ("pipeline",   OpConstructor(pipeline::new)),
    ("pop",        OpConstructor(pipeline::pop)),
    ("proj",       OpConstructor(proj::new)),
    ("push",       OpConstructor(pipeline::push)),
];
// A BTreeMap would have been a better choice for BUILTIN_OPERATORS, except
// for the annoying fact that it cannot be compile-time const-constructed.

/// Handle instantiation of built-in operators, as defined in
/// `BUILTIN_OPERATORS` above.
pub(crate) fn builtin(name: &str) -> Result<OpConstructor, Error> {
    for p in BUILTIN_OPERATORS {
        if p.0 == name {
            return Ok(p.1);
        }
    }
    Err(Error::NotFound(name.to_string(), String::default()))
}

// ----- S T R U C T   O P C O N S T R U C T O R ---------------------------------------

/// Blueprint for the overall instantiation of an operator.
/// OpConstructor needs to be a newtype, rather than a type alias,
/// since we must implement the Debug-trait for OpConstructor (to
/// make auto derive of the Debug-trait work for any derived type).
pub struct OpConstructor(pub fn(args: &RawParameters, ctx: &dyn Context) -> Result<Op, Error>);

// Cannot autoderive the Debug trait
impl core::fmt::Debug for OpConstructor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OpConstructor")
    }
}

// ----- S T R U C T   I N N E R O P ---------------------------------------------------

/// Blueprint for the functions doing the actual transformation work.
/// InnerOp needs to be a newtype, rather than a type alias, since we
/// must implement the Debug-trait for InnerOp (to make auto derive
/// of the Debug-trait work for any derived type).
pub struct InnerOp(pub fn(op: &Op, ctx: &dyn Context, operands: &mut [Coord]) -> usize);

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

fn noop_placeholder(_params: &Op, _ctx: &dyn Context, _operands: &mut [Coord]) -> usize {
    // Consider whether this should return an Err-value if used as a placeholder for a
    // non-existing or non-implemented inverse operation
    0
}
