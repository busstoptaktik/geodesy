mod cartesians;
mod geodesics;
pub(crate) mod latitudes;
mod meridians;

use super::internal::*;

// A HashMap would have been a better choice,for the OPERATOR_LIST, except
// for the annoying fact that it cannot be compile-time constructed
#[rustfmt::skip]
const ELLIPSOID_LIST: [(&str, &str, &str, &str, &str); 47] = [
    ("MERIT",     "6378137",       "6378137",      "298.257",            "MERIT 1983"),
    ("SGS85",     "6378136",       "6378136",      "298.257",            "Soviet Geodetic System 85"),
    ("GRS80",     "6378137",       "6378137",      "298.2572221008827",  "GRS 1980(IUGG, 1980)"),
    ("IAU76",     "6378140",       "6378140",      "298.257",            "IAU 1976"),
    ("airy",      "6377563.396",   "6377563.396",  "299.3249646",        "Airy 1830"),
    ("APL4.9",    "6378137",       "6378137.0",    "298.25",             "Appl. Physics. 1965"),
    ("NWL9D",     "6378145",       "6378145.0",    "298.25",             "Naval Weapons Lab., 1965"),
    ("mod_airy",  "6377340.189",   "6377340.189",  "299.3249373654824",  "Modified Airy"),
    ("andrae",    "6377104.43 ",   "6377104.43",   "300.0",              "Andrae 1876 (Denmark, Iceland)"),
    ("danish",    "6377019.2563",  "6377019.2563", "300.0",              "Andrae 1876 (Denmark, Iceland)"),
    ("aust_SA",   "6378160",       "6378160",      "298.25",             "Australian Natl & S. Amer. 1969"),
    ("GRS67",     "6378160",       "6378160",      "298.2471674270",     "GRS 67(IUGG 1967)"),
    ("GSK2011",   "6378136.5",     "6378136.5",    "298.2564151",        "GSK-2011"),
    ("bessel",    "6377397.155",   "6377397.155",  "299.1528128",        "Bessel 1841"),
    ("bess_nam",  "6377483.865",   "6377483.865",  "299.1528128",        "Bessel 1841 (Namibia)"),
    ("clrk66",    "6378206.4",     "6378206.4",    "294.9786982138982",  "Clarke 1866"),
    ("clrk80",    "6378249.145",   "6378249.145",  "293.4663",           "Clarke 1880 mod."),
    ("clrk80ign", "6378249.2",     "6378249.2",    "293.4660212936269",  "Clarke 1880 (IGN)"),
    ("CPM",       "6375738.7",     "6375738.7",    "334.29",             "Comm. des Poids et Mesures 1799"),
    ("delmbr",    "6376428",       "6376428",      "311.5",              "Delambre 1810 (Belgium)"),
    ("engelis",   "6378136.05",    "6378136.05",   "298.2566",           "Engelis 1985"),
    ("evrst30",   "6377276.345",   "6377276.345",  "300.8017",           "Everest 1830"),
    ("evrst48",   "6377304.063",   "6377304.063",  "300.8017",           "Everest 1948"),
    ("evrst56",   "6377301.243",   "6377301.243",  "300.8017",           "Everest 1956"),
    ("evrst69",   "6377295.664",   "6377295.664",  "300.8017",           "Everest 1969"),
    ("evrstSS",   "6377298.556",   "6377298.556",  "300.8017",           "Everest (Sabah & Sarawak)"),
    ("fschr60",   "6378166",       "6378166",      "298.3",              "Fischer (Mercury Datum) 1960"),
    ("fschr60m",  "6378155",       "6378155",      "298.3",              "Modified Fischer 1960"),
    ("fschr68",   "6378150",       "6378150",      "298.3",              "Fischer 1968"),
    ("helmert",   "6378200",       "6378200",      "298.3",              "Helmert 1906"),
    ("hough",     "6378270",       "6378270",      "297.",               "Hough"),
    ("intl",      "6378388",       "6378388",      "297.",               "International 1909 (Hayford)"),
    ("krass",     "6378245",       "6378245",      "298.3",              "Krassovsky, 1942"),
    ("kaula",     "6378163",       "6378163",      "298.24",             "Kaula 1961"),
    ("lerch",     "6378139",       "6378139",      "298.257",            "Lerch 1979"),
    ("mprts",     "6397300",       "6397300",      "191.",               "Maupertius 1738"),
    ("new_intl",  "6378157.5",     "6378157.5",    "298.2496153900135",  "New International 1967"),
    ("plessis",   "6376523",       "6376523.",     "308.64099709583735", "Plessis 1817 (France)"),
    ("PZ90",      "6378136",       "6378136",      "298.25784",          "PZ-90"),
    ("SEasia",    "6378155",       "6378155",      "298.3000002408657",  "Southeast Asia"),
    ("walbeck",   "6376896",       "6376896",      "302.78000018165636", "Walbeck"),
    ("WGS60",     "6378165",       "6378165",      "298.3",              "WGS 60"),
    ("WGS66",     "6378145",       "6378145",      "298.25",             "WGS 66"),
    ("WGS72",     "6378135",       "6378135",      "298.26",             "WGS 72"),
    ("WGS84",     "6378137",       "6378137",      "298.257223563",      "WGS 84"),
    ("sphere",    "6370997",       "6370997",      "0.",                 "Normal Sphere (r=6370997)"),
    ("unitsphere",      "1",             "1",      "0.",                 "Unit Sphere (r=1)"),
];

/// Representation of a (potentially triaxial) ellipsoid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ellipsoid {
    a: f64,
    ay: f64,
    f: f64,
}

/// GRS80 is the default ellipsoid.
impl Default for Ellipsoid {
    fn default() -> Ellipsoid {
        Ellipsoid::new(6_378_137.0, 1. / 298.257_222_100_882_7)
    }
}

impl Ellipsoid {
    /// User defined ellipsoid
    #[must_use]
    pub fn new(semimajor_axis: f64, flattening: f64) -> Ellipsoid {
        Ellipsoid {
            a: semimajor_axis,
            ay: semimajor_axis,
            f: flattening,
        }
    }

    pub fn triaxial(semimajor_x_axis: f64, semimajor_y_axis: f64, flattening: f64) -> Ellipsoid {
        Ellipsoid {
            a: semimajor_x_axis,
            ay: semimajor_y_axis,
            f: flattening,
        }
    }

    /// Predefined ellipsoid; built-in or defined in asset collections
    pub fn named(name: &str) -> Result<Ellipsoid, Error> {
        // Is it one of the few builtins?
        if let Some(index) = ELLIPSOID_LIST.iter().position(|&ellps| ellps.0 == name) {
            let e = ELLIPSOID_LIST[index];
            let ax: f64 = e.1.parse().unwrap();
            let ay: f64 = e.2.parse().unwrap();
            let rf: f64 = e.3.parse().unwrap();
            let f = if rf != 0.0 { 1.0 / rf } else { rf };
            return Ok(Ellipsoid::triaxial(ax, ay, f));
        }

        // The "semiminor, reciproque-flattening" form, e.g. "6378137, 298.3"
        loop {
            let a_rf = name.split(',').collect::<Vec<_>>();
            if a_rf.len() != 2_usize {
                break;
            }
            if let Ok(a) = a_rf[0].trim().parse::<f64>() {
                if let Ok(rf) = a_rf[1].trim().parse::<f64>() {
                    return Ok(Ellipsoid::new(a, 1. / rf));
                }
            }
            break;
        }

        // TODO: Search asset collection
        Err(Error::NotFound(
            String::from(name),
            String::from("Ellipsoid::named()"),
        ))
    }

    // ----- Eccentricities --------------------------------------------------------

    /// The linear eccentricity *E* = sqrt(a² - b²). Negative if b > a.
    #[must_use]
    pub fn linear_eccentricity(&self) -> f64 {
        let b = self.semiminor_axis();
        let le = self.a * self.a - b * b;
        if self.a > b {
            return le.sqrt();
        }
        -(-le).sqrt()
    }

    /// The squared eccentricity *e² = (a² - b²) / a²*.
    #[must_use]
    pub fn eccentricity_squared(&self) -> f64 {
        self.f * (2_f64 - self.f)
    }

    /// The eccentricity *e*
    #[must_use]
    pub fn eccentricity(&self) -> f64 {
        self.eccentricity_squared().sqrt()
    }

    /// The squared second eccentricity *e'² = (a² - b²) / b² = e² / (1 - e²)*
    #[must_use]
    pub fn second_eccentricity_squared(&self) -> f64 {
        let es = self.eccentricity_squared();
        es / (1.0 - es)
    }

    /// The second eccentricity *e'*
    #[must_use]
    pub fn second_eccentricity(&self) -> f64 {
        self.second_eccentricity_squared().sqrt()
    }

    /// The semimajor axis, *a*
    #[must_use]
    pub fn semimajor_axis(&self) -> f64 {
        self.a
    }

    /// The semimedian axis, *ay*
    #[must_use]
    pub fn semimedian_axis(&self) -> f64 {
        self.ay
    }

    /// The semiminor axis, *b*
    #[must_use]
    pub fn semiminor_axis(&self) -> f64 {
        self.a * (1.0 - self.f)
    }

    // ----- Flattenings -----------------------------------------------------------

    /// The flattening, *f = (a - b)/a*
    #[must_use]
    pub fn flattening(&self) -> f64 {
        self.f
    }

    /// The second flattening, *f = (a - b) / b*
    #[must_use]
    pub fn second_flattening(&self) -> f64 {
        let b = self.semiminor_axis();
        (self.a - b) / b
    }

    /// The third flattening, *n = (a - b) / (a + b) = f / (2 - f)*
    #[must_use]
    pub fn third_flattening(&self) -> f64 {
        self.f / (2.0 - self.f)
    }

    /// The aspect ratio, *b / a  =  1 - f  =  sqrt(1 - e²)*
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        1.0 - self.f
    }

    // ----- Curvatures ------------------------------------------------------------

    /// The radius of curvature in the prime vertical, *N*
    #[must_use]
    pub fn prime_vertical_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.f == 0.0 {
            return self.a;
        }
        self.a / (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).sqrt()
    }

    /// The meridian radius of curvature, *M*
    #[must_use]
    pub fn meridian_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.f == 0.0 {
            return self.a;
        }
        let num = self.a * (1.0 - self.eccentricity_squared());
        let denom = (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).powf(1.5);
        num / denom
    }

    /// The polar radius of curvature, *c*
    #[must_use]
    pub fn polar_radius_of_curvature(&self) -> f64 {
        self.a * self.a / self.semiminor_axis()
    }
}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ellipsoid() -> Result<(), Error> {
        // Constructors
        let ellps = Ellipsoid::named("intl")?;
        assert_eq!(ellps.flattening(), 1. / 297.);

        let ellps = Ellipsoid::named("6378137, 298.25")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25);

        let ellps = Ellipsoid::named("GRS80")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25722_21008_82711_24316);

        assert!((ellps.normalized_meridian_arc_unit() - 0.9983242984230415).abs() < 1e-13);
        assert!((4.0 * ellps.meridian_quadrant() - 40007862.9169218).abs() < 1e-7);
        Ok(())
    }

    #[test]
    fn shape_and_size() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        let ellps = Ellipsoid::new(ellps.semimajor_axis(), ellps.flattening());
        let ellps = Ellipsoid::triaxial(ellps.a, ellps.a - 1., ellps.f);
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25722_21008_82711_24316);

        // Additional shape descriptors
        assert!((ellps.eccentricity() - 0.081819191).abs() < 1.0e-10);
        assert!((ellps.eccentricity_squared() - 0.00669_43800_22903_41574).abs() < 1.0e-10);

        // Additional size descriptors
        assert!((ellps.semiminor_axis() - 6_356_752.31414_0347).abs() < 1e-9);
        assert!((ellps.semimajor_axis() - 6_378_137.0).abs() < 1e-9);

        let ellps = Ellipsoid::named("unitsphere")?;
        assert!((ellps.semimajor_axis() - 1.0) < 1e-10);
        assert_eq!(ellps.flattening(), 0.);

        // Test a few of the ellipsoids imported from PROJ
        let ellps = Ellipsoid::named("krass")?;
        assert_eq!(ellps.semimajor_axis(), 6378245.0);
        assert_eq!(ellps.flattening(), 1. / 298.3);

        let ellps = Ellipsoid::named("MERIT")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.257);
        Ok(())
    }

    #[test]
    fn curvatures() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        // The curvatures at the North Pole
        assert!(
            (ellps.meridian_radius_of_curvature(90_f64.to_radians()) - 6_399_593.6259).abs() < 1e-4
        );
        assert!(
            (ellps.prime_vertical_radius_of_curvature(90_f64.to_radians()) - 6_399_593.6259).abs()
                < 1e-4
        );
        assert!(
            (ellps.prime_vertical_radius_of_curvature(90_f64.to_radians())
                - ellps.meridian_radius_of_curvature(90_f64.to_radians()))
            .abs()
                < 1e-5
        );
        assert!(
            (ellps.polar_radius_of_curvature()
                - ellps.meridian_radius_of_curvature(90_f64.to_radians()))
            .abs()
                < 1e-6
        );

        // The curvatures at the Equator
        assert!((ellps.meridian_radius_of_curvature(0.0) - 6_335_439.3271).abs() < 1.0e-4);
        assert!(
            (ellps.prime_vertical_radius_of_curvature(0.0) - ellps.semimajor_axis()).abs() < 1.0e-4
        );
        Ok(())
    }
}
