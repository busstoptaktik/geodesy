use phf::phf_map;

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
    pub a: f64,
    pub f: f64,
}

// We use a map based on a perfect hash function (phf) here - not because
// perfect hashing is important for the core functionality, but because
// the phf::map can be constructed at compile time. This is not the case for
// ordinary hashmaps.
static ELLIPSOIDS: phf::Map<&'static str, Ellipsoid> = phf_map! {
    "GRS80"  =>  Ellipsoid {a: 6378137.0,   f: 1./298.257222100882711243},
    "intl"   =>  Ellipsoid {a: 6378388.0,   f: 1./297.},
    "Helmert"=>  Ellipsoid {a: 6378200.0,   f: 1./298.3},
    "clrk66" =>  Ellipsoid {a: 6378206.4,   f: 1./294.9786982},
    "clrk80" =>  Ellipsoid {a: 6378249.145, f: 1./293.465}
};

pub fn ellipsoid(name: &str) -> Ellipsoid {
    if ELLIPSOIDS.contains_key(name) {
        return ELLIPSOIDS[name];
    }
    ELLIPSOIDS["GRS80"]
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
        return (self.tan() / (1.0 - f)).atan();
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
