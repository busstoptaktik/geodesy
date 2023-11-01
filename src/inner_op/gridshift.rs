/// Datum shift using grid interpolation.
use crate::authoring::*;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grids = &op.params.grids;
    let use_null_grid = op.params.boolean("null_grid");

    let mut successes = 0_usize;
    let n = operands.len();

    'points: for i in 0..n {
        let mut coord = operands.get_coord(i);

        for within in [0.0, 0.5] {
            for grid in grids.iter() {
                if let Some(d) = grid.interpolation(&coord, within) {
                    // Geoid
                    if grid.bands() == 1 {
                        coord[2] -= d[0];
                        operands.set_coord(i, &coord);
                        successes += 1;

                        continue 'points;
                    }

                    // Datum shift
                    coord[0] += d[0];
                    coord[1] += d[1];
                    operands.set_coord(i, &coord);
                    successes += 1;

                    continue 'points;
                }
            }
        }

        if use_null_grid {
            successes += 1;
            continue;
        }

        // No grid found so we stomp on the coordinate
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

    'points: for i in 0..n {
        let mut coord = operands.get_coord(i);

        for within in [0.0, 0.5] {
            for grid in grids.iter().rev() {
                if let Some(t) = grid.interpolation(&coord, within) {
                    // Geoid
                    if grid.bands() == 1 {
                        coord[2] += t[0];
                        operands.set_coord(i, &coord);
                        successes += 1;

                        continue 'points;
                    }

                    // Datum shift - here we need to iterate in the inverse case
                    let mut t = coord - t;

                    'iterate: for _ in 0..10 {
                        if let Some(t2) = grid.interpolation(&t, within) {
                            let d = t - coord + t2;
                            t = t - d;
                            // i.e. d.dot(d).sqrt() < 1e-10
                            if d.dot(d) < 1e-20 {
                                break 'iterate;
                            }
                            continue 'iterate;
                        }

                        if use_null_grid {
                            successes += 1;
                            break 'iterate;
                        }

                        // The iteration has wondered off the grid so we stomp on the coordinate
                        t = Coor4D::nan();
                        break 'iterate;
                    }

                    operands.set_coord(i, &t);
                    successes += 1;

                    continue 'points;
                }
            }
        }

        if use_null_grid {
            successes += 1;
            continue;
        }

        // No grid found so we stomp on the coordinate
        operands.set_coord(i, &Coor4D::nan());
    }
    successes
}
// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 3] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Texts { key: "grids", default: None },
    OpParameter::Real { key: "padding", default: Some(0.5) },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    for grid_name in params.texts("grids")?.clone() {
        if grid_name.ends_with("@null") {
            params.boolean.insert("null_grid");
            // @null can only be used as the final grid
            break;
        }

        if grid_name.contains("@") {
            if let Ok(grid) = ctx.get_grid(&grid_name) {
                params.grids.push(grid);
            }
            continue;
        }

        let grid = ctx.get_grid(&grid_name)?;
        params.grids.push(grid);
    }

    let fwd = InnerOp(fwd);
    let inv = InnerOp(inv);
    let descriptor = OpDescriptor::new(def, fwd, Some(inv));
    let steps = Vec::new();
    let id = OpHandle::new();

    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// ----- T E S T S ------------------------------------------------------------------

//#[cfg(with_plain)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gridshift() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=../../geodesy/datum/test.datum")?;
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
    fn multiple_grids() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx
            .op("gridshift grids=../../geodesy/datum/test.datum,../../geodesy/datum/test.datum")?;
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
    fn fails_without_null_grid() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=../../geodesy/datum/test.datum")?;

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
        let op = ctx.op("gridshift grids=../../geodesy/datum/test.datum, @null")?;

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
        let op = ctx.op("gridshift grids=@optional.gsb,../../geodesy/datum/test.datum")?;
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
    fn missing_grid() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=missing.gsb");
        assert!(op.is_err());

        Ok(())
    }
}

// See additional tests in src/grid/mod.rs
