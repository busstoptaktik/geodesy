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
    /// [Karney (2010)](crate::Bibliography::Kar10) eq. (29), elaborated in
    /// [Deakin et al (2012)](crate::Bibliography::Dea12) eq. (41)
    #[must_use]
    pub fn rectifying_radius(&self) -> f64 {
        let n = self.third_flattening();
        self.semimajor_axis() / (1. + n)
            * crate::math::horner(n * n, &constants::MERIDIAN_ARC_COEFFICIENTS)
    }

    /// The rectifying radius, *A*, following [Bowring (1983)](crate::Bibliography::Bow83):
    /// An utterly elegant way of writing out the series truncated after the *n⁴* term.
    /// In general, however, prefer using the *n⁸* version implemented as
    /// [rectifying_radius](Ellipsoid::rectifying_radius), based on
    /// [Karney (2010)](crate::Bibliography::Kar10) eq. (29), as elaborated in
    /// [Deakin et al (2012)](crate::Bibliography::Dea12) eq. (41)
    #[must_use]
    pub fn rectifying_radius_bowring(&self) -> f64 {
        // A is the rectifying radius - truncated after the n⁴ term
        let n = self.third_flattening();
        let m = 1. + n * n / 8.;
        self.semimajor_axis() * m * m / (1. + n)
    }

    /// The Meridian Quadrant, *Qm*, is the distance from the equator to one of the poles.
    /// i.e. *π/2 · Qn · a*, where *Qn* is the
    /// [normalized meridian arc unit](Ellipsoid::normalized_meridian_arc_unit)
    #[must_use]
    pub fn meridian_quadrant(&self) -> f64 {
        self.a * FRAC_PI_2 * self.normalized_meridian_arc_unit()
    }

    /// The distance, *M*, along a meridian from the equator to the given
    /// latitude is a special case of a geodesic length.
    ///
    /// This implementation follows the remarkably simple algorithm
    /// by [Bowring (1983)](crate::Bibliography::Bow83).
    ///
    /// See also
    /// [Wikipedia: Transverse Mercator](https://en.wikipedia.org/wiki/Transverse_Mercator:_Bowring_series).
    ///
    /// [Deakin et al](crate::Bibliography::Dea12) provides a higher order (*n⁸*) derivation.
    #[must_use]
    #[allow(non_snake_case)] // make it possible to mimic math notation from the original paper
    #[allow(clippy::many_single_char_names)] // ditto
    pub fn meridian_latitude_to_distance(&self, latitude: f64) -> f64 {
        let n = self.third_flattening();

        // The rectifying radius - using a slightly more accurate series than in Bowring (1983)
        let A = self.rectifying_radius();

        let B = 9. * (1. - 3. * n * n / 8.0);
        let (s, c) = (2. * latitude).sin_cos();
        let x = 1. + 13. / 12. * n * c;
        let y = 0. + 13. / 12. * n * s;
        let r = y.hypot(x);
        let v = y.atan2(x);
        let theta = latitude - B * r.powf(-2. / 13.) * (2. * v / 13.).sin();
        A * theta
    }

    /// Compute the latitude of a point, given *M*, its distance from the equator,
    /// along its local meridian.
    ///
    /// This implementation follows the remarkably simple algorithm
    /// by [Bowring (1983)](crate::Bibliography::Bow83).
    ///
    /// See also
    /// [meridian_latitude_to_distance](Ellipsoid::meridian_latitude_to_distance)
    #[must_use]
    #[allow(non_snake_case)] // make it possible to mimic math notation from the original paper
    #[allow(clippy::many_single_char_names)] // ditto
    pub fn meridian_distance_to_latitude(&self, distance_from_equator: f64) -> f64 {
        let n = self.third_flattening();

        // Rectifying radius - using a slightly more accurate series than in Bowring (1983)
        let A = self.rectifying_radius();

        let theta = distance_from_equator / A;
        let (s, c) = (2. * theta).sin_cos();
        let x = 1. - 155. / 84. * n * c;
        let y = 0. + 155. / 84. * n * s;
        let r = y.hypot(x);
        let v = y.atan2(x);

        let C = 1. - 9. * n * n / 16.;
        theta + 63. / 4. * C * r.powf(8. / 155.) * (8. / 155. * v).sin()
    }
}

// ----- Tests ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meridional_distance() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;

        // The rectifying radius, A, is an important component for conputing
        // the meridian distance, so we start by testing A.
        assert!((ellps.rectifying_radius() - 6_367_449.145_771_043).abs() < 1e-9);

        // Compare with Bowring's version (truncated after the n⁴ term).
        // Note that for GRS80, the difference from 4th order and 8th order
        // is at nanometer scale, so Bowring's elegant version is perfectly
        // fine for all earth-like applications
        assert!((ellps.rectifying_radius() - ellps.rectifying_radius_bowring()).abs() < 1e-9);

        // --------------------------------------------------------------------
        // Meridional distance, M
        // --------------------------------------------------------------------

        // Internal consistency: Check that at 90°, the meridional distance
        // is identical to the meridian quadrant.
        assert!(
            (ellps.meridian_latitude_to_distance(FRAC_PI_2) - ellps.meridian_quadrant()).abs()
                < 1e-15
        );
        assert!(
            (ellps.meridian_distance_to_latitude(ellps.meridian_quadrant()) - FRAC_PI_2).abs()
                < 1e-15
        );

        // Internal consistency: Roundtrip replication accuracy.
        for i in 0..10 {
            // latitude -> distance -> latitude
            let b = (10. * i as f64).to_radians();
            assert!(
                (ellps.meridian_distance_to_latitude(ellps.meridian_latitude_to_distance(b)) - b)
                    .abs()
                    < 5e-11
            );

            // distance -> latitude -> distance;
            let d = 1_000_000. * i as f64;
            assert!(
                (ellps.meridian_latitude_to_distance(ellps.meridian_distance_to_latitude(d)) - d)
                    .abs()
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
            assert!((ellps.meridian_latitude_to_distance(angle) - s[i]).abs() < 6e-6);
            assert!((ellps.meridian_distance_to_latitude(s[i]) - angle).abs() < 6e-11);
        }

        // Since we suspect the deviation might be worst at 45°, we check that as well
        let angle = 45f64.to_radians();
        let length = 4984944.377857987;
        assert!((ellps.meridian_latitude_to_distance(angle) - length).abs() < 4e-6);
        assert!((ellps.meridian_distance_to_latitude(length) - angle).abs() < 4e-6);
        Ok(())
    }
}
