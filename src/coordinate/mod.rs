use crate::prelude::*;
pub mod coor2d;
pub mod coor32;
pub mod coor3d;
pub mod coor4d;
pub mod set;

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
        res.update(&[x.to_degrees(), y.to_degrees()]);
        res
    }

    /// Convert the first two elements of `self` from radians to degrees
    fn to_arcsec(&self) -> Self {
        let (x, y) = self.xy();
        let mut res = *self;
        res.update(&[x.to_degrees() * 3600., y.to_degrees() * 3600.]);
        res
    }

    /// Convert the first two elements of `self` from degrees to radians
    fn to_radians(&self) -> Self {
        let (x, y) = self.xy();
        let mut res = *self;
        res.update(&[x.to_radians(), y.to_radians()]);
        res
    }

    /// Convert-and-swap the first two elements of `self` from radians to degrees
    fn to_geo(&self) -> Self {
        let (x, y) = self.xy();
        let mut res = *self;
        res.update(&[y.to_degrees(), x.to_degrees()]);
        res
    }
}

/// For Rust Geodesy, the ISO-19111 concept of `DirectPosition` is represented
/// as a `geodesy::Coo4D`.
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

/// CoordinateSet is the fundamental coordinate access interface in ISO-19111.
/// Strictly speaking, it is not a set, but (in abstract terms) rather an
/// indexed list, or (in more concrete terms): An array.
///
/// Here it is implemented simply as an accessor trait, that allows us to
/// access any user provided data model by iterating over its elements,
/// represented as a `Coor4D`
pub trait CoordinateSet: CoordinateMetadata {
    /// Number of coordinate tuples in the set
    fn len(&self) -> usize;

    /// Native dimension of the underlying coordinates (they will always be returned as converted to [`Coor4D`](super::Coor4D))
    fn dim(&self) -> usize;

    /// Access the `index`th coordinate tuple
    fn get_coord(&self, index: usize) -> Coor4D;

    /// Overwrite the `index`th coordinate tuple
    fn set_coord(&mut self, index: usize, value: &Coor4D);

    /// Companion to `len()`
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Replace the two first elements of the `index`th `CoordinateTuple`
    /// with `x` and `y`.
    /// Consider providing a type specific version, when implementing
    /// the CoordinateSet trait for a concrete data type: The default
    ///  version is straightforward, but not very efficient.
    fn set_xy(&mut self, index: usize, x: f64, y: f64) {
        let mut coord = self.get_coord(index);
        coord.set_nth_unchecked(0, x);
        coord.set_nth_unchecked(1, y);
        self.set_coord(index, &coord);
    }

    /// Access the two first elements of the `index`th `CoordinateTuple`.
    /// Consider providing a type specific version, when implementing
    /// the CoordinateSet trait for a concrete data type: The default
    /// version is straightforward, but not very efficient.
    fn xy(&self, index: usize) -> (f64, f64) {
        self.get_coord(index).xy()
    }

    /// Set all coordinate tuples in the set to NaN
    fn stomp(&mut self) {
        let nanny = Coor4D::nan();
        for i in 0..self.len() {
            self.set_coord(i, &nanny);
        }
    }
}

/// CoordinateTuple is the ISO-19111 atomic spatial/spatiotemporal
/// referencing element. So loosely speaking, a CoordinateSet is a
///  collection of CoordinateTuples.
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
#[rustfmt::skip]
pub trait CoordinateTuple {
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
        if n < self.dim() { self.nth_unchecked(n) } else {f64::NAN}
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
        if self.dim() > 1 { self.nth_unchecked(1) } else {f64::NAN}
    }

    /// Pragmatically named accessor for the third element of the CoordinateTuple.
    fn z(&self) -> f64 {
        if self.dim() > 2 { self.nth_unchecked(2) } else {f64::NAN}
    }

    /// Pragmatically named accessor for the fourth element of the CoordinateTuple.
    fn t(&self) -> f64 {
        if self.dim() > 3 { self.nth_unchecked(3) } else {f64::NAN}
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
        (self.x().to_degrees(), self.y().to_degrees(), self.z(), self.t())
    }

    /// A tuple containing the first two components of the CoordinateTuple,
    /// converted from radians to seconds-of-arc
    fn xy_to_arcsec(&self) -> (f64, f64) {
        (self.x().to_degrees()*3600., self.y().to_degrees()*3600.)
    }

    /// A tuple containing the first three components of the CoordinateTuple,
    /// with the first two converted to seconds-of-arc
    fn xyz_to_arcsec(&self) -> (f64, f64, f64) {
        (self.x().to_degrees()*3600., self.y().to_degrees()*3600., self.z())
    }

    /// A tuple containing the first four components of the CoordinateTuple,
    /// with the first two converted to seconds-of-arc
    fn xyzt_to_arcsec(&self) -> (f64, f64, f64, f64) {
        (self.x().to_degrees()*3600., self.y().to_degrees()*3600., self.z(), self.t())
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
        (self.x().to_radians(), self.y().to_radians(), self.z(), self.t())
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
    /// [`distance`](crate::ellipsoid::Ellipsoid::distance)
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
    where Self: Sized {
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
    /// [`distance`](crate::ellipsoid::Ellipsoid::distance)
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
    where Self: Sized {
        if self.dim() < 3 {
            return f64::NAN;
        }
        let (u, v, w) = self.xyz();
        let (x, y, z) = other.xyz();
        (u - x).hypot(v - y).hypot(w - z)
    }


    fn scale(&self, factor: f64) -> Self
    where Self: Sized+Copy {
        let mut res = *self;
        for i in 0..self.dim() {
            res.set_nth(i, self.nth(i) * factor);
        }
        res
    }

    fn dot(&self, other: Self) -> f64
    where Self: Sized {
        let mut res = 0.;
        for i in 0..self.dim() {
            res += self.nth(i) * other.nth(i);
        }
        res
    }

}

// CoordinateTuple implementations for the Geodesy data types,
// Coor2D, Coor32, Coor3D, Coor4D

impl CoordinateTuple for Coor2D {
    fn dim(&self) -> usize {
        2
    }

    fn nth_unchecked(&self, n: usize) -> f64 {
        self.0[n]
    }

    fn set_nth_unchecked(&mut self, n: usize, value: f64) {
        self.0[n] = value;
    }
}

impl CoordinateTuple for Coor3D {
    fn dim(&self) -> usize {
        3
    }

    fn nth_unchecked(&self, n: usize) -> f64 {
        self.0[n]
    }

    fn set_nth_unchecked(&mut self, n: usize, value: f64) {
        self.0[n] = value;
    }
}

impl CoordinateTuple for Coor4D {
    fn dim(&self) -> usize {
        4
    }

    fn nth_unchecked(&self, n: usize) -> f64 {
        self.0[n]
    }

    fn set_nth_unchecked(&mut self, n: usize, value: f64) {
        self.0[n] = value;
    }
}

impl CoordinateTuple for Coor32 {
    fn dim(&self) -> usize {
        2
    }

    fn nth_unchecked(&self, n: usize) -> f64 {
        self.0[n] as f64
    }

    fn set_nth_unchecked(&mut self, n: usize, value: f64) {
        self.0[n] = value as f32;
    }
}

// And let's also implement it for a plain 2D f64 tuple

#[rustfmt::skip]
impl CoordinateTuple for (f64, f64) {
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
