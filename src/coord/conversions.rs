use super::Coord;

impl Coord {
    /// Transform the first two elements of a `CoordinateTuple` from degrees to radians
    #[must_use]
    pub fn to_radians(self) -> Coord {
        Coord([self[0].to_radians(), self[1].to_radians(), self[2], self[3]])
    }

    /// Transform the first two elements of a `CoordinateTuple` from radians to degrees
    #[must_use]
    pub fn to_degrees(self) -> Coord {
        Coord([self[0].to_degrees(), self[1].to_degrees(), self[2], self[3]])
    }

    /// Transform the internal lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    #[must_use]
    pub fn to_geo(self) -> Coord {
        Coord([self[1].to_degrees(), self[0].to_degrees(), self[2], self[3]])
    }

    /// For an entire data set: Transform the internal lon/lat/h/t-in-radians to lat/lon/h/t-in-degrees
    pub fn geo_all(operands: &mut [Coord]) {
        for coord in operands {
            *coord = coord.to_geo();
        }
    }

    /// For an entire data set: Transform the first two elements of a `CoordinateTuple` from radians to degrees
    pub fn degrees_all(operands: &mut [Coord]) {
        for coord in operands {
            *coord = coord.to_degrees();
        }
    }

    /// For an entire data set: Transform the first two elements of a `CoordinateTuple` from degrees to radians
    pub fn radians_all(operands: &mut [Coord]) {
        for coord in operands {
            *coord = coord.to_radians();
        }
    }

    /// Simplistic transformation from degrees, minutes and seconds-with-decimals
    /// to degrees-with-decimals. No sanity check: Sign taken from degree-component,
    /// minutes forced to unsigned by i16 type, but passing a negative value for
    /// seconds leads to undefined behaviour.
    pub fn dms_to_dd(d: i32, m: u16, s: f64) -> f64 {
        d.signum() as f64 * (d.abs() as f64 + (m as f64 + s / 60.) / 60.)
    }

    /// Simplistic transformation from degrees and minutes-with-decimals
    /// to degrees-with-decimals. No sanity check: Sign taken from
    /// degree-component, but passing a negative value for minutes leads
    /// to undefined behaviour.
    pub fn dm_to_dd(d: i32, m: f64) -> f64 {
        d.signum() as f64 * (d.abs() as f64 + (m as f64 / 60.))
    }

    /// Simplistic transformation from the NMEA DDDMM.mmm format to
    /// to degrees-with-decimals. No sanity check: Invalid input,
    /// such as 5575.75 (where the number of minutes exceed 60) leads
    /// to undefined behaviour.
    pub fn nmea_to_dd(nmea: f64) -> f64 {
        let sign = nmea.signum();
        let dm = nmea.abs() as u32;
        let fraction = nmea.abs() - dm as f64;
        let d = dm / 100;
        let m = (dm - d * 100) as f64 + fraction;
        sign * (d as f64 + (m as f64 / 60.))
    }

    /// Transformation from degrees-with-decimals to the NMEA DDDMM.mmm format.
    pub fn dd_to_nmea(dd: f64) -> f64 {
        let sign = dd.signum();
        let dd = dd.abs();
        let d = dd.floor();
        let m = (dd - d) * 60.;
        sign * (d * 100. + m)
    }

    /// Simplistic transformation from the extended NMEA DDDMMSS.sss
    /// format to degrees-with-decimals. No sanity check: Invalid input,
    /// such as 557575.75 (where the number of minutes and seconds both
    /// exceed 60) leads to undefined behaviour.
    pub fn nmeass_to_dd(nmeass: f64) -> f64 {
        let sign = nmeass.signum();
        let dms = nmeass.abs() as u32;
        let fraction = nmeass.abs() - dms as f64;
        let d = dms / 10000;
        let ms = dms - d * 10000;
        let m = ms / 100;
        let s = (ms - m * 100) as f64 + fraction;
        sign * (d as f64 + ((s as f64 / 60.) + m as f64) / 60.)
    }

    /// Transformation from degrees-with-decimals to the extended
    /// NMEA DDDMMSS.sss format.
    pub fn dd_to_nmeass(dd: f64) -> f64 {
        let sign = dd.signum();
        let dd = dd.abs();
        let d = dd.floor();
        let mm = (dd - d) * 60.;
        let m = mm.floor();
        let s = (mm - m) * 60.;
        sign * (d * 10000. + m * 100. + s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        let c = Coord::raw(12., 55., 100., 0.).to_radians();
        let d = Coord::gis(12., 55., 100., 0.);
        assert_eq!(c, d);
        assert_eq!(d[0], 12f64.to_radians());
        let e = d.to_degrees();
        assert_eq!(e[0], c.to_degrees()[0]);

        assert_eq!(Coord::dms_to_dd(55, 30, 36.), 55.51);
        assert_eq!(Coord::dm_to_dd(55, 30.60), 55.51);

        // nmea + nmeass
        assert!((Coord::nmea_to_dd(5530.60) - 55.51).abs() < 1e-10);
        assert!((Coord::nmea_to_dd(15530.60) - 155.51).abs() < 1e-10);
        assert!((Coord::nmea_to_dd(-15530.60) + 155.51).abs() < 1e-10);
        assert!((Coord::nmeass_to_dd(553036.0) - 55.51).abs() < 1e-10);
        assert_eq!(Coord::dd_to_nmea(55.5025), 5530.15);
        assert_eq!(Coord::dd_to_nmea(-55.5025), -5530.15);
        assert_eq!(Coord::dd_to_nmeass(55.5025), 553009.);
        assert_eq!(Coord::dd_to_nmeass(-55.51), -553036.);

        assert_eq!(Coord::nmea_to_dd(5500.), 55.);
        assert_eq!(Coord::nmea_to_dd(-5500.), -55.);
        assert_eq!(Coord::nmea_to_dd(5530.60), -Coord::nmea_to_dd(-5530.60));
        assert_eq!(
            Coord::nmeass_to_dd(553036.),
            -Coord::nmeass_to_dd(-553036.00)
        );
    }
}
