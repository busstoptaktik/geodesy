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

// An experiment with an extended version of Kyle Barron's CoordTrait<DIMENSION, MEASURE> PR
// over at https://github.com/georust/geo/pull/1157

pub trait CoordNum {}
impl CoordNum for f32 {}
impl CoordNum for f64 {}

/// A trait for accessing data from a generic Coord.
pub trait CoordTrait {
    type T: CoordNum;
    const DIMENSION: usize;
    const MEASURE: bool;

    // Required implementations

    /// Accessors for the coordinate tuple components
    fn x(&self) -> Self::T;
    fn y(&self) -> Self::T;
    fn z(&self) -> Self::T;
    fn t(&self) -> Self::T;
    fn m(&self) -> Self::T;

    /// Accessors for the coordinate tuple components converted to f64
    fn x_as_f64(&self) -> f64;
    fn y_as_f64(&self) -> f64;
    fn z_as_f64(&self) -> f64;
    fn t_as_f64(&self) -> f64;
    fn m_as_f64(&self) -> f64;

    // Provided implementations

    /// Returns a tuple that contains the two first components of the coord.
    fn xy(&self) -> (Self::T, Self::T) {
        (self.x(), self.y())
    }

    /// Returns a tuple that contains the three first components of the coord.
    fn xyz(&self) -> (Self::T, Self::T, Self::T) {
        (self.x(), self.y(), self.z())
    }

    /// Returns a tuple that contains the three first components of the coord.
    fn xyzt(&self) -> (Self::T, Self::T, Self::T, Self::T) {
        (self.x(), self.y(), self.z(), self.t())
    }

    /// Returns a tuple that contains the two first components of the coord converted to f64.
    fn xy_as_f64(&self) -> (f64, f64) {
        (self.x_as_f64(), self.y_as_f64())
    }

    /// Returns a tuple that contains the three first components of the coord converted to f64.
    fn xyz_as_f64(&self) -> (f64, f64, f64) {
        (self.x_as_f64(), self.y_as_f64(), self.z_as_f64())
    }

    /// Returns a tuple that contains the three first components of the coord converted to f64.
    fn xyzt_as_f64(&self) -> (f64, f64, f64, f64) {
        (
            self.x_as_f64(),
            self.y_as_f64(),
            self.z_as_f64(),
            self.t_as_f64(),
        )
    }
}

impl CoordTrait for Coor2D {
    type T = f64;
    const DIMENSION: usize = 2;
    const MEASURE: bool = false;

    /// Accessors for the coordinate tuple components
    fn x(&self) -> Self::T {
        self.0[0]
    }
    fn y(&self) -> Self::T {
        self.0[1]
    }
    fn z(&self) -> Self::T {
        f64::NAN
    }
    fn t(&self) -> Self::T {
        f64::NAN
    }
    fn m(&self) -> Self::T {
        f64::NAN
    }

    /// Accessors for the coordinate tuple components converted to f64
    fn x_as_f64(&self) -> f64 {
        self.0[0]
    }
    fn y_as_f64(&self) -> f64 {
        self.0[1]
    }
    fn z_as_f64(&self) -> f64 {
        f64::NAN
    }
    fn t_as_f64(&self) -> f64 {
        f64::NAN
    }
    fn m_as_f64(&self) -> f64 {
        f64::NAN
    }
}

impl CoordTrait for Coor32 {
    type T = f32;
    const DIMENSION: usize = 2;
    const MEASURE: bool = false;

    /// Accessors for the coordinate tuple components
    fn x(&self) -> Self::T {
        self.0[0]
    }
    fn y(&self) -> Self::T {
        self.0[1]
    }
    fn z(&self) -> Self::T {
        f32::NAN
    }
    fn t(&self) -> Self::T {
        f32::NAN
    }
    fn m(&self) -> Self::T {
        f32::NAN
    }

    /// Accessors for the coordinate tuple components converted to f64
    fn x_as_f64(&self) -> f64 {
        self.0[0] as f64
    }
    fn y_as_f64(&self) -> f64 {
        self.0[1] as f64
    }
    fn z_as_f64(&self) -> f64 {
        f64::NAN
    }
    fn t_as_f64(&self) -> f64 {
        f64::NAN
    }
    fn m_as_f64(&self) -> f64 {
        f64::NAN
    }
}

impl CoordTrait for Coor3D {
    type T = f64;
    const DIMENSION: usize = 3;
    const MEASURE: bool = false;

    /// Accessors for the coordinate tuple components
    fn x(&self) -> Self::T {
        self.0[0]
    }
    fn y(&self) -> Self::T {
        self.0[1]
    }
    fn z(&self) -> Self::T {
        self.0[2]
    }
    fn t(&self) -> Self::T {
        f64::NAN
    }
    fn m(&self) -> Self::T {
        f64::NAN
    }

    /// Accessors for the coordinate tuple components converted to f64
    fn x_as_f64(&self) -> f64 {
        self.0[0]
    }
    fn y_as_f64(&self) -> f64 {
        self.0[1]
    }
    fn z_as_f64(&self) -> f64 {
        self.0[2]
    }
    fn t_as_f64(&self) -> f64 {
        f64::NAN
    }
    fn m_as_f64(&self) -> f64 {
        f64::NAN
    }
}

impl CoordTrait for Coor4D {
    type T = f64;
    const DIMENSION: usize = 4;
    const MEASURE: bool = false;

    /// Accessors for the coordinate tuple components
    fn x(&self) -> Self::T {
        self.0[0]
    }
    fn y(&self) -> Self::T {
        self.0[1]
    }
    fn z(&self) -> Self::T {
        self.0[2]
    }
    fn t(&self) -> Self::T {
        self.0[3]
    }
    fn m(&self) -> Self::T {
        f64::NAN
    }

    /// Accessors for the coordinate tuple components converted to f64
    fn x_as_f64(&self) -> f64 {
        self.0[0]
    }
    fn y_as_f64(&self) -> f64 {
        self.0[1]
    }
    fn z_as_f64(&self) -> f64 {
        self.0[2]
    }
    fn t_as_f64(&self) -> f64 {
        self.0[3]
    }
    fn m_as_f64(&self) -> f64 {
        f64::NAN
    }
}
