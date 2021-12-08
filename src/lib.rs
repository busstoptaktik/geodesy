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
#![doc = include_str!("../README.md")]

// Most details are hidden: Few `pub mod`s below
mod bibliography;
mod coordinate;
mod ellipsoid;
mod internals;
mod operator;
pub mod resource;

// But we add `pub`-ness to some important `struct`s and traits.
pub use coordinate::CoordinateTuple;
pub use ellipsoid::Ellipsoid;
pub use resource::gys::{GysArgs, GysResource};
pub use resource::minimal::MinimalResourceProvider as Minimal;
pub use resource::plain::PlainResourceProvider as Plain;
pub use resource::plain::SearchLevel as SearchLevel;
pub use resource::Provider;

// The bibliography needs `pub`-ness in order to be able to build the docs.
pub use bibliography::Bibliography;

// These need `pub`-ness in order to support user-defined operators
pub use operator::{Operator, OperatorCore};
pub type OperatorConstructor =
    fn(args: &GysResource, provider: &dyn Provider) -> Result<Operator, GeodesyError>;

/// Indicate that a two-way operator, function, or method, should run in the *forward* direction.
pub const FWD: bool = true;
/// Indicate that a two-way operator, function, or method, should run in the *inverse* direction.
pub const INV: bool = false;

/// And obviously the GeodesyError enum needs `pub`-ness
use thiserror::Error;
#[derive(Error, Debug)]
pub enum GeodesyError {
    // Convert any std::io::Error to GeodesyError::Io
    #[error(transparent)]
    Io(#[from] std::io::Error),

    // General error message
    #[error("error: {0}")]
    General(&'static str),

    #[error("syntax error: {0}")]
    Syntax(String),

    #[error("{0}: {1}")]
    Operator(&'static str, &'static str),

    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },

    #[error("operator {0} not found")]
    NotFound(String),

    #[error("too deep recursion for {0}")]
    Recursion(String),

    #[error("unknown error")]
    Unknown,
}
