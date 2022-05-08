#![doc = include_str!("../README.md")]

pub(crate) mod bibliography;
pub(crate) mod coord;
pub(crate) mod ellipsoid;
pub(crate) mod grid;

#[cfg(doc)]
pub use crate::bibliography::Bibliography;

mod inner_op;
mod op;
mod provider;

// The bread-and-butter
pub use crate::coord::Coord;
pub use crate::ellipsoid::Ellipsoid;
pub use crate::grid::Grid;
pub use crate::op::Op;
pub use crate::provider::Minimal;
pub use crate::provider::Provider;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct OpHandle(uuid::Uuid);
impl Default for OpHandle {
    fn default() -> Self {
        OpHandle(uuid::Uuid::new_v4())
    }
}

/// The bread-and-butter, shrink-wrapped for external use
pub mod preamble {
    pub use crate::coord::Coord;
    pub use crate::Direction;
    pub use crate::Direction::Fwd;
    pub use crate::Direction::Inv;
    pub use crate::Ellipsoid;
    pub use crate::Error;
    pub use crate::Minimal;
    pub use crate::Op;
    pub use crate::OpHandle;
    pub use crate::Provider;
}

/// Preamble for InnerOp modules (built-in or user defined)
pub mod inner_op_authoring {
    pub use crate::preamble::*;
    pub use log::error;
    pub use log::info;
    pub use log::trace;
    pub use log::warn;

    pub use crate::Grid;

    pub use crate::inner_op::InnerOp;
    pub use crate::inner_op::OpConstructor;

    pub use crate::op::OpDescriptor;
    pub use crate::op::OpParameter;
    pub use crate::op::ParsedParameters;
    pub use crate::op::RawParameters;

    #[cfg(test)]
    pub use crate::some_basic_coordinates;
}

/// Preamble for crate-internal modules
pub(crate) mod internal {
    pub use crate::inner_op_authoring::*;
    pub use std::collections::BTreeMap;
    pub use std::collections::BTreeSet;
    pub use uuid::Uuid;
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
#[derive(Debug, PartialEq)]
pub enum Direction {
    Fwd,
    Inv,
}

#[cfg(test)]
pub fn some_basic_coordinates() -> [Coord; 2] {
    let copenhagen = Coord::raw(55., 12., 0., 0.);
    let stockholm = Coord::raw(59., 18., 0., 0.);
    [copenhagen, stockholm]
}
