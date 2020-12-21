pub mod foundations;
pub mod operators;

// ----------------- TYPES -------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct CoordinateTuple(f64, f64, f64, f64);
impl CoordinateTuple {
    pub fn new(x: f64, y: f64, z: f64, t: f64) -> CoordinateTuple {
        CoordinateTuple{0: x, 1: y, 2: z, 3: t}
    }

    pub fn first(&self) -> f64 {
        self.0
    }
    pub fn second(&self) -> f64 {
        self.1
    }
    pub fn third(&self) -> f64 {
        self.2
    }
    pub fn fourth(&self) -> f64 {
        self.3
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CoordType {}

