use crate::prelude::*;

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
impl<T> CoordinateMetadata for T where T: ?Sized { }

pub trait CoordinateSet: CoordinateMetadata {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> DirectPosition;
    fn set(&mut self, index: usize, value: &DirectPosition);
}

impl<const N: usize> CoordinateSet for [Coord; N] {
    fn len(&self) -> usize {
        N
    }

    fn get(&self, index: usize) -> Coord {
        self[index]
    }

    fn set(&mut self, index: usize, value: &Coord) {
        self[index] = *value;
    }
}


impl CoordinateSet for Vec<Coord> {
    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, index: usize) -> Coord {
        self[index]
    }

    fn set(&mut self, index: usize, value: &Coord) {
        self[index] = *value;
    }
}


// ----- Implementations: DirectPosition --------------------------------------

// impl DirectPosition for Coord {
//     fn into(&self) -> Coord {
//         *self
//     }
//     fn from(&mut self, value: Coord) {
//         *self = value;
//     }
// }

// ----- Implementations: Coordinate Metadata ---------------------------------
impl MdIdentifier {
    pub fn new() -> Self {
        MdIdentifier(uuid::Uuid::new_v4())
    }
}
impl Default for MdIdentifier {
    fn default() -> Self {
        MdIdentifier(uuid::Uuid::new_v4())
    }
}

impl DataEpoch {
    pub fn new() -> Self {
        DataEpoch(f64::NAN)
    }
}
