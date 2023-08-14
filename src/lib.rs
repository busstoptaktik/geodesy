#![doc = include_str!("../README.md")]

mod bibliography;
mod context;
mod coordinate;
mod ellipsoid;
mod grid;
mod inner_op;
pub mod math;
mod op;

// The bread-and-butter

// Context trait and types
pub use crate::context::minimal::Minimal;
#[cfg(feature = "with_plain")]
pub use crate::context::plain::Plain;
pub use crate::context::Context;

// Used to specify which operator to apply in `Context::apply(...)`
pub use crate::op::OpHandle;

// Coordinate traits and types
pub use crate::coordinate::coor2d::Coor2D;
pub use crate::coordinate::coor32::Coor32;
pub use crate::coordinate::coor3d::Coor3D;
pub use crate::coordinate::coor4d::Coor4D;
pub use crate::coordinate::AngularUnits;
pub use crate::coordinate::CoordinateMetadata;
pub use crate::coordinate::CoordinateSet;

// Additional types
pub use crate::ellipsoid::Ellipsoid;
pub use crate::math::jacobian::Factors;
pub use crate::math::jacobian::Jacobian;

// Constants - directionality in `Context::apply(...)`
pub use crate::Direction::Fwd;
pub use crate::Direction::Inv;

// PROJ interoperability
pub use crate::op::parse_proj;

/// The bread-and-butter, shrink-wrapped for external use
pub mod prelude {
    pub use crate::Context;
    pub use crate::Minimal;
    #[cfg(feature = "with_plain")]
    pub use crate::Plain;

    pub use crate::AngularUnits;
    pub use crate::CoordinateMetadata;
    pub use crate::CoordinateSet;

    pub use crate::Coor2D;
    pub use crate::Coor32;
    pub use crate::Coor3D;
    pub use crate::Coor4D;

    pub use crate::Ellipsoid;
    pub use crate::Factors;
    pub use crate::Jacobian;

    pub use crate::OpHandle;

    pub use crate::Direction;
    pub use crate::Direction::Fwd;
    pub use crate::Direction::Inv;
    pub use crate::Error;

    pub use crate::parse_proj;

    #[cfg(test)]
    pub fn some_basic_coor4dinates() -> [Coor4D; 2] {
        let copenhagen = Coor4D::raw(55., 12., 0., 0.);
        let stockholm = Coor4D::raw(59., 18., 0., 0.);
        [copenhagen, stockholm]
    }
    #[cfg(test)]
    pub fn some_basic_coor3dinates() -> [Coor3D; 2] {
        let copenhagen = Coor3D::raw(55., 12., 0.);
        let stockholm = Coor3D::raw(59., 18., 0.);
        [copenhagen, stockholm]
    }
    #[cfg(test)]
    pub fn some_basic_coor2dinates() -> [Coor2D; 2] {
        let copenhagen = Coor2D::raw(55., 12.);
        let stockholm = Coor2D::raw(59., 18.);
        [copenhagen, stockholm]
    }
}

/// Prelude for authoring Contexts and InnerOp modules (built-in or user defined)
pub mod authoring {
    pub use crate::prelude::*;

    pub use crate::grid::Grid;
    pub use crate::inner_op::InnerOp;
    pub use crate::inner_op::OpConstructor;
    pub use crate::math::*;
    pub use crate::op::split_into_steps;
    pub use crate::op::Op;
    pub use crate::op::OpDescriptor;
    pub use crate::op::OpParameter;
    pub use crate::op::ParsedParameters;
    pub use crate::op::RawParameters;

    // All new contexts are supposed to support these
    pub use crate::context::BUILTIN_ADAPTORS;

    // External material
    pub use log::error;
    pub use log::info;
    pub use log::trace;
    pub use log::warn;
    pub use std::collections::BTreeMap;
}

use thiserror::Error;
/// The *Rust Geodesy* error messaging enumeration. Badly needs reconsideration
#[derive(Error, Debug)]
pub enum Error {
    #[error("i/o error")]
    Io(#[from] std::io::Error),

    #[error("error: {0}")]
    General(&'static str),

    #[error("syntax error: {0}")]
    Syntax(String),

    #[error("{0}: {1}")]
    Operator(&'static str, &'static str),

    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("{message:?} (expected {expected:?}, found {found:?})")]
    Unexpected {
        message: String,
        expected: String,
        found: String,
    },

    #[error("operator {0} not found{1}")]
    NotFound(String, String),

    #[error("recursion too deep for {0}, at {1}")]
    Recursion(String, String),

    #[error("attempt to invert a non-invertible item: {0}")]
    NonInvertible(String),

    #[error("missing required parameter {0}")]
    MissingParam(String),

    #[error("malformed value for parameter {0}: {1}")]
    BadParam(String, String),

    #[error("unknown error")]
    Unknown,
}

/// `Fwd`: Indicate that a two-way operator, function, or method,
/// should run in the *forward* direction.
/// `Inv`: Indicate that a two-way operator, function, or method,
/// should run in the *inverse* direction.
#[derive(Debug, PartialEq, Eq)]
pub enum Direction {
    Fwd,
    Inv,
}

#[cfg(doc)]
pub use crate::bibliography::Bibliography;
