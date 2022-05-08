use crate::internal::*;

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

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Grid {
    lat_0: f64, // Latitude of the first (typically northernmost) row of the grid
    lat_1: f64, // Latitude of the last (typically southernmost) row of the grid
    lon_0: f64, // Longitude of the first (typically westernmost) column of each row
    lon_1: f64, // Longitude of the last (typically easternmost) column of each row
    dlat: f64,  // Signed distance between two consecutive rows
    dlon: f64,  // Signed distance between two consecutive columns
    rows: usize,
    cols: usize,
    pub bands: usize,
    offset: usize,
    last_valid_record_start: usize,
    grid: Vec<f32>,
}

impl Grid {
    pub fn plain(raw: &[f64]) -> Result<Self, Error> {
        if raw.len() < 6 {
            return Err(Error::General("Incomplete grid"));
        }

        let lat_0 = raw[1];
        let lat_1 = raw[0];
        let lon_0 = raw[2];
        let lon_1 = raw[3];
        let dlat = -raw[4];
        let dlon = raw[5];
        let rows = ((lat_1 - lat_0) / dlat + 1.5).floor() as usize;
        let cols = ((lon_1 - lon_0) / dlon + 1.5).floor() as usize;
        let bands = (raw.len() - 6_usize) / (rows * cols);
        let offset = 6;

        let elements = rows * cols * bands;
        if elements == 0 || elements + offset > raw.len() || bands < 1 {
            return Err(Error::General("Incomplete grid"));
        }

        // Extract the grid part, and convert it to f32
        let grid: Vec<f32> = raw[offset..].iter().map(|x| *x as f32).collect();
        if elements != grid.len() {
            return Err(Error::General("Malformed grid"));
        }

        let offset = 0;
        let last_valid_record_start = offset + (rows * cols - 1) * bands;

        Ok(Grid {
            lat_0,
            lat_1,
            lon_0,
            lon_1,
            dlat,
            dlon,
            rows,
            cols,
            bands,
            offset,
            last_valid_record_start,
            grid,
        })
    }

    // Since we store the entire grid in a single vector, the interpolation
    // routine here looks strongly like a case of "writing Fortran 77 in Rust".
    // It is, however, one of the cases where a more extensive use of abstractions
    // leads to a significantly larger code base, much harder to maintain and
    // comprehend.
    pub fn interpolation(&self, coord: &Coord, grid: Option<&[f32]>) -> Coord {
        let grid = grid.unwrap_or(&self.grid);

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
            self.offset + self.bands * (self.cols *  row      + col    ),
            self.offset + self.bands * (self.cols *  row      + col + 1),
            self.offset + self.bands * (self.cols * (row - 1) + col + 1),
            self.offset + self.bands * (self.cols * (row - 1) + col    ),
        );

        // Cell relative, cell unit coordinates in a right handed CS (hence .abs())
        let rlon = (coord[0] - (self.lon_0 + col as f64 * self.dlon)) / self.dlon.abs();
        let rlat = (coord[1] - (self.lat_0 + row as f64 * self.dlat)) / self.dlat.abs();

        // Interpolate
        let mut left = Coord::origin();
        for i in 0..self.bands {
            left[i] = (1. - rlat) * grid[ll + i] as f64 + rlat * grid[ul + i] as f64;
        }
        let mut right = Coord::origin();
        for i in 0..self.bands {
            right[i] = (1. - rlat) * grid[lr + i] as f64 + rlat * grid[ur + i] as f64;
        }

        let mut result = Coord::origin();
        for i in 0..self.bands {
            result[i] = (1. - rlon) * left[i] + rlon * right[i];
        }
        result
    }
}

// ----- T E S T S ------------------------------------------------------------------

// The tests for Grid are placed in src/inner_op/gridshift.rs
