//! Ellipsoids

use crate::CoordinateTuple;
use std::f64::consts::FRAC_PI_2;

/// Representation of an ellipsoid.
#[derive(Clone, Copy, Debug)]
pub struct Ellipsoid {
    name: &'static str,
    a: f64,
    f: f64,
}


// A hashmap indexed by the ellipsoid name would be better, but Rust cannot
// statically initialize a hashmap, so we put the name into the struct and
// use a static array instead.
static ELLIPSOIDS: [Ellipsoid; 5] =  [
    Ellipsoid {name: "GRS80"  ,  a: 6378137.0,   f: 1./298.25722_21008_82711_24316},
    Ellipsoid {name: "intl"   ,  a: 6378388.0,   f: 1./297.},
    Ellipsoid {name: "Helmert",  a: 6378200.0,   f: 1./298.3},
    Ellipsoid {name: "clrk66" ,  a: 6378206.4,   f: 1./294.9786982},
    Ellipsoid {name: "clrk80" ,  a: 6378249.145, f: 1./293.465}
];


impl Ellipsoid {
    /// User defined ellipsoid
    pub fn new(semimajor_axis: f64, flattening: f64) -> Ellipsoid {
        Ellipsoid{
            name: "",
            a: semimajor_axis,
            f: flattening,
        }
    }

    /// Predefined ellipsoid
    pub fn named(name: &str) -> Ellipsoid {
        for e in ELLIPSOIDS.iter() {
            if e.name == name {
                return *e;
            }
        }
        ELLIPSOIDS[0]
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    // ----- Eccentricities --------------------------------------------------------

    /// The squared eccentricity *e² = (a² - b²) / a²*.
    pub fn eccentricity_squared(&self) -> f64 {
        self.f*(2_f64 - self.f)
    }


    /// The eccentricity *e*
    pub fn eccentricity(&self) -> f64 {
        self.eccentricity_squared().sqrt()
    }


    /// The squared second eccentricity *e'² = (a² - b²) / b² = e² / (1 - e²)*
    pub fn second_eccentricity_squared(&self) -> f64 {
        let es = self.eccentricity_squared();
        es / (1.0 - es)
    }


    /// The second eccentricity *e'*
    pub fn second_eccentricity(&self) -> f64 {
        self.second_eccentricity_squared().sqrt()
    }


    // ----- Axes ------------------------------------------------------------------


    /// The semiminor axis, *b*
    pub fn semiminor_axis(&self) -> f64 {
        self.a * (1.0 - self.f)
    }


    /// The semimajor axis, *a*
    pub fn semimajor_axis(&self) -> f64 {
        self.a
    }


    // ----- Flatteningss ----------------------------------------------------------


    /// The flattening, *f = (a - b)/a*
    pub fn flattening(&self) -> f64 {
        self.f
    }


    /// The second flattening, *f = (a - b) / b*
    pub fn second_flattening(&self) -> f64 {
        let b = self.semiminor_axis();
        (self.a - b) / b
    }


    /// The third flattening, *n = (a - b) / (a + b) = f / (2 - f)*
    pub fn third_flattening(&self) -> f64 {
        self.f / (2.0 - self.f)
    }


    // ----- Curvatures ------------------------------------------------------------


    /// The radius of curvature in the prime vertical, *N*
    pub fn prime_vertical_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.f == 0.0 {
            return self.a;
        }
        self.a / (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).sqrt()
    }


    /// The meridian radius of curvature, *M*
    pub fn meridian_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.f == 0.0 {
            return self.a;
        }
        let num = self.a * (1.0 - self.eccentricity_squared());
        let denom = (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).powf(1.5);
        num / denom
    }


    /// The polar radius of curvature, *c*
    pub fn polar_radius_of_curvature(&self) -> f64 {
        self.a * self.a / self.semiminor_axis()
    }


    // ----- Meridian geometry -----------------------------------------------------


    /// The Normalized Meridian Arc Unit, *Qn*, is the mean length of one radian
    ///  of the meridian. "Normalized", because we measure it in units of the
    /// semimajor axis, *a*.
    ///
    /// König und Weise p.50 (96), p.19 (38b), p.5 (2), here using the extended
    /// version from [Deakin et al 2012](crate::Bibliography::Dea12) eq. (41)
    pub fn normalized_meridian_arc_unit(&self) -> f64 {
        let n = self.third_flattening();
        let nn = n*n;
        (1. + nn*(1./4. + nn*(1./64. + nn*(1./256. + 25.*nn/16384.)))) / (1.0 + n)
    }


    /// The rectifying radius, *A*, is the radius of a sphere of the same circumference
    /// as the length of a full meridian on the ellipsoid.
    ///
    /// Closely related to the [normalized meridian arc unit](Ellipsoid::normalized_meridian_arc_unit).
    ///
    /// [Deakin et al 2012](crate::Bibliography::Dea12) eq. (41)
    pub fn rectifying_radius(&self) -> f64 {
        let n = self.third_flattening();
        let nn = n*n;
        let d = (1. + nn*(1./4. + nn*(1./64. + nn*(1./256. + 25.*nn/16384.)))) / (1.0 + n);

        self.a * d / (1.+ n)
    }


    /// The Meridian Quadrant, *Qm*, is the distance from the equator to one of the poles.
    /// i.e. *π/2 ·	Qn · a*, where *Qn* is the
    /// [normalized meridian arc unit](Ellipsoid::normalized_meridian_arc_unit)
    pub fn meridian_quadrant(&self) -> f64 {
        self.a * FRAC_PI_2 * self.normalized_meridian_arc_unit()
    }


    /// The distance, *M*, along a meridian from the equator to the given
    /// latitude.
    ///
    /// Following the remarkably simple algorithm by Bowring (1983).
    ///
    /// B. R. Bowring (1983), New equations for meridional distance.
    /// [Bull. Geodesique 57, 374–381](https://doi.org/10.1007/BF02520940).
    ///
    ///
    /// See also
    /// [Wikipedia: Transverse Mercator](https://en.wikipedia.org/wiki/Transverse_Mercator:_Bowring_series).
    ///
    /// [Deakin et al](crate::Bibliography::Dea12) provides a higher order (*n⁸*) derivation.
    ///
    #[allow(non_snake_case)]
    pub fn meridional_distance(&self, latitude: f64, forward: bool) -> f64 {
        let n = self.third_flattening();
        let m = 1. + n * n / 8.;

        // Rectifying radius - truncated after the n⁴ term
        let A = self.a * m * m / (1. + n);

        if forward {
            let B = 9. * (1. - 3. * n * n / 8.0);
            let x = 1. + 13./12. * n * (2.* latitude).cos();
            let y = 0. + 13./12. * n * (2.* latitude).sin();
            let r = y.hypot(x);
            let v = y.atan2(x);
            let theta = latitude - B * r.powf(-2./13.) * (2. * v / 13.).sin();
            return A * theta;
        }

        let C = 1. - 9. * n * n / 16.;
        let theta = latitude / A;
        let x = 1. - 155. / 84. * n * (2. * theta).cos();
        let y = 0. + 155. / 84. * n * (2. * theta).sin();
        let r = y.hypot(x);
        let v = y.atan2(x);

        theta + 63./4. * C * r.powf(8./155.) * (8./155. * v).sin()
    }


    // Charles F.F. Karney: Algorithms for Geodesics. https://arxiv.org/pdf/1109.4448.pdf
    // Rust implementation: https://docs.rs/crate/geographiclib-rs/0.2.0


    // ----- Latitudes -------------------------------------------------------------


    /// Geographic latitude to geocentric latitude
    /// (or vice versa if `forward` is `false`).
    pub fn geocentric_latitude(&self, latitude: f64, forward: bool) -> f64 {
        if forward {
            return ((1.0 - self.f * (2.0 - self.f)) * latitude.tan()).atan();
        }
        (latitude.tan() / (1.0 - self.eccentricity_squared())).atan()
    }


    /// Geographic latitude to reduced latitude
    /// (or vice versa if `forward` is  `false`).
    pub fn reduced_latitude(&self, latitude: f64, forward: bool) -> f64 {
        if forward {
            return ((1.0 - self.f) * latitude.tan()).atan();
        }
        (latitude.tan() / (1.0 - self.f)).atan()
    }

    /// Isometric latitude, 𝜓
    pub fn isometric_latitude(&self, latitude: f64) -> f64 {
        let e = self.eccentricity();
        latitude.tan().asinh() - (e * latitude.sin()).atanh() * e
    }


    // ----- Cartesian <--> Geographic conversion ----------------------------------


    /// Geographic to cartesian conversion.
    ///
    /// Follows the the derivation given by
    /// Bowring ([1976](crate::Bibliography::Bow76) and
    /// [1985](crate::Bibliography::Bow85))
    #[allow(non_snake_case)]
    pub fn cartesian(&self, geographic: &CoordinateTuple) -> CoordinateTuple {
        let lam = geographic.first();
        let phi = geographic.second();
        let h = geographic.third();
        let t = geographic.fourth();

        let N = self.prime_vertical_radius_of_curvature(phi);
        let cosphi = phi.cos();
        let sinphi = phi.sin();
        let coslam = lam.cos();
        let sinlam = lam.sin();

        let X = (N + h) * cosphi * coslam;
        let Y = (N + h) * cosphi * sinlam;
        let Z = (N * (1.0 - self.eccentricity_squared()) + h) * sinphi;

        CoordinateTuple::new(X, Y, Z, t)
    }


    /// Cartesian to geogaphic conversion.
    ///
    /// Follows the the derivation given by
    /// Bowring ([1976](crate::Bibliography::Bow76) and
    /// [1985](crate::Bibliography::Bow85))
    #[allow(non_snake_case)]
    pub fn geographic(&self, cartesian: &CoordinateTuple) -> CoordinateTuple {
        let X = cartesian.first();
        let Y = cartesian.second();
        let Z = cartesian.third();
        let t = cartesian.fourth();

        // We need a few additional ellipsoidal parameters
        let b = self.semiminor_axis();
        let eps = self.second_eccentricity_squared();
        let es = self.eccentricity_squared();

        // The longitude is straightforward
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
            return CoordinateTuple::new(lam, phi, h, t);
        }

        // HM eq. (5-36) and (5-37), with some added numerical efficiency due to
        // Even Rouault, who replaced 3 trigs with a hypot and two divisions, cf.
        // https://github.com/OSGeo/PROJ/commit/78c1df51e0621a4e0b2314f3af9478627e018db3
        let theta_num = Z * self.a;
        let theta_denom = p * b;
        let length = theta_num.hypot(theta_denom);
        let c = theta_denom / length; // i.e. cos(theta)
        let s = theta_num / length;   // i.e. sin(theta)

        let phi_num = Z + eps * b * s.powi(3);
        let phi_denom = p - es * self.a * c.powi(3);
        let phi = phi_num.atan2(phi_denom);
        let lenphi = phi_num.hypot(phi_denom);
        let sinphi = phi_num/lenphi;
        let cosphi = phi_denom/lenphi;

        // We already have sinphi and es, so we can compute the radius
        // of curvature faster by inlining, rather than calling the
        // prime_vertical_radius_of_curvature() method.
        let N = self.a / (1.0 - sinphi.powi(2) * es).sqrt();

        // Bowring (1985), as quoted by Burtch (2006), suggests this expression
        // as more accurate than the commonly used h = p / cosphi - N;
        let h = p*cosphi + Z * sinphi - self.a*self.a/N;

        CoordinateTuple::new(lam, phi, h, t)
    }

}


// ----- Tests ---------------------------------------------------------------------


#[cfg(test)]
mod tests {
    #[test]
    fn test_ellipsoid() {
        use std::f64::consts::FRAC_PI_2;
        use super::Ellipsoid;
        use super::CoordinateTuple;

        // Constructors
        let ellps = Ellipsoid::named("intl");
        assert_eq!(ellps.f, 1. / 297.);
        assert_eq!(ellps.name(), "intl");

        let ellps = Ellipsoid::named("GRS80");
        assert_eq!(ellps.a, 6378137.0);
        assert_eq!(ellps.f, 1. / 298.25722_21008_82711_24316);
        assert_eq!(ellps.name, "GRS80");

        assert!((ellps.normalized_meridian_arc_unit() - 0.9983242984230415).abs() < 1e-13);
        assert!((4.0 * ellps.meridian_quadrant() - 40007862.9169218).abs() < 1e-7);

        let ellps = Ellipsoid::new(ellps.a, ellps.f);
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25722_21008_82711_24316);
        assert_eq!(ellps.name(), "");

        // Additional shape descriptors
        assert!((ellps.eccentricity() - 0.081819191).abs() < 1.0e-10);
        assert!((ellps.eccentricity_squared() - 0.00669_43800_22903_41574).abs() < 1.0e-10);

        // Additional size descriptors
        assert!((ellps.semiminor_axis() - 6_356_752.31414_0347).abs() < 1e-9);
        assert!((ellps.semimajor_axis() - 6_378_137.0).abs() < 1e-9);

        // The curvatures at the North Pole
        assert!((ellps.meridian_radius_of_curvature(90_f64.to_radians()) - 6_399_593.6259).abs() < 1e-4);
        assert!((ellps.prime_vertical_radius_of_curvature(90_f64.to_radians()) - 6_399_593.6259).abs() < 1e-4);
        assert!((ellps.prime_vertical_radius_of_curvature(90_f64.to_radians()) - ellps.meridian_radius_of_curvature(90_f64.to_radians())).abs() < 1e-5);
        assert!((ellps.polar_radius_of_curvature() - ellps.meridian_radius_of_curvature(90_f64.to_radians())).abs() < 1e-6);

        // The curvatures at the Equator
        assert!((ellps.meridian_radius_of_curvature(0.0) - 6_335_439.3271).abs() < 1.0e-4);
        assert!((ellps.prime_vertical_radius_of_curvature(0.0) - ellps.semimajor_axis()).abs() < 1.0e-4);


        // Roundtrip geographic <-> cartesian
        let geo = CoordinateTuple::new(12_f64.to_radians(), 55_f64.to_radians(), 100.0, 0.);
        let cart = ellps.cartesian(&geo);
        let geo2 = ellps.geographic(&cart);
        assert!((geo.0-geo2.0).abs() < 1.0e-12);
        assert!((geo.1-geo2.1).abs() < 1.0e-12);
        assert!((geo.2-geo2.2).abs() < 1.0e-9);

        // Roundtrip geocentric latitude
        let lat = 55_f64.to_radians();
        let lat2 = ellps.geocentric_latitude(ellps.geocentric_latitude(lat, true), false);
        assert!((lat-lat2) < 1.0e-12);
        assert!(ellps.geocentric_latitude(0.0, true).abs() < 1.0e-10);
        assert!((ellps.geocentric_latitude(FRAC_PI_2, true) - FRAC_PI_2).abs() < 1.0e-10);

        // Roundtrip reduced latitude
        let lat = 55_f64.to_radians();
        let lat2 = ellps.reduced_latitude(ellps.reduced_latitude(lat, true), false);
        assert!((lat-lat2) < 1.0e-12);
        assert!(ellps.reduced_latitude(0.0, true).abs() < 1.0e-10);
        assert!((ellps.reduced_latitude(FRAC_PI_2, true) - FRAC_PI_2).abs() < 1.0e-10);

        // Isometric latitude, 𝜓
        assert!((ellps.isometric_latitude(45f64.to_radians()) - 50.227465815385806f64.to_radians()).abs() < 1e-15);

        // Rectifying radius, A
        assert!((ellps.rectifying_radius() - 6356774.720017125).abs() < 1e-9);

        // --------------------------------------------------------------------
        // Meridional distance, M
        // --------------------------------------------------------------------

        // Internal consistency: Check that at 90°, the meridional distance
        // is identical to the meridian quadrant.
        assert!((ellps.meridional_distance(FRAC_PI_2, true) - ellps.meridian_quadrant()).abs() < 1e-15);
        assert!((ellps.meridional_distance(ellps.meridian_quadrant(), false) - FRAC_PI_2).abs() < 1e-15);

        // Internal consistency: Roundtrip replication accuracy.
        for i in 0..10 {
            // latitude -> distance -> latitude
            let b = (10. * i as f64).to_radians();
            assert!((
                ellps.meridional_distance(
                    ellps.meridional_distance(b, true), false
                ) - b
            ).abs() < 5e-11);

            // distance -> latitude -> distance;
            let d = 1_000_000. * i as f64;
            assert!((
                ellps.meridional_distance(
                    ellps.meridional_distance(d, false), true
                ) - d
            ).abs() < 6e-5);
        }

        // Compare with Karney's algorithm for geodesics.
        // We expect deviations to be less than 6 𝜇m.

        // Meridional distances for angles 0, 10, 20, 30 ... 90, obtained from Charles Karney's
        // online geodesic solver, https://geographiclib.sourceforge.io/cgi-bin/GeodSolve
        let s = [
            0000000.000000000, 1105854.833198446, 2212366.254102976, 3320113.397845014, 4429529.030236580,
            5540847.041560960, 6654072.819367435, 7768980.727655508, 8885139.871836751, 10001965.729230457
        ];
        for i in 0..s.len() {
            assert!((ellps.meridional_distance((10.0*i as f64).to_radians(), true) - s[i]).abs() < 6e-6);
            assert!((ellps.meridional_distance(s[i], false) - (10.0*i as f64).to_radians()).abs() < 6e-11);
        }

        // Since we suspect the deviation might be worst at 45°, we check that as well
        assert!((ellps.meridional_distance(45f64.to_radians(), true) - 4984944.377857987).abs() < 4e-6);
        assert!((ellps.meridional_distance(4984944.377857987, false) - 45f64.to_radians()).abs() < 4e-6);

    }
}
