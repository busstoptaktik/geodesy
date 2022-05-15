mod local;
mod minimal;

pub use local::Local;
pub use minimal::Minimal;

use super::internal::*;

// ----- T H E   P R O V I D E R   T R A I T -------------------------------------------

/// The `Provider` trait defines the mode of communication between *Rust Geodesy* internals
/// and the external context (i.e. typically resources like grids, transformation definitions,
/// or ellipsoid parameters).
pub trait Provider {
    fn new(resources: Option<BTreeMap<&'static str, String>>) -> Self
    where Self: Sized;

    /// Instantiate the operation given by `definition`
    fn op(&mut self, definition: &str) -> Result<OpHandle, Error>;
    /// Apply operation `op` to `operands`
    fn apply(
        &self,
        op: OpHandle,
        direction: Direction,
        operands: &mut [Coord],
    ) -> Result<usize, Error>;

    /// Globally defined default values (typically just `ellps=GRS80`)
    fn globals(&self) -> BTreeMap<String, String>;

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
    fn get_grid(&self, name: &str) -> Result<Grid, Error>;
}
