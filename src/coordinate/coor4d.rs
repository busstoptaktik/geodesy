use super::*;
use crate::math::angular;
use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};

/// Generic 4D coordinate tuple, with no fixed interpretation of the elements
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Coor4D(pub [f64; 4]);

// ----- O P E R A T O R   T R A I T S -------------------------------------------------

impl Index<usize> for Coor4D {
    type Output = f64;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for Coor4D {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

impl Add for Coor4D {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Coor4D([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }
}

impl Add<&Coor4D> for Coor4D {
    type Output = Self;
    fn add(self, other: &Self) -> Self {
        Coor4D([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }
}

impl Sub for Coor4D {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Coor4D([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            self.0[3] - other.0[3],
        ])
    }
}

impl Mul for Coor4D {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Coor4D([
            self.0[0] * other.0[0],
            self.0[1] * other.0[1],
            self.0[2] * other.0[2],
            self.0[3] * other.0[3],
        ])
    }
}

impl Div for Coor4D {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Coor4D([
            self.0[0] / other.0[0],
            self.0[1] / other.0[1],
            self.0[2] / other.0[2],
            self.0[3] / other.0[3],
        ])
    }
}

// ----- A N G U L A R   U N I T S -------------------------------------------

impl AngularUnits for Coor4D {
    /// Transform the first two elements of a `Coor4D` from degrees to radians
    #[must_use]
    fn to_radians(self) -> Self {
        Coor4D::raw(self[0].to_radians(), self[1].to_radians(), self[2], self[3])
    }

    /// Transform the first two elements of a `Coor4D` from radians to degrees
    #[must_use]
    fn to_degrees(self) -> Self {
        Coor4D::raw(self[0].to_degrees(), self[1].to_degrees(), self[2], self[3])
    }

    /// Transform the first two elements of a `Coor4D` from radians to seconds
    /// of arc.
    #[must_use]
    fn to_arcsec(self) -> Self {
        Coor4D::raw(
            self[0].to_degrees() * 3600.,
            self[1].to_degrees() * 3600.,
            self[2],
            self[3],
        )
    }

    /// Transform the internal lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    #[must_use]
    fn to_geo(self) -> Self {
        Coor4D::raw(self[1].to_degrees(), self[0].to_degrees(), self[2], self[3])
    }
}

// ----- C O N S T R U C T O R S ---------------------------------------------

/// Constructors
impl Coor4D {
    /// A `Coor4D` from latitude/longitude/height/time, with the angular input in degrees
    #[must_use]
    pub fn geo(latitude: f64, longitude: f64, height: f64, time: f64) -> Coor4D {
        Coor4D([longitude.to_radians(), latitude.to_radians(), height, time])
    }

    /// A `Coor4D` from longitude/latitude/height/time, with the angular input in seconds
    /// of arc. Mostly for handling grid shift elements.
    #[must_use]
    pub fn arcsec(longitude: f64, latitude: f64, height: f64, time: f64) -> Coor4D {
        Coor4D([
            longitude.to_radians() / 3600.,
            latitude.to_radians() / 3600.,
            height,
            time,
        ])
    }

    /// A `Coor4D` from longitude/latitude/height/time, with the angular input in degrees
    #[must_use]
    pub fn gis(longitude: f64, latitude: f64, height: f64, time: f64) -> Coor4D {
        Coor4D([longitude.to_radians(), latitude.to_radians(), height, time])
    }

    /// A `Coor4D` from longitude/latitude/height/time, with the angular input in radians
    #[must_use]
    pub fn raw(first: f64, second: f64, third: f64, fourth: f64) -> Coor4D {
        Coor4D([first, second, third, fourth])
    }

    /// A `Coor4D` from latitude/longitude/height/time, with
    /// the angular input in the ISO-6709 DDDMM.mmmmm format
    #[must_use]
    pub fn iso_dm(latitude: f64, longitude: f64, height: f64, time: f64) -> Coor4D {
        let longitude = angular::iso_dm_to_dd(longitude);
        let latitude = angular::iso_dm_to_dd(latitude);
        Coor4D([longitude.to_radians(), latitude.to_radians(), height, time])
    }

    /// A `Coor4D` from latitude/longitude/height/time, with the
    /// angular input in the ISO-6709 DDDMMSS.sssss format
    #[must_use]
    pub fn iso_dms(latitude: f64, longitude: f64, height: f64, time: f64) -> Coor4D {
        let longitude = angular::iso_dms_to_dd(longitude);
        let latitude = angular::iso_dms_to_dd(latitude);
        Coor4D::geo(latitude, longitude, height, time)
    }

    /// A `Coor4D` consisting of 4 `NaN`s
    #[must_use]
    pub fn nan() -> Coor4D {
        Coor4D([f64::NAN, f64::NAN, f64::NAN, f64::NAN])
    }

    /// A `Coor4D` consisting of 4 `0`s
    #[must_use]
    pub fn origin() -> Coor4D {
        Coor4D([0., 0., 0., 0.])
    }

    /// A `Coor4D` consisting of 4 `1`s
    #[must_use]
    pub fn ones() -> Coor4D {
        Coor4D([1., 1., 1., 1.])
    }

    /// Arithmetic (also see the operator trait implementations `add, sub, mul, div`)

    /// Multiply by a scalar
    #[must_use]
    pub fn scale(&self, factor: f64) -> Coor4D {
        let mut result = Coor4D::nan();
        for i in 0..4 {
            result[i] = self[i] * factor;
        }
        result
    }

    /// Scalar product
    #[must_use]
    pub fn dot(&self, other: Coor4D) -> f64 {
        let mut result = 0_f64;
        for i in 0..4 {
            result += self[i] * other[i];
        }
        result
    }
}

// ----- D I S T A N C E S ---------------------------------------------------

impl Coor4D {
    /// Euclidean distance between two points in the 2D plane.
    ///
    /// Primarily used to compute the distance between two projected points
    /// in their projected plane. Typically, this distance will differ from
    /// the actual distance in the real world.
    ///
    /// The distance is computed in the subspace spanned by the first and
    /// second coordinate of the `Coor4D`s
    ///
    /// # See also:
    ///
    /// [`hypot3`](Coor4D::hypot3),
    /// [`distance`](crate::ellipsoid::Ellipsoid::distance)
    ///
    /// # Examples
    ///
    /// ```
    /// use geodesy::prelude::*;
    /// let t = 1000 as f64;
    /// let p0 = Coor4D::origin();
    /// let p1 = Coor4D::raw(t, t, 0., 0.);
    /// assert_eq!(p0.hypot2(&p1), t.hypot(t));
    /// ```
    #[must_use]
    pub fn hypot2(&self, other: &Self) -> f64 {
        (self[0] - other[0]).hypot(self[1] - other[1])
    }

    /// Euclidean distance between two points in the 3D space.
    ///
    /// Primarily used to compute the distance between two points in the
    /// 3D cartesian space. The typical case is GNSS-observations, in which
    /// case, the distance computed will reflect the actual distance
    /// in the real world.
    ///
    /// The distance is computed in the subspace spanned by the first,
    /// second and third coordinate of the `Coor4D`s
    ///
    /// # See also:
    ///
    /// [`hypot2`](Coor4D::hypot2),
    /// [`distance`](crate::ellipsoid::Ellipsoid::distance)
    ///
    /// # Examples
    ///
    /// ```
    /// use geodesy::prelude::*;
    /// let t = 1000 as f64;
    /// let p0 = Coor4D::origin();
    /// let p1 = Coor4D::raw(t, t, t, 0.);
    /// assert_eq!(p0.hypot3(&p1), t.hypot(t).hypot(t));
    /// ```
    #[must_use]
    pub fn hypot3(&self, other: &Self) -> f64 {
        (self[0] - other[0])
            .hypot(self[1] - other[1])
            .hypot(self[2] - other[2])
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
        let dms = Coor4D::geo(lat, lon, 0., 2020.);
        let geo = Coor4D::geo(55.51, 12.76, 0., 2020.);
        let e = Ellipsoid::default();
        assert!(e.distance(&geo, &dms) < 1e-10);
    }

    #[test]
    fn coord() {
        let c = Coor4D::raw(12., 55., 100., 0.).to_radians();
        let d = Coor4D::gis(12., 55., 100., 0.);
        assert_eq!(c, d);
        assert_eq!(d[0], 12f64.to_radians());
        let e = d.to_degrees();
        assert_eq!(e[0], c.to_degrees()[0]);
    }

    #[test]
    fn array() {
        let b = Coor4D::raw(7., 8., 9., 10.);
        let c = [b[0], b[1], b[2], b[3], f64::NAN, f64::NAN];
        assert_eq!(b[0], c[0]);
    }

    #[test]
    fn arithmetic() {
        let a = Coor4D([1., 2., 3., 4.]);
        let b = Coor4D([4., 3., 2., 1.]);
        let t = Coor4D([12., 12., 12., 12.]);

        let c = a.add(b);
        assert_eq!(c, Coor4D([5., 5., 5., 5.]));

        let d = c.scale(2.);
        assert_eq!(d, Coor4D([10., 10., 10., 10.]));

        let e = t.div(b);
        assert_eq!(e, Coor4D([3., 4., 6., 12.]));

        assert_eq!(e.mul(b), t);
        assert_eq!(a.dot(b), 20.)
    }
}
