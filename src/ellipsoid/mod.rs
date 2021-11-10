mod cartesians;
mod geodesics;
pub(crate) mod latitudes;
mod meridians;

use crate::GeodesyError;

/// Representation of a (potentially triaxial) ellipsoid.
#[derive(Clone, Copy, Debug)]
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

    // fn from_yaml(definition: &str) -> Option<Ellipsoid> {
    //     use yaml_rust::YamlLoader;
    //     if let Ok(docs) = YamlLoader::load_from_str(definition) {
    //         let doc = &docs[0];
    //         if let Some(a) = doc["ellipsoid"]["shortcut"]["a"].as_f64() {
    //             if let Some(rf) = doc["ellipsoid"]["shortcut"]["rf"].as_f64() {
    //                 let f = if rf == 0.0 { 0.0 } else { 1.0 / (rf) };
    //                 if let Some(ay) = doc["ellipsoid"]["shortcut"]["ay"].as_f64() {
    //                     return Some(Ellipsoid::triaxial(a, ay, f));
    //                 }
    //                 return Some(Ellipsoid::new(a, f));
    //             }
    //         }
    //     }
    //     None
    // }

    /// Predefined ellipsoid; built-in or defined in asset collections
    #[must_use]
    pub fn named(name: &str) -> Result<Ellipsoid, GeodesyError> {
        // Is it one of the few builtins?
        if name == "GRS80" {
            return Ok(Ellipsoid::new(6_378_137.0, 1. / 298.257_222_100_882_7));
        } else if name == "WGS84" {
            return Ok(Ellipsoid::new(6_378_137.0, 1. / 298.257_223_563));
        } else if name == "intl" {
            return Ok(Ellipsoid::new(6_378_388.0, 1. / 297.0));
        } else if name == "Helmert" {
            return Ok(Ellipsoid::new(6_378_200.0, 1. / 298.3));
        } else if name == "clrk66" {
            return Ok(Ellipsoid::new(6_378_206.4, 1. / 294.978_698_2));
        } else if name == "clrk80" {
            return Ok(Ellipsoid::new(6_378_249.145, 1. / 293.465));
        } else if name == "bessel" {
            return Ok(Ellipsoid::new(6_377_397.155, 1. / 299.152_812_8));
        }

        Err(GeodesyError::NotFound(String::from(name)))
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
    fn test_ellipsoid() -> Result<(), GeodesyError> {
        // Constructors
        let ellps = Ellipsoid::named("intl")?;
        assert_eq!(ellps.flattening(), 1. / 297.);

        // let ellps = Ellipsoid::named("APL4.9")?;
        // assert_eq!(ellps.flattening(), 1. / 298.25);

        let ellps = Ellipsoid::named("GRS80")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25722_21008_82711_24316);

        assert!((ellps.normalized_meridian_arc_unit() - 0.9983242984230415).abs() < 1e-13);
        assert!((4.0 * ellps.meridian_quadrant() - 40007862.9169218).abs() < 1e-7);
        Ok(())
    }

    #[test]
    fn shape_and_size() -> Result<(), GeodesyError> {
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

        // let ellps = Ellipsoid::named("unitsphere")?;
        // assert!((ellps.semimajor_axis() - 1.0) < 1e-10);
        // assert!(ellps.flattening() < 1e-20);

        // Test a few of the ellipsoids imported from PROJ
        // let ellps = Ellipsoid::named("krass")?;
        // assert_eq!(ellps.semimajor_axis(), 6378245.0);
        // assert_eq!(ellps.flattening(), 1. / 298.3);
        //
        // let ellps = Ellipsoid::named("MERIT")?;
        // assert_eq!(ellps.semimajor_axis(), 6378137.0);
        // assert_eq!(ellps.flattening(), 1. / 298.257);
        Ok(())
    }

    #[test]
    fn curvatures() -> Result<(), GeodesyError> {
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
