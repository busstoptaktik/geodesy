use crate::ellipsoid::Ellipsoid;
use crate::CoordinateTuple;
use crate::FWD;

// ----- Geodesics -------------------------------------------------------------
impl Ellipsoid {
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
        let U1 = self.reduced_latitude(B1, FWD);
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

        CoordinateTuple::raw(L2, B2, aa2, f64::from(i))
    }

    /// See [`geodesic_fwd`](crate::Ellipsoid::geodesic_fwd)
    #[must_use]
    #[allow(non_snake_case)] // allow math-like notation
    pub fn geodesic_inv(&self, from: &CoordinateTuple, to: &CoordinateTuple) -> CoordinateTuple {
        let B1 = from[1];
        let B2 = to[1];
        let B = B2 - B1;

        let L1 = from[0];
        let L2 = to[0];
        let L = L2 - L1;

        // Below the micrometer level, we don't care about directions
        if L.hypot(B) < 1e-15 {
            return CoordinateTuple::geo(0., 0., 0., 0.);
        }

        let U1 = self.reduced_latitude(B1, FWD);
        let U2 = self.reduced_latitude(B2, FWD);

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
        CoordinateTuple::raw(a1, a2, s, f64::from(i))
    }

    /// Geodesic distance between two points. Assumes the first coordinate
    /// is longitude, second is latitude.
    ///
    /// # See also:
    ///
    /// [`hypot2`](crate::coordinate::CoordinateTuple::hypot2),
    /// [`hypot3`](crate::coordinate::CoordinateTuple::hypot3)
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Compute the distance between Copenhagen and Paris
    /// use geodesy::Ellipsoid;
    /// use geodesy::CoordinateTuple;
    /// let ellps = Ellipsoid::named("GRS80");
    /// let p0 = CoordinateTuple::geo(55., 12., 0., 0.);
    /// let p1 = CoordinateTuple::geo(49., 2., 0., 0.);
    /// let d = ellps.distance(&p0, &p1);
    /// assert!((d - 956_066.231_959).abs() < 1e-5);
    /// ```
    #[must_use]
    pub fn distance(&self, from: &CoordinateTuple, to: &CoordinateTuple) -> f64 {
        self.geodesic_inv(from, to)[2]
    }
}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::GeodesyError;

    use super::*;
    #[test]
    fn geodesics() -> Result<(), GeodesyError>{
        let ellps = Ellipsoid::named("GRS80")?;

        // (expected values from Karney: https://geographiclib.sourceforge.io/cgi-bin/GeodSolve)

        // Copenhagen (Denmark)--Paris (France)
        // Expect distance good to 0.01 mm, azimuths to a nanodegree
        let p1 = CoordinateTuple::gis(12., 55., 0., 0.);
        let p2 = CoordinateTuple::gis(2., 49., 0., 0.);

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
        let p2 = CoordinateTuple::gis(7., 34., 0., 0.);

        let d = ellps.geodesic_inv(&p1, &p2);
        assert!((d[0].to_degrees() - (-168.48914418666)).abs() < 1e-9);
        assert!((d[1].to_degrees() - (-172.05461964948)).abs() < 1e-9);
        assert!((d[2] - 2365723.367715).abs() < 1e-4);

        // And the other way round...
        let b = ellps.geodesic_fwd(&p1, d[0], d[2]).to_degrees();
        assert!((b[0] - p2[0].to_degrees()).abs() < 1e-9);
        assert!((b[1] - p2[1].to_degrees()).abs() < 1e-9);
        Ok(())
    }
}
