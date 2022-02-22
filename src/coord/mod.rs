use super::internal::*;
use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};

pub mod conversions;
pub mod distances;

/// Generic 4D coordinate tuple, with no fixed interpretation of the elements
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Coord(pub [f64; 4]);

// ----- O P E R A T O R   T R A I T S -------------------------------------------------

impl Index<usize> for Coord {
    type Output = f64;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for Coord {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

impl Add for Coord {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Coord([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }
}

impl Add<&Coord> for Coord {
    type Output = Self;
    fn add(self, other: &Self) -> Self {
        Coord([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }
}

impl Sub for Coord {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Coord([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            self.0[3] - other.0[3],
        ])
    }
}

impl Mul for Coord {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Coord([
            self.0[0] * other.0[0],
            self.0[1] * other.0[1],
            self.0[2] * other.0[2],
            self.0[3] * other.0[3],
        ])
    }
}

impl Div for Coord {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Coord([
            self.0[0] / other.0[0],
            self.0[1] / other.0[1],
            self.0[2] / other.0[2],
            self.0[3] / other.0[3],
        ])
    }
}

impl Coord {
    /// Constructors

    /// A `Coord` from latitude/longitude/height/time, with the angular input in degrees
    #[must_use]
    pub fn geo(latitude: f64, longitude: f64, height: f64, time: f64) -> Coord {
        Coord([longitude, latitude, height, time]).to_radians()
    }

    /// A `Coord` from longitude/latitude/height/time, with the angular input in degrees
    #[must_use]
    pub fn gis(longitude: f64, latitude: f64, height: f64, time: f64) -> Coord {
        Coord([longitude, latitude, height, time]).to_radians()
    }

    /// A `Coord` from longitude/latitude/height/time, with the angular input in radians
    #[must_use]
    pub fn raw(first: f64, second: f64, third: f64, fourth: f64) -> Coord {
        Coord([first, second, third, fourth])
    }

    /// A `Coord` from latitude/longitude/height/time,
    /// with the angular input in NMEA format: DDDMM.mmmmm
    #[must_use]
    pub fn nmea(latitude: f64, longitude: f64, height: f64, time: f64) -> Coord {
        let longitude = Coord::nmea_to_dd(longitude);
        let latitude = Coord::nmea_to_dd(latitude);
        Coord([longitude, latitude, height, time]).to_radians()
    }

    /// A `Coord` from latitude/longitude/height/time, with
    /// the angular input in extended NMEA format: DDDMMSS.sssss
    #[must_use]
    pub fn nmeass(latitude: f64, longitude: f64, height: f64, time: f64) -> Coord {
        let longitude = Coord::nmeass_to_dd(longitude);
        let latitude = Coord::nmeass_to_dd(latitude);
        Coord::geo(latitude, longitude, height, time)
    }

    /// A `Coord` consisting of 4 `NaN`s
    #[must_use]
    pub fn nan() -> Coord {
        Coord([f64::NAN, f64::NAN, f64::NAN, f64::NAN])
    }

    /// A `Coord` consisting of 4 `0`s
    #[must_use]
    pub fn origin() -> Coord {
        Coord([0., 0., 0., 0.])
    }

    /// A `Coord` consisting of 4 `1`s
    #[must_use]
    pub fn ones() -> Coord {
        Coord([1., 1., 1., 1.])
    }

    /// Accessors

    /// First coordinate of the `Coord`
    #[must_use]
    pub fn first(&self) -> f64 {
        self[0]
    }

    /// Second coordinate of the `Coord`
    #[must_use]
    pub fn second(&self) -> f64 {
        self[1]
    }

    /// Third coordinate of the `Coord`
    #[must_use]
    pub fn third(&self) -> f64 {
        self[2]
    }

    /// Fourth coordinate of the `Coord`
    #[must_use]
    pub fn fourth(&self) -> f64 {
        self[3]
    }

    /// Arithmetic (also see the operator trait implementations `add, sub, mul, div`)

    /// Multiply by a scalar
    #[must_use]
    pub fn scale(&self, factor: f64) -> Coord {
        let mut result = Coord::nan();
        for i in 0..4 {
            result[i] = self[i] * factor;
        }
        result
    }

    /// Scalar product
    #[must_use]
    pub fn dot(&self, other: Coord) -> f64 {
        let mut result = 0_f64;
        for i in 0..4 {
            result += self[i] * other[i];
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord() {
        let c = Coord::raw(12., 55., 100., 0.).to_radians();
        let d = Coord::gis(12., 55., 100., 0.);
        assert_eq!(c, d);
        assert_eq!(d[0], 12f64.to_radians());
        let e = d.to_degrees();
        assert_eq!(e[0], c.to_degrees()[0]);
    }

    #[test]
    fn array() {
        let b = Coord::raw(7., 8., 9., 10.);
        let c = [b[0], b[1], b[2], b[3], f64::NAN, f64::NAN];
        assert_eq!(b[0], c[0]);
    }

    #[test]
    fn arithmetic() {
        let a = Coord([1., 2., 3., 4.]);
        let b = Coord([4., 3., 2., 1.]);
        let t = Coord([12., 12., 12., 12.]);

        let c = a.add(b);
        assert_eq!(c, Coord([5., 5., 5., 5.]));

        let d = c.scale(2.);
        assert_eq!(d, Coord([10., 10., 10., 10.]));

        let e = t.div(b);
        assert_eq!(e, Coord([3., 4., 6., 12.]));

        assert_eq!(e.mul(b), t);
        assert_eq!(a.dot(b), 20.)
    }
}
