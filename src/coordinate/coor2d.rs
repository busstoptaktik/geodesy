use super::*;
use crate::math::angular;
use std::ops::{Index, IndexMut};

/// Generic 2D Coordinate tuple, with no fixed interpretation of the elements
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Coor2D(pub [f64; 2]);

// ----- O P E R A T O R   T R A I T S -------------------------------------------------

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

// ----- A N G U L A R   U N I T S -------------------------------------------

impl AngularUnits for Coor2D {
    /// Transform the first two elements of a `Coor2D` from degrees to radians
    #[must_use]
    fn to_radians(self) -> Self {
        Coor2D([self[0].to_radians(), self[1].to_radians()])
    }

    /// Transform the first two elements of a `Coor2D` from radians to degrees
    #[must_use]
    fn to_degrees(self) -> Self {
        Coor2D([self[0].to_degrees(), self[1].to_degrees()])
    }

    /// Transform the elements of a `Coor2D` from radians to seconds of arc.
    #[must_use]
    fn to_arcsec(self) -> Self {
        Coor2D([self[0].to_degrees() * 3600., self[1].to_degrees() * 3600.])
    }

    /// Transform the internal lon/lat-in-radians to lat/lon-in-degrees
    #[must_use]
    fn to_geo(self) -> Self {
        Coor2D([self[1].to_degrees(), self[0].to_degrees()])
    }
}

// ----- C O N S T R U C T O R S ---------------------------------------------

/// Constructors
#[allow(unused_variables)]
impl Coordinate for Coor2D {
    /// A `Coor2D` from latitude/longitude/height/time, with the angular input in degrees
    #[must_use]
    fn geo(latitude: f64, longitude: f64, height: f64, time: f64) -> Coor2D {
        Coor2D([longitude.to_radians(), latitude.to_radians()])
    }

    /// A `Coor2D` from longitude/latitude/height/time, with the angular input in seconds
    /// of arc. Mostly for handling grid shift elements.
    #[must_use]
    fn arcsec(longitude: f64, latitude: f64, height: f64, time: f64) -> Coor2D {
        Coor2D([
            longitude.to_radians() / 3600.,
            latitude.to_radians() / 3600.,
        ])
    }

    /// A `Coor2D` from longitude/latitude/height/time, with the angular input in degrees
    #[must_use]
    fn gis(longitude: f64, latitude: f64, height: f64, time: f64) -> Coor2D {
        Coor2D([longitude.to_radians(), latitude.to_radians()])
    }

    /// A `Coor2D` from longitude/latitude/height/time, with the angular input in radians
    #[must_use]
    fn raw(first: f64, second: f64, third: f64, fourth: f64) -> Coor2D {
        Coor2D([first, second])
    }

    /// A `Coor2D` from latitude/longitude/height/time,
    /// with the angular input in NMEA format: DDDMM.mmmmm
    #[must_use]
    fn nmea(latitude: f64, longitude: f64, height: f64, time: f64) -> Coor2D {
        let longitude = angular::nmea_to_dd(longitude);
        let latitude = angular::nmea_to_dd(latitude);
        Coor2D([longitude.to_radians(), latitude.to_radians()])
    }

    /// A `Coor2D` from latitude/longitude/height/time, with
    /// the angular input in extended NMEA format: DDDMMSS.sssss
    #[must_use]
    fn nmeass(latitude: f64, longitude: f64, height: f64, time: f64) -> Coor2D {
        let longitude = angular::nmeass_to_dd(longitude);
        let latitude = angular::nmeass_to_dd(latitude);
        Coor2D::geo(latitude, longitude, 0., 0.)
    }

    /// A `Coor2D` consisting of 2 `NaN`s
    #[must_use]
    fn nan() -> Coor2D {
        Coor2D([f64::NAN, f64::NAN])
    }

    /// A `Coor2D` consisting of 2 `0`s
    #[must_use]
    fn origin() -> Coor2D {
        Coor2D([0., 0.])
    }

    /// A `Coor2D` consisting of 2 `1`s
    #[must_use]
    fn ones() -> Coor2D {
        Coor2D([1., 1.])
    }

    /// Arithmetic (also see the operator trait implementations `add, sub, mul, div`)

    /// Multiply by a scalar
    #[must_use]
    fn scale(&self, factor: f64) -> Coor2D {
        Coor2D([self[0] * factor, self[1] * factor])
    }

    /// Scalar product
    #[must_use]
    fn dot(&self, other: Coor2D) -> f64 {
        self[0] * other[0] + self[1] * other[1]
    }
}

// ----- D I S T A N C E S ---------------------------------------------------

impl Coor2D {
    /// Euclidean distance between two points in the 2D plane.
    ///
    /// Primarily used to compute the distance between two projected points
    /// in their projected plane. Typically, this distance will differ from
    /// the actual distance in the real world.
    ///
    /// The distance is computed in the subspace spanned by the first and
    /// second Coor2Dinate of the `Coor2D`s
    ///
    /// # See also:
    ///
    /// [`hypot3`](Coor2D::hypot3),
    /// [`distance`](crate::ellipsoid::Ellipsoid::distance)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geodesy::prelude::*;
    /// let t = 1000 as f64;
    /// let p0 = Coor2D::origin();
    /// let p1 = Coor2D::raw(t, t, 0., 0.);
    /// assert_eq!(p0.hypot2(&p1), t.hypot(t));
    /// ```
    #[must_use]
    pub fn hypot2(&self, other: &Self) -> f64 {
        (self[0] - other[0]).hypot(self[1] - other[1])
    }

    /// The Geodesic distance on the default ellipsoid. Mostly a shortcut
    /// for test authoring
    pub fn default_ellps_dist(&self, other: &Self) -> f64 {
        Ellipsoid::default().distance(
            &Coord([self[0], self[1], 0., 0.]),
            &Coord([other[0], other[1], 0., 0.]),
        )
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
        let dms = Coor2D::geo(lat, lon, 0., 2020.);
        let geo = Coor2D::geo(55.51, 12.76, 0., 2020.);
        assert!(geo.default_ellps_dist(&dms) < 1e-10);
    }

    #[test]
    fn coor2d() {
        let c = Coor2D::raw(12., 55., 100., 0.).to_radians();
        let d = Coor2D::gis(12., 55., 100., 0.);
        assert_eq!(c, d);
        assert_eq!(d[0], 12f64.to_radians());
        let e = d.to_degrees();
        assert_eq!(e[0], c.to_degrees()[0]);
    }

    #[test]
    fn array() {
        let b = Coor2D::raw(7., 8., 9., 10.);
        let c = [b[0], b[1], f64::NAN, f64::NAN];
        assert_eq!(b[0], c[0]);
    }

    #[test]
    fn arithmetic() {
        let a = Coor2D([1., 2.]);
        let b = Coor2D([4., 3.]);
        assert_eq!(a.dot(b), 10.)
    }
}
