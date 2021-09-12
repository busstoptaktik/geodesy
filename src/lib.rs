//! *A playground for experimentation with alternative models for geodetic
//! data flow and coordinate representation*.
//!
//! Geodesy
//! =======
//!
//! A crate designed to facilitate development of new geodetic transformations,
//! and to investigate potential solutions to identified/perceived/suspected
//! shortcomings in the [PROJ](https://proj.org) data flow, and the
//! [ISO-19111](https://www.iso.org/standard/74039.html)
//! model for referencing by coordinates.
//!
//! Et cetera
//! ---------
//!
//! Copyright by Thomas Knudsen, knudsen.thomas@gmail.com, 2020/2021
//!
//!

// Most details are hidden: No `pub mod`s below
mod bibliography;
mod context;
mod coordinate;
mod ellipsoid;
mod internals;
mod operator;

// But we add `pub`-ness to a few important `struct`s.
pub use context::Context;
pub use coordinate::CoordinateTuple;
pub use ellipsoid::Ellipsoid;

// The bibliography needs `pub`-ness in order to be able to build the docs.
pub use bibliography::Bibliography;

/// The operator construction toolkit. Needs `pub`-ness in order to support
/// the construction of user defined operators.
pub mod operator_construction {
    mod gas;
    mod operatorargs;
    pub use crate::operator::Operator;
    pub use crate::operator::OperatorCore;
    use crate::Context;
    pub use gas::Gas;
    pub use operatorargs::OperatorArgs;
    pub type OperatorConstructor =
        fn(args: &mut OperatorArgs, ctx: &mut Context) -> Result<Operator, &'static str>;
}

/// Indicate that a two-way operator, function, or method, should run in the *forward* direction.
#[allow(non_upper_case_globals)]
pub const fwd: bool = true;
/// Indicate that a two-way operator, function, or method, should run in the *inverse* direction.
#[allow(non_upper_case_globals)]
pub const inv: bool = false;
