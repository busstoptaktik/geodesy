#![doc = include_str!("../README.md")]

/// The bread-and-butter, shrink-wrapped and ready to use
pub mod prelude {
    pub use crate::Error;
    pub use crate::coord::*;
    pub use crate::ctx::*;
    pub use crate::ellps::*;
}

/// Extended prelude for authoring Contexts and InnerOp modules
pub mod authoring {
    pub use crate::grd::*;
    pub use crate::math::*;
    pub use crate::ops::*;
    pub use crate::parse::*;
    pub use crate::prelude::*;

    // All new contexts are supposed to support these
    pub use crate::context::BUILTIN_ADAPTORS;

    // Map projection characteristics
    pub use crate::math::jacobian::Factors;
    pub use crate::math::jacobian::Jacobian;

    // External material
    pub use log::debug;
    pub use log::error;
    pub use log::info;
    pub use log::trace;
    pub use log::warn;
    pub use std::collections::BTreeMap;
}

/// Context related elements
pub mod ctx {
    pub use crate::Direction;
    pub use crate::Direction::Fwd;
    pub use crate::Direction::Inv;
    pub use crate::context::Context;
    pub use crate::context::minimal::Minimal;
    #[cfg(feature = "with_plain")]
    pub use crate::context::plain::Plain;
    pub use crate::op::OpHandle;
}

/// Ellipsoid related elements
pub mod ellps {
    pub use crate::ellipsoid::EllipsoidBase;
    pub use crate::ellipsoid::biaxial::Ellipsoid;
    pub use crate::ellipsoid::geocart::GeoCart;
    pub use crate::ellipsoid::geodesics::Geodesics;
    pub use crate::ellipsoid::gravity::Gravity;
    pub use crate::ellipsoid::latitudes::Latitudes;
    pub use crate::ellipsoid::meridians::Meridians;
    pub use crate::ellipsoid::triaxial::TriaxialEllipsoid;
}

/// Coordinate related elements
pub mod coord {
    // Coordinate types
    pub use crate::coordinate::coor2d::Coor2D;
    pub use crate::coordinate::coor3d::Coor3D;
    pub use crate::coordinate::coor4d::Coor4D;
    pub use crate::coordinate::coor32::Coor32;
    // Coordinate traits
    pub use crate::coordinate::AngularUnits;
    pub use crate::coordinate::CoordinateMetadata;
    pub use crate::coordinate::set::CoordinateSet;
    pub use crate::coordinate::tuple::CoordinateTuple;
    pub use crate::math::angular;
}

/// Elements for building operators
mod ops {
    pub use crate::inner_op::InnerOp;
    pub use crate::inner_op::OpConstructor;
    pub use crate::op::Op;
    pub use crate::op::OpDescriptor;
    pub use crate::op::OpParameter;
    pub use crate::op::ParsedParameters;
    pub use crate::op::RawParameters;
}

/// Elements for handling grids
mod grd {
    pub use crate::grid::BaseGrid;
    pub use crate::grid::Grid;
    pub use crate::grid::grids_at;
    pub use crate::grid::ntv2::Ntv2Grid;
}

/// Elements for parsing both Geodesy and PROJ syntax
mod parse {
    // Tokenizing Rust Geodesy operations
    pub use crate::token::Tokenize;
    // PROJ interoperability
    pub use crate::token::parse_proj;
}

use thiserror::Error;
/// The *Rust Geodesy* error messaging enumeration. Badly needs reconsideration
#[derive(Error, Debug)]
pub enum Error {
    #[error("i/o error")]
    Io(#[from] std::io::Error),

    #[error("General error: '{0}'")]
    General(&'static str),

    #[error("Syntax error: '{0}'")]
    Syntax(String),

    #[error("{0}: {1}")]
    Operator(&'static str, &'static str),

    #[error("Invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("{message:?} (expected {expected:?}, found {found:?})")]
    Unexpected {
        message: String,
        expected: String,
        found: String,
    },

    #[error("Operator '{0}' not found{1}")]
    NotFound(String, String),

    #[error("Recursion too deep for '{0}', at {1}")]
    Recursion(String, String),

    #[error("Attempt to invert a non-invertible item: '{0}'")]
    NonInvertible(String),

    #[error("Missing required parameter '{0}'")]
    MissingParam(String),

    #[error("Malformed value for parameter '{0}': '{1}'")]
    BadParam(String, String),

    #[error("Unsupported: {0}")]
    Unsupported(String),

    #[error("Invalid: {0}")]
    Invalid(String),

    #[error("UTF8 error")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("Unknown")]
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

mod bibliography;
mod context;
mod coordinate;
mod ellipsoid;
mod grid;
mod inner_op;
mod math;
mod op;
mod token;

/// Some generic coordinates for test composition
#[cfg(test)]
mod test_data {
    pub fn coor4d() -> [crate::coord::Coor4D; 2] {
        let copenhagen = crate::coord::Coor4D::raw(55., 12., 0., 0.);
        let stockholm = crate::coord::Coor4D::raw(59., 18., 0., 0.);
        [copenhagen, stockholm]
    }

    pub fn coor3d() -> [crate::coord::Coor3D; 2] {
        let copenhagen = crate::coord::Coor3D::raw(55., 12., 0.);
        let stockholm = crate::coord::Coor3D::raw(59., 18., 0.);
        [copenhagen, stockholm]
    }

    pub fn coor2d() -> [crate::coord::Coor2D; 2] {
        let copenhagen = crate::coord::Coor2D::raw(55., 12.);
        let stockholm = crate::coord::Coor2D::raw(59., 18.);
        [copenhagen, stockholm]
    }

    pub fn coor32() -> [crate::coord::Coor32; 2] {
        let copenhagen = crate::coord::Coor32::raw(55., 12.);
        let stockholm = crate::coord::Coor32::raw(59., 18.);
        [copenhagen, stockholm]
    }
}

// ---- Documentation: Bibliography ----
#[cfg(doc)]
pub use crate::bibliography::Bibliography;
