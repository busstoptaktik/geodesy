// Datum shift using grid interpolation.
use crate::operator_authoring::*;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grid = &op.params.grids["grid"];
    let mut successes = 0_usize;
    let n = operands.len();

    // Geoid
    if grid.bands == 1 {
        for i in 0..n {
            let mut coord = operands.get(i);
            let d = grid.interpolation(&coord, None);
            coord[2] -= d[0];
            successes += 1;
            operands.set(i, &coord);
        }
        return successes;
    }

    // Datum shift
    for i in 0..n {
        let mut coord = operands.get(i);
        let d = grid.interpolation(&coord, None);
        coord[0] += d[0];
        coord[1] += d[1];
        operands.set(i, &coord);
        successes += 1;
    }
    successes
}

// ----- I N V E R S E --------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grid = &op.params.grids["grid"];
    let mut successes = 0_usize;
    let n = operands.len();

    // Geoid
    if grid.bands == 1 {
        for i in 0..n {
            let mut coord = operands.get(i);
            let t = grid.interpolation(&coord, None);
            coord[2] += t[0];
            operands.set(i, &coord);
            successes += 1;
        }
        return successes;
    }

    // Datum shift - here we need to iterate in the inverse case
    for i in 0..n {
        let coord = operands.get(i);
        let mut t = coord - grid.interpolation(&coord, None);

        for _ in 0..10 {
            let d = t - coord + grid.interpolation(&t, None);
            t = t - d;
            // i.e. d.dot(d).sqrt() < 1e-10
            if d.dot(d) < 1e-20 {
                break;
            }
        }

        operands.set(i, &t);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 3] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "grids", default: None },
    OpParameter::Real { key: "padding", default: Some(0.5) },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    let grid_file_name = params.text("grids")?;
    let buf = ctx.get_blob(&grid_file_name)?;

    let grid = Grid::gravsoft(&buf)?;
    params.grids.insert("grid", grid);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gridshift() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=test.datum")?;
        let cph = Coord::geo(55., 12., 0., 0.);
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
}

// See additional tests in src/grid/mod.rs
