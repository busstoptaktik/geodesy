use super::*;

// ----- Latitudes -------------------------------------------------------------
impl Ellipsoid {
    /// Geographic latitude to geocentric latitude
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

    pub fn geodetic_to_rectifying_latitude_coefficients(&self, direction: Direction) -> Vec<f64> {
        let n = self.third_flattening();
        let n2 = n*n;
        let mut d = n;

        let mut result = vec![];
        result.push(crate::math::horner(n2, &constants::MERIDIAN_ARC_COEFFICIENTS) / (1. + n));

        let coefs: &[f64] = if direction == Direction::Fwd {
            &constants::GEODETIC_TO_RECTIFYING_LATITUDE_COEFFICIENTS
        } else {
            &constants::RECTIFYING_TO_GEODETIC_LATITUDE_COEFFICIENTS
        };
        let degree = constants::RECTIFYING_TO_GEODETIC_LATITUDE_DEGREE;
        let mut first_coef = 0_usize;
        for i in 1_usize..=degree {
            let n_coefs = (degree - i) / 2;
            let c = &coefs[first_coef..first_coef+n_coefs];
            result.push(d * crate::math::horner(n2, c));
            d *= n;
            first_coef += n_coefs + 1;
        }
            // for (int l = 0, o = 0; l < Lmax; ++l) {
            //     int m = (Lmax - l - 1) / 2;
            //     en[l + 1       ] = d * polyval(n2, coeff_mu_phi + o, m);
            //     en[l + 1 + Lmax] = d * polyval(n2, coeff_phi_mu + o, m);
            //     d *= n;
            //     o += m + 1;
            // }
        result
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

        // Roundtrip reduced latitude
        let lat = 55_f64.to_radians();
        let lat2 = ellps.reduced_latitude(ellps.reduced_latitude(lat, Fwd), Inv);
        assert!((lat - lat2) < 1.0e-12);
        assert!(ellps.reduced_latitude(0.0, Fwd).abs() < 1.0e-10);
        assert!((ellps.reduced_latitude(FRAC_PI_2, Fwd) - FRAC_PI_2).abs() < 1.0e-10);

        // Isometric latitude, 𝜓
        let angle = 45f64.to_radians();
        let isometric = 50.227465815385806f64.to_radians();
        assert!((ellps.isometric_latitude(angle, Fwd) - isometric).abs() < 1e-15);
        assert!((ellps.isometric_latitude(isometric, Inv) - angle).abs() < 1e-15);
        Ok(())
    }
}
