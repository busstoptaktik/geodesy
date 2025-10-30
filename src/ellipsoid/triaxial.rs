use crate::prelude::*;

/// A triaxial ellipsoid. Currently mostly a placeholder for future use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriaxialEllipsoid {
    a: f64,
    ay: f64,
    f: f64,
}

/// GRS80 is the default ellipsoid.
impl Default for TriaxialEllipsoid {
    fn default() -> TriaxialEllipsoid {
        TriaxialEllipsoid::new(6_378_137.0, 6_378_137.0, 1. / 298.257_222_100_882_7)
    }
}

impl EllipsoidBase for TriaxialEllipsoid {
    fn semimajor_axis(&self) -> f64 {
        self.a
    }

    fn semimedian_axis(&self) -> f64 {
        self.ay
    }

    fn flattening(&self) -> f64 {
        self.f
    }
}

/// Constructors for `TriaxialEllipsoid`
impl TriaxialEllipsoid {
    /// User defined ellipsoid
    #[must_use]
    pub fn new(semimajor_axis: f64, semimedian_axis: f64, flattening: f64) -> TriaxialEllipsoid {
        TriaxialEllipsoid {
            a: semimajor_axis,
            ay: semimedian_axis,
            f: flattening,
        }
    }

    /// Predefined ellipsoid; built-in, defined in asset collections, or given as a string formatted
    /// (a, rf) or (ax, ay, rx) tuple, e.g. "6378137, 298.25" or "6378137, 6345678, 300"
    pub fn named(name: &str) -> Result<TriaxialEllipsoid, Error> {
        // Is it one of the few builtins?
        if let Some(index) = super::constants::ELLIPSOID_LIST
            .iter()
            .position(|&ellps| ellps.0 == name)
        {
            let e = super::constants::ELLIPSOID_LIST[index];
            let ax: f64 = e.1.parse().unwrap();
            let ay: f64 = e.2.parse().unwrap();
            let rf: f64 = e.3.parse().unwrap();
            // EPSG convention: zero reciproque flattening indicates zero flattening
            let f = if rf != 0.0 { 1.0 / rf } else { rf };
            return Ok(TriaxialEllipsoid::new(ax, ay, f));
        }

        // Remove optional parenthesis
        let mut name = name;
        if name.starts_with('(') && name.ends_with(')') {
            name = name.strip_prefix('(').unwrap().strip_suffix(')').unwrap();
        }

        // The "semimajor, semimedian, reciproque-flattening" form, e.g. "6378137, 6345678, 298.3"
        let a_and_rf = name.split(',').collect::<Vec<_>>();
        let n = a_and_rf.len();
        let semimedian_index = if n == 2 { 0 } else { 1 };
        if [2usize, 3].contains(&n) {
            if let Ok(ax) = a_and_rf[0].trim().parse::<f64>() {
                if let Ok(ay) = a_and_rf[semimedian_index].trim().parse::<f64>() {
                    if let Ok(rf) = a_and_rf[semimedian_index + 1].trim().parse::<f64>() {
                        return Ok(TriaxialEllipsoid::new(ax, ay, 1. / rf));
                    }
                }
            }
        }

        // TODO: Search asset collection
        Err(Error::NotFound(
            String::from(name),
            String::from("Ellipsoid::named()"),
        ))
    }
}

// ----- Tests ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::Error;
    use crate::ellps::EllipsoidBase;
    use crate::ellps::Meridians;
    use crate::ellps::TriaxialEllipsoid;

    #[test]
    fn test_triaxial_ellipsoid() -> Result<(), Error> {
        // Constructors
        let ellps = TriaxialEllipsoid::named("intl")?;
        assert_eq!(ellps.flattening(), 1. / 297.);

        let ellps = TriaxialEllipsoid::named("6378137, 298.25")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25);

        let ellps = TriaxialEllipsoid::named("(6378137, 298.25)")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25);

        let ellps = TriaxialEllipsoid::named("6378137, 6345678, 298.25")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.semimedian_axis(), 6345678.0);
        assert_eq!(ellps.flattening(), 1. / 298.25);

        let ellps = TriaxialEllipsoid::named("GRS80")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.257_222_100_882_7);

        assert!((ellps.normalized_meridian_arc_unit() - 0.998_324_298_423_041_5).abs() < 1e-13);
        assert!((4.0 * ellps.meridian_quadrant() - 40_007_862.916_921_8).abs() < 1e-7);
        Ok(())
    }
}
