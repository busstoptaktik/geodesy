#![doc = include_str!("../README.md")]

mod bibliography;
mod context;
mod coord;
mod ellipsoid;
mod grid;
mod inner_op;
mod math;
mod op;

// The bread-and-butter
pub use crate::context::Context;
pub use crate::context::Minimal;
pub use crate::context::Plain;
pub use crate::coord::Coord;
pub use crate::ellipsoid::Ellipsoid;
pub use crate::Direction::Fwd;
pub use crate::Direction::Inv;

/// The bread-and-butter, shrink-wrapped for external use
pub mod preamble {
    pub use crate::context::Context;
    pub use crate::context::Minimal;
    pub use crate::context::Plain;
    pub use crate::coord::Coord;
    pub use crate::ellipsoid::Ellipsoid;
    pub use crate::op::Op;
    pub use crate::op::OpHandle;
    pub use crate::Direction;
    pub use crate::Direction::Fwd;
    pub use crate::Direction::Inv;
    pub use crate::Error;
}

/// Preamble for InnerOp modules (built-in or user defined)
pub mod operator_authoring {
    pub use crate::preamble::*;
    pub use log::error;
    pub use log::info;
    pub use log::trace;
    pub use log::warn;

    pub use crate::grid::Grid;
    pub use crate::inner_op::InnerOp;
    pub use crate::inner_op::OpConstructor;
    pub use crate::math::*;
    pub use crate::op::OpDescriptor;
    pub use crate::op::OpParameter;
    pub use crate::op::ParsedParameters;
    pub use crate::op::RawParameters;

    #[cfg(test)]
    pub fn some_basic_coordinates() -> [Coord; 2] {
        let copenhagen = Coord::raw(55., 12., 0., 0.);
        let stockholm = Coord::raw(59., 18., 0., 0.);
        [copenhagen, stockholm]
    }
}

/// Preamble for Contexts (built-in or user defined)
pub mod context_authoring {
    pub use crate::context::BUILTIN_ADAPTORS;
    pub use crate::operator_authoring::*;
    pub use std::collections::BTreeMap;
}

use thiserror::Error;
/// The *Rust Geodesy* errror messaging enumeration. Badly needs reconsideration
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
