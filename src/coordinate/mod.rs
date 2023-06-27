use crate::prelude::*;
pub mod coor2d;
pub mod coor32;
pub mod coor3d;
pub mod coor4d;
pub mod set;

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
#[allow(dead_code)]
type DirectPosition = Coor4D;
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

#[derive(Debug, Default, PartialEq, PartialOrd, Clone)]
pub enum Crs {
    #[default]
    Unknown,
    RegisterItem(String, String),
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
    fn get_coord(&self, index: usize) -> Coor4D;
    fn set_coord(&mut self, index: usize, value: &Coor4D);
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
