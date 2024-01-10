//! Grid characteristics and interpolation.

pub mod ntv2;
use crate::prelude::*;
use std::{fmt::Debug, io::BufRead, sync::Arc};

pub trait Grid: Debug + Sync + Send {
    fn bands(&self) -> usize;
    /// Returns true if `coord` is contained by `self` or lies within a margin of
    /// `margin` grid cell units. Typically `margin` should be on the order of 1
    fn contains(&self, coord: &Coor4D, margin: f64) -> bool;
    /// Returns `None` if the grid or any of its sub-grids do not contain the point.
    /// **Contain** is in the sense of the `contains` method, i.e. the point is
    /// considered contained if it is inside a margin of `margin` grid units of
    /// the grid.
    fn at(&self, at: &Coor4D, margin: f64) -> Option<Coor4D>;
}

/// Grid characteristics and interpolation.
///
/// The actual grid may be part of the `BaseGrid` struct, or
/// provided externally (presumably by a [Context](crate::Context)).
///
/// In principle grid format agnostic, but includes a parser for
/// geodetic grids in the Gravsoft format.
#[derive(Debug, Default, Clone)]
pub struct BaseGrid {
    lat_n: f64, // Latitude of the first (typically northernmost) row of the grid
    lat_s: f64, // Latitude of the last (typically southernmost) row of the grid
    lon_w: f64, // Longitude of the first (typically westernmost) column of each row
    lon_e: f64, // Longitude of the last (typically easternmost) column of each row
    dlat: f64,  // Signed distance between two consecutive rows
    dlon: f64,  // Signed distance between two consecutive columns
    rows: usize,
    cols: usize,
    pub bands: usize,
    offset: usize,  // typically 0, but may be any number for externally stored grids
    grid: Vec<f32>, // May be zero sized in cases where the Context provides access to an externally stored grid
}

impl Grid for BaseGrid {
    fn bands(&self) -> usize {
        self.bands
    }

    /// Determine whether a given coordinate falls within the grid borders + margin.
    /// "On the border" qualifies as within.
    fn contains(&self, position: &Coor4D, margin: f64) -> bool {
        // We start by assuming that the last row (latitude) is the southernmost
        let mut min = self.lat_s;
        let mut max = self.lat_n;

        // If it's not, we swap
        if self.dlat > 0. {
            (min, max) = (max, min)
        }

        let grace = margin * self.dlat.abs();
        if position[1] != position[1].clamp(min - grace, max + grace) {
            return false;
        }

        // The default assumption is the other way round for columns (longitudes)
        min = self.lon_w;
        max = self.lon_e;
        // If it's not, we swap
        if self.dlon < 0. {
            (min, max) = (max, min)
        }

        let grace = margin * self.dlon.abs();
        if position[0] != position[0].clamp(min - grace, max + grace) {
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
    fn at(&self, at: &Coor4D, margin: f64) -> Option<Coor4D> {
        if !self.contains(at, margin) {
            return None;
        };

        let grid = &self.grid;

        // For now, we support top-to-bottom, left-to-right scan order only.
        // This is the common case for most non-block grid formats, with
        // NTv2 the odd man out. But since we normalize the NTv2 scan order
        // during parsing, we just cruise along here
        let dlat = self.dlat.abs();
        let dlon = self.dlon.abs();

        // The interpolation coordinate relative to the grid origin
        let rlon = at[0] - self.lon_w;
        let rlat = self.lat_n - at[1];

        // The (row, column) of the lower left node of the grid cell containing
        // the interpolation coordinate - or, in the case of extrapolation:
        // the nearest cell inside the grid.
        let row = (rlat / dlat).ceil() as i64;
        let col = (rlon / dlon).floor() as i64;

        let col = col.clamp(0_i64, (self.cols - 2) as i64) as usize;
        let row = row.clamp(1_i64, (self.rows - 1) as i64) as usize;

        // Index of the first band element of each corner value
        #[rustfmt::skip]
        let (ll, lr, ul, ur) = (
            self.offset + self.bands * (self.cols *  row      + col    ),
            self.offset + self.bands * (self.cols *  row      + col + 1),
            self.offset + self.bands * (self.cols * (row - 1) + col    ),
            self.offset + self.bands * (self.cols * (row - 1) + col + 1),
        );

        let ll_lon = self.lon_w + col as f64 * dlon;
        let ll_lat = self.lat_n - row as f64 * dlat;

        // Cell relative, cell unit coordinates in a right handed CS
        let rlon = (at[0] - ll_lon) / dlon;
        let rlat = (at[1] - ll_lat) / dlat;

        // We cannot return more than 4 bands in a Coor4D, so we ignore
        // any exceeding bands
        let bands = self.bands.min(4);
        let mut left = Coor4D::origin();

        // Interpolate (or extrapolate, if we're outside of the physical grid)
        for i in 0..bands {
            let lower = grid[ll + i] as f64;
            let upper = grid[ul + i] as f64;
            left[i] = (1. - rlat) * lower + rlat * upper;
        }
        let mut right = Coor4D::origin();
        for i in 0..bands {
            let lower = grid[lr + i] as f64;
            let upper = grid[ur + i] as f64;
            right[i] = (1. - rlat) * lower + rlat * upper;
        }

        let mut result = Coor4D::origin();
        for i in 0..bands {
            result[i] = (1. - rlon) * left[i] + rlon * right[i];
        }

        Some(result)
    }
}

impl BaseGrid {
    pub fn plain(
        header: &[f64],
        grid: Option<&[f32]>,
        offset: Option<usize>,
    ) -> Result<Self, Error> {
        if header.len() < 7 {
            return Err(Error::General("Malformed header"));
        }

        let lat_n = header[0];
        let lat_s = header[1];
        let lon_w = header[2];
        let lon_e = header[3];
        let dlat = header[4].copysign(lat_s - lat_n);
        let dlon = header[5].copysign(lon_e - lon_w);
        let bands = header[6] as usize;
        let rows = ((lat_s - lat_n) / dlat + 1.5).floor() as usize;
        let cols = ((lon_e - lon_w) / dlon + 1.5).floor() as usize;
        let elements = rows * cols * bands;

        let offset = offset.unwrap_or(0);

        let grid = Vec::from(grid.unwrap_or(&[]));

        if elements == 0 || (offset == 0 && elements > grid.len()) || bands < 1 {
            return Err(Error::General("Malformed grid"));
        }

        Ok(BaseGrid {
            lat_n,
            lat_s,
            lon_w,
            lon_e,
            dlat,
            dlon,
            rows,
            cols,
            bands,
            offset,
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

    // For horizontal datum shifts, the grid values are in seconds-of-arc
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

    // The Gravsoft header has lat_s before lat_n
    header.swap(0, 1);

    // Count the number of bands
    let lat_n = header[0];
    let lat_s = header[1];
    let lon_w = header[2];
    let lon_e = header[3];

    // The Gravsoft header has inverted sign for dlat. We force
    // the two deltas to have signs compatible with the grid
    // organization
    let dlat = header[4].copysign(lat_s - lat_n);
    let dlon = header[5].copysign(lon_e - lon_w);
    let rows = ((lat_s - lat_n) / dlat + 1.5).floor() as usize;
    let cols = ((lon_e - lon_w) / dlon + 1.5).floor() as usize;
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

/// Find the most appropriate grid value from a stack (i.e. slice) of grids.
/// Search the grids in slice order and return the first hit.
/// If no hits are found, try once more, this time adding a half grid-cell
/// margin around each grid
pub fn grids_at(grids: &[Arc<dyn Grid>], coord: &Coor4D, use_null_grid: bool) -> Option<Coor4D> {
    for margin in [0.0, 0.5] {
        for grid in grids.iter() {
            let d = grid.at(coord, margin);
            if d.is_some() {
                return d;
            }
        }
    }

    if use_null_grid {
        return Some(Coor4D::origin());
    }

    None
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // lat_n, lat_s, lon_w, lon_e, dlat, dlon
    const HEADER: [f64; 6] = [58., 54., 8., 16., -1., 1.];

    #[allow(dead_code)]
    #[rustfmt::skip]
    const DATUM: [f32; 5*2*9] = [
        58., 08., 58., 09., 58., 10., 58., 11., 58., 12., 58., 13., 58., 14., 58., 15., 58., 16.,
        57., 08., 57., 09., 57., 10., 57., 11., 57., 12., 57., 13., 57., 14., 57., 15., 57., 16.,
        56., 08., 56., 09., 56., 10., 56., 11., 56., 12., 56., 13., 56., 14., 56., 15., 56., 16.,
        55., 08., 55., 09., 55., 10., 55., 11., 55., 12., 55., 13., 55., 14., 55., 15., 55., 16.,
        54., 08., 54., 09., 54., 10., 54., 11., 54., 12., 54., 13., 54., 14., 54., 15., 54., 16.,
    ];

    #[rustfmt::skip]
    const GEOID: [f32; 5*9] = [
        58.08, 58.09, 58.10, 58.11, 58.12, 58.13, 58.14, 58.15, 58.16,
        57.08, 57.09, 57.10, 57.11, 57.12, 57.13, 57.14, 57.15, 57.16,
        56.08, 56.09, 56.10, 56.11, 56.12, 56.13, 56.14, 56.15, 56.16,
        55.08, 55.09, 55.10, 55.11, 55.12, 55.13, 55.14, 55.15, 55.16,
        54.08, 54.09, 54.10, 54.11, 54.12, 54.13, 54.14, 54.15, 54.16,
    ];

    #[test]
    fn grid_header() -> Result<(), Error> {
        // Create a datum correction grid (2 bands)
        let mut datum_header = Vec::from(HEADER);

        // Since we use normalize_gravsoft...(...) to handle angular normalization,
        // we need a Gravsoft style header here
        datum_header.swap(0, 1);
        datum_header[4] = -datum_header[4];
        datum_header.push(2_f64); // 2 bands
        let mut datum_grid = Vec::from(DATUM);
        normalize_gravsoft_grid_values(&mut datum_header, &mut datum_grid);

        // But Since we use BaseGrid::plain(...) to instantiate, we need a plain header here
        datum_header.swap(0, 1);
        datum_header[4] = -datum_header[4];
        let datum = BaseGrid::plain(&datum_header, Some(&datum_grid), None)?;

        // Extrapolation
        let c = Coor4D::geo(100., 50., 0., 0.);
        // ...with output converted back to arcsec
        let d = datum.at(&c, 100.0).unwrap().to_arcsec();

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
        assert!(datum.contains(&c, 0.0));
        // ...with output converted back to arcsec
        let d = datum.at(&c, 0.0).unwrap().to_arcsec();
        // We can do slightly better for interpolation than for extrapolation,
        // but the grid values are f32, so we have only approx 7 significant
        // figures...
        assert!(c.to_degrees().hypot2(&d) < 1e-5);

        // Create a geoid grid (1 band)
        let mut geoid_header = datum_header.clone();
        geoid_header[6] = 1.0; // 1 band
        let geoid_grid = Vec::from(GEOID);
        let geoid = BaseGrid::plain(&geoid_header, Some(&geoid_grid), None)?;

        let c = Coor4D::geo(58.75, 08.25, 0., 0.);
        assert!(!geoid.contains(&c, 0.0));
        assert!(geoid.contains(&c, 1.0));

        let n = geoid.at(&c, 1.0).unwrap();
        assert!((n[0] - (58.75 + 0.0825)).abs() < 0.0001);
        Ok(())
    }
}

// Additional tests for Grid in src/inner_op/gridshift.rs
