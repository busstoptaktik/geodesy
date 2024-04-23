use crate::prelude::*;
pub mod set;
pub mod tuple;

pub mod coor2d;
pub mod coor32;
pub mod coor3d;
pub mod coor4d;

/// Methods for changing the coordinate representation of angles.
/// Dimensionality untold, the methods operate on the first two
/// dimensions only.
pub trait AngularUnits {
    /// Transform the first two elements of a coordinate tuple from degrees to radians
    fn to_radians(&self) -> Self;

    /// Transform the first two elements of a coordinate tuple from radians to degrees
    fn to_degrees(&self) -> Self;

    /// Transform the first two elements of a coordinate tuple from radians to seconds
    /// of arc.
    fn to_arcsec(&self) -> Self;

    /// Transform the internal lon/lat(/h/t)-in-radians to lat/lon(/h/t)-in-degrees
    fn to_geo(&self) -> Self;
}

impl<T> AngularUnits for T
where
    T: CoordinateTuple + Copy,
{
    /// Convert the first two elements of `self` from radians to degrees
    fn to_degrees(&self) -> Self {
        let (x, y) = self.xy();
        let mut res = *self;
        res.set_xy(x.to_degrees(), y.to_degrees());
        res
    }

    /// Convert the first two elements of `self` from radians to degrees
    fn to_arcsec(&self) -> Self {
        let (x, y) = self.xy();
        let mut res = *self;
        res.set_xy(x.to_degrees() * 3600., y.to_degrees() * 3600.);
        res
    }

    /// Convert the first two elements of `self` from degrees to radians
    fn to_radians(&self) -> Self {
        let (x, y) = self.xy();
        let mut res = *self;
        res.set_xy(x.to_radians(), y.to_radians());
        res
    }

    /// Convert-and-swap the first two elements of `self` from radians to degrees
    fn to_geo(&self) -> Self {
        let (x, y) = self.xy();
        let mut res = *self;
        res.set_xy(y.to_degrees(), x.to_degrees());
        res
    }
}

/// For Rust Geodesy, the ISO-19111 concept of `DirectPosition` is represented
/// as a `geodesy::Coor4D`.
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
