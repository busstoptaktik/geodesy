use crate::prelude::*;
pub mod coor2d;
pub mod coor32;
pub mod coor3d;
pub mod coor4d;
pub mod set;

/// Methods for changing the coordinate representation of angles.
/// Dimensionality untold, the methods operate on the first two
/// dimensions only.
pub trait AngularUnits {
    /// Transform the first two elements of a coordinate tuple from degrees to radians
    fn to_radians(self) -> Self;

    /// Transform the first two elements of a coordinate tuple from radians to degrees
    fn to_degrees(self) -> Self;

    /// Transform the first two elements of a coordinate tuple from radians to seconds
    /// of arc.
    fn to_arcsec(self) -> Self;

    /// Transform the internal lon/lat(/h/t)-in-radians to lat/lon(/h/t)-in-degrees
    fn to_geo(self) -> Self;
}

/// For Rust Geodesy, the ISO-19111 concept of `DirectPosition` is represented
/// as a `geodesy::Coo4D`.
///
/// The strict connection between the ISO19107 "DirectPosition" datatype
/// and the ISO19111/OGC Topic 2 "CoordinateSet" interface (i.e. trait)
/// is unclear to me: The DirectPosition, according to 19107, includes
/// metadata which in the 19111 CoordinateSet interface is lifted from the
/// DirectPosition to the CoordinateSet level. Nevertheless, the interface
/// consists of an array of DirectPositions, and further derives from the
/// CoordinateMetadata interface...
#[allow(dead_code)]
type DirectPosition = Coor4D;

// ----- Coordinate Metadata --------------------------------------------------

/// OGC 18-005r5, section 7.4 https://docs.ogc.org/as/18-005r5/18-005r5.html#12
#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone)]
pub struct DataEpoch(f64);

/// The metadataidentifier (CRS id) is represented by an UUID placeholder
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct MdIdentifier(uuid::Uuid);

/// CRS given as a register item
#[derive(Debug, Default, PartialEq, PartialOrd, Clone)]
pub enum Crs {
    #[default]
    Unknown,
    RegisterItem(String, String),
}

// ----- Interface: Coordinate Metadata ---------------------------------------

/// The ISO-19111 Coordinate Metadata gamut includes an optional
///  epoch and one of two possible ways of representing the CRS
pub trait CoordinateMetadata {
    fn crs_id(&self) -> Option<MdIdentifier> {
        None
    }
    fn crs(&self) -> Option<Crs> {
        Some(Crs::Unknown)
    }
    fn coordinate_epoch(&self) -> Option<DataEpoch> {
        None
    }
    // constraints
    fn is_valid(&self) -> bool {
        if self.crs_id().is_none() && self.crs().is_none() {
            return false;
        }
        true
        // TODO: check for coordinate_epoch.is_some() for dynamic crs
    }
}

// Preliminary empty blanket implementation: Defaults for all items, for all types
impl<T> CoordinateMetadata for T where T: ?Sized {}

/// CoordinateSet is the fundamental coordinate access interface in ISO-19111.
///
/// Here it is implemented simply as an accessor trait, that allows us to
/// access any user provided data model by iterating over its elements,
/// represented as a `Coor4D`
pub trait CoordinateSet: CoordinateMetadata {
    /// Number of coordinate tuples in the set
    fn len(&self) -> usize;
    /// Access the `index`th coordinate tuple
    fn get_coord(&self, index: usize) -> Coor4D;
    /// Overwrite the `index`th coordinate tuple
    fn set_coord(&mut self, index: usize, value: &Coor4D);
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Set all coordinate tuples in the set to NaN
    fn stomp(&mut self) {
        let nanny = Coor4D::nan();
        for i in 0..self.len() {
            self.set_coord(i, &nanny);
        }
    }
}

//-----------------------------------------------------------------------
// An experiment with an extended version of Kyle Barron's CoordTrait PR
// over at https://github.com/georust/geo/pull/1157
//-----------------------------------------------------------------------

// The next 8 lines are mostly copied from georust/geo/geo-types/src/lib.rs
// Although I added ToPrimitive in the first line
use core::fmt::Debug;
use num_traits::{Float, Num, NumCast, ToPrimitive};
pub trait CoordinateType: Num + Copy + NumCast + PartialOrd + Debug {}
impl<T: Num + Copy + NumCast + PartialOrd + Debug> CoordinateType for T {}
pub trait CoordNum: CoordinateType + Debug {}
impl<T: CoordinateType + Debug> CoordNum for T {}
pub trait CoordFloat: CoordNum + Float {}
impl<T: CoordNum + Float> CoordFloat for T {}

// And here is Kyle Barron's CoordTrait from https://github.com/georust/geo/pull/1157
// extended with z(), t(), m(), and associated consts DIMENSION and MEASURE
pub trait CoordTrait {
    type T: CoordNum;
    const DIMENSION: usize;
    const MEASURE: bool;

    /// Accessors for the coordinate tuple components
    fn x(&self) -> Self::T;
    fn y(&self) -> Self::T;
    fn z(&self) -> Self::T;
    fn t(&self) -> Self::T;
    fn m(&self) -> Self::T;

    /// Returns a tuple that contains the two first components of the coord.
    fn x_y(&self) -> (Self::T, Self::T) {
        (self.x(), self.y())
    }
}

// The CoordTuples trait is blanket-implemented for anything that
// CoordTrait is implemented for
impl<C: CoordTrait> CoordTuples for C {}

// And here the actual implementation, which takes any CoordTrait implementing
// data type, and lets us access the contents as geodesy-compatible f64 tuples
#[rustfmt::skip]
pub trait CoordTuples: CoordTrait {
    /// Accessors for the coordinate tuple components converted to f64
    fn x_as_f64(&self) -> f64 { self.x().to_f64().unwrap_or(f64::NAN) }
    fn y_as_f64(&self) -> f64 { self.y().to_f64().unwrap_or(f64::NAN) }
    fn z_as_f64(&self) -> f64 { self.z().to_f64().unwrap_or(f64::NAN) }
    fn t_as_f64(&self) -> f64 { self.t().to_f64().unwrap_or(f64::NAN) }
    fn m_as_f64(&self) -> f64 { self.m().to_f64().unwrap_or(f64::NAN) }

    fn xy_as_f64(&self) -> (f64, f64) {
        (self.x_as_f64(), self.y_as_f64())
    }

    fn xyz_as_f64(&self) -> (f64, f64, f64) {
        (self.x_as_f64(), self.y_as_f64(), self.z_as_f64())
    }

    fn xyzt_as_f64(&self) -> (f64, f64, f64, f64) {
        (self.x_as_f64(), self.y_as_f64(), self.z_as_f64(), self.t_as_f64())
    }
}

// We must still implement the foundational CoordTrait trait for
// the Geodesy data types Coor2D, Coor32, Coor3D, Coor4D

#[rustfmt::skip]
impl CoordTrait for Coor2D {
    type T = f64;
    const DIMENSION: usize = 2;
    const MEASURE: bool = false;
    fn x(&self) -> Self::T { self.0[0] }
    fn y(&self) -> Self::T { self.0[1] }
    fn z(&self) -> Self::T { f64::NAN  }
    fn t(&self) -> Self::T { f64::NAN  }
    fn m(&self) -> Self::T { f64::NAN  }
}

#[rustfmt::skip]
impl CoordTrait for Coor32 {
    type T = f32;
    const DIMENSION: usize = 2;
    const MEASURE: bool = false;
    fn x(&self) -> Self::T { self.0[0] }
    fn y(&self) -> Self::T { self.0[1] }
    fn z(&self) -> Self::T { f32::NAN  }
    fn t(&self) -> Self::T { f32::NAN  }
    fn m(&self) -> Self::T { f32::NAN  }
}

#[rustfmt::skip]
impl CoordTrait for Coor3D {
    type T = f64;
    const DIMENSION: usize = 3;
    const MEASURE: bool = false;
    fn x(&self) -> Self::T { self.0[0] }
    fn y(&self) -> Self::T { self.0[1] }
    fn z(&self) -> Self::T { self.0[2] }
    fn t(&self) -> Self::T { f64::NAN  }
    fn m(&self) -> Self::T { f64::NAN  }
}

#[rustfmt::skip]
impl CoordTrait for Coor4D {
    type T = f64;
    const DIMENSION: usize = 4;
    const MEASURE: bool = false;
    fn x(&self) -> Self::T { self.0[0] }
    fn y(&self) -> Self::T { self.0[1] }
    fn z(&self) -> Self::T { self.0[2] }
    fn t(&self) -> Self::T { self.0[3] }
    fn m(&self) -> Self::T { f64::NAN  }
}

// And let's also implement it for a plain 2D f64 tuple
#[rustfmt::skip]
impl CoordTrait for (f64, f64) {
    type T = f64;
    const DIMENSION: usize = 2;
    const MEASURE: bool = false;
    fn x(&self) -> Self::T { self.0    }
    fn y(&self) -> Self::T { self.1    }
    fn z(&self) -> Self::T { f64::NAN  }
    fn t(&self) -> Self::T { f64::NAN  }
    fn m(&self) -> Self::T { f64::NAN  }
}
