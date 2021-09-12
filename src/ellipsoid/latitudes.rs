use crate::Ellipsoid;

// ----- Latitudes -------------------------------------------------------------
impl Ellipsoid {
    /// Geographic latitude to geocentric latitude
    /// (or vice versa if `forward` is `false`).
    #[must_use]
    pub fn geocentric_latitude(&self, latitude: f64, forward: bool) -> f64 {
        if forward {
            return ((1.0 - self.f * (2.0 - self.f)) * latitude.tan()).atan();
        }
        (latitude.tan() / (1.0 - self.eccentricity_squared())).atan()
    }

    /// Geographic latitude to reduced latitude, 𝛽
    /// (or vice versa if `forward` is  `false`).
    #[must_use]
    pub fn reduced_latitude(&self, latitude: f64, forward: bool) -> f64 {
        if forward {
            return latitude.tan().atan2(1. / (1. - self.f));
        }
        latitude.tan().atan2(1. - self.f)
    }

    /// Geographic latitude to Isometric latitude, 𝜓
    /// (or vice versa if `forward` is  `false`).
    #[must_use]
    pub fn isometric_latitude(&self, latitude: f64, forward: bool) -> f64 {
        let e = self.eccentricity();
        if forward {
            return latitude.tan().asinh() - (e * latitude.sin()).atanh() * e;
        }
        crate::internals::sinhpsi_to_tanphi(latitude.sinh(), e).atan()
    }
}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fwd;
    use crate::inv;
    use std::f64::consts::FRAC_PI_2;
    #[test]
    fn latitudes() {
        let ellps = Ellipsoid::named("GRS80");
        // Roundtrip geocentric latitude
        let lat = 55_f64.to_radians();
        let lat2 = ellps.geocentric_latitude(ellps.geocentric_latitude(lat, fwd), inv);
        assert!((lat - lat2) < 1.0e-12);
        assert!(ellps.geocentric_latitude(0.0, fwd).abs() < 1.0e-10);
        assert!((ellps.geocentric_latitude(FRAC_PI_2, fwd) - FRAC_PI_2).abs() < 1.0e-10);

        // Roundtrip reduced latitude
        let lat = 55_f64.to_radians();
        let lat2 = ellps.reduced_latitude(ellps.reduced_latitude(lat, fwd), inv);
        assert!((lat - lat2) < 1.0e-12);
        assert!(ellps.reduced_latitude(0.0, fwd).abs() < 1.0e-10);
        assert!((ellps.reduced_latitude(FRAC_PI_2, fwd) - FRAC_PI_2).abs() < 1.0e-10);

        // Isometric latitude, 𝜓
        let angle = 45f64.to_radians();
        let isometric = 50.227465815385806f64.to_radians();
        assert!((ellps.isometric_latitude(angle, fwd) - isometric).abs() < 1e-15);
        assert!((ellps.isometric_latitude(isometric, inv) - angle).abs() < 1e-15);
    }
}
