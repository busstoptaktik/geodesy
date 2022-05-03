use super::*;
use crate::Provider;
use std::io::BufRead;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let grid = &op.params.series["grid"];
    let h = GridHeader::gravsoft(grid)?;
    let mut successes = 0_usize;

    // Geoid
    if h.bands == 1 {
        for coord in operands {
            let d = h.interpolation(coord, grid);
            coord[2] -= d[0];
            successes += 1;
        }
        return Ok(successes);
    }

    // Datum shift
    for coord in operands {
        let d = h.interpolation(coord, grid);
        if h.bands == 1 {
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
    let grid = &op.params.series["grid"];
    let h = GridHeader::gravsoft(grid)?;
    let mut successes = 0_usize;

    // Geoid
    if h.bands == 1 {
        for coord in operands {
            let t = h.interpolation(coord, grid);
            coord[2] += t[0];
            successes += 1;
        }
        return Ok(successes);
    }

    // Datum shift - here we need to iterate in the inverse case
    for coord in operands {
        let mut t = *coord - h.interpolation(coord, grid);

        for _ in 0..5 {
            let d = t - *coord + h.interpolation(&t, grid);
            t = t - d;
            if d.dot(d).sqrt() < 1e-10 {
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
    params.series.insert("grid", grid);

    let fwd = InnerOp(fwd);
    let inv = InnerOp(inv);
    let descriptor = OpDescriptor::new(def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    let id = OpHandle::default();

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
    for i in 0..4 {
        if grid[i].abs() > 720. {
            return;
        }
    }

    // The header values are in decimal degrees
    for i in 0..6 {
        grid[i] = grid[i].to_radians();
    }

    // If we're handling a geoid grid, we're done: Grid values are in meters
    let h = GridHeader::gravsoft(grid).unwrap_or_default();
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

// Clamp input to range min..max
fn clamp<T>(input: T, min: T, max: T) -> T
where
    T: PartialOrd<T>,
{
    if input < min {
        return min;
    }
    if input > max {
        return max;
    }
    input
}

#[derive(Debug, Default)]
struct GridHeader {
    lat_0: f64, // Latitude of the first (typically northernmost) row of the grid
    lat_1: f64, // Latitude of the last (typically southernmost) row of the grid
    lon_0: f64, // Longitude of the first (typically westernmost) column of each row
    lon_1: f64, // Longitude of the last (typically easternmost) column of each row
    dlat: f64,  // Signed distance between two consecutive rows
    dlon: f64,  // Signed distance between two consecutive columns
    rows: usize,
    cols: usize,
    bands: usize,
    header_length: usize,
    last_valid_record_start: usize,
}

impl GridHeader {
    fn gravsoft(grid: &[f64]) -> Result<Self, Error> {
        let lat_0 = grid[1];
        let lat_1 = grid[0];
        let lon_0 = grid[2];
        let lon_1 = grid[3];
        let dlat = -grid[4];
        let dlon = grid[5];
        let rows = ((lat_1 - lat_0) / dlat + 1.5).floor() as usize;
        let cols = ((lon_1 - lon_0) / dlon + 1.5).floor() as usize;
        let bands = (grid.len() - 6_usize) / (rows * cols);
        let header_length = 6;
        let last_valid_record_start = header_length + (rows * cols - 1) * bands;

        let elements = rows * cols * bands;
        if elements == 0 || elements + header_length > grid.len() || bands < 1 {
            return Err(Error::General("Incomplete grid"));
        }

        Ok(GridHeader {
            lat_0,
            lat_1,
            lon_0,
            lon_1,
            dlat,
            dlon,
            rows,
            cols,
            bands,
            header_length,
            last_valid_record_start,
        })
    }

    #[allow(dead_code)]
    pub fn to_degrees(&self) -> Self {
        let lat_0 = self.lat_0.to_degrees();
        let lat_1 = self.lat_1.to_degrees();
        let lon_0 = self.lon_0.to_degrees();
        let lon_1 = self.lon_1.to_degrees();
        let dlat = self.dlat.to_degrees();
        let dlon = self.dlon.to_degrees();
        let rows = self.rows;
        let cols = self.cols;
        let bands = self.bands;
        let header_length = self.header_length;
        let last_valid_record_start = self.last_valid_record_start;
        Self {
            lat_0,
            lat_1,
            lon_0,
            lon_1,
            dlat,
            dlon,
            rows,
            cols,
            bands,
            header_length,
            last_valid_record_start,
        }
    }

    // Since we store the entire grid+header in a single vector, the interpolation
    // routine here looks strongly like a case of "writing Fortran 77 in Rust".
    // It is, however, one of the cases where a more extensive use of abstractions
    // leads to a significantly larger code base, much harder to maintain and
    // comprehend.
    pub fn interpolation(&self, coord: &Coord, grid: &[f64]) -> Coord {
        // The interpolation coordinate relative to the grid origin
        let rlon = coord[0] - self.lon_0;
        let rlat = coord[1] - self.lat_0;

        // The (row, column) of the lower left node of the grid cell containing
        // coord or, in the case of extrapolation, the nearest cell inside the grid.
        let row = (rlat / self.dlat).floor() as i64;
        let col = (rlon / self.dlon).floor() as i64;

        let col = clamp(col, 0_i64, (self.cols - 2) as i64) as usize;
        let row = clamp(row, 1_i64, (self.rows - 1) as i64) as usize;

        // Index of the first band element of each corner value
        #[rustfmt::skip]
        let (ll, lr, ur, ul) = (
            self.header_length + self.bands * (self.cols *  row      + col    ),
            self.header_length + self.bands * (self.cols *  row      + col + 1),
            self.header_length + self.bands * (self.cols * (row - 1) + col + 1),
            self.header_length + self.bands * (self.cols * (row - 1) + col    ),
        );

        // Cell relative, cell unit coordinates in a right handed CS (hence .abs())
        let rlon = (coord[0] - (self.lon_0 + col as f64 * self.dlon)) / self.dlon.abs();
        let rlat = (coord[1] - (self.lat_0 + row as f64 * self.dlat)) / self.dlat.abs();

        // Interpolate
        let mut left = Coord::origin();
        for i in 0..self.bands {
            left[i] = (1. - rlat) * grid[ll + i] + rlat * grid[ul + i];
        }
        let mut right = Coord::origin();
        for i in 0..self.bands {
            right[i] = (1. - rlat) * grid[lr + i] + rlat * grid[ur + i];
        }

        let mut result = Coord::origin();
        for i in 0..self.bands {
            result[i] = (1. - rlon) * left[i] + rlon * right[i];
        }
        result
    }
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
        let datum = GridHeader::gravsoft(&datumgrid)?;

        let mut geoidgrid = Vec::from(HEADER);
        geoidgrid.extend_from_slice(&GEOID[..]);
        normalize_gravsoft_grid_values(&mut geoidgrid);
        let geoid = GridHeader::gravsoft(&geoidgrid)?;

        let c = Coord::geo(58.75, 08.25, 0., 0.);

        let n = geoid.interpolation(&c, &geoidgrid);
        assert!((n[0] - 58.83).abs() < 0.1);

        let d = datum.interpolation(&c, &datumgrid);
        assert!(c.default_ellps_dist(&d.to_arcsec().to_radians()) < 1e-4);

        let c = Coord::geo(100., 50., 0., 0.);
        let d = datum.interpolation(&c, &datumgrid);
        assert!(c.default_ellps_dist(&d.to_arcsec().to_radians()) < 1e-4);

        Ok(())
    }

    #[test]
    fn gravsoft() -> Result<(), Error> {
        let mut prv = Minimal::default();
        let op = prv.op("gridshift grids=test.datum")?;
        let cph = Coord::geo(55.,12.,0.,0.);
        let mut data = [cph];

        prv.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        dbg!(res);
        assert!((res[0] - 55.0 * ( 1. + 1./3600.)).abs() < 1e-10);
        assert!((res[1] - 12.0 * ( 1. + 1./3600.)).abs() < 1e-10);

        prv.apply(op, Inv, &mut data)?;
        dbg!(data[0].to_degrees());
        assert!((data[0][0] - cph[0]).abs() < 1e-10);
        assert!((data[0][1] - cph[1]).abs() < 1e-10);

        Ok(())
    }
}
