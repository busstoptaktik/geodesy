//! Ellipsoids
use std::f64::consts::FRAC_PI_2;

use crate::fwd;
use crate::operand::*;

/// Representation of a (potentially triaxial) ellipsoid.
#[derive(Clone, Copy, Debug)]
pub struct Ellipsoid {
    a: f64,
    ay: f64,
    f: f64,
}

/// Overrepresentation of an ellipsoid: Precompute additional useful constants.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
pub struct FatEllipsoid {
    // Axes
    pub a: f64, // Semimajor axis
    ay: f64,    // Equatoreal semiminor axis
    b: f64,     // Semiminor axis
    ra: f64,    // 1 / a
    ray: f64,   // 1 / ay
    rb: f64,    // 1 / b

    // Flattenings
    f: f64,
    fe: f64, // Equatoreal flattening (0 for biaxial)
    g: f64,  // Second flattening
    n: f64,  // Third flattening
    rf: f64, // 1 / f
    rn: f64, // 1 / n
    ar: f64, // Aspect ratio: 1 - f = b / a

    // Eccentricities
    e: f64,       // Eccentricity
    ee: f64,      // Equatoreal eccentricity (0 for biaxial)
    es: f64,      // Eccentricity squared
    ees: f64,     // Equatoreal eccentricity squared
    e4: f64,      // Eccentricity squared, squared
    ep: f64,      // Second eccentricity
    eps: f64,     // Second eccentricity squared
    E: f64,       // Linear eccentricity
    one_es: f64,  // 1 - es
    rone_es: f64, // 1 / one_es
}

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct CartographicFoundations {
    lat_0: f64,
    lat_1: f64,
    lat_2: f64,
    lat_3: f64,
    lat_4: f64,

    lon_0: f64,
    lon_1: f64,
    lon_2: f64,
    lon_3: f64,
    lon_4: f64,

    N0: f64,
    N1: f64,
    N2: f64,
    N3: f64,
    N4: f64,

    E0: f64,
    E1: f64,
    E2: f64,
    E3: f64,
    E4: f64,

    k_0: f64,
}

/// GRS80 is the default ellipsoid.
impl Default for Ellipsoid {
    fn default() -> Ellipsoid {
        Ellipsoid::new(6_378_137.0, 1. / 298.257_222_100_882_7)
    }
}

pub trait Axes {
    fn a(&self) -> f64;
    fn ay(&self) -> f64;
    fn f(&self) -> f64;

    /// The semimajor axis, *a*
    #[must_use]
    fn semimajor_axis(&self) -> f64 {
        self.a()
    }

    /// The semimedian axis, *ay*
    #[must_use]
    fn semimedian_axis(&self) -> f64 {
        self.ay()
    }

    /// The semiminor axis, *b*
    #[must_use]
    fn semiminor_axis(&self) -> f64 {
        self.a() * (1.0 - self.f())
    }
}

impl Axes for Ellipsoid {
    fn a(&self) -> f64 {
        self.a
    }
    fn ay(&self) -> f64 {
        self.ay
    }
    fn f(&self) -> f64 {
        self.f
    }
}

impl Axes for FatEllipsoid {
    fn a(&self) -> f64 {
        self.a
    }
    fn ay(&self) -> f64 {
        self.ay
    }
    fn f(&self) -> f64 {
        self.f
    }
}

pub trait Axen<T> {
    fn a(&self) -> f64;
    fn ay(&self) -> f64;
    fn f(&self) -> f64;

    /// The semimajor axis, *a*
    #[must_use]
    fn semimajor_axis(&self) -> f64 {
        self.a()
    }

    /// The semimedian axis, *ay*
    #[must_use]
    fn semimedian_axis(&self) -> f64 {
        self.ay()
    }

    /// The semiminor axis, *b*
    #[must_use]
    fn semiminor_axis(&self) -> f64 {
        self.a() * (1.0 - self.f())
    }
}

impl Ellipsoid {
    /// User defined ellipsoid
    #[must_use]
    pub fn new(semimajor_axis: f64, flattening: f64) -> Ellipsoid {
        Ellipsoid {
            a: semimajor_axis,
            ay: semimajor_axis,
            f: flattening,
        }
    }

    pub fn triaxial(semimajor_x_axis: f64, semimajor_y_axis: f64, flattening: f64) -> Ellipsoid {
        Ellipsoid {
            a: semimajor_x_axis,
            ay: semimajor_y_axis,
            f: flattening,
        }
    }

    /// Predefined ellipsoid
    #[must_use]
    pub fn named(name: &str) -> Ellipsoid {
        match name {
            "GRS80" => Ellipsoid::new(6_378_137.0, 1. / 298.257_222_100_882_7),
            "intl" => Ellipsoid::new(6_378_388.0, 1. / 297.0),
            "Helmert" => Ellipsoid::new(6_378_200.0, 1. / 298.3),
            "clrk66" => Ellipsoid::new(6_378_206.4, 1. / 294.978_698_2),
            "clrk80" => Ellipsoid::new(6_378_249.145, 1. / 293.465),
            _ => Ellipsoid::new(6_378_137.0, 1. / 298.257_222_100_882_7),
        }
    }

    // ----- Eccentricities --------------------------------------------------------

    /// The linear eccentricity *E* = sqrt(a² - b²). Negative if b > a.
    #[must_use]
    pub fn linear_eccentricity(&self) -> f64 {
        let b = self.semiminor_axis();
        let le = self.a * self.a - b * b;
        if self.a > b {
            return le.sqrt();
        }
        -(-le).sqrt()
    }

    /// The squared eccentricity *e² = (a² - b²) / a²*.
    #[must_use]
    pub fn eccentricity_squared(&self) -> f64 {
        self.f * (2_f64 - self.f)
    }

    /// The eccentricity *e*
    #[must_use]
    pub fn eccentricity(&self) -> f64 {
        self.eccentricity_squared().sqrt()
    }

    /// The squared second eccentricity *e'² = (a² - b²) / b² = e² / (1 - e²)*
    #[must_use]
    pub fn second_eccentricity_squared(&self) -> f64 {
        let es = self.eccentricity_squared();
        es / (1.0 - es)
    }

    /// The second eccentricity *e'*
    #[must_use]
    pub fn second_eccentricity(&self) -> f64 {
        self.second_eccentricity_squared().sqrt()
    }

    /// The semimajor axis, *a*
    #[must_use]
    pub fn semimajor_axis(&self) -> f64 {
        self.a
    }

    /// The semimedian axis, *ay*
    #[must_use]
    pub fn semimedian_axis(&self) -> f64 {
        self.ay
    }

    /// The semiminor axis, *b*
    #[must_use]
    pub fn semiminor_axis(&self) -> f64 {
        self.a * (1.0 - self.f)
    }

    // ----- Flattenings -----------------------------------------------------------

    /// The flattening, *f = (a - b)/a*
    #[must_use]
    pub fn flattening(&self) -> f64 {
        self.f
    }

    /// The second flattening, *f = (a - b) / b*
    #[must_use]
    pub fn second_flattening(&self) -> f64 {
        let b = self.semiminor_axis();
        (self.a - b) / b
    }

    /// The third flattening, *n = (a - b) / (a + b) = f / (2 - f)*
    #[must_use]
    pub fn third_flattening(&self) -> f64 {
        self.f / (2.0 - self.f)
    }

    /// The aspect ratio, *b / a  =  1 - f  =  sqrt(1 - e²)*
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        1.0 - self.f
    }

    // ----- Curvatures ------------------------------------------------------------

    /// The radius of curvature in the prime vertical, *N*
    #[must_use]
    pub fn prime_vertical_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.f == 0.0 {
            return self.a;
        }
        self.a / (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).sqrt()
    }

    /// The meridian radius of curvature, *M*
    #[must_use]
    pub fn meridian_radius_of_curvature(&self, latitude: f64) -> f64 {
        if self.f == 0.0 {
            return self.a;
        }
        let num = self.a * (1.0 - self.eccentricity_squared());
        let denom = (1.0 - latitude.sin().powi(2) * self.eccentricity_squared()).powf(1.5);
        num / denom
    }

    /// The polar radius of curvature, *c*
    #[must_use]
    pub fn polar_radius_of_curvature(&self) -> f64 {
        self.a * self.a / self.semiminor_axis()
    }

    // ----- Latitudes -------------------------------------------------------------

    /// Geographic latitude to geocentric latitude
    /// (or vice versa if `forward` is `false`).
    #[must_use]
    pub fn geocentric_latitude(&self, latitude: f64, forward: bool) -> f64 {
        if forward {
            return ((1.0 - self.f * (2.0 - self.f)) * latitude.tan()).atan();
        }
        (latitude.tan() / (1.0 - self.eccentricity_squared())).atan()
    }

    /// Geographic latitude to reduced latitude
    /// (or vice versa if `forward` is  `false`).
    #[must_use]
    pub fn reduced_latitude(&self, latitude: f64, forward: bool) -> f64 {
        if forward {
            return latitude.tan().atan2(1. / (1. - self.f));
        }
        latitude.tan().atan2(1. - self.f)
    }

    /// Isometric latitude, 𝜓
    #[must_use]
    pub fn isometric_latitude(&self, latitude: f64) -> f64 {
        let e = self.eccentricity();
        latitude.tan().asinh() - (e * latitude.sin()).atanh() * e
    }

    // ----- Meridian geometry -----------------------------------------------------

    /// The Normalized Meridian Arc Unit, *Qn*, is the mean length of one radian
    ///  of the meridian. "Normalized", because we measure it in units of the
    /// semimajor axis, *a*.
    ///
    /// König und Weise p.50 (96), p.19 (38b), p.5 (2), here using the extended
    /// version from [Deakin et al 2012](crate::Bibliography::Dea12) eq. (41)
    #[must_use]
    pub fn normalized_meridian_arc_unit(&self) -> f64 {
        let n = self.third_flattening();
        let nn = n * n;
        (1. + nn * (1. / 4. + nn * (1. / 64. + nn * (1. / 256. + 25. * nn / 16384.)))) / (1.0 + n)
    }

    /// The rectifying radius, *A*, is the radius of a sphere of the same circumference
    /// as the length of a full meridian on the ellipsoid.
    ///
    /// Closely related to the [normalized meridian arc unit](Ellipsoid::normalized_meridian_arc_unit).
    ///
    /// [Deakin et al 2012](crate::Bibliography::Dea12) eq. (41)
    #[must_use]
    pub fn rectifying_radius(&self) -> f64 {
        let n = self.third_flattening();
        let nn = n * n;
        let d = (1. + nn * (1. / 4. + nn * (1. / 64. + nn * (1. / 256. + 25. * nn / 16384.))))
            / (1. + n);

        self.a * d / (1. + n)
    }

    /// The Meridian Quadrant, *Qm*, is the distance from the equator to one of the poles.
    /// i.e. *π/2 · Qn · a*, where *Qn* is the
    /// [normalized meridian arc unit](Ellipsoid::normalized_meridian_arc_unit)
    #[must_use]
    pub fn meridian_quadrant(&self) -> f64 {
        self.a * FRAC_PI_2 * self.normalized_meridian_arc_unit()
    }

    // ----- Geodesics -------------------------------------------------------------

    /// The distance, *M*, along a meridian from the equator to the given
    /// latitude is a special case of a geodesic length.
    ///
    /// This implementation follows the
    /// [remarkably simple algorithm](crate::Bibliography::Bow83) by Bowring (1983).
    /// The forward case computes the meridian distance given a latitude.
    /// The inverse case computes the latitude given a meridian distance.
    ///
    /// See also
    /// [Wikipedia: Transverse Mercator](https://en.wikipedia.org/wiki/Transverse_Mercator:_Bowring_series).
    ///
    /// [Deakin et al](crate::Bibliography::Dea12) provides a higher order (*n⁸*) derivation.
    ///
    #[must_use]
    #[allow(non_snake_case)] // make it possible to mimic math notation from original paper
    #[allow(clippy::many_single_char_names)] // ditto
    pub fn meridional_distance(&self, latitude: f64, forward: bool) -> f64 {
        let n = self.third_flattening();
        let m = 1. + n * n / 8.;

        // Rectifying radius - truncated after the n⁴ term
        let A = self.a * m * m / (1. + n);

        if forward {
            let B = 9. * (1. - 3. * n * n / 8.0);
            let x = 1. + 13. / 12. * n * (2. * latitude).cos();
            let y = 0. + 13. / 12. * n * (2. * latitude).sin();
            let r = y.hypot(x);
            let v = y.atan2(x);
            let theta = latitude - B * r.powf(-2. / 13.) * (2. * v / 13.).sin();
            return A * theta;
        }

        let C = 1. - 9. * n * n / 16.;
        let theta = latitude / A;
        let x = 1. - 155. / 84. * n * (2. * theta).cos();
        let y = 0. + 155. / 84. * n * (2. * theta).sin();
        let r = y.hypot(x);
        let v = y.atan2(x);

        theta + 63. / 4. * C * r.powf(8. / 155.) * (8. / 155. * v).sin()
    }

    /// For general geodesics, we use the algorithm by Vincenty
    /// ([1975](crate::Bibliography::Vin75)), with updates by the same author
    /// ([1976](crate::Bibliography::Vin76)).
    /// The Vincenty algorithm is relatively simple to implement, but for near-antipodal
    /// cases, it suffers from lack of convergence and loss of accuracy.
    ///
    /// Karney ([2012](crate::Bibliography::Kar12), [2013](crate::Bibliography::Kar13))
    /// presented an algorithm which is exact to machine precision, and converges everywhere.
    /// The crate [geographiclib-rs](https://crates.io/crates/geographiclib-rs), by
    /// Federico Dolce and Michael Kirk, provides a Rust implementation of Karney's algorithm.
    #[must_use]
    #[allow(non_snake_case)]
    pub fn geodesic_fwd(
        &self,
        from: &CoordinateTuple,
        azimuth: f64,
        distance: f64,
    ) -> CoordinateTuple {
        // Coordinates of the point of origin, P1
        let B1 = from[1];
        let L1 = from[0];

        // The latitude of P1 projected onto the auxiliary sphere
        let U1 = self.reduced_latitude(B1, fwd);
        let U1cos = U1.cos();
        let U1sin = U1.sin();

        // σ_1, here ss1, is the angular distance on the aux sphere from P1 to equator
        let azicos = azimuth.cos();
        let ss1 = ((1. - self.f) * B1.tan()).atan2(azicos);

        // α, the forward azimuth of the geodesic at equator
        let aasin = U1cos * azimuth.sin();
        let aasin2 = aasin * aasin;
        let aacos2 = 1. - aasin2;

        // A and B according to Vincenty's update (1976)
        let eps = self.second_eccentricity_squared();
        let us = aacos2 * eps;
        let t = (1. + us).sqrt();
        let k1 = (t - 1.) / (t + 1.);
        let A = (1. + k1 * k1 / 4.) / (1. - k1);
        let B = k1 * (1. - 3. * k1 * k1 / 8.);

        // Initial estimate for λ, the longitude on the auxiliary sphere
        let b = self.semiminor_axis();
        let mut ss = distance / (b * A);
        let mut i: i32 = 0;
        let mut t1 = 0.;
        let mut ssmx2cos = 0.;

        while i < 1000 {
            i += 1;

            // 2σ_m, where σ_m is the latitude of the midpoint on the aux sphere
            let ssmx2 = 2. * ss1 + ss;

            // dσ = dss: The correction term for σ
            ssmx2cos = ssmx2.cos();
            let ssmx2cos2 = ssmx2cos * ssmx2cos;
            t1 = -1. + 2. * ssmx2cos2;
            let t2 = -3. + 4. * ssmx2cos2;
            let sssin = ss.sin();
            let sscos = ss.cos();
            let t3 = -3. + 4. * sssin * sssin;
            let dss = B * sssin * (ssmx2cos + B / 4. * (sscos * t1 - B / 6. * ssmx2cos * t2 * t3));

            let prevss = ss;
            ss = distance / (b * A) + dss;

            // Stop criterion: Last update of σ made little difference
            if (prevss - ss).abs() < 1e-13 {
                break;
            }
        }

        // B2: Latitude of destination
        let sssin = ss.sin();
        let sscos = ss.cos();
        let t4 = U1cos * azicos * sssin;
        let t5 = U1cos * azicos * sscos;
        let B2 = (U1sin * sscos + t4).atan2((1. - self.f) * aasin.hypot(U1sin * sssin - t5));

        // L2: Longitude of destination
        let azisin = azimuth.sin();
        let ll = (sssin * azisin).atan2(U1cos * sscos - U1sin * sssin * azicos);
        let C = (4. + self.f * (4. - 3. * aacos2)) * self.f * aacos2 / 16.;
        let L = ll - (1. - C) * self.f * aasin * (ss + C * sssin * (ssmx2cos + C * sscos * t1));
        let L2 = L1 + L;

        // Return azimuth
        let aa2 = aasin.atan2(U1cos * sscos * azicos - U1sin * sssin);

        CoordinateTuple::new(L2, B2, aa2, f64::from(i))
    }

    /// See [`geodesic_fwd`](crate::Ellipsoid::geodesic_fwd)
    #[must_use]
    #[allow(non_snake_case)] // allow math-like notation
    pub fn geodesic_inv(&self, from: &CoordinateTuple, to: &CoordinateTuple) -> CoordinateTuple {
        let B1 = from[1];
        let B2 = to[1];

        let L1 = from[0];
        let L2 = to[0];
        let L = L2 - L1;

        let U1 = self.reduced_latitude(B1, fwd);
        let U2 = self.reduced_latitude(B2, fwd);

        let U1cos = U1.cos();
        let U2cos = U2.cos();
        let U1sin = U1.sin();
        let U2sin = U2.sin();
        let eps = self.second_eccentricity_squared();

        // Initial estimate for λ, the longitude on the auxiliary sphere
        let mut ll = L;

        let mut aacos2 = 0.;
        let mut ssmx2cos = 0.;
        let mut sscos = 0.;
        let mut sssin = 0.;
        let mut ss = 0.;
        let mut llsin = 0.;
        let mut llcos = 1.;

        let mut i: i32 = 0;

        while i < 1000 {
            i += 1;

            // σ, the angular separation between the points
            llsin = ll.sin();
            llcos = ll.cos();
            let t1 = U2cos * llsin;
            let t2 = U1cos * U2sin - U2cos * U1sin * llcos;
            sssin = t1.hypot(t2);
            sscos = U1sin * U2sin + U1cos * U2cos * llcos;
            ss = sssin.atan2(sscos);

            // α, the forward azimuth of the geodesic at equator
            let aasin = U1cos * U2cos * llsin / sssin;
            aacos2 = 1. - aasin * aasin;

            // cosine of 2 times σ_m, the angular separation from the midpoint to the equator
            ssmx2cos = sscos - 2. * U1sin * U2sin / aacos2;
            let C = (4. + self.f * (4. - 3. * aacos2)) * self.f * aacos2 / 16.;
            let ll_next = L
                + (1. - C)
                    * self.f
                    * aasin
                    * (ss + C * sssin * (ssmx2cos + C * sscos * (-1. + 2. * ssmx2cos * ssmx2cos)));
            let dl = (ll - ll_next).abs();
            ll = ll_next;
            if dl < 1e-12 {
                break;
            }
        }

        // A and B according to Vincenty's update (1976)
        let us = aacos2 * eps;
        let t = (1. + us).sqrt();
        let k1 = (t - 1.) / (t + 1.);
        let A = (1. + k1 * k1 / 4.) / (1. - k1);
        let B = k1 * (1. - 3. * k1 * k1 / 8.);

        // The difference between the dist on the aux sphere and on the ellipsoid.
        let t1 = -1. + 2. * ssmx2cos * ssmx2cos;
        let t2 = -3. + 4. * sssin * sssin;
        let t3 = -3. + 4. * ssmx2cos * ssmx2cos;
        let dss = B * sssin * (ssmx2cos + B / 4. * (sscos * t1 - B / 6. * ssmx2cos * t2 * t3));

        // Distance, forward azimuth, return azimuth
        let s = self.semiminor_axis() * A * (ss - dss);
        let a1 = (U2cos * llsin).atan2(U1cos * U2sin - U1sin * U2cos * llcos);
        let a2 = (U1cos * llsin).atan2(-U1sin * U2cos + U1cos * U2sin * llcos);
        CoordinateTuple::new(a1, a2, s, f64::from(i))
    }

    /// Geodesic distance between two points. Assumes the first coordinate
    /// is longitude, second is latitude.
    ///
    /// # See also:
    ///
    /// [`hypot2`](crate::coordinates::CoordinateTuple::hypot2),
    /// [`hypot3`](crate::coordinates::CoordinateTuple::hypot3)
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Compute the distance between Copenhagen and Paris
    /// use geodesy::Ellipsoid;
    /// use geodesy::operand::*;
    /// let ellps = Ellipsoid::named("GRS80");
    /// let p0 = CoordinateTuple::deg(12., 55., 0., 0.);
    /// let p1 = CoordinateTuple::deg(2., 49., 0., 0.);
    /// let d = ellps.distance(&p0, &p1);
    /// assert!((d - 956_066.231_959).abs() < 1e-5);
    /// ```
    #[must_use]
    pub fn distance(&self, from: &CoordinateTuple, to: &CoordinateTuple) -> f64 {
        self.geodesic_inv(from, to)[2]
    }

    // ----- Cartesian <--> Geographic conversion ----------------------------------

    /// Geographic to cartesian conversion.
    ///
    /// Follows the the derivation given by
    /// Bowring ([1976](crate::Bibliography::Bow76) and
    /// [1985](crate::Bibliography::Bow85))
    #[must_use]
    #[allow(non_snake_case)] // make it possible to mimic math notation from original paper
    #[allow(clippy::many_single_char_names)] // ditto
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
    #[must_use]
    #[allow(non_snake_case)] // make it possible to mimic math notation from original paper
    #[allow(clippy::many_single_char_names)] // ditto
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
        // let theta_num = Z * self.a;
        // let theta_denom = p * b;
        // let length = theta_num.hypot(theta_denom);
        // let c = theta_denom / length; // i.e. cos(theta)
        // let s = theta_num / length; // i.e. sin(theta)

        // Fukushima (1999), Appendix B: Equivalent to Even Rouault's, implementation,
        // but not as clear - although a bit faster due to the substitution of sqrt
        // for hypot.
        let T = (Z * self.a) / (p * b);
        let c = 1.0 / (1.0 + T * T).sqrt();
        let s = c * T;

        let phi_num = Z + eps * b * s.powi(3);
        let phi_denom = p - es * self.a * c.powi(3);
        let phi = phi_num.atan2(phi_denom);

        let lenphi = phi_num.hypot(phi_denom);
        let sinphi = phi_num / lenphi;
        let cosphi = phi_denom / lenphi;

        // We already have sinphi and es, so we can compute the radius
        // of curvature faster by inlining, rather than calling the
        // prime_vertical_radius_of_curvature() method.
        let N = self.a / (1.0 - sinphi.powi(2) * es).sqrt();

        // Bowring (1985), as quoted by Burtch (2006), suggests this expression
        // as more accurate than the commonly used h = p / cosphi - N;
        let h = p * cosphi + Z * sinphi - self.a * self.a / N;

        CoordinateTuple::new(lam, phi, h, t)
    }
}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inv;
    #[test]
    fn test_ellipsoid() {
        // Constructors
        let ellps = Ellipsoid::named("intl");
        assert_eq!(ellps.flattening(), 1. / 297.);

        let ellps = Ellipsoid::named("GRS80");
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25722_21008_82711_24316);

        assert!((ellps.normalized_meridian_arc_unit() - 0.9983242984230415).abs() < 1e-13);
        assert!((4.0 * ellps.meridian_quadrant() - 40007862.9169218).abs() < 1e-7);
    }

    #[test]
    fn shape_and_size() {
        let ellps = Ellipsoid::named("GRS80");
        let ellps = Ellipsoid::new(ellps.semimajor_axis(), ellps.flattening());
        let ellps = Ellipsoid::triaxial(ellps.a, ellps.a - 1., ellps.f);
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25722_21008_82711_24316);

        // Additional shape descriptors
        assert!((ellps.eccentricity() - 0.081819191).abs() < 1.0e-10);
        assert!((ellps.eccentricity_squared() - 0.00669_43800_22903_41574).abs() < 1.0e-10);

        // Additional size descriptors
        assert!((ellps.semiminor_axis() - 6_356_752.31414_0347).abs() < 1e-9);
        assert!((ellps.semimajor_axis() - 6_378_137.0).abs() < 1e-9);
    }

    #[test]
    fn curvatures() {
        let ellps = Ellipsoid::named("GRS80");
        // The curvatures at the North Pole
        assert!(
            (ellps.meridian_radius_of_curvature(90_f64.to_radians()) - 6_399_593.6259).abs() < 1e-4
        );
        assert!(
            (ellps.prime_vertical_radius_of_curvature(90_f64.to_radians()) - 6_399_593.6259).abs()
                < 1e-4
        );
        assert!(
            (ellps.prime_vertical_radius_of_curvature(90_f64.to_radians())
                - ellps.meridian_radius_of_curvature(90_f64.to_radians()))
            .abs()
                < 1e-5
        );
        assert!(
            (ellps.polar_radius_of_curvature()
                - ellps.meridian_radius_of_curvature(90_f64.to_radians()))
            .abs()
                < 1e-6
        );

        // The curvatures at the Equator
        assert!((ellps.meridian_radius_of_curvature(0.0) - 6_335_439.3271).abs() < 1.0e-4);
        assert!(
            (ellps.prime_vertical_radius_of_curvature(0.0) - ellps.semimajor_axis()).abs() < 1.0e-4
        );
    }

    #[test]
    fn geo_to_cart() {
        let ellps = Ellipsoid::named("GRS80");
        // Roundtrip geographic <-> cartesian
        let geo = CoordinateTuple::deg(12., 55., 100., 0.);
        let cart = ellps.cartesian(&geo);
        let geo2 = ellps.geographic(&cart);
        assert_eq!(geo[0], geo2[0]);
        assert!((geo[0] - geo2[0]).abs() < 1.0e-12);
        assert!((geo[1] - geo2[1]).abs() < 1.0e-12);
        assert!((geo[2] - geo2[2]).abs() < 1.0e-9);
    }

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
        assert!(
            (ellps.isometric_latitude(45f64.to_radians()) - 50.227465815385806f64.to_radians())
                .abs()
                < 1e-15
        );
    }

    #[test]
    fn meridional_distance() {
        let ellps = Ellipsoid::named("GRS80");

        // Rectifying radius, A
        assert!((ellps.rectifying_radius() - 6356774.720017125).abs() < 1e-9);

        // --------------------------------------------------------------------
        // Meridional distance, M
        // --------------------------------------------------------------------

        // Internal consistency: Check that at 90°, the meridional distance
        // is identical to the meridian quadrant.
        assert!(
            (ellps.meridional_distance(FRAC_PI_2, fwd) - ellps.meridian_quadrant()).abs() < 1e-15
        );
        assert!(
            (ellps.meridional_distance(ellps.meridian_quadrant(), inv) - FRAC_PI_2).abs() < 1e-15
        );

        // Internal consistency: Roundtrip replication accuracy.
        for i in 0..10 {
            // latitude -> distance -> latitude
            let b = (10. * i as f64).to_radians();
            assert!(
                (ellps.meridional_distance(ellps.meridional_distance(b, fwd), inv) - b).abs()
                    < 5e-11
            );

            // distance -> latitude -> distance;
            let d = 1_000_000. * i as f64;
            assert!(
                (ellps.meridional_distance(ellps.meridional_distance(d, inv), fwd) - d).abs()
                    < 6e-5
            );
        }

        // Compare with Karney's algorithm for geodesics.
        // We expect deviations to be less than 6 𝜇m.

        // Meridional distances for angles 0, 10, 20, 30 ... 90, obtained from Charles Karney's
        // online geodesic solver, https://geographiclib.sourceforge.io/cgi-bin/GeodSolve
        let s = [
            0000000.000000000,
            1105854.833198446,
            2212366.254102976,
            3320113.397845014,
            4429529.030236580,
            5540847.041560960,
            6654072.819367435,
            7768980.727655508,
            8885139.871836751,
            10001965.729230457,
        ];

        for i in 0..s.len() {
            assert!(
                (ellps.meridional_distance((10.0 * i as f64).to_radians(), fwd) - s[i]).abs()
                    < 6e-6
            );
            assert!(
                (ellps.meridional_distance(s[i], inv) - (10.0 * i as f64).to_radians()).abs()
                    < 6e-11
            );
        }

        // Since we suspect the deviation might be worst at 45°, we check that as well
        assert!(
            (ellps.meridional_distance(45f64.to_radians(), fwd) - 4984944.377857987).abs() < 4e-6
        );
        assert!(
            (ellps.meridional_distance(4984944.377857987, inv) - 45f64.to_radians()).abs() < 4e-6
        );
    }

    #[test]
    fn geodesics() {
        let ellps = Ellipsoid::named("GRS80");

        // (expected values from Karney: https://geographiclib.sourceforge.io/cgi-bin/GeodSolve)

        // Copenhagen (Denmark)--Paris (France)
        // Expect distance good to 0.01 mm, azimuths to a nanodegree
        let p1 = CoordinateTuple::deg(12., 55., 0., 0.);
        let p2 = CoordinateTuple::deg(2., 49., 0., 0.);

        let d = ellps.geodesic_inv(&p1, &p2);
        assert!((d[0].to_degrees() - (-130.15406042072)).abs() < 1e-9);
        assert!((d[1].to_degrees() - (-138.05257941874)).abs() < 1e-9);
        assert!((d[2] - 956066.231959).abs() < 1e-5);

        // And the other way round...
        let b = ellps.geodesic_fwd(&p1, d[0], d[2]);
        assert!((b[0].to_degrees() - 2.).abs() < 1e-9);
        assert!((b[1].to_degrees() - 49.).abs() < 1e-9);

        // Copenhagen (Denmark)--Rabat (Morocco)
        // Expect distance good to 0.1 mm, azimuths to a nanodegree
        let p2 = CoordinateTuple::deg(7., 34., 0., 0.);

        let d = ellps.geodesic_inv(&p1, &p2);
        assert!((d[0].to_degrees() - (-168.48914418666)).abs() < 1e-9);
        assert!((d[1].to_degrees() - (-172.05461964948)).abs() < 1e-9);
        assert!((d[2] - 2365723.367715).abs() < 1e-4);

        // And the other way round...
        let b = ellps.geodesic_fwd(&p1, d[0], d[2]).to_degrees();
        assert!((b[0] - p2[0].to_degrees()).abs() < 1e-9);
        assert!((b[1] - p2[1].to_degrees()).abs() < 1e-9);
    }
}
