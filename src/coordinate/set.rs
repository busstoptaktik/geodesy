/// `CoordinateSet` is the fundamental coordinate access interface in ISO-19111.
/// Strictly speaking, it is not a set, but (in abstract terms) rather an
/// indexed list, or (in more concrete terms): An array.
///
/// Here it is implemented simply as an accessor trait, that allows us to
/// access any user provided data model by iterating over its elements,
/// represented as a `Coor4D`
pub trait CoordinateSet: CoordinateMetadata {
    /// Number of coordinate tuples in the set
    fn len(&self) -> usize;

    /// Native dimension of the underlying coordinates (they will always be
    /// returned by [`Self::get_coord()`] as converted to [`Coor4D`](super::Coor4D))
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
    ///  version is straightforward, but not necessarily efficient
    fn set_xy(&mut self, index: usize, x: f64, y: f64) {
        let mut coord = self.get_coord(index);
        coord[0] = x;
        coord[1] = y;
        self.set_coord(index, &coord);
    }

    /// Access the two first elements of the `index`th `CoordinateTuple`.
    /// Consider providing a type specific version, when implementing
    /// the CoordinateSet trait for a concrete data type: The default
    /// version is straightforward, but not necessarily efficient
    fn xy(&self, index: usize) -> (f64, f64) {
        self.get_coord(index).xy()
    }

    /// Replace the three first elements of the `index`th `CoordinateTuple`
    /// with `x`, `y` and `z`.
    /// Consider providing a type specific version, when implementing
    /// the CoordinateSet trait for a concrete data type: The default
    ///  version is straightforward, but not necessarily efficient
    fn set_xyz(&mut self, index: usize, x: f64, y: f64, z: f64) {
        let mut coord = self.get_coord(index);
        coord[0] = x;
        coord[1] = y;
        coord[2] = z;
        self.set_coord(index, &coord);
    }

    /// Access the three first elements of the `index`th `CoordinateTuple`.
    /// Consider providing a type specific version, when implementing
    /// the CoordinateSet trait for a concrete data type: The default
    /// version is straightforward, but not necessarily efficient
    fn xyz(&self, index: usize) -> (f64, f64, f64) {
        self.get_coord(index).xyz()
    }

    /// Replace the four elements of the `index`th `CoordinateTuple`
    /// with `x`, `y`, `z` and `t`. Syntactic sugar for [`Self::set_coord`]
    fn set_xyzt(&mut self, index: usize, x: f64, y: f64, z: f64, t: f64) {
        self.set_coord(index, &Coor4D([x, y, z, t]));
    }

    /// Access the four elements of the `index`th `CoordinateTuple`.
    /// Syntactic sugar for [`Self::get_coord`]
    fn xyzt(&self, index: usize) -> (f64, f64, f64, f64) {
        self.get_coord(index).xyzt()
    }

    /// Set all coordinate tuples in the set to NaN
    fn stomp(&mut self) {
        let nanny = Coor4D::nan();
        for i in 0..self.len() {
            self.set_coord(i, &nanny);
        }
    }
}

use super::*;

// Some helper macros, simplifying the macros for the actual data types

// Produce the correct len() method for arrays, slices, and vecs
macro_rules! length {
    (array) => {
        fn len(&self) -> usize {
            N
        }
    };

    (slice) => {
        fn len(&self) -> usize {
            (**self).len()
        }
    };

    (vec) => {
        fn len(&self) -> usize {
            self.len()
        }
    };
}

macro_rules! coordinate_set_impl_2d_subset {
    ($dim:expr, $len:ident) => {
        length!($len);

        fn dim(&self) -> usize {
            $dim
        }

        fn xy(&self, index: usize) -> (f64, f64) {
            self[index].xy()
        }

        fn set_xy(&mut self, index: usize, x: f64, y: f64) {
            self[index].set_xy(x, y);
        }
    };
}

macro_rules! coordinate_set_impl_3d_subset {
    ($dim:expr, $len:ident) => {
        coordinate_set_impl_2d_subset!($dim, $len);

        fn xyz(&self, index: usize) -> (f64, f64, f64) {
            self[index].xyz()
        }
        fn set_xyz(&mut self, index: usize, x: f64, y: f64, z: f64) {
            self[index].set_xyz(x, y, z);
        }
    };
}

// ----- CoordinateSet implementations for some Coor2D containers ------------

/// By default, the CoordinateSet implementations for Coor2D return `0` and `f64::NAN`
/// as third and fourth coordinate value in `get_coord()`. In the common use case of
/// handling 2D geographical or projected coordinates in a static reference frame,
/// this will usually be what you need:
///
/// - The `0` as the third coordinate will make transformations behave as if the points
///   are placed immediately on the reference ellipsoid, `h==0`
///
/// - The `f64::NAN` as the fourth coordinate will spill into the plane coordinate
///   values if passing these static coordinates through any dynamic transformations,
///   requiring a proper time coordinate, hence giving a very noisy debugging signal
///
/// If other fixed values for third and fourth coordinate are needed, the
/// `CoordinateSet` trait is also blanket-implemented for the tuple
/// `(T, f64, f64) where T: CoordinateSet`, so any data structure implementing the
/// `CoordinateSet` trait can be combined with two fixed values for third and fourth
/// coordinate dimension.

macro_rules! coordinate_set_impl_for_coor2d {
    ($kind:ident) => {
        coordinate_set_impl_2d_subset!(2, $kind);

        fn get_coord(&self, index: usize) -> Coor4D {
            Coor4D([self[index][0], self[index][1], 0., f64::NAN])
        }

        fn set_coord(&mut self, index: usize, value: &Coor4D) {
            self[index] = Coor2D([value[0], value[1]]);
        }
    };
}

impl<const N: usize> CoordinateSet for [Coor2D; N] {
    coordinate_set_impl_for_coor2d!(array);
}

impl CoordinateSet for &mut [Coor2D] {
    coordinate_set_impl_for_coor2d!(slice);
}

impl CoordinateSet for Vec<Coor2D> {
    coordinate_set_impl_for_coor2d!(vec);
}

// ----- CoordinateSet implementations for some Coor32 containers ------------

macro_rules! coordinate_set_impl_for_coor32 {
    ($kind:ident) => {
        coordinate_set_impl_2d_subset!(2, $kind);

        fn get_coord(&self, index: usize) -> Coor4D {
            Coor4D([self[index][0] as f64, self[index][1] as f64, 0., f64::NAN])
        }

        fn set_coord(&mut self, index: usize, value: &Coor4D) {
            self[index] = Coor32([value[0] as f32, value[1] as f32]);
        }
    };
}

impl<const N: usize> CoordinateSet for [Coor32; N] {
    coordinate_set_impl_for_coor32!(array);
}

impl CoordinateSet for &mut [Coor32] {
    coordinate_set_impl_for_coor32!(slice);
}

impl CoordinateSet for Vec<Coor32> {
    coordinate_set_impl_for_coor32!(vec);
}

// ----- CoordinateSet implementations for some Coor3D containers ------------

macro_rules! coordinate_set_impl_for_coor3d {
    ($kind:ident) => {
        coordinate_set_impl_3d_subset!(3, $kind);

        fn get_coord(&self, index: usize) -> Coor4D {
            Coor4D([self[index][0], self[index][1], self[index][2], f64::NAN])
        }

        fn set_coord(&mut self, index: usize, value: &Coor4D) {
            self[index] = Coor3D([value[0], value[1], value[2]]);
        }
    };
}

impl<const N: usize> CoordinateSet for [Coor3D; N] {
    coordinate_set_impl_for_coor3d!(array);
}

impl CoordinateSet for &mut [Coor3D] {
    coordinate_set_impl_for_coor3d!(slice);
}

impl CoordinateSet for Vec<Coor3D> {
    coordinate_set_impl_for_coor3d!(vec);
}

// ----- CoordinateSet implementations for some Coor4D containers ------------

macro_rules! coordinate_set_impl_for_coor4d {
    ($kind:ident) => {
        coordinate_set_impl_3d_subset!(4, $kind);

        fn get_coord(&self, index: usize) -> Coor4D {
            self[index]
        }

        fn set_coord(&mut self, index: usize, value: &Coor4D) {
            self[index] = *value;
        }

        fn xyzt(&self, index: usize) -> (f64, f64, f64, f64) {
            self[index].xyzt()
        }

        fn set_xyzt(&mut self, index: usize, x: f64, y: f64, z: f64, t: f64) {
            self[index].set_xyzt(x, y, z, t);
        }
    };
}

impl<const N: usize> CoordinateSet for [Coor4D; N] {
    coordinate_set_impl_for_coor4d!(array);
}

impl CoordinateSet for &mut [Coor4D] {
    coordinate_set_impl_for_coor4d!(slice);
}

impl CoordinateSet for Vec<Coor4D> {
    coordinate_set_impl_for_coor4d!(vec);
}

/// User defined values for third and fourth coordinate dimension.
/// Intended as a way to supply a fixed height and epoch to a set
/// of 2D coordinates
impl<T> CoordinateSet for (T, f64, f64)
where
    T: CoordinateSet,
{
    fn len(&self) -> usize {
        self.0.len()
    }
    fn dim(&self) -> usize {
        4
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        let c = self.0.get_coord(index);
        Coor4D([c[0], c[1], self.1, self.2])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self.0.set_coord(index, value);
    }
}

/// User defined values for fourth coordinate dimension.
/// Intended as a way to supply a fixed epoch to a set
/// of 3D coordinates
impl<T> CoordinateSet for (T, f64)
where
    T: CoordinateSet,
{
    fn len(&self) -> usize {
        self.0.len()
    }
    fn dim(&self) -> usize {
        4
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        let c = self.0.get_coord(index);
        Coor4D([c[0], c[1], c[2], self.1])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self.0.set_coord(index, value);
    }
}

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

// ----- T E S T S ---------------------------------------------------

#[cfg(test)]
mod tests {
    // Also see the "coor2d" test in tmerc, where 2D and 4D operands are
    // used in otherwise identical setups.
    use super::*;
    // Test the "impl<const N: usize> CoordinateSet for [Coor4D; N]"
    #[test]
    fn array() {
        let mut operands = crate::test_data::coor4d();
        assert_eq!(operands.len(), 2);
        assert!(!operands.is_empty());

        let cph = operands.get_coord(0);
        assert_eq!(cph[0], 55.);
        assert_eq!(cph[1], 12.);

        let sth = operands.get_coord(1);
        assert_eq!(sth[0], 59.);
        assert_eq!(sth[1], 18.);

        // Turn Copenhagen into Stockholm
        operands.set_coord(0, &sth);
        let cph = operands.get_coord(0);
        assert_eq!(cph[0], 59.);
        assert_eq!(cph[1], 18.);
    }

    // Test the "impl CoordinateSet for Vec<Coor4D>"
    #[test]
    fn vector() {
        let mut operands = Vec::from(crate::test_data::coor2d());
        assert_eq!(operands.len(), 2);
        assert!(!operands.is_empty());

        let cph = operands.get_coord(0);
        assert_eq!(cph[0], 55.);
        assert_eq!(cph[1], 12.);

        let sth = operands.get_coord(1);
        assert_eq!(sth[0], 59.);
        assert_eq!(sth[1], 18.);

        // Turn Copenhagen into Stockholm
        operands.set_coord(0, &sth);
        let cph = operands.get_coord(0);
        assert_eq!(cph[0], 59.);
        assert_eq!(cph[1], 18.);
    }

    // Test the "AngularUnits" conversion trait
    #[test]
    fn angular() {
        let operands = crate::test_data::coor2d();
        let cph = operands.get_coord(0);

        // Note the different usage patterns when using the AngularUnits trait with
        // a Coor4D and a CoordinateSet: For the latter, the blanket implementation
        // is for an `&mut T where T: CoordinateSet`, and we just mutate the contents
        // in situ. For the former, we return a newly computed `Coor4D`.
        let cph = cph.to_radians();
        assert_eq!(cph[0], operands.get_coord(0).to_radians()[0]);
        assert_eq!(cph[1], operands.get_coord(0).to_radians()[1]);

        assert_eq!(
            cph[0].to_degrees() * 3600.,
            operands.get_coord(0).to_radians().to_arcsec()[0]
        );
        assert_eq!(
            cph[1].to_degrees() * 3600.,
            operands.get_coord(0).to_radians().to_arcsec()[1]
        );
    }

    #[test]
    fn setting_and_getting_as_f64() {
        let first = Coor4D([11., 12., 13., 14.]);
        let second = Coor4D([21., 22., 23., 24.]);
        let mut operands = Vec::from([first, second]);
        let (x, y) = operands.xy(0);
        assert_eq!((x, y), (11., 12.));
        let (x, y) = operands.xy(1);
        assert_eq!((x, y), (21., 22.));
        operands.set_xy(0, x, y);
        let (x, y) = operands.xy(0);
        assert_eq!((x, y), (21., 22.));

        let mut operands = Vec::from([first, second]);
        let (x, y, z) = operands.xyz(0);
        assert_eq!((x, y, z), (11., 12., 13.));
        let (x, y, z) = operands.xyz(1);
        assert_eq!((x, y, z), (21., 22., 23.));
        operands.set_xyz(0, x, y, z);
        let (x, y, z) = operands.xyz(0);
        assert_eq!((x, y, z), (21., 22., 23.));

        let mut operands = Vec::from([first, second]);
        let (x, y, z, t) = operands.xyzt(0);
        assert_eq!((x, y, z, t), (11., 12., 13., 14.));
        let (x, y, z, t) = operands.xyzt(1);
        assert_eq!((x, y, z, t), (21., 22., 23., 24.));
        operands.set_xyzt(0, x, y, z, t);
        let (x, y, z, t) = operands.xyzt(0);
        assert_eq!((x, y, z, t), (21., 22., 23., 24.));
    }
}
