use super::{BaseGrid, GridHeader, GridSource};
use crate::Error;
use std::io::{BufRead, BufReader};

// Example of a Golden Software ASCII grid (GSA) header
//
// DSAA                # "magic bytes" identifying the GSA format
// 301 313             # columns, rows
// 0 50                # lon_min, lon_max
// 49 75               # lat_min, lat_max
// -1.542 0.421        # grid_val_min, grid_val_max

pub fn gsa(name: &str, buf: &[u8]) -> Result<BaseGrid, Error> {
    let grids = gsa_grid_reader(name, buf)?;
    if grids.is_empty() {
        return Err(Error::Invalid("Empty grid: ".to_string() + name));
    }

    let mut header = grids[0].0.clone();
    let bands = grids.len();
    header.bands = bands;
    let cols = header.cols;
    let rows = header.rows;
    let stride = cols * bands;

    // Read the grid node information and store it row-wise, taking into account
    // that in a GSA file, the rows are stored southernmost-first.
    let mut grid = Vec::new();
    grid.resize(rows * cols * bands, 0f32);
    for b in 0..bands {
        let values = &grids[b].1;
        if grids[b].0 != grids[0].0 {
            return Err(Error::Invalid(
                "Mismatched headers for bands in grid: ".to_string() + name,
            ));
        }
        for r in 0..rows {
            let gsa_row_start = (rows - 1 - r) * cols;
            for c in 0..cols {
                grid[r * stride + c * bands + b] = values[gsa_row_start + c];
            }
        }
    }
    let grid = GridSource::Internal { values: grid };
    BaseGrid::new(name, header, grid)
}

// Read a Golden Software grid. Discard '#'-style comments
pub fn gsa_grid_reader(name: &str, buf: &[u8]) -> Result<Vec<(GridHeader, Vec<f32>)>, Error> {
    let all = BufReader::new(buf);
    let mut grid_values = Vec::<f32>::new();
    let mut header_values = Vec::<f64>::new();
    let mut grids = Vec::new();
    const SUBGRID_DELIMITER: &str = "DSAA";
    let mut magic_bytes_found = false;

    for line in all.lines() {
        let line = line?;
        // Remove comments and blank lines
        let line = line.split('#').collect::<Vec<_>>()[0].trim();
        if line.is_empty() {
            continue;
        }

        // Expect to find the magic bytes as the first item after top level comments
        if !magic_bytes_found {
            if !line.starts_with(SUBGRID_DELIMITER) {
                return Err(Error::Invalid(
                    "Grid: `".to_string() + name + "` is not in GSA format",
                ));
            }
            magic_bytes_found = true;
        }

        // Beginning of a new band? - construct the previous grid
        if line.starts_with(SUBGRID_DELIMITER) {
            // The DSAA string at the top of the file is not a subgrid delimiter, so
            // if we have not read anything yet, we just go on with the next line
            if header_values.is_empty() {
                continue;
            }

            let grid = gsa_grid_interpreter(&header_values, &mut grid_values)?;
            grids.push(grid);
            grid_values.clear();
            header_values.clear();
            continue;
        }

        // Convert to f64
        for item in line.split_whitespace() {
            let value = item.parse::<f64>().unwrap_or(f64::NAN);
            // In GSA grids, the header is the first 8 numbers of the file
            if header_values.len() < 8 {
                header_values.push(value);
            } else {
                // GSA uses the value 1.70141E+38 for NODATA
                grid_values.push(if value.abs() > 1e30 {
                    f32::NAN
                } else {
                    value as f32
                });
            }
        }
    }

    // Construct the last grid
    let grid = gsa_grid_interpreter(&header_values, &mut grid_values)?;
    grids.push(grid);

    Ok(grids)
}

// transform raw gravsoft data into a proper BaseGrid
pub fn gsa_grid_interpreter(
    header: &[f64],
    grid: &mut [f32],
) -> Result<(GridHeader, Vec<f32>), Error> {
    // The GSA header has lat_s before lat_n
    let lat_n = header[5];
    let lat_s = header[4];
    let lon_w = header[2];
    let lon_e = header[3];

    let rows = header[1] as usize;
    let cols = header[0] as usize;

    let dlat = (lat_s - lat_n) / (rows - 1) as f64;
    let dlon = (lon_e - lon_w) / (cols - 1) as f64;

    // Bands is handled at the top level gsa function
    let bands = 0;

    if (rows * cols) != grid.len() {
        return Err(Error::General("Unrecognized material at end of GSA grid"));
    }

    let header = GridHeader {
        lat_n: lat_n.to_radians(),
        lat_s: lat_s.to_radians(),
        lon_w: lon_w.to_radians(),
        lon_e: lon_e.to_radians(),
        dlat: dlat.to_radians(),
        dlon: dlon.to_radians(),
        rows,
        cols,
        bands,
    };
    Ok((header, grid.into()))
}

#[cfg(test)]
mod tests {
    use crate::authoring::*;
    #[test]
    fn gsa_plain() -> Result<(), Error> {
        let buf = include_bytes!("../../geodesy/gsa/egm96_15_subset.gsa");
        let grid = super::gsa("egm96_15_subset", buf)?;
        assert_eq!(grid.name, "egm96_15_subset");

        assert_eq!(grid.subgrids.len(), 0);
        assert_eq!(grid.header.bands, 3);
        assert_eq!(grid.header.rows, 17);
        assert_eq!(grid.header.cols, 33);
        assert_eq!(grid.header.bands, 3);
        assert_eq!(grid.header.lat_n, 58f64.to_radians());
        assert_eq!(grid.header.lat_s, 54f64.to_radians());
        assert_eq!(grid.header.lon_w, 8f64.to_radians());
        assert_eq!(grid.header.lon_e, 16f64.to_radians());

        let ul = Coor4D::geo(58., 8., 0., 0.);
        let ur = Coor4D::geo(58., 16., 0., 0.);
        let lr = Coor4D::geo(54., 16., 0., 0.);
        let ul = grid.at(None, ul, 0.).unwrap();
        let ur = grid.at(None, ur, 0.).unwrap();
        let lr = grid.at(None, lr, 0.).unwrap();
        if let GridSource::Internal { values } = &grid.grid {
            assert_eq!(ul[0] as f32, *values.first().unwrap());
            // In the present case, the geoid undulation is larger in west, than in east
            assert!(ul[0] > ur[0]);
            assert_eq!(lr[2] as f32, *values.last().unwrap());
        } else {
            panic!("Unexpected GridSource enum")
        }

        // Now for a point that needs interpolation
        let pt = Coor4D::geo(56.1, 12.1, 0., 0.);
        let n = grid.at(None, pt, 0.).unwrap();
        assert!((n[0] - 36.6803439331055).abs() < 1e-8);
        assert!((n[1] - 136.6803430176).abs() < 1e-8);
        assert!((n[2] - 236.6803430176).abs() < 1e-8);
        Ok(())
    }
}
