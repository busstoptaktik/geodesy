//! Read a GTX style grid, according to the format description
//! at <https://vdatum.noaa.gov/docs/gtx_info.html#dev_gtx_binary>

use super::*;
use std::io::{BufReader, Read};

pub fn gtx(name: &str, buf: &[u8]) -> Result<BaseGrid, Error> {
    let mut buf = BufReader::new(buf);

    // Read the 44-byte header
    let mut header = [0u8; 40];
    buf.read_exact(&mut header)?;

    // Interpret the header information
    let lat_s = f64::from_be_bytes(header[0..8].try_into().unwrap());
    let lon_w = f64::from_be_bytes(header[8..16].try_into().unwrap());
    let dlat = f64::from_be_bytes(header[16..24].try_into().unwrap());
    let dlon = f64::from_be_bytes(header[24..32].try_into().unwrap());
    let nrows = u32::from_be_bytes(header[32..36].try_into().unwrap()) as usize;
    let ncols = u32::from_be_bytes(header[36..40].try_into().unwrap()) as usize;

    // Read grid node information and store row-wise
    let mut rows = Vec::with_capacity(nrows);
    let mut node = [0u8; 4];
    for _ in 0..nrows {
        let mut row = Vec::with_capacity(ncols);
        for _ in 0..ncols {
            buf.read_exact(&mut node)?;
            let mut node_value = f32::from_be_bytes(node);
            if node_value == -88.8888 {
                node_value = f32::NAN;
            }
            row.push(node_value);
        }
        rows.push(row);
    }

    // The grid is stored upside-down, so we add the rows to the final
    // grid in reverse order
    let mut grid = Vec::with_capacity(nrows * ncols);
    for _ in 0..nrows {
        grid.append(&mut rows.pop().unwrap());
    }

    // Construct the BaseGrid elements

    let header = GridHeader {
        lat_n: (lat_s + (nrows - 1) as f64 * dlat).to_radians(),
        lat_s: lat_s.to_radians(),
        lon_w: lon_w.to_radians(),
        lon_e: (lon_w + (ncols - 1) as f64 * dlon).to_radians(),
        dlat: -dlat.to_radians(),
        dlon: dlon.to_radians(),
        rows: nrows,
        cols: ncols,
        bands: 1,
    };

    let grid = GridSource::Internal { values: grid };
    let subgrids = Vec::new();
    let name = name.to_string();

    Ok(BaseGrid {
        name,
        header,
        grid,
        subgrids,
    })
}

#[cfg(test)]
mod tests {
    use crate::authoring::*;
    #[test]
    fn gtx_plain() -> Result<(), Error> {
        let buf = include_bytes!("../../geodesy/gtx/egm96_15_subset.gtx");
        let grid = super::gtx("egm96_15_subset", buf)?;
        assert_eq!(grid.name, "egm96_15_subset");

        assert_eq!(grid.subgrids.len(), 0);
        assert_eq!(grid.header.bands, 1);
        assert_eq!(grid.header.rows, 17);
        assert_eq!(grid.header.cols, 33);
        assert_eq!(grid.header.lat_n, 58f64.to_radians());
        assert_eq!(grid.header.lat_s, 54f64.to_radians());
        assert_eq!(grid.header.lon_w, 8f64.to_radians());
        assert_eq!(grid.header.lon_e, 16f64.to_radians());

        let ul = Coor4D::geo(58., 8., 0., 0.);
        let lr = Coor4D::geo(54., 16., 0., 0.);
        let u = grid.at(None, ul, 0.).unwrap();
        let v = grid.at(None, lr, 0.).unwrap();
        if let GridSource::Internal { values } = &grid.grid {
            assert_eq!(u[0] as f32, *values.first().unwrap());
            assert_eq!(v[0] as f32, *values.last().unwrap());
        } else {
            panic!("Unexpected GridSource enum")
        }

        // Now for a point that needs interpolation
        let pt = Coor4D::geo(56.1, 12.1, 0., 0.);
        let n = grid.at(None, pt, 0.).unwrap()[0];
        assert!((n - 36.6803439331055).abs() < 1e-8);
        Ok(())
    }
}
