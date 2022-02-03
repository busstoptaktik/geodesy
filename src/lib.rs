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

// No public modules,
pub(crate) mod bibliography;
pub(crate) mod coordinate;
pub(crate) mod ellipsoid;

pub use crate::coordinate::CoordinateTuple;
pub use crate::ellipsoid::Ellipsoid;
pub use crate::provider::Provider;
pub use crate::provider::Minimal;
pub use crate::op::Op;

pub mod inner_op;
pub mod op;
pub mod op_descriptor;
pub mod parsed_parameters;

pub mod etc;
pub mod parameter;
pub mod provider;
pub mod raw_parameters;

use log::error;
use std::io;
use thiserror::Error;

/// Preamble for InnerOp modules (built-in or user defined)
pub mod inner_op_authoring {
    pub use log::error;
    pub use log::warn;

    pub use crate::Error;
    pub use crate::coordinate::CoordinateTuple;
    pub use crate::ellipsoid::Ellipsoid;

    pub use crate::provider::Provider;
    pub use crate::provider::Minimal;

    pub use crate::op::Op;
    pub use crate::inner_op::InnerOp;
    pub use crate::op_descriptor::OpDescriptor;
    pub use crate::inner_op::OpConstructor;

    pub use crate::parameter::OpParameter;
    pub use crate::parsed_parameters::ParsedParameters;
    pub use crate::raw_parameters::RawParameters;

    pub use crate::Direction;
    pub use crate::Direction::Fwd;
    pub use crate::Direction::Inv;

    pub use crate::etc;

}

/// Preamble for crate-internal modules
pub(crate) mod internal {
    pub use std::collections::BTreeMap;
    pub use std::collections::BTreeSet;

    pub use log::error;
    pub use log::warn;
    pub use uuid::Uuid;

    pub use crate::coordinate::CoordinateTuple;
    pub use crate::ellipsoid::Ellipsoid;
    pub use crate::provider::Provider;
    pub use crate::provider::Minimal;
    pub use crate::op::Op;
    pub use crate::Error;

    pub use crate::etc;
    pub use crate::inner_op::InnerOp;
    pub use crate::inner_op::OpConstructor;

    pub use crate::op_descriptor::OpDescriptor;
    pub use crate::parameter::OpParameter;
    pub use crate::parsed_parameters::ParsedParameters;

    // pub use crate::provider::Minimal;
    pub use crate::raw_parameters::RawParameters;
    pub use crate::Direction;
    pub use crate::Direction::Fwd;
    pub use crate::Direction::Inv;
    // pub use crate::coordinate::CoordinateTuple as CoordinateTuple;
}

/// Preamble for external use
pub mod preamble {
    pub use crate::op::Op as Op;
    pub use crate::provider::Provider;
    pub use crate::provider::Minimal as Minimal;
    pub use crate::Direction;
    pub use crate::Error;
    pub use crate::CoordinateTuple;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("i/o error")]
    Io(#[from] io::Error),

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

