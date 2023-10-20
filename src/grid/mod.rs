//! Grid characteristics and interpolation.

#[cfg(feature = "ntv2")]
pub mod ntv2;
use crate::prelude::*;
use std::{fmt::Debug, io::BufRead};

pub trait Grid: Debug {
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

// NOTE: Should this be renamed PlainGrid? Then rename the trait to Grid?
/// Grid characteristics and interpolation.
///
/// The actual grid may be part of the `BaseGrid` struct, or
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
        let mut min = self.lat_1;
        let mut max = self.lat_0;

        // If it's not, we swap
        if self.dlat > 0. {
            (min, max) = (max, min)
        }

        let grace = margin * self.dlat.abs();
        if position[1] != position[1].clamp(min - grace, max + grace) {
            return false;
        }

        // The default assumption is the other way round for columns (longitudes)
        min = self.lon_0;
        max = self.lon_1;
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

        // The interpolation coordinate relative to the grid origin
        let rlon = at[0] - self.lon_0;
        let rlat = at[1] - self.lat_0;

        // The (row, column) of the lower left node of the grid cell containing
        // the interpolation coordinate - or, in the case of extrapolation:
        // the nearest cell inside the grid.
        let row = (rlat / self.dlat).floor() as i64;
        let col = (rlon / self.dlon).floor() as i64;

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

        let ll_lon = self.lon_0 + col as f64 * self.dlon;
        let ll_lat = self.lat_0 + row as f64 * self.dlat;

        // Cell relative, cell unit coordinates in a right handed CS
        let rlon = (at[0] - ll_lon) / self.dlon;
        let rlat = (at[1] - ll_lat) / -self.dlat;

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
            return Err(Error::General("Incomplete grid"));
        }

        let lat_0 = header[0];
        let lat_1 = header[1];
        let lon_0 = header[2];
        let lon_1 = header[3];
        let dlat = header[4].copysign(lat_1 - lat_0);
        let dlon = header[5].copysign(lon_1 - lon_0);
        let bands = header[6] as usize;
        let rows = ((lat_1 - lat_0) / dlat + 1.5).floor() as usize;
        let cols = ((lon_1 - lon_0) / dlon + 1.5).floor() as usize;
        let elements = rows * cols * bands;

        let offset = offset.unwrap_or(0);

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

    // The Gravsoft header has lat_1 before lat_0
    header.swap(0, 1);

    // Count the number of bands
    let lat_0 = header[0];
    let lat_1 = header[1];
    let lon_0 = header[2];
    let lon_1 = header[3];

    // The Gravsoft header has inverted sign for dlat. We force
    // the two deltas to have signs compatible with the grid
    // organization
    let dlat = header[4].copysign(lat_1 - lat_0);
    let dlon = header[5].copysign(lon_1 - lon_0);
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

    // lat_0, lat_1, lon_0, lon_1, dlat, dlon
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

    // A geoid in inverse row order
    #[rustfmt::skip]
    const UPSIDE_DOWN_GEOID: [f32; 5*9] = [
        54.08, 54.09, 54.10, 54.11, 54.12, 54.13, 54.14, 54.15, 54.16,
        55.08, 55.09, 55.10, 55.11, 55.12, 55.13, 55.14, 55.15, 55.16,
        56.08, 56.09, 56.10, 56.11, 56.12, 56.13, 56.14, 56.15, 56.16,
        57.08, 57.09, 57.10, 57.11, 57.12, 57.13, 57.14, 57.15, 57.16,
        58.08, 58.09, 58.10, 58.11, 58.12, 58.13, 58.14, 58.15, 58.16,
    ];

    #[rustfmt::skip]
    const MIRRORED_GEOID: [f32; 5*9] = [
        58.16, 58.15, 58.14, 58.13, 58.12, 58.11, 58.10, 58.09, 58.08,
        57.16, 57.15, 57.14, 57.13, 57.12, 57.11, 57.10, 57.09, 57.08,
        56.16, 56.15, 56.14, 56.13, 56.12, 56.11, 56.10, 56.09, 56.08,
        55.16, 55.15, 55.14, 55.13, 55.12, 55.11, 55.10, 55.09, 55.08,
        54.16, 54.15, 54.14, 54.13, 54.12, 54.11, 54.10, 54.09, 54.08,
    ];

    #[rustfmt::skip]
    const MIRRORED_UPSIDE_DOWN_GEOID: [f32; 5*9] = [
        54.16, 54.15, 54.14, 54.13, 54.12, 54.11, 54.10, 54.09, 54.08,
        55.16, 55.15, 55.14, 55.13, 55.12, 55.11, 55.10, 55.09, 55.08,
        56.16, 56.15, 56.14, 56.13, 56.12, 56.11, 56.10, 56.09, 56.08,
        57.16, 57.15, 57.14, 57.13, 57.12, 57.11, 57.10, 57.09, 57.08,
        58.16, 58.15, 58.14, 58.13, 58.12, 58.11, 58.10, 58.09, 58.08,
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

        let c = Coor4D::geo(55.06, 12.03, 0., 0.);
        let d = datum.at(&c, 1.0).unwrap();
        assert!(c.default_ellps_dist(&d.to_arcsec().to_radians()) < 1.0);

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
        assert_eq!(datum.contains(&c, 0.0), true);
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
        assert_eq!(geoid.contains(&c, 0.0), false);
        assert_eq!(geoid.contains(&c, 1.0), true);

        let n = geoid.at(&c, 1.0).unwrap();
        assert!((n[0] - (58.75 + 0.0825)).abs() < 0.0001);

        // Create an upside-down geoid grid (1 band)
        let mut geoid_header = datum_header.clone();
        geoid_header.swap(0, 1); // lat_0=54, lat_1=58
        geoid_header[6] = 1.0; // 1 band
        let geoid_grid = Vec::from(UPSIDE_DOWN_GEOID);
        let geoid = BaseGrid::plain(&geoid_header, Some(&geoid_grid), None)?;

        let c = Coor4D::geo(58.75, 08.25, 0., 0.);
        assert_eq!(geoid.contains(&c, 0.0), false);
        assert_eq!(geoid.contains(&c, 1.0), true);

        let n = geoid.at(&c, 1.0).unwrap();
        assert!((n[0] - 58.83).abs() < 0.1);

        let c = Coor4D::geo(53.25, 8.0, 0., 0.);
        assert_eq!(geoid.contains(&c, 0.0), false);
        assert_eq!(geoid.contains(&c, 1.0), true);

        let n = geoid.at(&c, 1.0).unwrap();
        assert!((n[0] - (53.25 + 0.08)).abs() < 0.0001);

        // Create a mirrored geoid grid (1 band)
        let mut geoid_header = datum_header.clone();
        geoid_header.swap(2, 3); // lon_0=16, lon_1=8
        geoid_header[5] = -geoid_header[5];
        geoid_header[6] = 1.0; // 1 band
        let geoid_grid = Vec::from(MIRRORED_GEOID);
        let geoid = BaseGrid::plain(&geoid_header, Some(&geoid_grid), None)?;

        let c = Coor4D::geo(58.75, 08.25, 0., 0.);
        assert_eq!(geoid.contains(&c, 0.0), false);
        assert_eq!(geoid.contains(&c, 1.0), true);

        let n = geoid.at(&c, 1.0).unwrap();
        assert!((n[0] - 58.83).abs() < 0.1);

        let c = Coor4D::geo(53.25, 8.0, 0., 0.);
        assert_eq!(geoid.contains(&c, 0.0), false);
        assert_eq!(geoid.contains(&c, 1.0), true);

        let n = geoid.at(&c, 1.0).unwrap();
        assert!((n[0] - (53.25 + 0.08)).abs() < 0.001);

        // Create a mirrored upside down geoid grid (1 band)
        geoid_header.swap(0, 1); // lon_0=16, lon_1=8
        geoid_header[4] = -geoid_header[4];
        let geoid_grid = Vec::from(MIRRORED_UPSIDE_DOWN_GEOID);
        let geoid = BaseGrid::plain(&geoid_header, Some(&geoid_grid), None)?;

        let c = Coor4D::geo(58.75, 08.25, 0., 0.);
        assert_eq!(geoid.contains(&c, 0.0), false);
        assert_eq!(geoid.contains(&c, 1.0), true);

        let n = geoid.at(&c, 1.0).unwrap();
        assert!((n[0] - 58.83).abs() < 0.1);

        let c = Coor4D::geo(53.25, 8.0, 0., 0.);
        assert_eq!(geoid.contains(&c, 0.0), false);
        assert_eq!(geoid.contains(&c, 1.0), true);

        let n = geoid.at(&c, 1.0).unwrap();
        assert!((n[0] - (53.25 + 0.08)).abs() < 0.001);

        Ok(())
    }
}

// Additional tests for Grid in src/inner_op/gridshift.rs
