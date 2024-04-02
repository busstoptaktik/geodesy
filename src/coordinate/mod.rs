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
/// Strictly speaking, it is not a set, but (in abstract terms) rather an
/// indexed list, or (in more concrete terms): An array.
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

    /// Companion to `len()`
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

/// The CoordinateTuple is the ISO-19111 atomic spatial/spatiotemporal
/// referencing element. Loosely speaking, a CoordinateSet consists of
/// CoordinateTuples.
/// 
/// Note that (despite the formal name) the underlying data structure
/// need not be a tuple: It can be any item, for which it makes sense
/// to implement the CoordinateTuple trait.
/// 
/// The CoordinateTuple trait provides a number of convenience accessors
/// for accessing single coordinate elements or tuples of subsets.
/// These accessors are pragmatically named (x, y, xy, etc.). While these
/// names may be geodetically naive, they are suggestive and practical, and
/// aligns well with the internal coordinate order convention of most
/// Geodesy operators.
/// 
/// All accessors have default implementations, except the
/// [`unchecked_nth()`](crate::coordinate::CoordinateTuple::unchecked_nth) function,
/// which must be provided by the implementer.
/// 
/// When accessing dimensions outside of the domain of the CoordinateTuple,
/// [NaN](f64::NAN) will be returned.
#[rustfmt::skip]
pub trait CoordinateTuple {
    const DIMENSION: usize;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// May panic if n >= DIMENSION.
    /// See also [`nth()`](crate::coordinate::CoordinateTuple::nth).
    fn unchecked_nth(&self, n: usize) -> f64;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// Returns NaN if `n >= DIMENSION`.
    /// See also [`unchecked_nth()`](crate::coordinate::CoordinateTuple::unchecked_nth).
    fn nth(&self, n: usize) -> f64 {
        if Self::DIMENSION < n { self.nth(n) } else {f64::NAN}
    }

    /// Alternative to the DIMENSION associated const. May take over in order to
    /// make the trait object safe.
    fn dim(&self) -> usize {
        Self::DIMENSION
    }

    // Note: We use unchecked_nth and explicitly check for dimension in
    // y(), z() and t(), rather than leaving the check to nth(...).
    // This is because the checks in these cases are constant expressions,
    // and hence can be eliminated by the compiler in the concrete cases
    // of implementation.

    /// Pragmatically named accessor for the first element of the CoordinateTuple.
    fn x(&self) -> f64 {
        self.unchecked_nth(0)
    }

    /// Pragmatically named accessor for the second element of the CoordinateTuple.
    fn y(&self) -> f64 {
        if Self::DIMENSION > 1 { self.unchecked_nth(1) } else {f64::NAN}
    }
    
    /// Pragmatically named accessor for the third element of the CoordinateTuple.
    fn z(&self) -> f64 {
        if Self::DIMENSION > 2 { self.unchecked_nth(2) } else {f64::NAN}
    }
    
    /// Pragmatically named accessor for the fourth element of the CoordinateTuple.
    fn t(&self) -> f64 {
        if Self::DIMENSION > 3 { self.unchecked_nth(3) } else {f64::NAN}
    }

    /// A tuple containing the first two components of the CoordinateTuple.
    fn xy(&self) -> (f64, f64) {
        (self.x(), self.y())
    }

    /// A tuple containing the first three components of the CoordinateTuple.
    fn xyz(&self) -> (f64, f64, f64) {
        (self.x(), self.y(), self.z())
    }

    /// A tuple containing the first four components of the CoordinateTuple.
    fn xyzt(&self) -> (f64, f64, f64, f64) {
        (self.x(), self.y(), self.z(), self.t())
    }
}

// We must still implement the CoordinateTuple trait for
// the Geodesy data types Coor2D, Coor32, Coor3D, Coor4D
impl CoordinateTuple for Coor2D {
    const DIMENSION: usize = 2;
    fn unchecked_nth(&self, n: usize) -> f64 {
        self.0[n]
    }
}

impl CoordinateTuple for Coor3D {
    const DIMENSION: usize = 3;
    fn unchecked_nth(&self, n: usize) -> f64 {
        self.0[n]
    }
}

impl CoordinateTuple for Coor4D {
    const DIMENSION: usize = 4;
    fn unchecked_nth(&self, n: usize) -> f64 {
        self.0[n]
    }
}

impl CoordinateTuple for Coor32 {
    const DIMENSION: usize = 2;
    fn unchecked_nth(&self, n: usize) -> f64 {
        self.0[n] as f64
    }
}

// And let's also implement it for a plain 2D f64 tuple
#[rustfmt::skip]
impl CoordinateTuple for (f64, f64) {
    const DIMENSION: usize = 2;
    fn unchecked_nth(&self, n: usize) -> f64 {
        match n {
            0 => self.0,
            1 => self.1,
            _ => panic!()
        }
    }
}
