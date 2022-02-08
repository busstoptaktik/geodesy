use super::inner_op_authoring::*;

// ----- B U I L T I N   O P E R A T O R S ---------------------------------------------

// Install new builtin operators by adding them in the pub(super) and
// BUILTIN_OPERATORS blocks below

mod adapt;
mod addone;
mod cart;
mod helmert;
mod lcc;
mod merc;
mod molodensky;
mod nmea;
mod noop;
pub(crate) mod pipeline;
mod tmerc;

#[rustfmt::skip]
const BUILTIN_OPERATORS: [(&str, OpConstructor); 12] = [
    ("adapt",      OpConstructor(adapt::new)),
    ("addone",     OpConstructor(addone::new)),
    ("cart",       OpConstructor(cart::new)),
    ("helmert",    OpConstructor(helmert::new)),
    ("lcc",        OpConstructor(lcc::new)),
    ("merc",       OpConstructor(merc::new)),
    ("molodensky", OpConstructor(molodensky::new)),
    ("nmea",       OpConstructor(nmea::new)),
    ("noop",       OpConstructor(noop::new)),
    ("tmerc",      OpConstructor(tmerc::new)),
    ("utm",        OpConstructor(tmerc::utm)),
    ("pipeline",   OpConstructor(pipeline::new)),
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

// ----- S T R U C T   O P C O N S T R U C T O R ---------------------------------------

/// Blueprint for the overall instantiation of an operator.
/// OpConstructor needs to be a newtype, rather than a type alias,
/// since we must implement the Debug-trait for OpConstructor (to
/// make auto derive of the Debug-trait work for any derived type).
pub struct OpConstructor(pub fn(args: &RawParameters, ctx: &dyn Provider) -> Result<Op, Error>);

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
pub struct InnerOp(
    pub fn(op: &Op, ctx: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error>,
);

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
    _operands: &mut [Coord],
) -> Result<usize, Error> {
    // Consider whether this should return an Err-value if used as a placeholder for a
    // non-existing or non-implemented inverse operation
    Ok(0)
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

// pj_tsfn is the equivalent of Charles Karney's PROJ function of the
// same name, which determines the function ts(phi) as defined in
// Snyder (1987), Eq. (7-10)
//
// ts is the exponential of the negated isometric latitude, i.e.
// exp(-𝜓), but evaluated in a numerically more stable way than
// the naive ellps.isometric_latitude(...).exp()
//
// This version is essentially identical to Charles Karney's PROJ
// version, including the majority of the comments.
//
// Inputs:
//   (sin phi, cos phi) = trigs of geographic latitude
//   e = eccentricity of the ellipsoid
// Output:
//   ts = exp(-psi) where psi is the isometric latitude (dimensionless)
//      = 1 / (tan(chi) + sec(chi))
// Here isometric latitude is defined by
//   psi = log( tan(pi/4 + phi/2) *
//              ( (1 - e*sin(phi)) / (1 + e*sin(phi)) )^(e/2) )
//       = asinh(tan(phi)) - e * atanh(e * sin(phi))
//       = asinh(tan(chi))
//   chi = conformal latitude
#[allow(dead_code)]
fn pj_tsfn(sincos: (f64, f64), e: f64) -> f64 {
    // exp(-asinh(tan(phi)))
    //    = 1 / (tan(phi) + sec(phi))
    //    = cos(phi) / (1 + sin(phi))  good for phi > 0
    //    = (1 - sin(phi)) / cos(phi)  good for phi < 0
    let factor = if sincos.0 > 0. {
        sincos.1 / (1. + sincos.0)
    } else {
        (1. - sincos.0) / sincos.1
    };
    (e * (e * sincos.0).atanh()).exp() * factor
}

// Snyder (1982) eq. 12-15, PROJ's pj_msfn()
#[allow(dead_code)]
fn pj_msfn(sincos: (f64, f64), es: f64) -> f64 {
    sincos.1 / (1. - sincos.0 * sincos.0 * es).sqrt()
}

// Equivalent to the PROJ pj_phi2 function
#[allow(dead_code)]
fn pj_phi2(ts0: f64, e: f64) -> f64 {
    sinhpsi_to_tanphi((1. / ts0 - ts0) / 2., e).atan()
}

// Ancillary function for computing the inverse isometric latitude. Follows
// [Karney, 2011](crate::Bibliography::Kar11), and the PROJ implementation
// in proj/src/phi2.cpp.
// Needs crate-visibility as it is also used in crate::ellipsoid::latitudes
pub(crate) fn sinhpsi_to_tanphi(taup: f64, e: f64) -> f64 {
    // min iterations = 1, max iterations = 2; mean = 1.954
    const MAX_ITER: usize = 5;

    // rooteps, tol and tmax are compile time constants, but currently
    // Rust cannot const-evaluate powers and roots, so we must either
    // evaluate these "constants" as lazy_statics, or just swallow the
    // penalty of an extra sqrt and two divisions on each call.
    // If this shows unbearable, we can just also assume IEEE-64 bit
    // arithmetic, and set rooteps = 0.000000014901161193847656
    let rooteps: f64 = f64::EPSILON.sqrt();
    let tol: f64 = rooteps / 10.; // the criterion for Newton's method
    let tmax: f64 = 2. / rooteps; // threshold for large arg limit exact

    let e2m = 1. - e * e;
    let stol = tol * taup.abs().max(1.0);

    // The initial guess.  70 corresponds to chi = 89.18 deg
    let mut tau = if taup.abs() > 70. {
        taup * (e * e.atanh()).exp()
    } else {
        taup / e2m
    };

    // Handle +/-inf, nan, and e = 1
    if (tau.abs() >= tmax) || tau.is_nan() {
        return tau;
    }

    for _ in 0..MAX_ITER {
        let tau1 = (1. + tau * tau).sqrt();
        let sig = (e * (e * tau / tau1).atanh()).sinh();
        let taupa = (1. + sig * sig).sqrt() * tau - sig * tau1;
        let dtau =
            (taup - taupa) * (1. + e2m * (tau * tau)) / (e2m * tau1 * (1. + taupa * taupa).sqrt());
        tau += dtau;

        if (dtau.abs() < stol) || tau.is_nan() {
            return tau;
        }
    }
    f64::NAN
}
