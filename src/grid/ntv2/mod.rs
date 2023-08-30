use self::parser::{parse_subgrid_grid, parse_subgrid_header, HEADER_SIZE};
use crate::{Coor2D, Coor4D, Error};
use parser::NTv2Parser;
mod parser;

/// Grid for using the NTv2 format.
/// Interpolation has been adapted from [projrs](https://github.com/3liz/proj4rs/blob/8b5eb762c6be65eed0ca0baea33f8c70d1cd56cb/src/nadgrids/grid.rs#L206C1-L252C6)
/// to work with Rust Geodesy
#[derive(Debug, Default, Clone)]
pub struct Ntv2Grid {
    nlat: f64,
    slat: f64,
    wlon: f64,
    elon: f64,
    dlat: f64,
    dlon: f64,
    num_rows: f64,
    row_size: f64,
    grid: Vec<Coor2D>,
    pub bands: usize,
}

impl Ntv2Grid {
    pub fn new(buf: &[u8]) -> Result<Self, Error> {
        let parser = NTv2Parser::new(buf.into());

        let num_sub_grids = parser.get_u32(40) as usize;
        if num_sub_grids != 11 && parser.cmp_str(8, "NUM_OREC") {
            return Err(Error::Unsupported("Wrong header".to_string()));
        }

        if num_sub_grids != 1 {
            // Multi grid support is out of scope for now given how few seem to exist
            return Err(Error::Unsupported(
                "Contains more than one subgrid".to_string(),
            ));
        }

        if !parser.cmp_str(56, "SECONDS") {
            return Err(Error::Invalid("Not in seconds".to_string()));
        }

        let (nlat, slat, wlon, elon, dlat, dlon, num_rows, row_size, num_nodes) =
            parse_subgrid_header(&parser, HEADER_SIZE)?;

        let grid_start_offset = HEADER_SIZE * 2;

        let grid = parse_subgrid_grid(
            &parser,
            grid_start_offset,
            num_nodes as usize,
            num_rows as usize,
            row_size as usize,
        )?;

        Ok(Self {
            nlat,
            slat,
            wlon,
            elon,
            dlat,
            dlon,
            num_rows,
            row_size,
            grid,
            // For compatibility with the `Grid` struct
            bands: 2,
        })
    }

    /// Checks if a `Coord4D` is within the grid limits
    pub fn contains(&self, position: Coor4D) -> bool {
        let lon = position[0];
        let lat = position[1];
        lat >= self.slat && lat <= self.nlat && lon >= self.wlon && lon <= self.elon
    }

    // Matches the `interpolation` method signature of the `Grid` struct
    // Implementation adapted from [projrs](https://github.com/3liz/proj4rs/blob/8b5eb762c6be65eed0ca0baea33f8c70d1cd56cb/src/nadgrids/grid.rs#L206C1-L252C6)
    pub fn interpolation(&self, coord: &Coor4D, _grid: Option<&Vec<Coor2D>>) -> Coor4D {
        // Normalise to the grid origin which is the SW corner
        let rlon = coord[0] - self.wlon;
        let rlat = coord[1] - self.slat;

        let (t_lon, t_lat) = (rlon / self.dlon, rlat / self.dlat);

        // NOTE: A robust implementation would need to throw outside of interpolation or change `interpolation` to return a Result
        let (i_lon, f_lon) = check_limits(t_lon, self.num_rows).unwrap();
        let (i_lat, f_lat) = check_limits(t_lat, self.row_size).unwrap();

        let mut index = (i_lat * self.row_size + i_lon) as usize;
        let f00 = &self.grid[index];
        let f10 = &self.grid[index + 1];
        index += self.row_size as usize;
        let f01 = &self.grid[index];
        let f11 = &self.grid[index + 1];

        let m00 = (1. - f_lon) * (1. - f_lat);
        let m01 = (1. - f_lon) * f_lat;
        let m10 = f_lon * (1. - f_lat);
        let m11 = f_lon * f_lat;

        let mut result = Coor4D::origin();
        result[0] = -(m00 * f00[0] + m10 * f10[0] + m01 * f01[0] + m11 * f11[0]); // lon
        result[1] = m00 * f00[1] + m10 * f10[1] + m01 * f01[1] + m11 * f11[1]; // lat

        result
    }
}

fn check_limits(t: f64, cols: f64) -> Result<(f64, f64), Error> {
    let mut i = t.floor();
    let mut f = t - i;
    if i < 0. {
        if i == -1. && f > 0.99999999999 {
            i += 1.;
            f = 0.
        } else {
            return Err(Error::General("Point outside nad shift area"));
        }
    } else {
        match i + 1. {
            n if n == cols && f < 1.0e-11 => {
                i -= 1.;
                f = 1.;
            }
            n if n > cols => return Err(Error::General("Point outside nad shift area")),
            _ => (),
        }
    }
    Ok((i, f))
}
