//! Grid characteristics and interpolation.

use crate::prelude::*;
use std::{fmt::Debug, io::BufRead};

pub trait Grid: Debug {
    fn bands(&self) -> usize;
    fn contains(&self, position: Coor4D) -> bool;
    // NOTE: `grid` is included for backwards compatibility but could be removed
    fn interpolation(&self, coord: &Coor4D, grid: Option<&[f32]>) -> Coor4D;
}

// NOTE: Should this be renamed PlainGrid? Then rename the trait to Grid?
/// Grid characteristics and interpolation.
///
/// The actual grid may be part of the `Grid` struct, or
/// provided externally (presumably by a [Context](crate::Context)).
///
/// In principle grid format agnostic, but includes a parser for
/// geodetic grids in the Gravsoft format.
#[derive(Debug, Default, Clone)]
pub struct BaseGrid {
    lat_0: f64, // Latitude of the first (typically northernmost) row of the grid
    lat_1: f64, // Latitude of the last (typically southernmost) row of the grid
    lon_0: f64, // Longitude of the first (typically westernmost) column of each row
    lon_1: f64, // Longitude of the last (typically easternmost) column of each row
    dlat: f64,  // Signed distance between two consecutive rows
    dlon: f64,  // Signed distance between two consecutive columns
    rows: usize,
    cols: usize,
    pub bands: usize,
    offset: usize, // typically 0, but may be any number for externally stored grids
    #[allow(dead_code)]
    last_valid_record_start: usize,
    grid: Vec<f32>, // May be zero sized in cases where the Context provides access to an externally stored grid
}

impl Grid for BaseGrid {
    fn bands(&self) -> usize {
        self.bands
    }

    /// Determine whether a given coordinate falls within the grid borders.
    /// "On the border" qualifies as within.
    fn contains(&self, position: Coor4D) -> bool {
        // We start by assuming that the last row (latitude) is the southernmost
        let mut min = self.lat_1;
        let mut max = self.lat_0;
        // If it's not, we swap
        if self.dlat > 0. {
            (min, max) = (max, min)
        }
        if position[1] != position[1].clamp(min, max) {
            return false;
        }

        // The default assumption is the other way round for columns (longitudes)
        min = self.lon_0;
        max = self.lon_1;
        // If it's not, we swap
        if self.dlon < 0. {
            (min, max) = (max, min)
        }
        if position[0] != position[0].clamp(min, max) {
            return false;
        }

        // If we fell through all the way to the bottom, we're inside the grid
        true
    }

    // Since we store the entire grid in a single vector, the interpolation
    // routine here looks strongly like a case of "writing Fortran 77 in Rust".
    // It is, however, one of the cases where a more extensive use of abstractions
    // leads to a significantly larger code base, much harder to maintain and
    // comprehend.
    fn interpolation(&self, coord: &Coor4D, grid: Option<&[f32]>) -> Coor4D {
        let grid = grid.unwrap_or(&self.grid);

        // The interpolation coordinate relative to the grid origin
        let rlon = coord[0] - self.lon_0;
        let rlat = coord[1] - self.lat_0;

        // The (row, column) of the lower left node of the grid cell containing
        // coord or, in the case of extrapolation, the nearest cell inside the grid.
        let row = (rlat / self.dlat).floor() as i64;
        let col = (rlon / self.dlon).floor() as i64;

        // let col = clamp(col, 0_i64, (self.cols - 2) as i64) as usize;
        // let row = clamp(row, 1_i64, (self.rows - 1) as i64) as usize;
        let col = col.clamp(0_i64, (self.cols - 2) as i64) as usize;
        let row = row.clamp(1_i64, (self.rows - 1) as i64) as usize;

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
        let mut left = Coor4D::origin();
        for i in 0..self.bands {
            left[i] = (1. - rlat) * grid[ll + i] as f64 + rlat * grid[ul + i] as f64;
        }
        let mut right = Coor4D::origin();
        for i in 0..self.bands {
            right[i] = (1. - rlat) * grid[lr + i] as f64 + rlat * grid[ur + i] as f64;
        }

        let mut result = Coor4D::origin();
        for i in 0..self.bands {
            result[i] = (1. - rlon) * left[i] + rlon * right[i];
        }
        result
    }
}

impl BaseGrid {
    pub fn plain(
        header: &[f64],
        grid: Option<&[f32]>,
        offset: Option<usize>,
    ) -> Result<Self, Error> {
        if header.len() < 7 {
            return Err(Error::General("Incomplete grid"));
        }

        let lat_0 = header[1];
        let lat_1 = header[0];
        let lon_0 = header[2];
        let lon_1 = header[3];
        let dlat = -header[4];
        let dlon = header[5];
        let bands = header[6] as usize;
        let rows = ((lat_1 - lat_0) / dlat + 1.5).floor() as usize;
        let cols = ((lon_1 - lon_0) / dlon + 1.5).floor() as usize;
        let elements = rows * cols * bands;

        let offset = offset.unwrap_or(0);
        let last_valid_record_start = offset + (rows * cols - 1) * bands;

        let grid = Vec::from(grid.unwrap_or(&[]));

        if elements == 0 || (offset == 0 && elements > grid.len()) || bands < 1 {
            return Err(Error::General("Malformed grid"));
        }

        Ok(BaseGrid {
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

    pub fn gravsoft(buf: &[u8]) -> Result<Self, Error> {
        let (header, grid) = gravsoft_grid_reader(buf)?;
        BaseGrid::plain(&header, Some(&grid), None)
    }
}

// If the Gravsoft grid appears to be in angular units, convert it to radians
fn normalize_gravsoft_grid_values(header: &mut [f64], grid: &mut [f32]) {
    // If any boundary is outside of [-720; 720], the grid must (by a wide margin) be
    // in projected coordinates and the correction in meters, so we simply return.
    for h in header.iter().take(4) {
        if h.abs() > 720. {
            return;
        }
    }

    // The header values are in decimal degrees
    for h in header.iter_mut().take(6) {
        *h = h.to_radians();
    }

    // If we're handling a geoid grid, we're done: Grid values are in meters
    let h = BaseGrid::plain(header, Some(grid), None).unwrap_or_default();
    if h.bands == 1 {
        return;
    }

    // For horizontal datum shifts, the grid values are in minutes-of-arc
    // and in latitude/longitude order. Swap them and convert into radians.
    if h.bands == 2 {
        for i in 0..grid.len() {
            grid[i] = (grid[i] / 3600.0).to_radians();
            if i % 2 == 1 {
                grid.swap(i, i - 1);
            }
        }
        return;
    }

    // For deformation grids, the grid values are in millimeters/year
    // and in latitude/longitude/height order. Swap them and convert
    // to meters/year
    if h.bands == 3 {
        for i in 0..grid.len() {
            if i % 3 == 0 {
                grid.swap(i, i + 1);
            }
            grid[i] /= 1000.0;
        }
    }
}

// Read a gravsoft grid. Discard '#'-style comments
fn gravsoft_grid_reader(buf: &[u8]) -> Result<(Vec<f64>, Vec<f32>), Error> {
    let all = std::io::BufReader::new(buf);
    let mut grid = Vec::<f32>::new();
    let mut header = Vec::<f64>::new();

    for line in all.lines() {
        // Remove comments
        let line = line?;
        let line = line.split('#').collect::<Vec<_>>()[0];
        // Convert to f64
        for item in line.split_whitespace() {
            let value = item.parse::<f64>().unwrap_or(f64::NAN);
            // In Gravsoft grids, the header is the first 6 numbers of the file
            if header.len() < 6 {
                header.push(value);
            } else {
                grid.push(value as f32);
            }
        }
    }

    if header.len() < 6 {
        return Err(Error::General("Incomplete Gravsoft header"));
    }

    // Count the number of bands
    let lat_0 = header[1];
    let lat_1 = header[0];
    let lon_0 = header[2];
    let lon_1 = header[3];
    let dlat = -header[4]; // minus because rows go from north to south
    let dlon = header[5];
    let rows = ((lat_1 - lat_0) / dlat + 1.5).floor() as usize;
    let cols = ((lon_1 - lon_0) / dlon + 1.5).floor() as usize;
    let bands = grid.len() / (rows * cols);
    if (rows * cols * bands) > grid.len() || bands < 1 {
        return Err(Error::General("Incomplete Gravsoft grid"));
    }

    if (rows * cols * bands) != grid.len() {
        return Err(Error::General(
            "Unrecognized material at end of Gravsoft grid",
        ));
    }

    if bands > 3 {
        return Err(Error::General(
            "Unsupported number of bands in Gravsoft grid",
        ));
    }

    header.push(bands as f64);

    // Handle linear/angular conversions
    normalize_gravsoft_grid_values(&mut header, &mut grid);
    Ok((header, grid))
}

// ----- T E S T S ------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: [f64; 6] = [54., 58., 8., 16., 1., 1.];

    #[rustfmt::skip]
    const GEOID: [f32; 5*9] = [
        58.08, 58.09, 58.10, 58.11, 58.12, 58.13, 58.14, 58.15, 58.16,
        57.08, 57.09, 57.10, 57.11, 57.12, 57.13, 57.14, 57.15, 57.16,
        56.08, 56.09, 56.10, 56.11, 56.12, 56.13, 56.14, 56.15, 56.16,
        55.08, 55.09, 55.10, 55.11, 55.12, 55.13, 55.14, 55.15, 55.16,
        54.08, 54.09, 54.10, 54.11, 54.12, 54.13, 54.14, 54.15, 54.16,
    ];

    #[allow(dead_code)]
    #[rustfmt::skip]
    const DATUM: [f32; 5*2*9] = [
        58., 08., 58., 09., 58., 10., 58., 11., 58., 12., 58., 13., 58., 14., 58., 15., 58., 16.,
        57., 08., 57., 09., 57., 10., 57., 11., 57., 12., 57., 13., 57., 14., 57., 15., 57., 16.,
        56., 08., 56., 09., 56., 10., 56., 11., 56., 12., 56., 13., 56., 14., 56., 15., 56., 16.,
        55., 08., 55., 09., 55., 10., 55., 11., 55., 12., 55., 13., 55., 14., 55., 15., 55., 16.,
        54., 08., 54., 09., 54., 10., 54., 11., 54., 12., 54., 13., 54., 14., 54., 15., 54., 16.,
    ];

    #[test]
    fn grid_header() -> Result<(), Error> {
        // Create a datum correction grid (2 bands)
        let mut datum_header = Vec::from(HEADER);
        datum_header.push(2_f64); // 2 bands
        let mut datum_grid = Vec::from(DATUM);
        normalize_gravsoft_grid_values(&mut datum_header, &mut datum_grid);
        let datum = BaseGrid::plain(&datum_header, Some(&datum_grid), None)?;

        // Create a geoid grid (1 band)
        let mut geoid_header = Vec::from(HEADER);
        geoid_header.push(1_f64); // 1 band
        let mut geoid_grid = Vec::from(GEOID);
        normalize_gravsoft_grid_values(&mut geoid_header, &mut geoid_grid);
        let geoid = BaseGrid::plain(&geoid_header, Some(&geoid_grid), None)?;

        let c = Coor4D::geo(58.75, 08.25, 0., 0.);
        assert_eq!(geoid.contains(c), false);

        let n = geoid.interpolation(&c, None);
        assert!((n[0] - 58.83).abs() < 0.1);

        let d = datum.interpolation(&c, None);
        assert!(c.default_ellps_dist(&d.to_arcsec().to_radians()) < 1.0);

        // Extrapolation
        let c = Coor4D::geo(100., 50., 0., 0.);
        // ...with output converted back to arcsec
        let d = datum.interpolation(&c, None).to_arcsec();

        // The grid is constructed to make the position in degrees equal to
        // the extrapolation value in arcsec.
        // Even for this case of extreme extrapolation, we expect the difference
        // to be less than 1/10_000 of an arcsec (i.e. approx 3 mm)
        assert!(c.to_degrees().hypot2(&d) < 1e-4);
        // Spelled out
        assert!((50.0 - d[0]).hypot(100.0 - d[1]) < 1e-4);

        // Interpolation
        let c = Coor4D::geo(55.06, 12.03, 0., 0.);
        // Check that we're not extrapolating
        assert_eq!(datum.contains(c), true);
        // ...with output converted back to arcsec
        let d = datum.interpolation(&c, None).to_arcsec();
        // We can do slightly better for interpolation than for extrapolation,
        // but the grid values are f32, so we have only approx 7 significant
        // figures...
        assert!(c.to_degrees().hypot2(&d) < 1e-5);

        Ok(())
    }
}

// Additional tests for Grid in src/inner_op/gridshift.rs
