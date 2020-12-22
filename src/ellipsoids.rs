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

    /// The squared eccentricity *e² = (a² - b²) / a²*.
    pub fn eccentricity_squared(&self) -> f64 {
        self.f*(2_f64 - self.f)
    }

    /// The eccentricity *e*
    pub fn eccentricity(&self) -> f64 {
        self.eccentricity_squared().sqrt()
    }

    /// The squared second eccentricity *e'² = (a² - b²) / b²*
    pub fn second_eccentricity_squared(&self) -> f64 {
        let b = self.semiminor_axis();
        let bb = b*b;
        (self.a.powi(2) - bb) / bb
    }

    /// The second eccentricity *e'*
    pub fn second_eccentricity(&self) -> f64 {
        self.second_eccentricity_squared().sqrt()
    }

    /// The semiminor axis, *b*
    pub fn semiminor_axis(&self) -> f64 {
        self.a * (1.0 - self.f)
    }

    /// The semimajor axis, *a*
    pub fn semimajor_axis(&self) -> f64 {
        self.a
    }

    /// The flattening, *f = (a - b)/a*
    pub fn flattening(&self) -> f64 {
        self.f
    }

    /// The second flattening, *f = (a - b) / b*
    pub fn second_flattening(&self) -> f64 {
        let b = self.semiminor_axis();
        (self.a - b) / b
    }

    /// The third flattening, *f = (a - b) / (a + b)*
    pub fn third_flattening(&self) -> f64 {
        let b = self.semiminor_axis();
        (self.a - b) / (self.a + b)
    }

    /// The radius of curvature in the prime vertical *N*
    pub fn prime_vertical_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.f == 0.0 {
            return self.a;
        }
        self.a / (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).sqrt()
    }

    /// The meridian curvature *M*
    pub fn meridian_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.f == 0.0 {
            return self.a;
        }
        let num = self.a * (1.0 - self.eccentricity_squared());
        let denom = (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).powf(1.5);
        num / denom
    }

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

    ///
    ///
    /// B.R. Bowring (1976): Transformation from spatial to geographical coordinates.
    ///     Survey Review 23(181), pp. 323–327
    /// B.R. Bowring (1985): The accuracy of geodetic latitude and height equations,
    ///     Survey Review, 28(218), pp.202-206, DOI: 10.1179/sre.1985.28.218.202
    ///

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
            // We have forced phi to one of the poles, so the height is Z - b
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
        println!("h = {}", h);
        CoordinateTuple::new(lam, phi, h, t)
    }

}


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

        let ellps = Ellipsoid::new(ellps.a, ellps.f);
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25722_21008_82711_24316);
        assert_eq!(ellps.name(), "");

        // Additional shape descriptors
        assert!((ellps.eccentricity() - 0.081819191).abs() < 1.0e-10);
        assert!((ellps.eccentricity_squared() - 0.00669_43800_22903_41574).abs() < 1.0e-10);

        // Additional size descriptors
        assert!((ellps.semiminor_axis() - 6_356_752.31414_0347).abs() < 1.0e-9);
        assert!((ellps.semimajor_axis() - 6_378_137.0).abs() < 1.0e-9);

        // The curvatures at the North Pole
        assert!((ellps.meridian_radius_of_curvature(90_f64.to_radians()) - 6_399_593.6259).abs() < 1.0e-4);
        assert!((ellps.prime_vertical_radius_of_curvature(90_f64.to_radians()) - 6_399_593.6259).abs() < 1.0e-4);

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

    }
}
