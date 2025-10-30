use crate::prelude::*;

/// An ellipsoid of revolution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ellipsoid {
    a: f64,
    f: f64,
}

/// GRS80 is the default ellipsoid.
impl Default for Ellipsoid {
    fn default() -> Ellipsoid {
        Ellipsoid::new(6_378_137.0, 1. / 298.257_222_100_882_7)
    }
}

impl EllipsoidBase for Ellipsoid {
    fn semimajor_axis(&self) -> f64 {
        self.a
    }

    fn flattening(&self) -> f64 {
        self.f
    }
}

/// Constructors for `Ellipsoid`
impl Ellipsoid {
    /// User defined ellipsoid
    #[must_use]
    pub fn new(semimajor_axis: f64, flattening: f64) -> Ellipsoid {
        Ellipsoid {
            a: semimajor_axis,
            f: flattening,
        }
    }

    /// Predefined ellipsoid; built-in, defined in asset collections, or given as a
    /// string formatted (a, rf) tuple, e.g. "6378137, 298.25"
    pub fn named(name: &str) -> Result<Ellipsoid, Error> {
        // Is it one of the few builtins?
        if let Some(index) = super::constants::ELLIPSOID_LIST
            .iter()
            .position(|&ellps| ellps.0 == name)
        {
            let e = super::constants::ELLIPSOID_LIST[index];
            let ax: f64 = e.1.parse().unwrap();
            let rf: f64 = e.3.parse().unwrap();
            // EPSG convention: zero reciproque flattening indicates zero flattening
            let f = if rf != 0.0 { 1.0 / rf } else { rf };
            return Ok(Ellipsoid::new(ax, f));
        }

        // Remove optional parenthesis
        let mut name = name;
        if name.starts_with('(') && name.ends_with(')') {
            name = name.strip_prefix('(').unwrap().strip_suffix(')').unwrap();
        }

        // The "semimajor, reciproque-flattening" form, e.g. "6378137, 298.3"
        let a_and_rf = name.split(',').collect::<Vec<_>>();
        if a_and_rf.len() == 2_usize {
            if let Ok(a) = a_and_rf[0].trim().parse::<f64>() {
                if let Ok(rf) = a_and_rf[1].trim().parse::<f64>() {
                    return Ok(Ellipsoid::new(a, 1. / rf));
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
    use crate::ellps::Ellipsoid;
    use crate::ellps::EllipsoidBase;
    use crate::ellps::Meridians;

    #[test]
    fn test_ellipsoid() -> Result<(), Error> {
        // Constructors
        let ellps = Ellipsoid::named("intl")?;
        assert_eq!(ellps.flattening(), 1. / 297.);

        let ellps = Ellipsoid::named("6378137, 298.25")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25);

        let ellps = Ellipsoid::named("(6378137, 298.25)")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.25);

        let ellps = Ellipsoid::named("GRS80")?;
        assert_eq!(ellps.semimajor_axis(), 6378137.0);
        assert_eq!(ellps.flattening(), 1. / 298.257_222_100_882_7);

        assert!((ellps.normalized_meridian_arc_unit() - 0.998_324_298_423_041_5).abs() < 1e-13);
        assert!((4.0 * ellps.meridian_quadrant() - 40_007_862.916_921_8).abs() < 1e-7);
        Ok(())
    }
}
