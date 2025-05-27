use super::*;

/// Generic 3D coordinate tuple, with no fixed interpretation of the elements
#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Coor3D(pub [f64; 3]);

impl CoordinateTuple for Coor3D {
    fn new(fill: f64) -> Self {
        Coor3D([fill; 3])
    }

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

// ----- C O N S T R U C T O R S ---------------------------------------------

/// Constructors
impl Coor3D {
    /// A `Coor3D` from latitude/longitude/height, with the angular input in degrees
    #[must_use]
    pub fn geo(latitude: f64, longitude: f64, height: f64) -> Coor3D {
        Coor3D([longitude.to_radians(), latitude.to_radians(), height])
    }

    /// A `Coor3D` from longitude/latitude/height, with the angular input in seconds
    /// of arc. Mostly for handling grid shift elements.
    #[must_use]
    pub fn arcsec(longitude: f64, latitude: f64, height: f64) -> Coor3D {
        Coor3D([
            longitude.to_radians() / 3600.,
            latitude.to_radians() / 3600.,
            height,
        ])
    }

    /// A `Coor3D` from longitude/latitude/height, with the angular input in degrees
    #[must_use]
    pub fn gis(longitude: f64, latitude: f64, height: f64) -> Coor3D {
        Coor3D([longitude.to_radians(), latitude.to_radians(), height])
    }

    /// A `Coor3D` from longitude/latitude/height/time, with the angular input in radians
    #[must_use]
    pub fn raw(first: f64, second: f64, third: f64) -> Coor3D {
        Coor3D([first, second, third])
    }

    /// A `Coor3D` from latitude/longitude/height,
    /// with the angular input in the ISO-6709 DDDMM.mmmmm format
    #[must_use]
    pub fn iso_dm(latitude: f64, longitude: f64, height: f64) -> Coor3D {
        let longitude = angular::iso_dm_to_dd(longitude);
        let latitude = angular::iso_dm_to_dd(latitude);
        Coor3D::geo(latitude, longitude, height)
    }

    /// A `Coor3D` from latitude/longitude/height, with
    /// the angular input in the ISO-6709 DDDMMSS.sssss format
    #[must_use]
    pub fn iso_dms(latitude: f64, longitude: f64, height: f64) -> Coor3D {
        let longitude = angular::iso_dms_to_dd(longitude);
        let latitude = angular::iso_dms_to_dd(latitude);
        Coor3D::geo(latitude, longitude, height)
    }

    /// A `Coor3D` consisting of 3 `NaN`s
    #[must_use]
    pub fn nan() -> Coor3D {
        Coor3D::new(f64::NAN) //([f64::NAN, f64::NAN, f64::NAN])
    }

    /// A `Coor3D` consisting of 3 `0`s
    #[must_use]
    pub fn origin() -> Coor3D {
        Coor3D([0., 0., 0.])
    }

    /// A `Coor3D` consisting of 3 `1`s
    #[must_use]
    pub fn ones() -> Coor3D {
        Coor3D([1., 1., 1.])
    }

    // Arithmetic (also see the operator trait implementations `add, sub, mul, div`)

    /// Multiply by a scalar
    #[must_use]
    pub fn scale(&self, factor: f64) -> Coor3D {
        let mut result = Coor3D::nan();
        for i in 0..3 {
            result[i] = self[i] * factor;
        }
        result
    }

    /// Scalar product
    #[must_use]
    pub fn dot(&self, other: Coor3D) -> f64 {
        let mut result = 0_f64;
        for i in 0..3 {
            result += self[i] * other[i];
        }
        result
    }
}

// ----- T E S T S ---------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::{Add, Div, Mul};

    #[test]
    fn distances() {
        let e = Ellipsoid::default();
        let lat = angular::dms_to_dd(55, 30, 36.);
        let lon = angular::dms_to_dd(12, 45, 36.);
        let dms = Coor3D::geo(lat, lon, 0.);
        let geo = Coor3D::geo(55.51, 12.76, 0.);
        assert!(e.distance(&geo, &dms) < 1e-10);
    }

    #[test]
    fn coord() {
        let c = Coor3D::raw(12., 55., 100.).to_radians();
        let d = Coor3D::gis(12., 55., 100.);
        assert_eq!(c, d);
        assert_eq!(d[0], 12f64.to_radians());
        let e = d.to_degrees();
        assert_eq!(e[0], c.to_degrees()[0]);
    }

    #[test]
    fn array() {
        let b = Coor3D::raw(7., 8., 9.);
        let c = [b[0], b[1], b[2], f64::NAN, f64::NAN, f64::NAN];
        assert_eq!(b[0], c[0]);
    }

    #[test]
    fn arithmetic() {
        let a = Coor3D([1., 2., 3.]);
        let b = Coor3D([4., 3., 2.]);
        let t = Coor3D([12., 12., 12.]);

        let c = a.add(b);
        assert_eq!(c, Coor3D([5., 5., 5.]));

        let d = c.scale(2.);
        assert_eq!(d, Coor3D([10., 10., 10.]));

        let e = t.div(b);
        assert_eq!(e, Coor3D([3., 4., 6.]));

        assert_eq!(e.mul(b), t);
        assert_eq!(a.dot(b), 16.)
    }
}
