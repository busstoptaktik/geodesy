/// Ellipsoidal curvature measures
use crate::authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let n = operands.len();
    let sliced = 0..n;
    let ellps = op.params.ellps(0);

    let prime = op.params.boolean("prime");
    let meridional = op.params.boolean("meridian");
    let gaussian = op.params.boolean("gaussian");
    let mean = op.params.boolean("mean");
    let azimuthal = op.params.boolean("azimuthal");

    let mut successes = 0_usize;

    if prime {
        for i in sliced {
            let (lat, lon) = operands.xy(i);
            let lat = ellps.prime_vertical_radius_of_curvature(lat.to_radians());
            operands.set_xy(i, lat, lon);
            successes += 1;
        }
        return successes;
    }

    if meridional {
        for i in sliced {
            let (lat, lon) = operands.xy(i);
            let lat = ellps.meridian_radius_of_curvature(lat.to_radians());
            operands.set_xy(i, lat, lon);
            successes += 1;
        }
        return successes;
    }

    if gaussian {
        for i in sliced {
            let (lat, lon) = operands.xy(i);
            let lat = lat.to_radians();
            let m = ellps.meridian_radius_of_curvature(lat);
            let n = ellps.prime_vertical_radius_of_curvature(lat);
            let lat = (n * m).sqrt();
            operands.set_xy(i, lat, lon);
            successes += 1;
        }
        return successes;
    }

    if mean {
        for i in sliced {
            let (lat, lon) = operands.xy(i);
            let lat = lat.to_radians();
            let m = ellps.meridian_radius_of_curvature(lat);
            let n = ellps.prime_vertical_radius_of_curvature(lat);
            let lat = 2.0 * (n.recip() + m.recip()).recip();
            operands.set_xy(i, lat, lon);
            successes += 1;
        }
        return successes;
    }

    if azimuthal {
        for i in sliced {
            let (lat, azi) = operands.xy(i).xy_to_radians();
            let m = ellps.meridian_radius_of_curvature(lat);
            let n = ellps.prime_vertical_radius_of_curvature(lat);
            let (s, c) = azi.sin_cos();
            let lat = (c * c / m + s * s / n).recip();
            operands.set_xy(i, lat, azi);
            successes += 1;
        }
        return successes;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 6] = [
    OpParameter::Flag { key: "prime" },
    OpParameter::Flag { key: "meridian" },
    OpParameter::Flag { key: "gaussian" },
    OpParameter::Flag { key: "mean" },
    OpParameter::Flag { key: "azimuthal" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") }
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let op = Op::plain(parameters, InnerOp(fwd), None, &GAMUT)?;
    let mut number_of_flags = 0;

    for parameter in GAMUT {
        number_of_flags += match parameter {
            OpParameter::Flag { key } => {
                if op.params.boolean(key) {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
    if number_of_flags != 1 {
        return Err(Error::MissingParam(
            "curvature: must specify exactly one of flags prime/meridian/gaussian/mean/azimuthal"
                .to_string(),
        ));
    }

    // Check that we have a proper ellipsoid
    if op.params.text("ellps").is_ok() {
        let _ = Ellipsoid::named(op.params.text("ellps")?.as_str())?;
    }

    Ok(op)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curvature() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        // Missing argument
        let op = ctx.op("curvature");
        assert!(matches!(op, Err(Error::MissingParam(_))));

        // Too many arguments
        let op = ctx.op("curvature meridian gaussian");
        assert!(matches!(op, Err(Error::MissingParam(_))));

        // Unknown ellipsoid name
        let op = ctx.op("curvature ellps=non_existing meridian");
        assert!(matches!(op, Err(Error::NotFound(_, _))));

        // Regression test: Curvatures for a random range of latitudes
        let latitudes = [
            50f64, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0,
        ];

        #[allow(clippy::excessive_precision)]
        let prime_vertical_radii_of_curvature = [
            6_390_702.044_256_360,
            6_391_069.984_921_544,
            6_391_435.268_276_582,
            6_391_797.447_784_556,
            6_392_156.080_476_415,
            6_392_510.727_498_910,
            6_392_860.954_658_516,
            6_393_206.332_960_654,
            6_393_546.439_143_487,
            6_393_880.856_205_599,
            6_394_209.173_926_849,
        ];

        #[allow(clippy::excessive_precision)]
        let meridian_radii_of_curvature = [
            6_372_955.925_709_509_0,
            6_374_056.745_916_700_0,
            6_375_149.741_260_880_0,
            6_376_233.572_673_635_0,
            6_377_306.911_183_843_0,
            6_378_368.439_577_595_0,
            6_379_416.854_048_849_0,
            6_380_450.865_838_609_0,
            6_381_469.202_860_374_0,
            6_382_470.611_309_607_5,
            6_383_453.857_254_997_0,
        ];

        // Prime vertical
        let op = ctx.op("curvature prime ellps=GRS80")?;

        let mut operands = Vec::new();
        for lat in latitudes {
            operands.push(Coor2D([lat, 0.0]));
        }
        ctx.apply(op, Fwd, &mut operands)?;

        for (i, coord) in operands.iter().enumerate() {
            let n = coord[0];
            assert!((n - prime_vertical_radii_of_curvature[i]).abs() < 1e-9);
        }

        // Meridian
        let op = ctx.op("curvature meridian ellps=GRS80")?;

        let mut operands = Vec::new();
        for lat in latitudes {
            operands.push(Coor2D([lat, 0.0]));
        }
        ctx.apply(op, Fwd, &mut operands)?;

        for (i, coord) in operands.iter().enumerate() {
            let m = coord[0];
            assert!((m - meridian_radii_of_curvature[i]).abs() < 1e-9);
        }

        // Azimuthal
        let op = ctx.op("curvature azimuthal ellps=GRS80")?;

        // The alpha = 90 case is identical to the prime vertical case
        let mut operands = Vec::new();
        for lat in latitudes {
            operands.push(Coor2D([lat, 90.0]));
        }
        ctx.apply(op, Fwd, &mut operands)?;

        for (i, coord) in operands.iter().enumerate() {
            let m = coord[0];
            assert!((m - prime_vertical_radii_of_curvature[i]).abs() < 1e-9);
        }

        // The alpha = 0 case is identical to the meridian case
        let mut operands = Vec::new();
        for lat in latitudes {
            operands.push(Coor2D([lat, 0.0]));
        }
        ctx.apply(op, Fwd, &mut operands)?;

        for (i, coord) in operands.iter().enumerate() {
            let m = coord[0];
            assert!((m - meridian_radii_of_curvature[i]).abs() < 1e-9);
        }

        Ok(())
    }
}
