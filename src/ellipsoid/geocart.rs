use super::*;

use std::f64::consts::FRAC_PI_2;

/// Geographic <--> Cartesian conversion
pub trait GeoCart: EllipsoidBase {
    /// Geographic to cartesian conversion.
    ///
    /// Follows the the derivation given by
    /// Bowring ([1976](crate::Bibliography::Bow76) and
    /// [1985](crate::Bibliography::Bow85))
    #[must_use]
    #[allow(non_snake_case)] // make it possible to mimic math notation from original paper
    #[allow(clippy::many_single_char_names)] // ditto
    fn cartesian<C: CoordinateTuple>(&self, geographic: &C) -> Coor4D {
        let (lam, phi, h, t) = geographic.xyzt();

        let N = self.prime_vertical_radius_of_curvature(phi);
        let cosphi = phi.cos();
        let sinphi = phi.sin();
        let coslam = lam.cos();
        let sinlam = lam.sin();

        let X = (N + h) * cosphi * coslam;
        let Y = (N + h) * cosphi * sinlam;
        let Z = (N * (1.0 - self.eccentricity_squared()) + h) * sinphi;

        Coor4D::raw(X, Y, Z, t)
    }

    /// Cartesian to geographic conversion.
    ///
    /// Follows the the derivation given by
    /// Bowring ([1976](crate::Bibliography::Bow76) and
    /// [1985](crate::Bibliography::Bow85))
    #[must_use]
    #[allow(non_snake_case)] // make it possible to mimic math notation from original paper
    #[allow(clippy::many_single_char_names)] // ditto
    fn geographic<C: CoordinateTuple>(&self, cartesian: &C) -> Coor4D {
        let (X, Y, Z, t) = cartesian.xyzt();

        // We need a few additional ellipsoidal parameters
        let b = self.semiminor_axis();
        let eps = self.second_eccentricity_squared();
        let es = self.eccentricity_squared();

        // The longitude is straightforward: Plain geometry in the equatoreal plane
        let lam = Y.atan2(X);

        // The perpendicular distance from the point coordinate to the Z-axis
        // (HM eq. 5-28)
        let p = X.hypot(Y);

        // For p < 1 picometer, we simplify things to avoid numerical havoc.
        if p < 1.0e-12 {
            // The sign of Z determines the hemisphere
            let phi = FRAC_PI_2.copysign(Z);
            // We have forced phi to one of the poles, so the height is |Z| - b
            let h = Z.abs() - self.semiminor_axis();
            return Coor4D::raw(lam, phi, h, t);
        }

        // HM eq. (5-36) and (5-37), with some added numerical efficiency due to
        // Even Rouault, who replaced 3 trigs with a hypot and two divisions, cf.
        // https://github.com/OSGeo/PROJ/commit/78c1df51e0621a4e0b2314f3af9478627e018db3
        // let theta_num = Z * self.a;
        // let theta_denom = p * b;
        // let length = theta_num.hypot(theta_denom);
        // let c = theta_denom / length; // i.e. cos(theta)
        // let s = theta_num / length; // i.e. sin(theta)

        // Fukushima (1999), Appendix B: Equivalent to Even Rouault's, implementation,
        // but not as clear - although a bit faster due to the substitution of sqrt
        // for hypot.
        let a = self.semimajor_axis();
        let T = (Z * a) / (p * b);
        let c = 1.0 / (1.0 + T * T).sqrt();
        let s = c * T;

        let phi_num = Z + eps * b * s.powi(3);
        let phi_denom = p - es * a * c.powi(3);
        let phi = phi_num.atan2(phi_denom);

        let lenphi = phi_num.hypot(phi_denom);
        let sinphi = phi_num / lenphi;
        let cosphi = phi_denom / lenphi;

        let a = self.semimajor_axis();

        // We already have sinphi and es, so we can compute the radius
        // of curvature faster by inlining, rather than calling the
        // prime_vertical_radius_of_curvature() method.
        let N = a / (1.0 - sinphi.powi(2) * es).sqrt();

        // Bowring (1985), as quoted by Burtch (2006), suggests this expression
        // as more accurate than the commonly used h = p / cosphi - N;
        let h = p * cosphi + Z * sinphi - a * a / N;

        Coor4D::raw(lam, phi, h, t)
    }
}

// ----- Tests ---------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn geo_to_cart() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;
        // Roundtrip geographic <-> cartesian
        let geo = Coor4D::geo(55., 12., 100., 0.);
        let cart = ellps.cartesian(&geo);
        let geo2 = ellps.geographic(&cart);
        assert!((geo[0] - geo2[0]).abs() < 1.0e-12);
        assert!((geo[1] - geo2[1]).abs() < 1.0e-12);
        assert!((geo[2] - geo2[2]).abs() < 1.0e-9);
        Ok(())
    }
}
