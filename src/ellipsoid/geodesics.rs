use crate::coordinate::tuple::CoordinateTuple;

use super::*;

/// Geodesics
pub trait Geodesics: EllipsoidBase {
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
    fn geodesic_fwd<C: CoordinateTuple>(&self, from: &C, azimuth: f64, distance: f64) -> Coor4D {
        // Coordinates of the point of origin, P1
        let (L1, B1) = from.xy();

        // The latitude of P1 projected onto the auxiliary sphere
        let U1 = self.latitude_geographic_to_reduced(B1);
        let (U1sin, U1cos) = U1.sin_cos();

        // σ_1, here ss1, is the angular distance on the aux sphere from P1 to equator
        let azicos = azimuth.cos();
        let ss1 = ((1. - self.flattening()) * B1.tan()).atan2(azicos);

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
            let (sssin, sscos) = ss.sin_cos();
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
        let (sssin, sscos) = ss.sin_cos();
        let f = self.flattening();
        let t4 = U1cos * azicos * sssin;
        let t5 = U1cos * azicos * sscos;
        let B2 = (U1sin * sscos + t4).atan2((1. - f) * aasin.hypot(U1sin * sssin - t5));

        // L2: Longitude of destination
        let azisin = azimuth.sin();
        let ll = (sssin * azisin).atan2(U1cos * sscos - U1sin * sssin * azicos);
        let C = (4. + f * (4. - 3. * aacos2)) * f * aacos2 / 16.;
        let L = ll - (1. - C) * f * aasin * (ss + C * sssin * (ssmx2cos + C * sscos * t1));
        let L2 = L1 + L;

        // Return azimuth
        let aa2 = aasin.atan2(U1cos * sscos * azicos - U1sin * sssin);

        Coor4D::raw(L2, B2, aa2, f64::from(i))
    }

    /// See [`geodesic_fwd`](Self::geodesic_fwd)
    #[must_use]
    #[allow(non_snake_case)] // So we can use the mathematical notation from the original text
    fn geodesic_inv<C: CoordinateTuple>(&self, from: &C, to: &C) -> Coor4D {
        let (L1, B1) = from.xy();
        let (L2, B2) = to.xy();
        let B = B2 - B1;
        let L = L2 - L1;

        // Below the micrometer level, we don't care about directions
        if L.hypot(B) < 1e-15 {
            return Coor4D::geo(0., 0., 0., 0.);
        }

        let U1 = self.latitude_geographic_to_reduced(B1);
        let U2 = self.latitude_geographic_to_reduced(B2);

        let (U1sin, U1cos) = U1.sin_cos();
        let (U2sin, U2cos) = U2.sin_cos();
        let eps = self.second_eccentricity_squared();
        let f = self.flattening();

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
            (llsin, llcos) = ll.sin_cos();
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
            let C = (4. + f * (4. - 3. * aacos2)) * f * aacos2 / 16.;
            let ll_next = L
                + (1. - C)
                    * f
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
        Coor4D::raw(a1, a2, s, f64::from(i))
    }

    /// Geodesic distance between two points. Assumes the first coordinate
    /// is longitude, second is latitude.
    ///
    /// # See also:
    ///
    /// [`hypot2`](crate::coordinate::tuple::CoordinateTuple::hypot2),
    /// [`hypot3`](crate::coordinate::tuple::CoordinateTuple::hypot3),
    ///
    /// # Examples
    ///
    /// ```
    /// // Compute the distance between Copenhagen and Paris
    /// use geodesy::prelude::*;
    /// if let Ok(ellps) = Ellipsoid::named("GRS80") {
    ///     let p0 = Coor2D::geo(55., 12.);
    ///     let p1 = Coor2D::geo(49., 2.);
    ///     let d = ellps.distance(&p0, &p1);
    ///     assert!((d - 956_066.231_959).abs() < 1e-5);
    /// }
    /// ```
    #[must_use]
    fn distance<G: CoordinateTuple>(&self, from: &G, to: &G) -> f64 {
        self.geodesic_inv(from, to)[2]
    }
}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::Coor2D;
    #[test]
    fn geodesics() -> Result<(), Error> {
        let ellps = Ellipsoid::named("GRS80")?;

        // (expected values from Karney: https://geographiclib.sourceforge.io/cgi-bin/GeodSolve)

        // Copenhagen (Denmark)--Paris (France)
        // Expect distance good to 0.01 mm, azimuths to a nanodegree
        let p1 = Coor2D::gis(12., 55.);
        let p2 = Coor2D::gis(2., 49.);

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
        let p2 = Coor2D::gis(7., 34.);

        let d = ellps.geodesic_inv(&p1, &p2);
        assert!((d[0].to_degrees() - (-168.48914418666)).abs() < 1e-9);
        assert!((d[1].to_degrees() - (-172.05461964948)).abs() < 1e-9);
        assert!((d[2] - 2365723.367715).abs() < 1e-4);

        // And the other way round...
        let b = ellps.geodesic_fwd(&p1, d[0], d[2]);
        assert!((b[0].to_degrees() - p2[0].to_degrees()).abs() < 1e-9);
        assert!((b[1].to_degrees() - p2[1].to_degrees()).abs() < 1e-9);
        Ok(())
    }
}
