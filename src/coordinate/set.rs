use super::*;

impl<T> CoordinateAngularUnitConversions for &mut T
where
    T: CoordinateSet,
{
    /// Transform the first two elements of a `Coord` from degrees to radians
    fn to_radians(self) -> Self {
        for i in 0..self.len() {
            self.set(i, &self.get(i).to_radians());
        }
        self
    }

    /// Transform the first two elements of a `Coord` from radians to degrees
    #[must_use]
    fn to_degrees(self) -> Self {
        for i in 0..self.len() {
            self.set(i, &self.get(i).to_degrees());
        }
        self
    }

    /// Transform the first two elements of a `Coord` from radians to seconds
    /// of arc.
    #[must_use]
    fn to_arcsec(self) -> Self {
        for i in 0..self.len() {
            self.set(i, &self.get(i).to_arcsec());
        }
        self
    }
    /// Transform the internal lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    fn to_geo(self) -> Self {
        for i in 0..self.len() {
            self.set(i, &self.get(i).to_geo());
        }
        self
    }
}

impl<const N: usize> CoordinateSet for [Coord; N] {
    fn len(&self) -> usize {
        N
    }

    fn get(&self, index: usize) -> Coord {
        self[index]
    }

    fn set(&mut self, index: usize, value: &Coord) {
        self[index] = *value;
    }
}

impl CoordinateSet for Vec<Coord> {
    fn len(&self) -> usize {
        self.len()
    }

    fn get(&self, index: usize) -> Coord {
        self[index]
    }

    fn set(&mut self, index: usize, value: &Coord) {
        self[index] = *value;
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
