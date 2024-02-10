mod cartesians;
mod constants;
mod geodesics;
mod gravity;
mod latitudes;
mod meridians;

use crate::prelude::*;

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
        if let Some(index) = constants::ELLIPSOID_LIST
            .iter()
            .position(|&ellps| ellps.0 == name)
        {
            let e = constants::ELLIPSOID_LIST[index];
            let ax: f64 = e.1.parse().unwrap();
            let ay: f64 = e.2.parse().unwrap();
            let rf: f64 = e.3.parse().unwrap();
            let f = if rf != 0.0 { 1.0 / rf } else { rf };
            return Ok(Ellipsoid::triaxial(ax, ay, f));
        }

        // The "semiminor, reciproque-flattening" form, e.g. "6378137, 298.3"
        let a_and_rf = name.split(',').collect::<Vec<_>>();
        if a_and_rf.len() == 2_usize {
            if let Ok(a) = a_and_rf[0].trim().parse::<f64>() {
                if let Ok(rf) = a_and_rf[1].trim().parse::<f64>() {
                    return Ok(Ellipsoid::new(a, 1. / rf));
                }
            }
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
        assert_eq!(ellps.flattening(), 1. / 298.257_222_100_882_7);

        assert!((ellps.normalized_meridian_arc_unit() - 0.998_324_298_423_041_5).abs() < 1e-13);
        assert!((4.0 * ellps.meridian_quadrant() - 40_007_862.916_921_8).abs() < 1e-7);
        Ok(())
    }

    #[test]
    fn shape_and_size() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        let ellps = Ellipsoid::new(ellps.semimajor_axis(), ellps.flattening());
        let ellps = Ellipsoid::triaxial(ellps.a, ellps.a - 1., ellps.f);
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.257_222_100_882_7);

        // Additional shape descriptors
        assert!((ellps.eccentricity() - 0.081819191).abs() < 1.0e-10);
        assert!((ellps.eccentricity_squared() - 0.006_694_380_022_903_416).abs() < 1.0e-10);

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
            (ellps.meridian_radius_of_curvature(90_f64.to_radians()) - 6_399_593.625_9).abs()
                < 1e-4
        );
        assert!(
            (ellps.prime_vertical_radius_of_curvature(90_f64.to_radians()) - 6_399_593.625_9).abs()
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
        assert!((ellps.meridian_radius_of_curvature(0.0) - 6_335_439.327_1).abs() < 1.0e-4);
        assert!(
            (ellps.prime_vertical_radius_of_curvature(0.0) - ellps.semimajor_axis()).abs() < 1.0e-4
        );

        // Regression test: Curvatures for a random range of latitudes
        let latitudes = [
            50f64, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0,
        ];

        #[allow(clippy::excessive_precision)]
        let prime_vertical_radii_of_curvature = [
            6390702.044256360,
            6391069.984921544,
            6391435.268276582,
            6391797.447784556,
            6392156.080476415,
            6392510.727498910,
            6392860.954658516,
            6393206.332960654,
            6393546.439143487,
            6393880.856205599,
            6394209.173926849,
        ];

        #[allow(clippy::excessive_precision)]
        let meridian_radii_of_curvature = [
            6372955.9257095090,
            6374056.7459167000,
            6375149.7412608800,
            6376233.5726736350,
            6377306.9111838430,
            6378368.4395775950,
            6379416.8540488490,
            6380450.8658386090,
            6381469.2028603740,
            6382470.6113096075,
            6383453.8572549970,
        ];

        for (i, lat) in latitudes.iter().enumerate() {
            let n = ellps.prime_vertical_radius_of_curvature(lat.to_radians());
            let m = ellps.meridian_radius_of_curvature(lat.to_radians());
            assert!((n - prime_vertical_radii_of_curvature[i]).abs() < 1e-9);
            assert!((m - meridian_radii_of_curvature[i]).abs() < 1e-9);
        }
        Ok(())
    }
}
