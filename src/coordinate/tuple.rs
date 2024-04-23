use super::*;

// ---- Indexing for the primary CoorND types ----

use std::ops::{Index, IndexMut};

macro_rules! coord_indexing {
    ($type:ty, $output:ty) => {
        impl Index<usize> for $type {
            type Output = $output;
            fn index(&self, i: usize) -> &Self::Output {
                &self.0[i]
            }
        }

        impl IndexMut<usize> for $type {
            fn index_mut(&mut self, i: usize) -> &mut Self::Output {
                &mut self.0[i]
            }
        }
    };
}

coord_indexing!(Coor2D, f64);
coord_indexing!(Coor3D, f64);
coord_indexing!(Coor4D, f64);
coord_indexing!(Coor32, f32);

// ---- Vector space operators for the primary CoorND types ----

use std::ops::{Add, Div, Mul, Sub};

// Helper for the coord_operator! macro
macro_rules! coor4d {
    ($symbol:tt, $self:ident, $other:ident) => {
        Coor4D([
            $self.0[0] $symbol ($other.0[0] as f64),
            $self.0[1] $symbol ($other.0[1] as f64),
            $self.0[2] $symbol ($other.0[2] as f64),
            $self.0[3] $symbol ($other.0[3] as f64),
        ])
    }
}

// Helper for the coord_operator! macro
macro_rules! coor3d {
    ($symbol:tt, $self:ident, $other:ident) => {
        Coor3D([
            $self.0[0] $symbol ($other.0[0] as f64),
            $self.0[1] $symbol ($other.0[1] as f64),
            $self.0[2] $symbol ($other.0[2] as f64),
        ])
    }
}

// Helper for the coord_operator! macro
macro_rules! coor2d {
    ($symbol:tt, $self:ident, $other:ident) => {
        Coor2D([
            $self.0[0] $symbol ($other.0[0] as f64),
            $self.0[1] $symbol ($other.0[1] as f64),
        ])
    }
}

// Helper for the coord_operator! macro
macro_rules! coor32 {
    ($symbol:tt, $self:ident, $other:ident) => {
        Coor32([
            $self.0[0] $symbol ($other.0[0] as f32),
            $self.0[1] $symbol ($other.0[1] as f32),
        ])
    }
}

// Generate the vector space operators Add, Sub, Mul, Div for $type
macro_rules! coord_operator {
    ($type:ty, $othertype:ty, $typemacro:ident, $op:ident, $symbol:tt, $function:ident) => {
        impl $op<$othertype> for $type {
            type Output = Self;
            fn $function(self, other: $othertype) -> Self {
                $typemacro!($symbol, self, other)
            }
        }
    };
}

// Generate the vector space operators Add, Sub, Mul, Div for $type
macro_rules! all_coord_operators {
    ($type:ty, $othertype:ty, $typemacro:ident) => {
        coord_operator!($type, $othertype, $typemacro, Add, +, add);
        coord_operator!($type, $othertype, $typemacro, Sub, -, sub);
        coord_operator!($type, $othertype, $typemacro, Mul, *, mul);
        coord_operator!($type, $othertype, $typemacro, Div, /, div);
    };
}

all_coord_operators!(Coor4D, &Coor4D, coor4d);
all_coord_operators!(Coor3D, &Coor3D, coor3d);
all_coord_operators!(Coor2D, &Coor2D, coor2d);
all_coord_operators!(Coor2D, &Coor32, coor2d);
all_coord_operators!(Coor32, &Coor32, coor32);

all_coord_operators!(Coor4D, Coor4D, coor4d);
all_coord_operators!(Coor3D, Coor3D, coor3d);
all_coord_operators!(Coor2D, Coor2D, coor2d);
all_coord_operators!(Coor2D, Coor32, coor2d);
all_coord_operators!(Coor32, Coor32, coor32);

/// CoordinateTuple is the ISO-19111 atomic spatial/spatiotemporal
/// referencing element. So loosely speaking, a CoordinateSet is a
/// collection of CoordinateTuples.
///
/// Note that (despite the formal name) the underlying data structure
/// need not be a tuple: It can be any item, for which it makes sense
/// to implement the CoordinateTuple trait.
///
/// The CoordinateTuple trait provides a number of convenience accessors
/// for accessing single coordinate elements or tuples of subsets.
/// These accessors are pragmatically named (x, y, xy, etc.). While these
/// names may be geodetically naïve, they are suggestive, practical, and
/// aligns well with the internal coordinate order convention of most
/// Geodesy operators.
///
/// All accessors have default implementations, except the 3 methods
/// [`nth_unchecked()`](Self::nth_unchecked()),
/// [`set_nth_unchecked()`](Self::set_nth_unchecked) and
/// [`dim()`](Self::dim()),
/// which must be provided by the implementer.
///
/// When accessing dimensions outside of the domain of the CoordinateTuple,
/// [NaN](f64::NAN) will be returned.
pub trait CoordinateTuple {
    /// Construct a new `CoordinateTuple``, with all elements set to `fill`
    fn new(fill: f64) -> Self;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// May panic if n >= DIMENSION.
    /// See also [`nth()`](Self::nth).
    fn nth_unchecked(&self, n: usize) -> f64;

    /// Replace the n'th (0-based) element of the `CoordinateTuple` with `value`.
    /// May panic if `n >=` [`dim()`](Self::dim()).
    /// See also [`set_nth()`](Self::set_nth).
    fn set_nth_unchecked(&mut self, n: usize, value: f64);

    /// Native dimension of the coordinate tuple
    fn dim(&self) -> usize;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// Returns NaN if `n >= DIMENSION`.
    /// See also [`nth()`](Self::nth_unchecked).
    fn nth(&self, n: usize) -> f64 {
        if n < self.dim() {
            self.nth_unchecked(n)
        } else {
            f64::NAN
        }
    }

    // Note: We use nth_unchecked and explicitly check for dimension in
    // y(), z() and t(), rather than leaving the check to nth(...).
    // This is because the checks in these cases are constant expressions, and
    // hence can be eliminated by the compiler in the concrete implementations.

    /// Pragmatically named accessor for the first element of the CoordinateTuple.
    fn x(&self) -> f64 {
        self.nth_unchecked(0)
    }

    /// Pragmatically named accessor for the second element of the CoordinateTuple.
    fn y(&self) -> f64 {
        if self.dim() > 1 {
            self.nth_unchecked(1)
        } else {
            f64::NAN
        }
    }

    /// Pragmatically named accessor for the third element of the CoordinateTuple.
    fn z(&self) -> f64 {
        if self.dim() > 2 {
            self.nth_unchecked(2)
        } else {
            f64::NAN
        }
    }

    /// Pragmatically named accessor for the fourth element of the CoordinateTuple.
    fn t(&self) -> f64 {
        if self.dim() > 3 {
            self.nth_unchecked(3)
        } else {
            f64::NAN
        }
    }

    /// A tuple containing the first two components of the CoordinateTuple.
    fn xy(&self) -> (f64, f64) {
        (self.x(), self.y())
    }

    /// A tuple containing the first three components of the CoordinateTuple.
    fn xyz(&self) -> (f64, f64, f64) {
        (self.x(), self.y(), self.z())
    }

    /// A tuple containing the first four components of the CoordinateTuple.
    fn xyzt(&self) -> (f64, f64, f64, f64) {
        (self.x(), self.y(), self.z(), self.t())
    }

    /// A tuple containing the first two components of the CoordinateTuple
    /// converted from radians to degrees
    fn xy_to_degrees(&self) -> (f64, f64) {
        (self.x().to_degrees(), self.y().to_degrees())
    }

    /// A tuple containing the first three components of the CoordinateTuple,
    /// with the first two converted from radians to degrees.
    fn xyz_to_degrees(&self) -> (f64, f64, f64) {
        (self.x().to_degrees(), self.y().to_degrees(), self.z())
    }

    /// A tuple containing the first four components of the CoordinateTuple,
    /// with the first two converted from radians to degrees.
    fn xyzt_to_degrees(&self) -> (f64, f64, f64, f64) {
        (
            self.x().to_degrees(),
            self.y().to_degrees(),
            self.z(),
            self.t(),
        )
    }

    /// A tuple containing the first two components of the CoordinateTuple,
    /// converted from radians to seconds-of-arc
    fn xy_to_arcsec(&self) -> (f64, f64) {
        (self.x().to_degrees() * 3600., self.y().to_degrees() * 3600.)
    }

    /// A tuple containing the first three components of the CoordinateTuple,
    /// with the first two converted to seconds-of-arc
    fn xyz_to_arcsec(&self) -> (f64, f64, f64) {
        (
            self.x().to_degrees() * 3600.,
            self.y().to_degrees() * 3600.,
            self.z(),
        )
    }

    /// A tuple containing the first four components of the CoordinateTuple,
    /// with the first two converted to seconds-of-arc
    fn xyzt_to_arcsec(&self) -> (f64, f64, f64, f64) {
        (
            self.x().to_degrees() * 3600.,
            self.y().to_degrees() * 3600.,
            self.z(),
            self.t(),
        )
    }

    /// A tuple containing the first two components of the CoordinateTuple,
    /// converted from degrees to radians
    fn xy_to_radians(&self) -> (f64, f64) {
        (self.x().to_radians(), self.y().to_radians())
    }

    /// A tuple containing the first three components of the CoordinateTuple,
    /// with the first two converted from degrees to radians
    fn xyz_to_radians(&self) -> (f64, f64, f64) {
        (self.x().to_radians(), self.y().to_radians(), self.z())
    }

    /// A tuple containing the first four components of the CoordinateTuple,
    /// with the first two converted from degrees to radians
    fn xyzt_to_radians(&self) -> (f64, f64, f64, f64) {
        (
            self.x().to_radians(),
            self.y().to_radians(),
            self.z(),
            self.t(),
        )
    }

    /// Fill all elements of `self` with `value`
    fn fill(&mut self, value: f64) {
        for n in 0..self.dim() {
            self.set_nth_unchecked(n, value);
        }
    }

    /// Replace the n'th (0-based) element of the `CoordinateTuple` with `value`.
    /// If `n >=` [`dim()`](Self::dim()) fill the coordinate with `f64::NAN`.
    /// See also [`set_nth_unchecked()`](Self::set_nth_unchecked).
    fn set_nth(&mut self, n: usize, value: f64) {
        if n < self.dim() {
            self.set_nth_unchecked(n, value)
        } else {
            self.fill(f64::NAN);
        }
    }

    /// Replace the two first elements of the `CoordinateTuple` with `x` and `y`.
    /// If the dimension is less than 2, fill the coordinate with `f64::NAN`.
    /// See also [`set_nth_unchecked()`](Self::set_nth_unchecked).
    fn set_xy(&mut self, x: f64, y: f64) {
        if self.dim() > 1 {
            self.set_nth_unchecked(0, x);
            self.set_nth_unchecked(1, y);
        } else {
            self.fill(f64::NAN);
        }
    }

    /// Replace the three first elements of the `CoordinateTuple` with `x`, `y` and `z`.
    /// If the dimension is less than 3, fill the coordinate with `f64::NAN`.
    fn set_xyz(&mut self, x: f64, y: f64, z: f64) {
        if self.dim() > 2 {
            self.set_nth_unchecked(0, x);
            self.set_nth_unchecked(1, y);
            self.set_nth_unchecked(2, z);
        } else {
            self.fill(f64::NAN);
        }
    }

    /// Replace the four first elements of the `CoordinateTuple` with `x`, `y` `z` and `t`.
    /// If the dimension is less than 4, fill the coordinate with `f64::NAN`.
    fn set_xyzt(&mut self, x: f64, y: f64, z: f64, t: f64) {
        if self.dim() > 3 {
            self.set_nth_unchecked(0, x);
            self.set_nth_unchecked(1, y);
            self.set_nth_unchecked(2, z);
            self.set_nth_unchecked(3, t);
        } else {
            self.fill(f64::NAN);
        }
    }

    /// Replace the `N` first (up to [`dim()`](Self::dim())) elements of `self` with the
    /// elements of `value`
    #[allow(clippy::needless_range_loop)]
    fn update(&mut self, value: &[f64]) {
        let n = value.len().min(self.dim());
        for i in 0..n {
            self.set_nth_unchecked(i, value[i])
        }
    }

    /// Euclidean distance between two points in the 2D plane.
    ///
    /// Primarily used to compute the distance between two projected points
    /// in their projected plane. Typically, this distance will differ from
    /// the actual distance in the real world.
    ///
    /// # See also:
    ///
    /// [`hypot3`](Self::hypot3),
    /// [`distance`](crate::ellps::Geodesics::distance)
    ///
    /// # Examples
    ///
    /// ```
    /// use geodesy::prelude::*;
    /// let t = 1000 as f64;
    /// let p0 = Coor2D::origin();
    /// let p1 = Coor2D::raw(t, t);
    /// assert_eq!(p0.hypot2(&p1), t.hypot(t));
    /// ```
    #[must_use]
    fn hypot2(&self, other: &Self) -> f64
    where
        Self: Sized,
    {
        let (u, v) = self.xy();
        let (x, y) = other.xy();
        (u - x).hypot(v - y)
    }

    /// Euclidean distance between two points in the 3D space.
    ///
    /// Primarily used to compute the distance between two points in the
    /// 3D cartesian space. The typical case is GNSS-observations, in which
    /// case, the distance computed will reflect the actual distance
    /// in the real world.
    ///
    /// The distance is computed in the subspace spanned by the first,
    /// second and third coordinate of the `Coor3D`s
    ///
    /// # See also:
    ///
    /// [`hypot2()`](Self::hypot2),
    /// [`distance`](crate::ellps::Geodesics::distance)
    ///
    /// # Examples
    ///
    /// ```
    /// use geodesy::prelude::*;
    /// let t = 1000 as f64;
    /// let p0 = Coor3D::origin();
    /// let p1 = Coor3D::raw(t, t, t);
    /// assert_eq!(p0.hypot3(&p1), t.hypot(t).hypot(t));
    /// ```
    #[must_use]
    fn hypot3(&self, other: &Self) -> f64
    where
        Self: Sized,
    {
        if self.dim() < 3 {
            return f64::NAN;
        }
        let (u, v, w) = self.xyz();
        let (x, y, z) = other.xyz();
        (u - x).hypot(v - y).hypot(w - z)
    }

    fn scale(&self, factor: f64) -> Self
    where
        Self: Sized + Copy,
    {
        let mut res = *self;
        for i in 0..self.dim() {
            res.set_nth(i, self.nth(i) * factor);
        }
        res
    }

    fn dot(&self, other: Self) -> f64
    where
        Self: Sized,
    {
        let mut res = 0.;
        for i in 0..self.dim() {
            res += self.nth(i) * other.nth(i);
        }
        res
    }
}

// The CoordiateTuple trait is implemented for the main
// newtypes Coor2D, Coor3D, Coor4D, Coor32 in their files
// below. But for good measure, let's also implement it
// for a plain 2D f64 tuple

#[rustfmt::skip]
impl CoordinateTuple for (f64, f64) {
    fn new(fill: f64) -> Self {
        (fill, fill)
    }

    fn dim(&self) -> usize { 2 }

    fn nth_unchecked(&self, n: usize) -> f64 {
        match n {
            0 => self.0,
            1 => self.1,
            _ => panic!()
        }
    }

    fn set_nth_unchecked(&mut self, n: usize, value: f64) {
        match n {
            0 => self.0 = value,
            1 => self.1 = value,
            _ => ()
        }
    }
}
