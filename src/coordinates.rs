#[derive(Clone, Copy, Debug)]
pub struct CoordinateTuple(pub f64, pub f64, pub f64, pub f64);
impl CoordinateTuple {
    pub fn new(x: f64, y: f64, z: f64, t: f64) -> CoordinateTuple {
        CoordinateTuple {
            0: x,
            1: y,
            2: z,
            3: t,
        }
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

#[derive(Clone, Copy, Debug)]
pub struct DMS {
    pub s: f32,
    pub d: i16,
    pub m: i8,
}

impl DMS {
    pub fn new(d: i16, m: i8, s: f32) -> DMS {
        DMS { d: d, m: m, s: s }
    }
    pub fn to_deg(&self) -> f64 {
        (self.s as f64 / 60. + self.m as f64) / 60. + self.d as f64
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dms() {
        let dms = super::DMS::new(60, 24, 36.);
        assert_eq!(dms.d, 60);
        assert_eq!(dms.m, 24);
        assert_eq!(dms.s, 36.);
        let d = dms.to_deg();
        assert_eq!(d, 60.41);
    }
}
