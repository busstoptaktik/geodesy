// Datum shift using grid interpolation.
use super::*;
use crate::Provider;
use std::io::BufRead;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let grid = &op.params.grids["grid"];
    let mut successes = 0_usize;

    // Geoid
    if grid.bands == 1 {
        for coord in operands {
            let d = grid.interpolation(coord, None);
            coord[2] -= d[0];
            successes += 1;
        }
        return Ok(successes);
    }

    // Datum shift
    for coord in operands {
        let d = grid.interpolation(coord, None);
        if grid.bands == 1 {
            coord[2] -= d[0];
            continue;
        }
        coord[0] += d[0];
        coord[1] += d[1];
        successes += 1;
    }
    Ok(successes)
}

// ----- I N V E R S E --------------------------------------------------------------

fn inv(op: &Op, _prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let grid = &op.params.grids["grid"];
    let mut successes = 0_usize;

    // Geoid
    if grid.bands == 1 {
        for coord in operands {
            let t = grid.interpolation(coord, None);
            coord[2] += t[0];
            successes += 1;
        }
        return Ok(successes);
    }

    // Datum shift - here we need to iterate in the inverse case
    for coord in operands {
        let mut t = *coord - grid.interpolation(coord, None);

        for _ in 0..10 {
            let d = t - *coord + grid.interpolation(&t, None);
            t = t - d;
            // i.e. d.dot(d).sqrt() < 1e-10
            if d.dot(d) < 1e-20 {
                break;
            }
        }

        *coord = t;
        successes += 1;
    }

    Ok(successes)
}

// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 3] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "grids", default: None },
    OpParameter::Real { key: "padding", default: Some(0.5) },
];

pub fn new(parameters: &RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    let grid_file_name = params.text("grids")?;
    let grid = gravsoft_grid_reader(&grid_file_name, provider)?;
    let grid = Grid::plain(&grid)?;
    params.grids.insert("grid", grid);

    let fwd = InnerOp(fwd);
    let inv = InnerOp(inv);
    let descriptor = OpDescriptor::new(def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    let id = OpHandle::new();

    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// If the Gravsoft grid appears to be in angular units, convert it to radians
fn normalize_gravsoft_grid_values(grid: &mut [f64]) {
    // If any boundary is outside of [-720; 720], the grid must (by a wide margin) be
    // in projected coordinates and the correction in meters, so we simply return.
    for g in grid.iter().take(4) {
        if g.abs() > 720. {
            return;
        }
    }

    // The header values are in decimal degrees
    for g in grid.iter_mut().take(6) {
        *g = g.to_radians();
    }

    // If we're handling a geoid grid, we're done: Grid values are in meters
    let h = Grid::plain(grid).unwrap_or_default();
    if h.bands < 2 {
        return;
    }

    // The grid values are in minutes-of-arc and in latitude/longitude order.
    // Swap them and convert into radians.
    // TODO: handle 3-D data with 3rd coordinate in meters
    for i in 6..grid.len() {
        grid[i] = (grid[i] / 3600.0).to_radians();
        if i % 2 == 1 {
            grid.swap(i, i - 1);
        }
    }
}

// Read a gravsoft grid. Discard '#'-style comments. Return everything as a single Vec
fn gravsoft_grid_reader(name: &str, provider: &dyn Provider) -> Result<Vec<f64>, Error> {
    let buf = provider.get_blob(name)?;
    let all = std::io::BufReader::new(buf.as_slice());
    let mut grid = Vec::<f64>::new();

    for line in all.lines() {
        // Remove comments
        let line = line?;
        let line = line.split('#').collect::<Vec<_>>()[0];
        // Convert to f64
        for item in line.split_whitespace() {
            grid.push(item.parse::<f64>().unwrap_or(0.));
        }
    }
    if grid.len() < 6 {
        return Err(Error::General("Incomplete grid"));
    }

    // Handle linear/angular conversions
    normalize_gravsoft_grid_values(&mut grid);
    Ok(grid)
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    const HEADER: [f64; 6] = [54., 58., 8., 16., 1., 1.];

    #[rustfmt::skip]
    const GEOID: [f64; 5*9] = [
        58.08, 58.09, 58.10, 58.11, 58.12, 58.13, 58.14, 58.15, 58.16,
        57.08, 57.09, 57.10, 57.11, 57.12, 57.13, 57.14, 57.15, 57.16,
        56.08, 56.09, 56.10, 56.11, 56.12, 56.13, 56.14, 56.15, 56.16,
        55.08, 55.09, 55.10, 55.11, 55.12, 55.13, 55.14, 55.15, 55.16,
        54.08, 54.09, 54.10, 54.11, 54.12, 54.13, 54.14, 54.15, 54.16,
    ];

    #[allow(dead_code)]
    #[rustfmt::skip]
    const DATUM: [f64; 5*2*9] = [
        58., 08., 58., 09., 58., 10., 58., 11., 58., 12., 58., 13., 58., 14., 58., 15., 58., 16.,
        57., 08., 57., 09., 57., 10., 57., 11., 57., 12., 57., 13., 57., 14., 57., 15., 57., 16.,
        56., 08., 56., 09., 56., 10., 56., 11., 56., 12., 56., 13., 56., 14., 56., 15., 56., 16.,
        55., 08., 55., 09., 55., 10., 55., 11., 55., 12., 55., 13., 55., 14., 55., 15., 55., 16.,
        54., 08., 54., 09., 54., 10., 54., 11., 54., 12., 54., 13., 54., 14., 54., 15., 54., 16.,
    ];

    #[test]
    fn grid_header() -> Result<(), Error> {
        let mut datumgrid = Vec::from(HEADER);
        datumgrid.extend_from_slice(&DATUM[..]);
        normalize_gravsoft_grid_values(&mut datumgrid);
        let datum = Grid::plain(&datumgrid)?;

        let mut geoidgrid = Vec::from(HEADER);
        geoidgrid.extend_from_slice(&GEOID[..]);
        normalize_gravsoft_grid_values(&mut geoidgrid);
        let geoid = Grid::plain(&geoidgrid)?;

        let c = Coord::geo(58.75, 08.25, 0., 0.);

        let n = geoid.interpolation(&c, None);
        assert!((n[0] - 58.83).abs() < 0.1);

        let d = datum.interpolation(&c, None);
        assert!(c.default_ellps_dist(&d.to_arcsec().to_radians()) < 1.0);

        // Extrapolation
        let c = Coord::geo(100., 50., 0., 0.);
        let d = datum.interpolation(&c, None);
        assert!(c.default_ellps_dist(&d.to_arcsec().to_radians()) < 25.0);

        Ok(())
    }

    #[test]
    fn gridshift() -> Result<(), Error> {
        let mut prv = Minimal::default();
        let op = prv.op("gridshift grids=test.datum")?;
        let cph = Coord::geo(55., 12., 0., 0.);
        let mut data = [cph];

        prv.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!((res[0] - 55.015278).abs() < 1e-6);
        assert!((res[1] - 12.003333).abs() < 1e-6);

        prv.apply(op, Inv, &mut data)?;
        assert!((data[0][0] - cph[0]).abs() < 1e-10);
        assert!((data[0][1] - cph[1]).abs() < 1e-10);

        Ok(())
    }
}
