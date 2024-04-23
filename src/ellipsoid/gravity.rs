use super::*;

// ----- Normal gravity -----------------------------------------------------

/// The normal gravity implementations are based solely on information
/// from [HandWiki](https://handwiki.org/wiki/Earth:Normal_gravity_formula).
///
/// Currently they remain untested in the absolute sense of the word:
/// The test suite checks for regressions, but the values used for comparison
/// are entirely internally sourced.
pub trait Gravity: EllipsoidBase {
    /// The Somigliana normal gravity formula. If the equatorial
    /// normal gravity, gamma_a, and/or the polar normal gravity,
    /// gamma_b, is given as `None`, the values for GRS80 is used.
    #[must_use]
    fn somigliana_gravity(&self, latitude: f64, gamma_a: Option<f64>, gamma_b: Option<f64>) -> f64 {
        let ga = gamma_a.unwrap_or(9.780_326_771_5);
        let gb = gamma_b.unwrap_or(9.832_186_368_5);

        let a = self.semimajor_axis();
        let b = self.semiminor_axis();
        let es = self.eccentricity_squared();

        let p = b * gb / (a * ga) - 1.0;
        let s = latitude.sin().powi(2);

        ga * (1.0 + p * s) / (1.0 - s * es).sqrt()
    }

    /// The international gravity formula 1930, for use with the
    /// international (Hayford) ellipsoid (or for mis-use with
    /// any other ellipsoid)
    #[must_use]
    fn cassinis_gravity_1930(&self, latitude: f64) -> f64 {
        const GAMMA_A: f64 = 9.780_49;
        const BETA_1: f64 = 5.2884e-3;
        const BETA_2: f64 = 5.9e-6;

        let s1 = latitude.sin().powi(2);
        let s2 = (latitude * 2.).sin().powi(2);
        GAMMA_A * (1.0 + s1 * BETA_1 - s2 * BETA_2)
    }

    /// Harold Jeffreys' 1948 improvement to the international
    /// gravity formula 1930
    #[must_use]
    fn jeffreys_gravity_1948(&self, latitude: f64) -> f64 {
        const GAMMA_A: f64 = 9.780_373;
        const BETA_1: f64 = 5.2884e-3;
        const BETA_2: f64 = 5.9e-6;

        let s1 = latitude.sin().powi(2);
        let s2 = (latitude * 2.).sin().powi(2);
        GAMMA_A * (1.0 + s1 * BETA_1 - s2 * BETA_2)
    }

    /// The GRS67 gravity formula. Differs from GRS80 at the mgal level
    #[must_use]
    fn grs67_gravity(&self, latitude: f64) -> f64 {
        const GAMMA_A: f64 = 9.780_318;
        const BETA_1: f64 = 5.3024e-3;
        const BETA_2: f64 = 5.9e-6;

        let s1 = latitude.sin().powi(2);
        let s2 = (latitude * 2.).sin().powi(2);
        GAMMA_A * (1.0 + s1 * BETA_1 - s2 * BETA_2)
    }

    /// The international gravity formula 1980, for use with
    /// systems based on GRS80
    #[must_use]
    fn grs80_gravity(&self, latitude: f64) -> f64 {
        // Equatorial normal gravity [m/s²]
        const GAMMA_A: f64 = 9.780_326_771_5;
        const C1: f64 = 5.279_041_4e-3;
        const C2: f64 = 2.327_180e-5;
        const C3: f64 = 1.262e-7;
        const C4: f64 = 7.0e-10;

        let s = latitude.sin().powi(2);
        GAMMA_A * (1.0 + s * (C1 + s * (C2 + s * (C3 + s * C4))))
    }

    /// Height correction according to Cassinis. Depends on the local rock
    /// density (in kg/m³). The value is to be **subtracted** from the
    /// normal gravity on the ellipsoid.
    #[must_use]
    fn cassinis_height_correction(&self, height: f64, density: f64) -> f64 {
        (3.08e-6 - 4.19e-10 * density) * height
    }

    /// The GRS67 height correction formula (used for all systems since GRS67).
    /// The value is to be **subtracted** from the normal gravity on the
    /// ellipsoid.
    #[must_use]
    fn grs67_height_correction(&self, latitude: f64, height: f64) -> f64 {
        ((3.0877e-6 - 4.3e-9 * latitude.sin().powi(2)) + 7.2e-13 * height) * height
    }

    /// The WELMEC method combines the latitudinal and the height correction
    /// of the normal gravity into a single formula.
    ///
    /// The HandWiki page states that the WELMEC formula is used in German standards
    /// labs (but probably EU+-wide, since the WELMEC organization covers the EU,
    /// EFTA, and a number of additional countries).
    ///
    /// Continuing: The free-fall acceleration g is calculated with respect to
    /// the average latitude φ and the average height above sea level H.
    ///
    /// **NOTE:** Orthometric height! And with the reference to averages of height
    /// and latitude, I suppose that this is for post-processing of surveys, not of
    /// individual observations? - or perhaps the HandWiki text is just slightly
    /// wrong here.
    #[must_use]
    fn welmec(&self, latitude: f64, height: f64) -> f64 {
        let s1 = latitude.sin().powi(2);
        let s2 = (latitude * 2.).sin().powi(2);
        (1.0 + 0.0053024 * s1 - 0.0000058 * s2) * 9.780318 - 0.000003085 * height
    }
}

// ----- Tests ---------------------------------------------------------------------

// Note that the tests in this section are **regression tests**: They values checked
// are not based on external authoritative sources. They only serve to be reasonably
// sure that future code changes do not change the functionality.

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn somigliana_gravity() {
        let ellps = Ellipsoid::named("GRS80").unwrap();
        assert!((ellps.somigliana_gravity(45_f64, None, None) - 9.81782912276995) < 1e-15);
    }

    #[test]
    fn cassinis_gravity_1930() {
        let ellps = Ellipsoid::named("GRS80").unwrap();
        //assert_eq!(ellps.cassinis_gravity_1930(45_f64.to_radians()), 10.0);
        assert!((ellps.cassinis_gravity_1930(45_f64.to_radians()) - 9.806293866767001) < 1e-15);
    }

    #[test]
    fn jeffreys_gravity_1948() {
        let ellps = Ellipsoid::named("GRS80").unwrap();
        //assert_eq!(ellps.jeffreys_gravity_1948(45_f64.to_radians()), 10.0);
        assert!((ellps.jeffreys_gravity_1948(45_f64.to_radians()) - 9.806176558085902) < 1e-15);
    }

    #[test]
    fn grs67_gravity() {
        let ellps = Ellipsoid::named("GRS67").unwrap();
        //assert_eq!(ellps.grs67_gravity(45_f64.to_radians()), 10.0);
        assert!((ellps.grs67_gravity(45_f64.to_radians()) - 9.806189875205401) < 1e-15);
    }

    #[test]
    fn grs80_gravity() {
        let ellps = Ellipsoid::named("GRS80").unwrap();
        // assert_eq!(ellps.grs80_gravity(45_f64.to_radians()), 10.0);
        assert!((ellps.grs80_gravity(45_f64.to_radians()) - 9.806199202630822) < 1e-15);
    }

    #[test]
    fn cassinis_height_correction() {
        let ellps = Ellipsoid::named("intl").unwrap();
        assert!(
            (ellps.cassinis_height_correction(1000., 2800.) - 0.0019068000000000002).abs() < 1e-9
        );
    }

    #[test]
    fn grs67_height_correction() {
        let ellps = Ellipsoid::named("GRS80").unwrap();
        assert!(
            (ellps.grs67_height_correction(45_f64.to_radians(), 1000.) - 0.00308627).abs() < 1e-8
        );
    }

    #[test]
    fn welmec() {
        let ellps = Ellipsoid::named("GRS80").unwrap();
        assert!((ellps.welmec(45_f64.to_radians(), 1000.) - 9.803105853237199).abs() < 1e-12);
    }
}
