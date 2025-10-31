/// Datum shift using grid interpolation.
use crate::authoring::*;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grids = &op.params.grids;
    let use_null_grid = op.params.boolean("null_grid");

    let mut successes = 0_usize;
    let n = operands.len();

    // Nothing to do?
    if grids.is_empty() {
        return n;
    }

    for i in 0..n {
        let mut coord = operands.get_coord(i);

        if let Some(d) = grids_at(grids, &coord, use_null_grid) {
            // Geoid
            if grids[0].bands() == 1 {
                coord[2] -= d[0];
                operands.set_coord(i, &coord);
                successes += 1;

                continue;
            }

            // Datum shift
            coord[0] += d[0];
            coord[1] += d[1];
            operands.set_coord(i, &coord);
            successes += 1;

            continue;
        }

        // No grid contained the point, so we stomp on the coordinate
        operands.set_coord(i, &Coor4D::nan());
    }

    successes
}

// ----- I N V E R S E --------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grids = &op.params.grids;
    let use_null_grid = op.params.boolean("null_grid");

    let mut successes = 0_usize;
    let n = operands.len();

    // Nothing to do?
    if grids.is_empty() {
        return n;
    }

    'points: for i in 0..n {
        let mut coord = operands.get_coord(i);
        if let Some(t) = grids_at(grids, &coord, use_null_grid) {
            // Geoid
            if grids[0].bands() == 1 {
                coord[2] += t[0];
                operands.set_coord(i, &coord);
                successes += 1;
                continue;
            }

            // Inverse case datum shift - iteration needed
            let mut t = coord - t;
            for _ in 0..10 {
                if let Some(t2) = grids_at(grids, &t, use_null_grid) {
                    let d = t - coord + t2;
                    t = t - d;
                    if d[0].hypot(d[1]) < 1e-12 {
                        operands.set_coord(i, &t);
                        successes += 1;
                        continue 'points;
                    }
                    continue;
                }

                // The iteration has wandered off the grids, so we stomp
                // on the coordinate and go on with the next
                operands.set_coord(i, &Coor4D::nan());
                continue 'points;
            }
        }
    }

    successes
}

// ----- C O N S T R U C T O R ------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 3] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Texts { key: "grids", default: None },
    OpParameter::Real { key: "padding", default: Some(0.5) },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.instantiated_as;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    for mut grid_name in params.texts("grids")?.clone() {
        let optional = grid_name.starts_with('@');
        if optional {
            grid_name = grid_name.trim_start_matches('@').to_string();
        }

        if grid_name == "null" {
            params.boolean.insert("null_grid");
            break; // ignore any additional grids after a null grid
        }

        match ctx.get_grid(&grid_name) {
            Ok(grid) => params.grids.push(grid),
            Err(e) => {
                if !optional {
                    return Err(e);
                }
            }
        }
    }

    let fwd = InnerOp(fwd);
    let inv = InnerOp(inv);
    let descriptor = OpDescriptor::new(def, fwd, Some(inv));

    Ok(Op {
        descriptor,
        params,
        steps: None,
    })
}

// ----- T E S T S ------------------------------------------------------------------

//#[cfg(with_plain)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::AngularUnits;

    #[test]
    fn gridshift() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=test.datum")?;
        let cph = Coor4D::geo(55., 12., 0., 0.);
        let mut data = [cph];

        ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!((res[0] - 55.015278).abs() < 1e-6);
        assert!((res[1] - 12.003333).abs() < 1e-6);

        ctx.apply(op, Inv, &mut data)?;
        assert!((data[0][0] - cph[0]).abs() < 1e-10);
        assert!((data[0][1] - cph[1]).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn ntv2() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=100800401.gsb")?;
        let bcn = Coor2D::geo(41.3874, 2.1686);
        let mut data = [bcn];

        ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!((res[0] - 41.38627500250805).abs() < 1e-8);
        assert!((res[1] - 2.167450821894838).abs() < 1e-8);

        ctx.apply(op, Inv, &mut data)?;
        assert!((data[0][0] - bcn[0]).abs() < 1e-10);
        assert!((data[0][1] - bcn[1]).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn multiple_grids() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=test.datum, test.datum")?;
        let cph = Coor4D::geo(55., 12., 0., 0.);
        let mut data = [cph];

        ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!((res[0] - 55.015278).abs() < 1e-6);
        assert!((res[1] - 12.003333).abs() < 1e-6);

        // Check that the reference counting works as expected
        Plain::clear_grids();
        Plain::clear_grids();

        ctx.apply(op, Inv, &mut data)?;
        assert!((data[0][0] - cph[0]).abs() < 1e-10);
        assert!((data[0][1] - cph[1]).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn fails_without_null_grid() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=test.datum")?;

        let ldn = Coor4D::geo(51.505, -0.09, 0., 0.);
        let mut data = [ldn];

        let successes = ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(successes, 0);
        assert!(data[0][0].is_nan());
        assert!(data[0][1].is_nan());

        Ok(())
    }

    #[test]
    fn passes_with_null_grid() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=test.datum, @null")?;

        let ldn = Coor4D::geo(51.505, -0.09, 0., 0.);
        let mut data = [ldn];

        let successes = ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert_eq!(successes, 1);
        assert_eq!(res[0], 51.505);
        assert_eq!(res[1], -0.09);

        let successes = ctx.apply(op, Inv, &mut data)?;
        assert_eq!(successes, 1);
        assert!((data[0][0] - ldn[0]).abs() < 1e-10);
        assert!((data[0][1] - ldn[1]).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn optional_grid() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=@test_subset.datum, @missing.gsb, test.datum")?;

        // Check that the Arc reference counting and the clear-GRIDS-by-instantiation
        // works together correctly (i.e. the Arcs used by op are kept alive when
        // GRIDS is cleared on the instantiation of ctx2)
        let mut ctx2 = Plain::default();
        let op2 = ctx2.op("gridshift grids=@test_subset.datum, @missing.gsb, test.datum")?;

        // Copenhagen is outside of the (optional, but present, subset grid)
        let cph = Coor4D::geo(55., 12., 0., 0.);
        let mut data = [cph];
        let mut data2 = [cph];

        ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!((res[0] - 55.015278).abs() < 1e-6);
        assert!((res[1] - 12.003333).abs() < 1e-6);

        // Same procedure on the other context gives same result
        ctx2.apply(op2, Fwd, &mut data2)?;
        assert_eq!(data, data2);

        ctx.apply(op, Inv, &mut data)?;
        assert!((data[0][0] - cph[0]).abs() < 1e-10);
        assert!((data[0][1] - cph[1]).abs() < 1e-10);

        // Same procedure on the other context gives same result
        ctx2.apply(op2, Inv, &mut data2)?;
        assert_eq!(data, data2);

        // Havnebyen (a small town with a large geodetic installation) is inside the subset grid
        let haby = Coor4D::geo(55.97, 11.33, 0., 0.);
        let mut data = [haby];
        let expected_correction = Coor4D([11.331, 55.971, 0., 0.]);
        ctx.apply(op, Fwd, &mut data)?;
        let correction = ((data[0] - haby) * Coor4D([3600., 3600., 3600., 3600.])).to_degrees();
        assert!((correction - expected_correction)[0].abs() < 1e-6);
        assert!((correction - expected_correction)[1].abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn missing_grid() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=missing.gsb");
        assert!(op.is_err());

        Ok(())
    }
}

// See additional tests in src/grid/mod.rs
