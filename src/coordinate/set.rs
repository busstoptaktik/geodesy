use super::*;

impl<T> AngularUnits for &mut T
where
    T: CoordinateSet,
{
    /// Transform the first two elements of a all `Coord`s in a
    /// coordinate set from degrees to radians
    fn to_radians(self) -> Self {
        for i in 0..self.len() {
            self.set_coord(i, &self.get_coord(i).to_radians());
        }
        self
    }

    /// Transform the first two elements of a all `Coord`s in a
    /// coordinate set from radians to degrees
    #[must_use]
    fn to_degrees(self) -> Self {
        for i in 0..self.len() {
            self.set_coord(i, &self.get_coord(i).to_degrees());
        }
        self
    }

    /// Transform the first two elements of a all `Coord`s in a
    /// coordinate set from radians to seconds of arc.
    #[must_use]
    fn to_arcsec(self) -> Self {
        for i in 0..self.len() {
            self.set_coord(i, &self.get_coord(i).to_arcsec());
        }
        self
    }

    /// Transform all `Coord`s in a coordinate set from the internal
    /// lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    fn to_geo(self) -> Self {
        for i in 0..self.len() {
            self.set_coord(i, &self.get_coord(i).to_geo());
        }
        self
    }
}

// ----- CoordinateSet implementations for plain Coord containers ------------
impl<const N: usize> CoordinateSet for [Coord; N] {
    fn len(&self) -> usize {
        N
    }
    fn get_coord(&self, index: usize) -> Coord {
        self[index]
    }
    fn set_coord(&mut self, index: usize, value: &Coord) {
        self[index] = *value;
    }
}

impl CoordinateSet for &mut [Coord] {
    fn len(&self) -> usize {
        (**self).len()
    }
    fn get_coord(&self, index: usize) -> Coord {
        self[index]
    }
    fn set_coord(&mut self, index: usize, value: &Coord) {
        self[index] = *value;
    }
}

impl CoordinateSet for Vec<Coord> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get_coord(&self, index: usize) -> Coord {
        self[index]
    }
    fn set_coord(&mut self, index: usize, value: &Coord) {
        self[index] = *value;
    }
}

// ----- CoordinateSet implementations for plain Coor2D containers ------------

impl<const N: usize> CoordinateSet for [Coor2D; N] {
    fn len(&self) -> usize {
        N
    }
    fn get_coord(&self, index: usize) -> Coord {
        Coord([self[index][0], self[index][1], 0., f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coord) {
        self[index] = Coor2D([value[0], value[1]]);
    }
}

impl CoordinateSet for Vec<Coor2D> {
    fn len(&self) -> usize {
        self.len()
    }
    fn get_coord(&self, index: usize) -> Coord {
        Coord([self[index][0], self[index][1], 0., f64::NAN])
    }
    fn set_coord(&mut self, index: usize, value: &Coord) {
        self[index] = Coor2D([value[0], value[1]]);
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
    // Test the "impl<const N: usize> CoordinateSet for [Coord; N]"
    #[test]
    fn array() {
        let mut operands = some_basic_coordinates();
        assert_eq!(operands.len(), 2);
        assert_eq!(operands.is_empty(), false);

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

    // Test the "impl CoordinateSet for Vec<Coord>"
    #[test]
    fn vector() {
        let mut operands = Vec::from(some_basic_coordinates());
        assert_eq!(operands.len(), 2);
        assert_eq!(operands.is_empty(), false);

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
        let mut operands = some_basic_coordinates();
        let cph = operands.get_coord(0);

        // Note the different usage patterns when using the AngularUnits trait with
        // a Coord and a CoordinateSet: For the latter, the blanket implementation
        // is for an `&mut T where T: CoordinateSet`, and we just mutate the contents
        // in situ. For the former, we mutate and return the mutated `Coord`.
        operands.to_radians();
        let cph = cph.to_radians();
        assert_eq!(cph[0], operands.get_coord(0)[0]);
        assert_eq!(cph[1], operands.get_coord(0)[1]);

        operands.to_arcsec();
        assert_eq!(cph[0].to_degrees() * 3600., operands.get_coord(0)[0]);
        assert_eq!(cph[1].to_degrees() * 3600., operands.get_coord(0)[1]);
    }
}
