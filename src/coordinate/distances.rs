use super::*;

impl CoordinateTuple {
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
    /// [`distance`](crate::ellipsoid::Ellipsoid::distance)
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
    /// second and third coordinate of the `CoordinateTuple`s
    ///
    /// # See also:
    ///
    /// [`hypot2`](CoordinateTuple::hypot2),
    /// [`distance`](crate::ellipsoid::Ellipsoid::distance)
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
    pub fn hypot3(&self, other: &Self) -> f64 {
        (self[0] - other[0])
            .hypot(self[1] - other[1])
            .hypot(self[2] - other[2])
    }

    /// The 3D distance between two points given as internal angular
    /// coordinates. Mostly a shortcut for test authoring
    pub fn default_ellps_3d_dist(&self, other: &Self) -> f64 {
        let e = Ellipsoid::default();
        e.cartesian(self).hypot3(&e.cartesian(other))
    }

    /// The Geodesic distance on the default ellipsoid. Mostly a shortcut
    /// for test authoring
    pub fn default_ellps_dist(&self, other: &Self) -> f64 {
        Ellipsoid::default().distance(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distances() {
        let lat = CoordinateTuple::dms_to_dd(55, 30, 36.);
        let lon = CoordinateTuple::dms_to_dd(12, 45, 36.);
        let dms = CoordinateTuple::geo(lat, lon, 0., 2020.);
        let geo = CoordinateTuple::geo(55.51, 12.76, 0., 2020.);
        assert!(geo.default_ellps_dist(&dms) < 1e-10);
    }
}
