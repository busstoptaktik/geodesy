pub mod conversions;
pub mod distances;

use super::internal::*;
use std::ops::{Index, IndexMut};

/// Generic 4D coordinate tuple, with no fixed interpretation of the elements
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Coord(pub [f64; 4]);

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

impl Coord {
    /// A `CoordinateTuple` from latitude/longitude/height/time, with the angular input in degrees
    #[must_use]
    pub fn geo(latitude: f64, longitude: f64, height: f64, time: f64) -> Coord {
        Coord([longitude, latitude, height, time]).to_radians()
    }

    /// A `CoordinateTuple` from longitude/latitude/height/time, with the angular input in degrees
    #[must_use]
    pub fn gis(longitude: f64, latitude: f64, height: f64, time: f64) -> Coord {
        Coord([longitude, latitude, height, time]).to_radians()
    }

    /// A `CoordinateTuple` from longitude/latitude/height/time, with the angular input in radians
    #[must_use]
    pub fn raw(first: f64, second: f64, third: f64, fourth: f64) -> Coord {
        Coord([first, second, third, fourth])
    }

    /// A `CoordinateTuple` from latitude/longitude/height/time,
    /// with the angular input in NMEA format: DDDMM.mmmmm
    #[must_use]
    pub fn nmea(latitude: f64, longitude: f64, height: f64, time: f64) -> Coord {
        let longitude = Coord::nmea_to_dd(longitude);
        let latitude = Coord::nmea_to_dd(latitude);
        Coord([longitude, latitude, height, time]).to_radians()
    }

    /// A `CoordinateTuple` from latitude/longitude/height/time, with
    /// the angular input in extended NMEA format: DDDMMSS.sssss
    #[must_use]
    pub fn nmeass(latitude: f64, longitude: f64, height: f64, time: f64) -> Coord {
        let longitude = Coord::nmeass_to_dd(longitude);
        let latitude = Coord::nmeass_to_dd(latitude);
        Coord::geo(latitude, longitude, height, time)
    }

    /// A `CoordinateTuple` consisting of 4 `NaN`s
    #[must_use]
    pub fn nan() -> Coord {
        Coord([f64::NAN, f64::NAN, f64::NAN, f64::NAN])
    }

    /// A `CoordinateTuple` consisting of 4 `0`s
    #[must_use]
    pub fn origin() -> Coord {
        Coord([0., 0., 0., 0.])
    }

    /// A `CoordinateTuple` consisting of 4 `1`s
    #[must_use]
    pub fn ones() -> Coord {
        Coord([1., 1., 1., 1.])
    }

    /// First coordinate of the `CoordinateTuple`
    #[must_use]
    pub fn first(&self) -> f64 {
        self[0]
    }

    /// Second coordinate of the `CoordinateTuple`
    #[must_use]
    pub fn second(&self) -> f64 {
        self[1]
    }

    /// Third coordinate of the `CoordinateTuple`
    #[must_use]
    pub fn third(&self) -> f64 {
        self[2]
    }

    /// Fourth coordinate of the `CoordinateTuple`
    #[must_use]
    pub fn fourth(&self) -> f64 {
        self[3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coordinatetuple() {
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
}
