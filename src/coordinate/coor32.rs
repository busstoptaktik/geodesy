/// Tiny coordinate type: 2D, 32 bits, only one fourth the weight of a Coord.
/// Probably only useful for small scale world maps, without too much zoom.
use super::*;
use crate::math::angular;
use std::ops::{Index, IndexMut};

/// Generic 2D Coordinate tuple, with no fixed interpretation of the elements
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Coor32(pub [f32; 2]);

// ----- O P E R A T O R   T R A I T S -------------------------------------------------

impl Index<usize> for Coor32 {
    type Output = f32;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for Coor32 {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

// ----- A N G U L A R   U N I T S -------------------------------------------

impl AngularUnits for Coor32 {
    /// Transform the first two elements of a `Coor32` from degrees to radians
    #[must_use]
    fn to_radians(self) -> Self {
        Coor32([self[0].to_radians(), self[1].to_radians()])
    }

    /// Transform the elements of a `Coor32` from radians to degrees
    #[must_use]
    fn to_degrees(self) -> Self {
        Coor32([self[0].to_degrees(), self[1].to_degrees()])
    }

    /// Transform the elements of a `Coor32` from radians to seconds of arc.
    #[must_use]
    fn to_arcsec(self) -> Self {
        Coor32([self[0].to_degrees() * 3600., self[1].to_degrees() * 3600.])
    }

    /// Transform the internal lon/lat-in-radians to lat/lon-in-degrees
    #[must_use]
    fn to_geo(self) -> Self {
        Coor32([self[1].to_degrees(), self[0].to_degrees()])
    }
}

// ----- C O N S T R U C T O R S ---------------------------------------------

/// Constructors
impl Coor32 {
    /// A `Coor32` from latitude/longitude/height/time, with the angular input in degrees,
    /// and height and time ignored.
    #[must_use]
    pub fn geo(latitude: f64, longitude: f64) -> Coor32 {
        Coor32([longitude.to_radians() as f32, latitude.to_radians() as f32])
    }

    /// A `Coor32` from longitude/latitude/height/time, with the angular input in seconds
    /// of arc, and height and time ignored. Mostly for handling grid shift elements.
    #[must_use]
    pub fn arcsec(longitude: f64, latitude: f64) -> Coor32 {
        Coor32([
            (longitude.to_radians() / 3600.) as f32,
            (latitude.to_radians() / 3600.) as f32,
        ])
    }

    /// A `Coor32` from longitude/latitude/height/time, with the angular input in degrees,
    /// and height and time ignored.
    #[must_use]
    pub fn gis(longitude: f64, latitude: f64) -> Coor32 {
        Coor32([longitude.to_radians() as f32, latitude.to_radians() as f32])
    }

    /// A `Coor32` from longitude/latitude/height/time, with the angular input in radians,
    /// and height and time ignored.
    #[must_use]
    pub fn raw(first: f64, second: f64) -> Coor32 {
        Coor32([first as f32, second as f32])
    }

    /// A `Coor32` from latitude/longitude/height/time, with
    /// the angular input in the ISO-6709 DDDMM.mmmmm format,
    /// and height and time ignored.
    #[must_use]
    pub fn iso_dm(latitude: f64, longitude: f64) -> Coor32 {
        let longitude = angular::iso_dm_to_dd(longitude);
        let latitude = angular::iso_dm_to_dd(latitude);
        Coor32([longitude.to_radians() as f32, latitude.to_radians() as f32])
    }

    /// A `Coor32` from latitude/longitude/height/time, with the
    /// angular input in the ISO-6709 DDDMMSS.sssss format,
    /// and height and time ignored.
    #[must_use]
    pub fn iso_dms(latitude: f64, longitude: f64) -> Coor32 {
        let longitude = angular::iso_dms_to_dd(longitude);
        let latitude = angular::iso_dms_to_dd(latitude);
        Coor32::geo(latitude, longitude)
    }

    /// A `Coor32` consisting of 2 `NaN`s
    #[must_use]
    pub fn nan() -> Coor32 {
        Coor32([f32::NAN, f32::NAN])
    }

    /// A `Coor32` consisting of 2 `0`s
    #[must_use]
    pub fn origin() -> Coor32 {
        Coor32([0., 0.])
    }

    /// A `Coor32` consisting of 2 `1`s
    #[must_use]
    pub fn ones() -> Coor32 {
        Coor32([1., 1.])
    }

    /// Arithmetic (also see the operator trait implementations `add, sub, mul, div`)

    /// Multiply by a scalar
    #[must_use]
    pub fn scale(&self, factor: f64) -> Coor32 {
        Coor32([self[0] * factor as f32, self[1] * factor as f32])
    }

    /// Scalar product
    #[must_use]
    pub fn dot(&self, other: Coor32) -> f64 {
        self[0] as f64 * other[0] as f64 + self[1] as f64 * other[1] as f64
    }
}

impl From<Coor32> for Coor4D {
    fn from(c: Coor32) -> Self {
        Coor4D([c[0] as f64, c[1] as f64, 0.0, f64::NAN])
    }
}

impl From<Coor4D> for Coor32 {
    fn from(xyzt: Coor4D) -> Self {
        Coor32::raw(xyzt[0], xyzt[1])
    }
}

// ----- D I S T A N C E S ---------------------------------------------------

impl Coor32 {
    /// Euclidean distance between two points in the 2D plane.
    ///
    /// Primarily used to compute the distance between two projected points
    /// in their projected plane. Typically, this distance will differ from
    /// the actual distance in the real world.
    ///
    /// # See also:
    ///
    /// [`distance`](crate::ellipsoid::Ellipsoid::distance)
    ///
    /// # Examples
    ///
    /// ```
    /// use geodesy::prelude::*;
    /// let t = 1000.;
    /// let p0 = Coor32::origin();
    /// let p1 = Coor32::raw(t, t);
    /// assert_eq!(p0.hypot2(&p1), t.hypot(t));
    /// ```
    #[must_use]
    pub fn hypot2(&self, other: &Self) -> f64 {
        (self[0] as f64 - other[0] as f64).hypot(self[1] as f64 - other[1] as f64)
    }
}

// ----- T E S T S ---------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distances() {
        let lat = angular::dms_to_dd(55, 30, 36.);
        let lon = angular::dms_to_dd(12, 45, 36.);
        let dms = Coor32::geo(lat, lon);
        let geo = Coor32::geo(55.51, 12.76);
        let e = &Ellipsoid::default();
        assert!(e.distance(&dms, &geo) < 1e-9);
    }

    #[test]
    fn coor32() {
        let c = Coor32::raw(12., 55.).to_radians();
        let d = Coor32::gis(12., 55.);
        assert_eq!(c, d);
        assert_eq!(d[0], 12f32.to_radians());
        let e = d.to_degrees();
        assert_eq!(e[0], c.to_degrees()[0]);
    }

    #[test]
    fn array() {
        let b = Coor32::raw(7., 8.);
        let c = [b[0], b[1], f32::NAN, f32::NAN];
        assert_eq!(b[0], c[0]);
    }

    #[test]
    fn arithmetic() {
        let a = Coor32([1., 2.]);
        let b = Coor32([4., 3.]);
        assert_eq!(a.dot(b), 10.)
    }
}
