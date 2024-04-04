use super::*;
use crate::math::angular;

/// Generic 2D Coordinate tuple, with no fixed interpretation of the elements
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Coor2D(pub [f64; 2]);

use std::ops::{Index, IndexMut};

impl Index<usize> for Coor2D {
    type Output = f64;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for Coor2D {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

// ----- C O N S T R U C T O R S ---------------------------------------------

/// Constructors
impl Coor2D {
    /// A `Coor2D` from latitude/longitude/height/time, with the angular input in degrees,
    /// and height and time ignored.
    #[must_use]
    pub fn geo(latitude: f64, longitude: f64) -> Coor2D {
        Coor2D([longitude.to_radians(), latitude.to_radians()])
    }

    /// A `Coor2D` from longitude/latitude/height/time, with the angular input in seconds
    /// of arc. Mostly for handling grid shift elements.
    #[must_use]
    pub fn arcsec(longitude: f64, latitude: f64) -> Coor2D {
        Coor2D([
            longitude.to_radians() / 3600.,
            latitude.to_radians() / 3600.,
        ])
    }

    /// A `Coor2D` from longitude/latitude/height/time, with the angular input in degrees.
    /// and height and time ignored.
    #[must_use]
    pub fn gis(longitude: f64, latitude: f64) -> Coor2D {
        Coor2D([longitude.to_radians(), latitude.to_radians()])
    }

    /// A `Coor2D` from longitude/latitude/height/time, with the angular input in radians,
    /// and third and fourth arguments ignored.
    #[must_use]
    pub fn raw(first: f64, second: f64) -> Coor2D {
        Coor2D([first, second])
    }

    /// A `Coor2D` from latitude/longitude/height/time, with
    /// the angular input in the ISO-6709 DDDMM.mmmmm format,
    /// and height and time ignored.
    #[must_use]
    pub fn iso_dm(latitude: f64, longitude: f64) -> Coor2D {
        let longitude = angular::iso_dm_to_dd(longitude);
        let latitude = angular::iso_dm_to_dd(latitude);
        Coor2D([longitude.to_radians(), latitude.to_radians()])
    }

    /// A `Coor2D` from latitude/longitude/height/time, with the
    /// angular input in the ISO-6709 DDDMMSS.sssss format,
    /// and height and time ignored.
    #[must_use]
    pub fn iso_dms(latitude: f64, longitude: f64) -> Coor2D {
        let longitude = angular::iso_dms_to_dd(longitude);
        let latitude = angular::iso_dms_to_dd(latitude);
        Coor2D::geo(latitude, longitude)
    }

    /// A `Coor2D` consisting of 2 `NaN`s
    #[must_use]
    pub fn nan() -> Coor2D {
        Coor2D([f64::NAN, f64::NAN])
    }

    /// A `Coor2D` consisting of 2 `0`s
    #[must_use]
    pub fn origin() -> Coor2D {
        Coor2D([0., 0.])
    }

    /// A `Coor2D` consisting of 2 `1`s
    #[must_use]
    pub fn ones() -> Coor2D {
        Coor2D([1., 1.])
    }

    /// Arithmetic (also see the operator trait implementations `add, sub, mul, div`)

    /// Multiply by a scalar
    #[must_use]
    pub fn scale(&self, factor: f64) -> Coor2D {
        Coor2D([self.x() * factor, self.y() * factor])
    }

    /// Scalar product
    #[must_use]
    pub fn dot(&self, other: Coor2D) -> f64 {
        self.x() * other.x() + self.y() * other.y()
    }
}

// ----- T E S T S ---------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distances() {
        let e = Ellipsoid::default();
        let lat = angular::dms_to_dd(55, 30, 36.);
        let lon = angular::dms_to_dd(12, 45, 36.);
        let dms = Coor2D::geo(lat, lon);
        let geo = Coor2D::geo(55.51, 12.76);
        assert!(e.distance(&geo, &dms) < 1e-10);
    }

    #[test]
    fn coor2d() {
        let c = Coor2D::raw(12., 55.).to_radians();
        let d = Coor2D::gis(12., 55.);
        assert_eq!(c, d);
        assert_eq!(d.x(), 12f64.to_radians());
        assert_eq!(d.x().to_degrees(), c.x().to_degrees());
    }

    #[test]
    fn array() {
        let b = Coor2D::raw(7., 8.);
        let c = [b.x(), b.y(), f64::NAN, f64::NAN];
        assert_eq!(b.x(), c[0]);
    }

    #[test]
    fn arithmetic() {
        let a = Coor2D([1., 2.]);
        let b = Coor2D([4., 3.]);
        assert_eq!(a.dot(b), 10.)
    }
}
