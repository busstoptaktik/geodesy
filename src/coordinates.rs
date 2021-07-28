use crate::operand::*;

impl CoordinatePrimitives for CoordinateTuple {
    fn new(x: f64, y: f64, z: f64, t: f64) -> CoordinateTuple {
        [x, y, z, t]
    }

    fn nan() -> CoordinateTuple {
        [f64::NAN, f64::NAN, f64::NAN, f64::NAN]
    }

    #[must_use]
    fn deg(x: f64, y: f64, z: f64, t: f64) -> CoordinateTuple {
        CoordinateTuple::new(x.to_radians(), y.to_radians(), z, t)
    }

    #[must_use]
    fn to_degrees(self) -> CoordinateTuple {
        CoordinateTuple::new(self[0].to_degrees(), self[1].to_degrees(), self[2], self[3])
    }

    #[must_use]
    fn to_radians(self) -> CoordinateTuple {
        CoordinateTuple::new(self[0].to_radians(), self[1].to_radians(), self[2], self[3])
    }

    #[must_use]
    fn first(&self) -> f64 {
        self[0]
    }

    #[must_use]
    fn second(&self) -> f64 {
        self[1]
    }

    #[must_use]
    fn third(&self) -> f64 {
        self[2]
    }

    #[must_use]
    fn fourth(&self) -> f64 {
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
    /// [`hypot3`](crate::coordinates::CoordinateTuple::hypot3),
    /// [`distance`](crate::ellipsoids::Ellipsoid::distance)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geodesy::operand::*;
    /// let t = 1000.;
    /// let p0 = CoordinateTuple::new(0., 0., 0., 0.);
    /// let p1 = CoordinateTuple::new(t, t, 0., 0.);
    /// assert_eq!(p0.hypot2(&p1), t.hypot(t));
    /// ```
    #[must_use]
    fn hypot2(&self, other: &CoordinateTuple) -> f64 {
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
    /// [`hypot2`](crate::coordinates::CoordinateTuple::hypot2),
    /// [`distance`](crate::ellipsoids::Ellipsoid::distance)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geodesy::operand::*;
    /// let t = 1000.;
    /// let p0 = CoordinateTuple::new(0., 0., 0., 0.);
    /// let p1 = CoordinateTuple::new(t, t, t, 0.);
    /// assert_eq!(p0.hypot3(&p1), t.hypot(t).hypot(t));
    /// ```
    #[must_use]
    fn hypot3(&self, other: &CoordinateTuple) -> f64 {
        (self[0] - other[0])
            .hypot(self[1] - other[1])
            .hypot(self[2] - other[2])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CoordType {}

#[derive(Clone, Copy, Debug, Default)]
pub struct DMS {
    pub s: f32,
    pub d: i16,
    pub m: i8,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dms() {
        let dms = DMS::new(60, 24, 36.);
        assert_eq!(dms.d, 60);
        assert_eq!(dms.m, 24);
        assert_eq!(dms.s, 36.);
        let d = dms.to_degrees();
        assert_eq!(d, 60.41);
    }

    #[test]
    fn coordinatetuple() {
        let c = CoordinateTuple::new(12., 55., 100., 0.).to_radians();
        let d = CoordinateTuple::deg(12., 55., 100., 0.);
        assert_eq!(c, d);
        assert_eq!(d[0], 12f64.to_radians());
        let e = d.to_degrees();
        assert_eq!(e[0], c.to_degrees()[0]);
    }

    #[test]
    fn array() {
        let b = CoordinateTuple::new(7., 8., 9., 10.);
        let c = [b[0], b[1], b[2], b[3], f64::NAN, f64::NAN];
        assert_eq!(b[0], c[0]);
    }
}
