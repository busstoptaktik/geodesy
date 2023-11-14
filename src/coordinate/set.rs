use super::*;

impl<T> AngularUnits for &mut T
where
    T: CoordinateSet,
{
    /// Transform the first two elements of all elements in a
    /// coordinate set from degrees to radians
    fn to_radians(self) -> Self {
        for i in 0..self.len() {
            self.set_coord(i, &self.get_coord(i).to_radians());
        }
        self
    }

    /// Transform the first two elements of a all elements in a
    /// coordinate set from radians to degrees
    #[must_use]
    fn to_degrees(self) -> Self {
        for i in 0..self.len() {
            self.set_coord(i, &self.get_coord(i).to_degrees());
        }
        self
    }

    /// Transform the first two elements of a all elements in a
    /// coordinate set from radians to seconds of arc.
    #[must_use]
    fn to_arcsec(self) -> Self {
        for i in 0..self.len() {
            self.set_coord(i, &self.get_coord(i).to_arcsec());
        }
        self
    }

    /// Transform all elements in a coordinate set from the internal
    /// lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    fn to_geo(self) -> Self {
        for i in 0..self.len() {
            self.set_coord(i, &self.get_coord(i).to_geo());
        }
        self
    }
}

// ----- CoordinateSet implementations for some Coor4D containers ------------
impl<const N: usize> CoordinateSet for [Coor4D; N] {
    fn len(&self) -> usize {
        N
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        self[index]
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = *value;
    }
}

impl CoordinateSet for &mut [Coor4D] {
    fn len(&self) -> usize {
        (**self).len()
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        self[index]
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = *value;
    }
}

impl CoordinateSet for Vec<Coor4D> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        self[index]
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = *value;
    }
}

// ----- CoordinateSet implementations for some Coor3D containers ------------
impl<const N: usize> CoordinateSet for [Coor3D; N] {
    fn len(&self) -> usize {
        N
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0], self[index][1], self[index][2], f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor3D([value[0], value[1], value[2]]);
    }
}

impl CoordinateSet for &mut [Coor3D] {
    fn len(&self) -> usize {
        (**self).len()
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0], self[index][1], self[index][2], f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor3D([value[0], value[1], value[2]]);
    }
}

impl CoordinateSet for Vec<Coor3D> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0], self[index][1], self[index][2], f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor3D([value[0], value[1], value[2]]);
    }
}

// ----- CoordinateSet implementations for some Coor2D containers ------------

/// By default, the CoordinateSet implementations for Coor2D return `0` and `f64::NAN`
/// as third and fourth coordinate value in `get_coord()`. In the common use case of
/// handling 2D geographical or projected coordinates in a static reference frame,
/// this will usually be what you need:
///
/// - The `0` as the third coordinate will make transformations behave as if the points
/// are placed immediately on the reference ellipsoid, `h==0`
///
/// - The `f64::NAN` as the fourth coordinate will spill into the plane coordinate
/// values if passing these static coordinates through any dynamic transformations,
/// requiring a proper time coordinate, hence giving a very noisy debugging signal
///
/// If other fixed values for third and fourth coordinate are needed, the
/// `CoordinateSet` trait is also blanket-implemented for the tuple
/// `(T, f64, f64) where T: CoordinateSet`, so any data structure implementing the
/// `CoordinateSet` trait can be combined with two fixed values for third and fourth
/// coordinate dimension.

impl<const N: usize> CoordinateSet for [Coor2D; N] {
    fn len(&self) -> usize {
        N
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0], self[index][1], 0., f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor2D([value[0], value[1]]);
    }
}

impl CoordinateSet for &mut [Coor2D] {
    fn len(&self) -> usize {
        (**self).len()
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0], self[index][1], 0.0, f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor2D([value[0], value[1]]);
    }
}

impl CoordinateSet for Vec<Coor2D> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0], self[index][1], 0., f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor2D([value[0], value[1]]);
    }
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
    fn get_coord(&self, index: usize) -> Coor4D {
        let c = self.0.get_coord(index);
        Coor4D([c[0], c[1], c[2], self.1])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self.0.set_coord(index, value);
    }
}

// ----- CoordinateSet implementations for some Coor32 containers ------------

impl<const N: usize> CoordinateSet for [Coor32; N] {
    fn len(&self) -> usize {
        N
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0] as f64, self[index][1] as f64, 0., f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor32::raw(value[0], value[1]);
    }
}

impl CoordinateSet for &mut [Coor32] {
    fn len(&self) -> usize {
        (**self).len()
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0] as f64, self[index][1] as f64, 0.0, f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor32::raw(value[0], value[1]);
    }
}

impl CoordinateSet for Vec<Coor32> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get_coord(&self, index: usize) -> Coor4D {
        Coor4D([self[index][0] as f64, self[index][1] as f64, 0., f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = Coor32::raw(value[0], value[1]);
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
        let mut operands = some_basic_coor4dinates();
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
        let mut operands = Vec::from(some_basic_coor4dinates());
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
        let mut operands = some_basic_coor2dinates();
        let cph = operands.get_coord(0);

        // Note the different usage patterns when using the AngularUnits trait with
        // a Coor4D and a CoordinateSet: For the latter, the blanket implementation
        // is for an `&mut T where T: CoordinateSet`, and we just mutate the contents
        // in situ. For the former, we return a newly computed `Coor4D`.
        operands.to_radians();
        let cph = cph.to_radians();
        assert_eq!(cph[0], operands.get_coord(0)[0]);
        assert_eq!(cph[1], operands.get_coord(0)[1]);

        operands.to_arcsec();
        assert_eq!(cph[0].to_degrees() * 3600., operands.get_coord(0)[0]);
        assert_eq!(cph[1].to_degrees() * 3600., operands.get_coord(0)[1]);
    }
}
