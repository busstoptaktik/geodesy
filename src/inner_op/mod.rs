use crate::authoring::*;

// ----- B U I L T I N   O P E R A T O R S ---------------------------------------------

// Install new builtin operators by adding them in the `mod` and
// `BUILTIN_OPERATORS` blocks below

mod adapt;
mod addone;
mod axisswap;
mod btmerc;
mod cart;
mod curvature;
mod deflection;
mod deformation;
mod geodesic;
mod gridshift;
mod helmert;
mod iso6709;
mod laea;
mod latitude;
mod lcc;
mod merc;
mod molodensky;
mod noop;
mod omerc;
pub(crate) mod pipeline; // Needed by Op for instantiation
mod pushpop;
mod somerc;
mod stack;
mod tmerc;
mod unitconvert;
mod units;
mod webmerc;

#[rustfmt::skip]
const BUILTIN_OPERATORS: [(&str, OpConstructor); 34] = [
    ("adapt",        OpConstructor(adapt::new)),
    ("addone",       OpConstructor(addone::new)),
    ("axisswap",     OpConstructor(axisswap::new)),
    ("btmerc",       OpConstructor(btmerc::new)),
    ("butm",         OpConstructor(btmerc::utm)),
    ("cart",         OpConstructor(cart::new)),
    ("curvature",    OpConstructor(curvature::new)),
    ("deflection",   OpConstructor(deflection::new)),
    ("deformation",  OpConstructor(deformation::new)),
    ("dm",           OpConstructor(iso6709::dm)),
    ("dms",          OpConstructor(iso6709::dms)),
    ("geodesic",     OpConstructor(geodesic::new)),
    ("gridshift",    OpConstructor(gridshift::new)),
    ("helmert",      OpConstructor(helmert::new)),
    ("laea",         OpConstructor(laea::new)),
    ("latitude",     OpConstructor(latitude::new)),
    ("lcc",          OpConstructor(lcc::new)),
    ("merc",         OpConstructor(merc::new)),
    ("webmerc",      OpConstructor(webmerc::new)),
    ("molodensky",   OpConstructor(molodensky::new)),
    ("omerc",        OpConstructor(omerc::new)),
    ("somerc",       OpConstructor(somerc::new)),
    ("tmerc",        OpConstructor(tmerc::new)),
    ("unitconvert",  OpConstructor(unitconvert::new)),
    ("utm",          OpConstructor(tmerc::utm)),

    // Pipeline handlers
    ("pipeline",     OpConstructor(pipeline::new)),
    ("pop",          OpConstructor(pushpop::pop)),
    ("push",         OpConstructor(pushpop::push)),
    ("stack",        OpConstructor(stack::new)),

    // Some commonly used noop-aliases
    ("noop",         OpConstructor(noop::new)),
    ("longlat",      OpConstructor(noop::new)),
    ("latlon",       OpConstructor(noop::new)),
    ("latlong",      OpConstructor(noop::new)),
    ("lonlat",       OpConstructor(noop::new)),
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
///
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
///
/// InnerOp needs to be a newtype, rather than a type alias, since we
/// must implement the Debug-trait for InnerOp (to make auto derive
/// of the Debug-trait work for any derived type).
pub struct InnerOp(pub fn(op: &Op, ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize);

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

fn noop_placeholder(_op: &Op, _ctx: &dyn Context, _operands: &mut dyn CoordinateSet) -> usize {
    // Consider whether this should return an Err-value if used as a placeholder for a
    // non-existing or non-implemented inverse operation
    0
}
