use super::*;
use crate::math::*;

// ----- Latitudes -------------------------------------------------------------
impl Ellipsoid {
    /// Geographic latitude, 𝜙 to geocentric latitude, 𝜙'
    /// (or vice versa if `forward` is `false`).
    #[must_use]
    pub fn geocentric_latitude(&self, latitude: f64, direction: Direction) -> f64 {
        if direction == Direction::Fwd {
            return ((1.0 - self.f * (2.0 - self.f)) * latitude.tan()).atan();
        }
        (latitude.tan() / (1.0 - self.eccentricity_squared())).atan()
    }

    /// Geographic latitude to reduced latitude, 𝛽
    /// (or vice versa if `forward` is  `false`).
    #[must_use]
    pub fn reduced_latitude(&self, latitude: f64, direction: Direction) -> f64 {
        if direction == Direction::Fwd {
            return latitude.tan().atan2(1. / (1. - self.f));
        }
        latitude.tan().atan2(1. - self.f)
    }

    /// Geographic latitude to Isometric latitude, 𝜓
    /// (or vice versa if `forward` is  `false`).
    #[must_use]
    pub fn isometric_latitude(&self, latitude: f64, direction: Direction) -> f64 {
        let e = self.eccentricity();
        if direction == Direction::Fwd {
            return crate::math::inverse_gudermannian(latitude) - (e * latitude.sin()).atanh() * e;
        }
        crate::math::sinhpsi_to_tanphi(latitude.sinh(), e).atan()
    }


    // Rectifying latitude, 𝜇
    #[must_use]
    pub fn rectifying_latitude(&self, latitude: f64, coefficients: &[f64], direction: Direction) -> Option<f64> {
        if coefficients.len() < 13 {
            return None;
        }

        if direction == Direction::Fwd {
            return Some(coefficients[0] * (latitude + clenshaw_sin(2.*latitude, &coefficients[1..7])));
        }
        let latitude = latitude / coefficients[0];
        Some(latitude + crate::math::clenshaw_sin(2.*latitude, &coefficients[7..]))
    }

    pub fn geodetic_to_rectifying_latitude_coefficients(&self) -> Vec<f64> {
        let mut result = Vec::<f64>::new();
        // The zero order coefficient
        result.push(self.normalized_meridian_arc_unit());
        // The forward coefficients
        self.geodetic_to_rectifying_latitude_coefficients_helper(&mut result, &constants::GEODETIC_TO_RECTIFYING_LATITUDE_COEFFICIENTS);
        // The inverse coefficients
        self.geodetic_to_rectifying_latitude_coefficients_helper(&mut result, &constants::RECTIFYING_TO_GEODETIC_LATITUDE_COEFFICIENTS);
        result
    }

    fn geodetic_to_rectifying_latitude_coefficients_helper(&self, result: &mut Vec<f64>, coefs: &[f64]) {
        let n = self.third_flattening();
        let n2 = n*n;
        let mut d = n;

        result.push(d * crate::math::horner(n2, &coefs[0..3]));
        d *= n;
        result.push(d * crate::math::horner(n2, &coefs[3..6]));
        d *= n;
        result.push(d * crate::math::horner(n2, &coefs[6..8]));
        d *= n;
        result.push(d * crate::math::horner(n2, &coefs[8..10]));
        d *= n;
        result.push(d*n2*coefs[10]);
        d *= n;
        result.push(d*n2*coefs[11]);
        println!("result: {:#?}", result);
    }

}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    use std::f64::consts::FRAC_PI_2;
    #[test]
    fn latitudes() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        // Roundtrip geocentric latitude
        let lat = 55_f64.to_radians();
        let lat2 = ellps.geocentric_latitude(ellps.geocentric_latitude(lat, Fwd), Inv);
        assert!((lat - lat2) < 1.0e-12);
        assert!(ellps.geocentric_latitude(0.0, Fwd).abs() < 1.0e-10);
        assert!((ellps.geocentric_latitude(FRAC_PI_2, Fwd) - FRAC_PI_2).abs() < 1.0e-10);

        // Roundtrip reduced latitude, 𝛽
        let lat = 55_f64.to_radians();
        let lat2 = ellps.reduced_latitude(ellps.reduced_latitude(lat, Fwd), Inv);
        assert!((lat - lat2) < 1.0e-12);
        assert!(ellps.reduced_latitude(0.0, Fwd).abs() < 1.0e-10);
        assert!((ellps.reduced_latitude(FRAC_PI_2, Fwd) - FRAC_PI_2).abs() < 1.0e-10);

        // Isometric latitude, 𝜓
        let angle = 45_f64.to_radians();
        let isometric = 50.227465815385806f64.to_radians();
        assert!((ellps.isometric_latitude(angle, Fwd) - isometric).abs() < 1e-15);
        assert!((ellps.isometric_latitude(isometric, Inv) - angle).abs() < 1e-15);

        // Rectifying latitude, 𝜇
        let coeffs = ellps.geodetic_to_rectifying_latitude_coefficients();
        // We expect `None` if the array slice of coefficients is too short
        assert!(ellps.rectifying_latitude(angle, &[1.,2.,3.], Fwd).is_none());
        // Roundtrip phi->mu->phi
        let mu = ellps.rectifying_latitude(angle, &coeffs, Fwd).unwrap();
        let phi = ellps.rectifying_latitude(mu, &coeffs, Inv).unwrap();
        assert!((angle-phi).abs().to_degrees() < 1e-10);
        // Symmetry
        assert_eq!(-mu, ellps.rectifying_latitude(-angle, &coeffs, Fwd).unwrap());

        Ok(())
    }

}
