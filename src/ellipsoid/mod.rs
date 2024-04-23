pub mod biaxial;
mod constants;
pub mod geocart;
pub mod geodesics;
pub mod gravity;
pub mod latitudes;
pub mod meridians;
pub mod triaxial;

use crate::prelude::*;

// Blanket implementations for all the Ellipsoidal traits
impl<T> Meridians for T where T: EllipsoidBase + ?Sized {}
impl<T> Latitudes for T where T: EllipsoidBase + ?Sized {}
impl<T> GeoCart for T where T: EllipsoidBase + ?Sized {}
impl<T> Geodesics for T where T: EllipsoidBase + ?Sized {}
impl<T> Gravity for T where T: EllipsoidBase + ?Sized {}

/// The fundamental "size and shape" parameters for an ellipsoid.
/// In general we assume that the ellipsoid is oblate and biaxial,
/// although triaxial and/or prolate ellipsoids are taken care of
/// in a few cases. Only required methods are `semimajor_axis` and
/// `flattening` (and, in the triaxial case, `semimedian_axis`)
pub trait EllipsoidBase {
    /// The semimajor axis, *a*
    fn semimajor_axis(&self) -> f64;

    /// The flattening, *f = (a - b)/a*
    fn flattening(&self) -> f64;

    /// Synonym for [Self::semimajor_axis]
    fn a(&self) -> f64 {
        self.semimajor_axis()
    }

    /// Synonym for [Self::flattening]
    fn f(&self) -> f64 {
        self.flattening()
    }

    // ----- Additional axes -------------------------------------------------------

    /// The semimedian axis, *ay*. Equals *a* unless the ellipsoid is triaxial.
    #[must_use]
    fn semimedian_axis(&self) -> f64 {
        self.semimajor_axis()
    }

    /// The semiminor axis, *b*
    #[must_use]
    fn semiminor_axis(&self) -> f64 {
        self.semimajor_axis() * (1.0 - self.flattening())
    }

    // ----- Additional Flattenings ------------------------------------------------

    /// The second flattening, *g  =  (a - b) / b*
    #[must_use]
    fn second_flattening(&self) -> f64 {
        let b = self.semiminor_axis();
        (self.a() - b) / b
    }

    /// The third flattening, *n  =  (a - b) / (a + b)  =  f / (2 - f)*
    #[must_use]
    fn third_flattening(&self) -> f64 {
        let flattening = self.flattening();
        flattening / (2.0 - flattening)
    }

    /// The aspect ratio, *a / b  =  1 / ( 1 - f )  =  1 / sqrt(1 - e²)*
    #[must_use]
    fn aspect_ratio(&self) -> f64 {
        (1.0 - self.flattening()).recip()
    }

    // ----- Eccentricities --------------------------------------------------------

    /// The linear eccentricity *E* = sqrt(a² - b²). Negative if b > a.
    #[must_use]
    fn linear_eccentricity(&self) -> f64 {
        let a = self.semimajor_axis();
        let b = self.semiminor_axis();
        let le = a * a - b * b;
        if a > b {
            return le.sqrt();
        }
        -(-le).sqrt()
    }

    /// The squared eccentricity *e² = (a² - b²) / a²*.
    #[must_use]
    fn eccentricity_squared(&self) -> f64 {
        self.flattening() * (2_f64 - self.flattening())
    }

    /// The eccentricity *e*
    #[must_use]
    fn eccentricity(&self) -> f64 {
        self.eccentricity_squared().sqrt()
    }

    /// The squared second eccentricity *e'² = (a² - b²) / b² = e² / (1 - e²)*
    #[must_use]
    fn second_eccentricity_squared(&self) -> f64 {
        let es = self.eccentricity_squared();
        es / (1.0 - es)
    }

    /// The second eccentricity *e'*
    #[must_use]
    fn second_eccentricity(&self) -> f64 {
        self.second_eccentricity_squared().sqrt()
    }

    // ----- Curvatures ------------------------------------------------------------

    /// The radius of curvature in the prime vertical, *N*
    #[must_use]
    fn prime_vertical_radius_of_curvature(&self, latitude: f64) -> f64 {
        let a = self.semimajor_axis();
        if self.flattening() == 0.0 {
            return a;
        }
        a / (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).sqrt()
    }

    /// The meridian radius of curvature, *M*
    #[must_use]
    fn meridian_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.flattening() == 0.0 {
            return self.semimajor_axis();
        }
        let num = self.semimajor_axis() * (1.0 - self.eccentricity_squared());
        let denom = (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).powf(1.5);
        num / denom
    }

    /// The polar radius of curvature, *c*
    #[must_use]
    fn polar_radius_of_curvature(&self) -> f64 {
        let a = self.semimajor_axis();
        a * a / self.semiminor_axis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_and_size() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        let ellps = Ellipsoid::new(ellps.semimajor_axis(), ellps.flattening());
        // let ellps = Ellipsoid::triaxial(ellps.a, ellps.a - 1., ellps.f);
        // assert_eq!(ellps.semimajor_axis(), 6378137.0);
        // assert_eq!(ellps.flattening(), 1. / 298.257_222_100_882_7);

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
