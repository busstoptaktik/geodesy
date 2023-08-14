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
            let mut coord = operands.get_coord(i);
            coord[0] = ellps.prime_vertical_radius_of_curvature(coord[0].to_radians());
            operands.set_coord(i, &coord);
            successes += 1;
        }
        return successes;
    }

    if meridional {
        for i in sliced {
            let mut coord = operands.get_coord(i);
            coord[0] = ellps.meridian_radius_of_curvature(coord[0].to_radians());
            operands.set_coord(i, &coord);
            successes += 1;
        }
        return successes;
    }

    if gaussian {
        for i in sliced {
            let mut coord = operands.get_coord(i);
            let lat = coord[0].to_radians();
            let m = ellps.meridian_radius_of_curvature(lat);
            let n = ellps.prime_vertical_radius_of_curvature(lat);
            coord[0] = (n * m).sqrt();
            operands.set_coord(i, &coord);
            successes += 1;
        }
        return successes;
    }

    if mean {
        for i in sliced {
            let mut coord = operands.get_coord(i);
            let lat = coord[0].to_radians();
            let m = ellps.meridian_radius_of_curvature(lat);
            let n = ellps.prime_vertical_radius_of_curvature(lat);
            coord[0] = 2.0 * (n.recip() + m.recip()).recip();
            operands.set_coord(i, &coord);
            successes += 1;
        }
        return successes;
    }

    if azimuthal {
        for i in sliced {
            let mut coord = operands.get_coord(i);
            let lat = coord[0].to_radians();
            let azi = coord[1].to_radians();
            let m = ellps.meridian_radius_of_curvature(lat);
            let n = ellps.prime_vertical_radius_of_curvature(lat);
            let (s, c) = azi.sin_cos();
            coord[0] = (c * c / m + s * s / n).recip();
            operands.set_coord(i, &coord);
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

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let op = Op::plain(parameters, InnerOp(fwd), None, &GAMUT, ctx)?;
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
        let prime_vertical_radii_of_curvature = [
            6390702.044256360,
            6391069.984921544,
            6391435.268276582,
            6391797.447784556,
            6392156.080476415,
            6392510.727498910,
            6392860.954658516,
            6393206.332960654,
            6393546.439143487,
            6393880.856205599,
            6394209.173926849,
        ];

        let meridian_radii_of_curvature = [
            6372955.9257095090,
            6374056.7459167000,
            6375149.7412608800,
            6376233.5726736350,
            6377306.9111838430,
            6378368.4395775950,
            6379416.8540488490,
            6380450.8658386090,
            6381469.2028603740,
            6382470.6113096075,
            6383453.8572549970,
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
