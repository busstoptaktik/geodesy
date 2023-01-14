use super::*;
use crate::math::*;

// ----- Latitudes -------------------------------------------------------------
impl Ellipsoid {
    // --- Classic latitudes: geographic, geocentric & reduced ---

    /// Geographic latitude, 𝜙 to geocentric latitude, 𝜃.
    /// See also [latitude_geocentric_to_geographic](Ellipsoid::latitude_geocentric_to_geographic)
    #[must_use]
    pub fn latitude_geographic_to_geocentric(&self, geographic: f64) -> f64 {
        ((1.0 - self.f * (2.0 - self.f)) * geographic.tan()).atan()
    }

    /// Geocentric latitude, 𝜃 to geographic latitude, 𝜙.
    /// See also [latitude_geographic_to_geocentric](Ellipsoid::latitude_geographic_to_geocentric)
    #[must_use]
    pub fn latitude_geocentric_to_geographic(&self, latitude: f64) -> f64 {
        (latitude.tan() / (1.0 - self.eccentricity_squared())).atan()
    }

    /// Geographic latitude to reduced latitude, 𝛽
    #[must_use]
    pub fn latitude_geographic_to_reduced(&self, latitude: f64) -> f64 {
        latitude.tan().atan2(1. / (1. - self.f))
    }

    /// Geographic latitude to reduced latitude, 𝛽
    #[must_use]
    pub fn latitude_reduced_to_geographic(&self, latitude: f64) -> f64 {
        latitude.tan().atan2(1. - self.f)
    }

    /// Geographic latitude to Isometric latitude, 𝜓
    /// (or vice versa if `forward` is  `false`).
    #[must_use]
    pub fn isometric_latitude(&self, latitude: f64, direction: Direction) -> f64 {
        let e = self.eccentricity();
        if direction == Direction::Fwd {
            return inverse_gudermannian(latitude) - (e * latitude.sin()).atanh() * e;
        }
        sinhpsi_to_tanphi(latitude.sinh(), e).atan()
    }

    // --- Auxiliary latitudes ---

    // --- Rectifying latitude ---

    /// Obtain the coefficients needed for working with rectifying latitudes
    pub fn coefficients_for_rectifying_latitude_computations(&self) -> FourierCoefficients {
        self.latitude_fourier_coefficients(&constants::RECTIFYING)
    }

    /// Geographic latitude, 𝜙, to rectifying, 𝜇
    pub fn latitude_geographic_to_rectifying(
        &self,
        geographic_latitude: f64,
        coefficients: &FourierCoefficients,
    ) -> f64 {
        coefficients.etc[0]
            * (geographic_latitude + clenshaw_sin(2. * geographic_latitude, &coefficients.fwd))
    }

    /// Rectifying latitude, 𝜇, to geographic, 𝜙
    pub fn latitude_rectifying_to_geographic(
        &self,
        rectifying_latitude: f64,
        coefficients: &FourierCoefficients,
    ) -> f64 {
        let rlat = rectifying_latitude / coefficients.etc[0];
        rlat + clenshaw_sin(2. * rlat, &coefficients.inv)
    }

    // --- Conformal latitude ---

    /// Obtain the coefficients needed for working with conformal latitudes
    pub fn coefficients_for_conformal_latitude_computations(&self) -> FourierCoefficients {
        self.latitude_fourier_coefficients(&constants::CONFORMAL)
    }

    /// Geographic latitude, 𝜙, to conformal, 𝜒
    pub fn latitude_geographic_to_conformal(
        &self,
        geographic_latitude: f64,
        coefficients: &FourierCoefficients,
    ) -> f64 {
        geographic_latitude + clenshaw_sin(2. * geographic_latitude, &coefficients.fwd)
    }

    /// conformal latitude, 𝜒, to geographic, 𝜙
    pub fn latitude_conformal_to_geographic(
        &self,
        conformal_latitude: f64,
        coefficients: &FourierCoefficients,
    ) -> f64 {
        conformal_latitude + clenshaw_sin(2. * conformal_latitude, &coefficients.inv)
    }

    fn latitude_fourier_coefficients(
        &self,
        coefficients: &PolynomialCoefficients,
    ) -> FourierCoefficients {
        let n = self.third_flattening();
        let mut result = fourier_coefficients(n, coefficients);
        result.etc[0] = self.normalized_meridian_arc_unit();
        result
    }
}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    // Geocentric latitude, 𝜃
    #[test]
    fn geocentric() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        let lats = Vec::from([35., 45., 55.]);
        for lat in &lats {
            let lat = *lat as f64;
            let theta = ellps.latitude_geographic_to_geocentric(lat.to_radians());
            let roundtrip = ellps.latitude_geocentric_to_geographic(theta).to_degrees();
            assert!((lat - roundtrip).abs() < 1e-15);
        }
        assert!(ellps.latitude_geographic_to_geocentric(0.0).abs() < 1.0e-10);
        assert!((ellps.latitude_geographic_to_geocentric(FRAC_PI_2) - FRAC_PI_2).abs() < 1.0e-10);
        Ok(())
    }

    // Reduced latitude, 𝛽
    #[test]
    fn reduced() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        let lat = 55_f64.to_radians();
        let lat1 = ellps.latitude_geographic_to_reduced(lat);
        let lat2 = ellps.latitude_reduced_to_geographic(lat1);
        assert!((lat - lat2) < 1.0e-12);
        assert!(ellps.latitude_geographic_to_reduced(0.0).abs() < 1.0e-10);
        assert!((ellps.latitude_geographic_to_reduced(FRAC_PI_2) - FRAC_PI_2).abs() < 1.0e-10);
        Ok(())
    }

    // Isometric latitude, 𝜓
    #[test]
    fn isometric() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        let angle = 45_f64.to_radians();
        let isometric = 50.227465815385806f64.to_radians();
        assert!((ellps.isometric_latitude(angle, Fwd) - isometric).abs() < 1e-15);
        assert!((ellps.isometric_latitude(isometric, Inv) - angle).abs() < 1e-15);
        Ok(())
    }

    // Rectifying latitude, 𝜇
    #[test]
    fn rectifying() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        let latitudes = vec![35., 45., 55., -35., -45., -55., 0., 90.];
        let coefficients = ellps.coefficients_for_rectifying_latitude_computations();
        // Roundtrip 𝜙 -> 𝜇 -> 𝜙
        for phi in latitudes {
            let lat = (phi as f64).to_radians();
            let mu = ellps.latitude_geographic_to_rectifying(lat, &coefficients);
            let phi = ellps.latitude_rectifying_to_geographic(mu, &coefficients);
            let ihp = ellps.latitude_rectifying_to_geographic(-mu, &coefficients);
            assert!((lat - phi).abs() < 1e-14);
            assert!((lat + ihp).abs() < 1e-14); // Symmetry
        }
        Ok(())
    }

    // Conformal latitude, 𝜒
    #[test]
    fn conformal() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        let latitudes = vec![35., 45., 55., -35., -45., -55., 0., 90.];
        #[rustfmt::skip]
        let conformal_latitudes = vec![
            34.819454814955349775,  44.807684055145067248,  54.819109023689023275, // Northern hemisphere
           -34.819454814955349775, -44.807684055145067248, -54.819109023689023275, // Symmetry wrt. the Equator
            0., 90., // Extreme values are invariant
        ];

        let chi_coefs = ellps.latitude_fourier_coefficients(&constants::CONFORMAL);
        let pairs = latitudes.iter().zip(conformal_latitudes.iter());

        for pair in pairs {
            // The casts are necessary, at least as of Rust 1.66
            let phi = (*(pair.0) as f64).to_radians();
            let chi = (*(pair.1) as f64).to_radians();
            assert!((chi - ellps.latitude_geographic_to_conformal(phi, &chi_coefs)).abs() < 1e-14);
            assert!((phi - ellps.latitude_conformal_to_geographic(chi, &chi_coefs)).abs() < 1e-14);
        }

        let lat = 55_f64.to_radians();
        let chi = ellps.latitude_geographic_to_conformal(lat, &chi_coefs);
        let phi = ellps.latitude_conformal_to_geographic(chi, &chi_coefs);
        assert!((chi.to_degrees() - 54.819109023689023275).abs() < 1e-12);
        assert_eq!(phi.to_degrees(), 55.0);
        Ok(())
    }
}

// From the Poder-Engsager implementation, as revitalized in Coopsy
// Conformal:      34.819454814955349775
// Geographic:     35.000000000000000000
//
// Conformal:      44.807684055145067248
// Geographic:     45.000000000000000000
//
// Conformal:      54.819109023689023275
// Geographic:     55.000000000000000000
//
// Conformal:     -54.819109023689023275
// Geographic:    -55.000000000000000000

// Geographic to conformal
// Coef[0] =   -0.00335655463626897662
// Coef[1] =   4.69457307327488333e-06
// Coef[2] =  -8.19449756752843304e-09
// Coef[3] =   1.55799671344272666e-11
// Coef[4] =    -3.103292317686079e-14
// Coef[5] =   6.38914768904757935e-17
//
// Conformal to geographic
// Coef[0] =    0.00335655148560440753
// Coef[1] =   6.57187326307206622e-06
// Coef[2] =   1.76467247399761524e-08
// Coef[3] =   5.38775389000947284e-11
// Coef[4] =   1.76400751591338953e-13
// Coef[5] =   6.05607405520758705e-16
