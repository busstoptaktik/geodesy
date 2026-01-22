use super::{BaseGrid, GridHeader, GridSource};
use crate::Error;
use std::io::BufRead;

pub fn gravsoft(name: &str, buf: &[u8]) -> Result<BaseGrid, Error> {
    let (header, values) = gravsoft_grid_reader(buf)?;
    BaseGrid::new(name, header, values)
}

// Read a gravsoft grid. Discard '#'-style comments
pub fn gravsoft_grid_reader(buf: &[u8]) -> Result<(GridHeader, GridSource), Error> {
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
    let lat_n = header[1];
    let lat_s = header[0];
    let lon_w = header[2];
    let lon_e = header[3];

    // The Gravsoft header has inverted sign for dlat. We force
    // the two deltas to have signs compatible with the grid
    // organization
    let dlat = header[4].copysign(lat_s - lat_n);
    let dlon = header[5].copysign(lon_e - lon_w);
    let rows = (((lat_s - lat_n) / dlat).abs() + 1.5).floor() as usize;
    let cols = (((lon_e - lon_w) / dlon).abs() + 1.5).floor() as usize;
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

    // Swap datum shift corrections into longitude/latitude order
    if bands == 2 {
        for i in 0..rows * cols {
            grid.swap(2 * i, 2 * i + 1);
        }
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

    let grid = GridSource::Internal { values: grid };

    Ok((header, grid))
}
