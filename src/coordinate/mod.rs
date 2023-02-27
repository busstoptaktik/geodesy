use crate::prelude::*;
pub mod coor2d;
pub mod coor32;
pub mod coord;
pub mod set;

/// Coordinate constructors
pub trait Coordinate {
    /// A `Coord` from latitude/longitude/height/time, with the angular input in degrees
    #[must_use]
    fn geo(latitude: f64, longitude: f64, height: f64, time: f64) -> Self;

    /// A `Coord` from longitude/latitude/height/time, with the angular input in seconds
    /// of arc. Mostly for handling grid shift elements.
    #[must_use]
    fn arcsec(longitude: f64, latitude: f64, height: f64, time: f64) -> Self;

    /// A `Coord` from longitude/latitude/height/time, with the angular input in degrees
    #[must_use]
    fn gis(longitude: f64, latitude: f64, height: f64, time: f64) -> Self;

    /// A `Coord` from longitude/latitude/height/time, with the angular input in radians
    #[must_use]
    fn raw(first: f64, second: f64, third: f64, fourth: f64) -> Self;

    /// A `Coord` from latitude/longitude/height/time,
    /// with the angular input in NMEA format: DDDMM.mmmmm
    #[must_use]
    fn nmea(latitude: f64, longitude: f64, height: f64, time: f64) -> Self;

    /// A `Coord` from latitude/longitude/height/time, with
    /// the angular input in extended NMEA format: DDDMMSS.sssss
    #[must_use]
    fn nmeass(latitude: f64, longitude: f64, height: f64, time: f64) -> Self;

    /// A `Coord` consisting of 4 `NaN`s
    #[must_use]
    fn nan() -> Self;

    /// A `Coord` consisting of 4 `0`s
    #[must_use]
    fn origin() -> Self;

    /// A `Coord` consisting of 4 `1`s
    #[must_use]
    fn ones() -> Self;

    /// Arithmetic (also see the operator trait implementations `add, sub, mul, div`)

    /// Multiply by a scalar
    #[must_use]
    fn scale(&self, factor: f64) -> Self;

    /// Scalar product
    #[must_use]
    fn dot(&self, other: Self) -> f64;
}

pub trait AngularUnits {
    /// Transform the first two elements of a `Coord` from degrees to radians
    fn to_radians(self) -> Self;

    /// Transform the first two elements of a `Coord` from radians to degrees
    fn to_degrees(self) -> Self;

    /// Transform the first two elements of a `Coord` from radians to seconds
    /// of arc.
    fn to_arcsec(self) -> Self;

    /// Transform the internal lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    fn to_geo(self) -> Self;
}

// For Rust Geodesy, a DirectPosition is represented as a geodesy::Coord.
type DirectPosition = Coord;
// The strict connection between the ISO19107 "DirectPosition" datatype
// and the ISO19111/OGC Topic 2 "CoordinateSet" interface (i.e. trait)
// is unclear to me: The DirectPosition, according to 19107, includes
// metadata which in the 19111 CoordinateSet interface is lifted from the
// DirectPosition to the CoordinateSet level. Nevertheless, the interface
// consists of an array of DirectPositions, and further derives from the
// CoordinateMetadata interface...

// ----- Coordinate Metadata --------------------------------------------------

// OGC 18-005r5, section 7.4 https://docs.ogc.org/as/18-005r5/18-005r5.html#12

#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone)]
pub struct DataEpoch(f64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct MdIdentifier(uuid::Uuid);

#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone)]
pub enum Crs {
    #[default]
    Unknown,
}

// ----- Interface: Coordinate Metadata ---------------------------------------

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

pub trait CoordinateSet: CoordinateMetadata {
    fn len(&self) -> usize;
    fn get_coord(&self, index: usize) -> DirectPosition;
    fn set_coord(&mut self, index: usize, value: &DirectPosition);
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
