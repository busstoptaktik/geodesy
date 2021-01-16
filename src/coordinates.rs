#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct CoordinateTuple(pub f64, pub f64, pub f64, pub f64);
impl CoordinateTuple {
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64, t: f64) -> CoordinateTuple {
        CoordinateTuple {
            0: x,
            1: y,
            2: z,
            3: t,
        }
    }

    #[must_use]
    pub fn deg(x: f64, y: f64, z: f64, t: f64) -> CoordinateTuple {
        CoordinateTuple {
            0: x.to_radians(),
            1: y.to_radians(),
            2: z,
            3: t,
        }
    }

    #[must_use]
    pub fn to_degrees(&self) -> CoordinateTuple {
        CoordinateTuple::new(self.0.to_degrees(), self.1.to_degrees(), self.2, self.3)
    }

    #[must_use]
    pub fn to_radians(&self) -> CoordinateTuple {
        CoordinateTuple::new(self.0.to_radians(), self.1.to_radians(), self.2, self.3)
    }

    #[must_use]
    pub fn first(&self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn second(&self) -> f64 {
        self.1
    }

    #[must_use]
    pub fn third(&self) -> f64 {
        self.2
    }

    #[must_use]
    pub fn fourth(&self) -> f64 {
        self.3
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
        DMS { d, m, s }
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
    fn test_dms() {
        let dms = DMS::new(60, 24, 36.);
        assert_eq!(dms.d, 60);
        assert_eq!(dms.m, 24);
        assert_eq!(dms.s, 36.);
        let d = dms.to_degrees();
        assert_eq!(d, 60.41);
    }

    #[test]
    fn test_coordinatetuple() {
        let c = CoordinateTuple::new(12., 55., 100., 0.).to_radians();
        let d = CoordinateTuple::deg(12., 55., 100., 0.);
        assert_eq!(c, d);
        assert_eq!(d.0, 12f64.to_radians());
        let e = d.to_degrees();
        assert_eq!(e.0, c.to_degrees().0);
    }

    #[test]
    fn test_array() {
        let b = CoordinateTuple::new(7., 8., 9., 10.);
        let c = [b.0, b.1, b.2, b.3, f64::NAN, f64::NAN];
        assert_eq!(b.0, c[0]);
    }
}
