use crate::CoordinateTuple;
use std::f64::consts::FRAC_PI_2;

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
        return (self.s as f64 / 60. + self.m as f64) / 60. + self.d as f64;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Ellipsoid {
    pub name: &'static str,
    pub a: f64,
    pub f: f64,
}

impl Ellipsoid {
    pub fn eccentricity_squared(&self) -> f64 {
        self.f*(2_f64 - self.f)
    }

    pub fn eccentricity(&self) -> f64 {
        self.eccentricity_squared().sqrt()
    }

    pub fn second_eccentricity_squared(&self) -> f64 {
        let b = self.semiminor_axis();
        let bb = b*b;
        (self.a.powi(2) - bb) / bb
    }

    pub fn second_eccentricity(&self) -> f64 {
        self.second_eccentricity_squared().sqrt()
    }

    pub fn semiminor_axis(&self) -> f64 {
        self.a * (1.0 - self.f)
    }

    pub fn prime_vertical_radius_of_curvature(&self, latitude: f64) -> f64 {
        let sinlat = latitude.sin();
        if self.f == 0.0 {
            return self.a;
        }
        self.a / (1.0 - sinlat * sinlat * self.eccentricity_squared()).sqrt()
    }

    #[allow(non_snake_case)]
    pub fn cartesian(&self, geographic: &CoordinateTuple) -> CoordinateTuple {
        let lam = geographic.first();
        let phi = geographic.second();
        let h = geographic.third();
        let t = geographic.fourth();

        let N = self.prime_vertical_radius_of_curvature(phi);
        let cosphi = phi.cos();
        let sinphi = phi.sin();
        let coslam = lam.cos();
        let sinlam = lam.sin();

        let X = (N + h) * cosphi * coslam;
        let Y = (N + h) * cosphi * sinlam;
        let Z = (N * (1.0 - self.eccentricity_squared()) + h) * sinphi;

        CoordinateTuple{0: X, 1: Y, 2: Z, 3: t}
    }


    #[allow(non_snake_case)]
    pub fn geographic(&self, cartesian: &CoordinateTuple) -> CoordinateTuple {
        let X = cartesian.first();
        let Y = cartesian.second();
        let Z = cartesian.third();
        let t = cartesian.fourth();

        // We need a few additional ellipsoidal parameters
        let b = self.semiminor_axis();
        let eps = self.second_eccentricity_squared();
        let es = self.eccentricity_squared();

        // The longitude is straightforward
        let lam = Y.atan2(X);

        // The perpendicular distance from the point coordinate to the Z-axis
        // (HM eq. 5-28)
        let p = X.hypot(Y);

        // For p < 1 picometer, we simplify things to avoid numerical havoc.
        if p < 1.0e-12 {
            let hemisphere = Z.signum();
            let phi = hemisphere * FRAC_PI_2;
            let h = Z.abs() - self.a;
            return CoordinateTuple{0: lam, 1: phi, 2: h, 3: t};
        }

        // HM eq. (5-36) and (5-37), with some added numerical efficiency due to
        // Even Rouault, who replaced 3 trigs with a hypot and two divisions.
        let theta_num = Z * self.a;
        let theta_denom = p * b;
        let length = theta_num.hypot(theta_denom);
        let c = theta_denom / length; // i.e. cos(theta)
        let s = theta_num / length;   // i.e. sin(theta)

        let phi_num = Z + eps * b * s.powi(3);
        let phi_denom = p - es * self.a * c.powi(3);
        let phi = phi_num.atan2(phi_denom);

        let h = p / phi.cos() - self.prime_vertical_radius_of_curvature(phi);
        CoordinateTuple{0: lam, 1: phi, 2: h, 3: t}
    }
}


// A hashmap indexed by the ellipsoid name would be better, but Rust cannot
// statically initialize a hashmap, so we put the name into the struct and
// use a static array instead.
static ELLIPSOIDS: [Ellipsoid; 5] =  [
    Ellipsoid {name: "GRS80"  ,  a: 6378137.0,   f: 1./298.257222100882711243},
    Ellipsoid {name: "intl"   ,  a: 6378388.0,   f: 1./297.},
    Ellipsoid {name: "Helmert",  a: 6378200.0,   f: 1./298.3},
    Ellipsoid {name: "clrk66" ,  a: 6378206.4,   f: 1./294.9786982},
    Ellipsoid {name: "clrk80" ,  a: 6378249.145, f: 1./293.465}
];

pub fn ellipsoid(name: &str) -> Ellipsoid {
    for e in ELLIPSOIDS.iter() {
        if e.name == name {
            return *e;
        }
    }
    ELLIPSOIDS[0]
}


/// Misc. auxiliary latitudes
pub trait Latitude {
    /// Geographic latitude to geocentric latitude
    /// or vice versa if `forward` is set to `false`.
    fn geocentric(&self, f: f64, forward: bool) -> f64;

    /// Geographic latitude to reduced latitude
    /// or vice versa if `forward` is set to `false`.
    fn reduced(&self, f: f64, forward: bool) -> f64;
}


impl Latitude for f64 {
    fn geocentric(&self, f: f64, forward: bool) -> f64 {
        // Geographic to geocentric?
        if forward {
            return ((1.0 - f * (2.0 - f)) * self.tan()).atan();
        }
        (self.tan() / (1.0 - f * (2.0 - f))).atan()
    }

    fn reduced(&self, f: f64, forward: bool) -> f64 {
        // Geographic to reduced?
        if forward {
            return ((1.0 - f) * self.tan()).atan();
        }
        (self.tan() / (1.0 - f)).atan()
    }
}

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

    #[test]
    fn test_ellipsoid() {
        let ellps = super::ellipsoid("GRS80");
        assert_eq!(ellps.a, 6378137.0);
        assert_eq!(ellps.f, 1. / 298.257222100882711243);

        let ellps = super::ellipsoid("intl");
        assert_eq!(ellps.f, 1. / 297.);

        let geo = super::CoordinateTuple{0: 12_f64.to_radians(), 1: 55_f64.to_radians(), 2: 100.0, 3: 0.};
        let cart = ellps.cartesian(&geo);
        let geo2 = ellps.geographic(&cart);
        assert!((geo.0-geo2.0) < 1.0e-12);
        assert!((geo.1-geo2.1) < 1.0e-12);

    }

    #[test]
    fn test_latitude() {
        use super::Latitude;
        let ellps = super::ellipsoid("GRS80");
        let phi = 60.0_f64.to_radians();
        let theta = phi.geocentric(ellps.f, true);
        let phi2 = theta.geocentric(ellps.f, false);
        assert!((phi - phi2).abs() < 1.0e-10);
        let theta2 = phi2.geocentric(ellps.f, true);
        assert!((theta - theta2).abs() < 1.0e-10);

        let beta = phi.reduced(ellps.f, true);
        let phi2 = beta.reduced(ellps.f, false);
        assert!((phi - phi2).abs() < 1.0e-10);
    }
}

/*
# Abbreviations used
#
# In general, for code readability, concepts are spelled out fully in
# identifiers (i.e. constants, variables, functions, classes).
#
# In a few very common cases, it makes sense to maintain short forms.
# These cases are limited to:
#
# lat:       latitude
# lon:       longitude
# f:         flattening
# rf:        inverse flattening
# e:         eccentricity
# e2:        eccentricity squared
# deg:       degree(s)
# rad:       radian(s)
#

def torad(deg):
    return pi*deg/180.
def todeg(rad):
    return 180*rad/pi
def dms2dd(dms):
    return (dms[2]/60. + dms[1])/60. + dms[0]


def geocentric_lat(geographic_lat, f):
    return atan((1 - f*(2 - f))*tan(geographic_lat))

def geographic_lat(geocentric_lat, f):
    return atan(tan(geocentric_lat) / (1 - f*(2 - f)))

def eccentricity_squared(f):
    return f*(2 - f)
*/
