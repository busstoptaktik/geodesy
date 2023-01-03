use super::*;
use std::f64::consts::FRAC_PI_2;

// ----- Meridian geometry -----------------------------------------------------
impl Ellipsoid {
    /// The Normalized Meridian Arc Unit, *Qn*, is the mean length of one radian
    ///  of the meridian. "Normalized", because we measure it in units of the
    /// semimajor axis, *a*.
    ///
    /// König und Weise p.50 (96), p.19 (38b), p.5 (2), here using the extended
    /// version from [Karney 2010](crate::Bibliography::Kar10) eq. (29)
    #[must_use]
    pub fn normalized_meridian_arc_unit(&self) -> f64 {
        let n = self.third_flattening();
        crate::math::horner(n * n, &constants::MERIDIAN_ARC_COEFFICIENTS) / (1. + n)
    }

    /// The rectifying radius, *A*, is the radius of a sphere of the same circumference
    /// as the length of a full meridian on the ellipsoid.
    ///
    /// Closely related to the [normalized meridian arc unit](Ellipsoid::normalized_meridian_arc_unit).
    ///
    /// [Karney 2010](crate::Bibliography::Kar10) eq. (29), elaborated in
    /// [Deakin et al 2012](crate::Bibliography::Dea12) eq. (41)
    #[must_use]
    pub fn rectifying_radius(&self) -> f64 {
        let n = self.third_flattening();
        self.a * crate::math::horner(n * n, &constants::MERIDIAN_ARC_COEFFICIENTS)
            / ((1. + n) * (1. + n))
    }

    /// The Meridian Quadrant, *Qm*, is the distance from the equator to one of the poles.
    /// i.e. *π/2 · Qn · a*, where *Qn* is the
    /// [normalized meridian arc unit](Ellipsoid::normalized_meridian_arc_unit)
    #[must_use]
    pub fn meridian_quadrant(&self) -> f64 {
        self.a * FRAC_PI_2 * self.normalized_meridian_arc_unit()
    }
}

// ----- Tests ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Direction::Fwd;
    use crate::Direction::Inv;
    use crate::Error;

    #[test]
    fn meridional_distance() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;

        // Rectifying radius, A
        assert!((ellps.rectifying_radius() - 6356774.720017125).abs() < 1e-9);

        // --------------------------------------------------------------------
        // Meridional distance, M
        // --------------------------------------------------------------------

        // Internal consistency: Check that at 90°, the meridional distance
        // is identical to the meridian quadrant.
        assert!(
            (ellps.meridional_distance(FRAC_PI_2, Fwd) - ellps.meridian_quadrant()).abs() < 1e-15
        );
        assert!(
            (ellps.meridional_distance(ellps.meridian_quadrant(), Inv) - FRAC_PI_2).abs() < 1e-15
        );

        // Internal consistency: Roundtrip replication accuracy.
        for i in 0..10 {
            // latitude -> distance -> latitude
            let b = (10. * i as f64).to_radians();
            assert!(
                (ellps.meridional_distance(ellps.meridional_distance(b, Fwd), Inv) - b).abs()
                    < 5e-11
            );

            // distance -> latitude -> distance;
            let d = 1_000_000. * i as f64;
            assert!(
                (ellps.meridional_distance(ellps.meridional_distance(d, Inv), Fwd) - d).abs()
                    < 6e-5
            );
        }

        // Compare with Karney's algorithm for geodesics.
        // We expect deviations to be less than 6 𝜇m.

        // Meridional distances for angles 0, 10, 20, 30 ... 90, obtained from Charles Karney's
        // online geodesic solver, https://geographiclib.sourceforge.io/cgi-bin/GeodSolve
        let s = [
            0_000_000.000000000,
            1_105_854.833198446,
            2_212_366.254102976,
            3_320_113.397845014,
            4_429_529.030236580,
            5_540_847.041560960,
            6_654_072.819367435,
            7_768_980.727655508,
            8_885_139.871836751,
            10_001_965.729230457,
        ];

        for i in 0..s.len() {
            let angle = (10.0 * i as f64).to_radians();
            assert!((ellps.meridional_distance(angle, Fwd) - s[i]).abs() < 6e-6);
            assert!((ellps.meridional_distance(s[i], Inv) - angle).abs() < 6e-11);
        }

        // Since we suspect the deviation might be worst at 45°, we check that as well
        let angle = 45f64.to_radians();
        let length = 4984944.377857987;
        assert!((ellps.meridional_distance(angle, Fwd) - length).abs() < 4e-6);
        assert!((ellps.meridional_distance(length, Inv) - angle).abs() < 4e-6);
        Ok(())
    }
}
