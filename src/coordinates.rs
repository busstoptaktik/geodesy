use std::ops::{Index, IndexMut};

/// Generic 4D coordinate tuple, with no fixed interpretation of the elements
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct CoordinateTuple(pub [f64; 4]);

impl CoordinateTuple {
    /// A `CoordinateTuple` from latitude/longitude/height/time, with the angular input in degrees
    #[must_use]
    pub fn geo(latitude: f64, longitude: f64, height: f64, time: f64) -> CoordinateTuple {
        CoordinateTuple([longitude, latitude, height, time]).to_radians()
    }

    /// A `CoordinateTuple` from longitude/latitude/height/time, with the angular input in degrees
    #[must_use]
    pub fn gis(longitude: f64, latitude: f64, height: f64, time: f64) -> CoordinateTuple {
        CoordinateTuple([longitude, latitude, height, time]).to_radians()
    }

    /// A `CoordinateTuple` from longitude/latitude/height/time, with the angular input in radians
    #[must_use]
    pub fn raw(first: f64, second: f64, third: f64, fourth: f64) -> CoordinateTuple {
        CoordinateTuple([first, second, third, fourth])
    }

    /// A `CoordinateTuple` consisting of 4 `NaN`s
    #[must_use]
    pub fn nan() -> CoordinateTuple {
        CoordinateTuple([f64::NAN, f64::NAN, f64::NAN, f64::NAN])
    }

    /// A `CoordinateTuple` consisting of 4 `0`s
    #[must_use]
    pub fn origin() -> CoordinateTuple {
        CoordinateTuple([0., 0., 0., 0.])
    }

    /// A `CoordinateTuple` consisting of 4 `1`s
    #[must_use]
    pub fn ones() -> CoordinateTuple {
        CoordinateTuple([1., 1., 1., 1.])
    }

    /// Transform the first two elements of a `CoordinateTuple` from degrees to radians
    #[must_use]
    pub fn to_radians(self) -> CoordinateTuple {
        CoordinateTuple([self[0].to_radians(), self[1].to_radians(), self[2], self[3]])
    }

    /// Transform the first two elements of a `CoordinateTuple` from radians to degrees
    #[must_use]
    pub fn to_degrees(self) -> CoordinateTuple {
        CoordinateTuple([self[0].to_degrees(), self[1].to_degrees(), self[2], self[3]])
    }

    /// Transform the internal lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    #[must_use]
    pub fn to_geo(self) -> CoordinateTuple {
        CoordinateTuple([self[1].to_degrees(), self[0].to_degrees(), self[2], self[3]])
    }

    /// For an entire data set: Transform the internal lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    pub fn geo_all(operands: &mut [CoordinateTuple]) {
        for coord in operands {
            *coord = coord.to_geo();
        }
    }

    /// For an entire data set: Transform the first two elements of a `CoordinateTuple` from radians to degrees
    pub fn degrees_all(operands: &mut [CoordinateTuple]) {
        for coord in operands {
            *coord = coord.to_degrees();
        }
    }

    /// For an entire data set: Transform the first two elements of a `CoordinateTuple` from degrees to radians
    pub fn radians_all(operands: &mut [CoordinateTuple]) {
        for coord in operands {
            *coord = coord.to_radians();
        }
    }

    /// First coordinate of the `CoordinateTuple`
    #[must_use]
    pub fn first(&self) -> f64 {
        self[0]
    }

    /// Second coordinate of the `CoordinateTuple`
    #[must_use]
    pub fn second(&self) -> f64 {
        self[1]
    }

    /// Third coordinate of the `CoordinateTuple`
    #[must_use]
    pub fn third(&self) -> f64 {
        self[2]
    }

    /// Fourth coordinate of the `CoordinateTuple`
    #[must_use]
    pub fn fourth(&self) -> f64 {
        self[3]
    }

    /// Euclidean distance between two points in the 2D plane.
    ///
    /// Primarily used to compute the distance between two projected points
    /// in their projected plane. Typically, this distance will differ from
    /// the actual distance in the real world.
    ///
    /// The distance is computed in the subspace spanned by the first and
    /// second coordinate of the `CoordinateTuple`s
    ///
    /// # See also:
    ///
    /// [`hypot3`](CoordinateTuple::hypot3),
    /// [`distance`](crate::ellipsoids::Ellipsoid::distance)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geodesy::CoordinateTuple as Coord;
    /// let t = 1000 as f64;
    /// let p0 = Coord::origin();
    /// let p1 = Coord::raw(t, t, 0., 0.);
    /// assert_eq!(p0.hypot2(&p1), t.hypot(t));
    /// ```
    #[must_use]
    pub fn hypot2(&self, other: &CoordinateTuple) -> f64 {
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
    /// second and third coordinate of the `CoordinateTuple`s
    ///
    /// # See also:
    ///
    /// [`hypot2`](CoordinateTuple::hypot2),
    /// [`distance`](crate::ellipsoids::Ellipsoid::distance)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geodesy::CoordinateTuple as Coord;
    /// let t = 1000 as f64;
    /// let p0 = Coord::origin();
    /// let p1 = Coord::raw(t, t, t, 0.);
    /// assert_eq!(p0.hypot3(&p1), t.hypot(t).hypot(t));
    /// ```
    #[must_use]
    pub fn hypot3(&self, other: &CoordinateTuple) -> f64 {
        (self[0] - other[0])
            .hypot(self[1] - other[1])
            .hypot(self[2] - other[2])
    }
}

impl Index<usize> for CoordinateTuple {
    type Output = f64;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for CoordinateTuple {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

/*
#[derive(Clone, Copy, Debug)]
pub struct CoordType {}

#[derive(Clone, Copy, Debug, Default)]
pub struct DMS {
    pub s: f32,
    pub d: i16,
    pub m: i8,
}

#[allow(dead_code)]
impl DMS {
    #[must_use]
    pub fn new(d: i16, m: i8, s: f32) -> DMS {
        DMS { s, d, m }
    }
    #[must_use]
    pub fn to_degrees(self) -> f64 {
        (f64::from(self.s) / 60. + f64::from(self.m)) / 60. + f64::from(self.d)
    }
    #[must_use]
    pub fn to_radians(self) -> f64 {
        self.to_degrees().to_radians()
    }
}

#[allow(dead_code)]
enum CoordinateKind {
    Linear,
    Angular,
    Parametric,
    Pass,
}

#[allow(dead_code)]
enum Coordinate {
    Northish {
        from: usize,
        to: usize,
        scale: f64,
        offset: f64,
        nan: f64,
        kind: CoordinateKind,
    },
    Eastish {},
    Upish {},
    Timeish {},
    Pass {},

    // An `enum` may either be `unit-like`,
    PageLoad,
    PageUnload,
    // like tuple structs,
    KeyPress(char),
    Paste(String),
    // or c-like structures.
    Click {
        x: i64,
        y: i64,
    },
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    /*
    #[test]
    fn dms() {
        let dms = DMS::new(60, 24, 36.);
        assert_eq!(dms.d, 60);
        assert_eq!(dms.m, 24);
        assert_eq!(dms.s, 36.);
        let d = dms.to_degrees();
        assert_eq!(d, 60.41);
    }
    */

    #[test]
    fn coordinatetuple() {
        let c = CoordinateTuple::raw(12., 55., 100., 0.).to_radians();
        let d = CoordinateTuple::gis(12., 55., 100., 0.);
        assert_eq!(c, d);
        assert_eq!(d[0], 12f64.to_radians());
        let e = d.to_degrees();
        assert_eq!(e[0], c.to_degrees()[0]);
    }

    #[test]
    fn array() {
        let b = CoordinateTuple::raw(7., 8., 9., 10.);
        let c = [b[0], b[1], b[2], b[3], f64::NAN, f64::NAN];
        assert_eq!(b[0], c[0]);
    }
}
