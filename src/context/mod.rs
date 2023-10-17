use std::rc::Rc;

use crate::authoring::*;
pub mod minimal;
pub use minimal::Minimal;

#[cfg(feature = "with_plain")]
pub mod plain;
#[cfg(feature = "with_plain")]
pub use plain::Plain;

// ----- T H E   C O N T E X T   T R A I T ---------------------------------------------

/// Modes of communication between the *Rust Geodesy* internals and the external
/// world (i.e. resources like grids, transformation definitions, or ellipsoid parameters).
pub trait Context {
    /// In general, implementations should make sure that `new` differs from `default`
    /// only by adding access to the builtin adaptors (`geo:in`, `gis:out` etc.)
    fn new() -> Self
    where
        Self: Sized;

    /// Instantiate the operation given by `definition`
    fn op(&mut self, definition: &str) -> Result<OpHandle, Error>;

    /// Apply operation `op` to `operands`
    fn apply(
        &self,
        op: OpHandle,
        direction: Direction,
        operands: &mut dyn CoordinateSet,
    ) -> Result<usize, Error>;

    /// Globally defined default values (typically just `ellps=GRS80`)
    fn globals(&self) -> BTreeMap<String, String>;

    /// Definitions of steps
    fn steps(&self, op: OpHandle) -> Result<&Vec<String>, Error>;

    /// Parsed parameters of a specific step
    fn params(&self, op: OpHandle, index: usize) -> Result<ParsedParameters, Error>;

    /// Register a new user-defined operator
    fn register_op(&mut self, name: &str, constructor: OpConstructor);
    /// Register a new user-defined resource (macro, ellipsoid parameter set...)
    fn register_resource(&mut self, name: &str, definition: &str);

    /// Helper for the `Op` instantiation logic in `Op::op(...)`
    fn get_op(&self, name: &str) -> Result<OpConstructor, Error>;
    /// Helper for the `Op` instantiation logic in `Op::op(...)`
    fn get_resource(&self, name: &str) -> Result<String, Error>;

    /// Access `blob`-like resources by identifier
    fn get_blob(&self, name: &str) -> Result<Vec<u8>, Error>;

    /// Access grid resources by identifier
    fn get_grid(&self, name: &str) -> Result<Rc<dyn GridTrait>, Error>;
}

/// Help context providers provide canonically named, built in coordinate adaptors
#[rustfmt::skip]
pub const BUILTIN_ADAPTORS: [(&str, &str); 8] = [
    ("geo:in",  "adapt from=neuf_deg"),
    ("geo:out", "adapt to=neuf_deg"  ),
    ("gis:in",  "adapt from=enuf_deg"),
    ("gis:out", "adapt to=enuf_deg"  ),
    ("neu:in",  "adapt from=neuf"    ),
    ("neu:out", "adapt to=neuf"      ),
    ("enu:in",  "adapt from=enuf"    ),
    ("enu:out", "adapt to=enuf"      ),
];
