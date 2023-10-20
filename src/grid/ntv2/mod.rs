use self::parser::{parse_subgrid_grid, parse_subgrid_header, HEADER_SIZE};
use crate::{Coor2D, Coor4D, Error, GridTrait};
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
        })
    }
}

impl GridTrait for Ntv2Grid {
    fn bands(&self) -> usize {
        2
    }

    /// Checks if a `Coord4D` is within the grid limits
    fn contains(&self, position: Coor4D) -> bool {
        let lon = position[0];
        let lat = position[1];
        lat >= self.slat && lat <= self.nlat && lon >= self.wlon && lon <= self.elon
    }

    // Matches the `interpolation` method signature of the `Grid` struct
    /// Implementation adapted from [projrs](https://github.com/3liz/proj4rs/blob/8b5eb762c6be65eed0ca0baea33f8c70d1cd56cb/src/nadgrids/grid.rs#L206C1-L252C6) && [proj4js](https://github.com/proj4js/proj4js/blob/d9faf9f93ebeccac4b79fa80f3e9ad8a7032828b/lib/datum_transform.js#L167)
    fn interpolation(&self, coord: &Coor4D, _grid: Option<&[f32]>) -> Coor4D {
        // Normalise to the grid origin which is the SW corner
        let rlon = coord[0] - self.wlon;
        let rlat = coord[1] - self.slat;

        let (t_lon, t_lat) = (rlon / self.dlon, rlat / self.dlat);
        let (i_lon, i_lat) = (t_lon.floor(), t_lat.floor());
        let (f_lon, f_lat) = (t_lon - 1.0 * i_lon, t_lat - 1.0 * i_lat);

        if i_lon < 0. || i_lon >= self.row_size {
            return Coor4D::raw(f64::NAN, f64::NAN, 0., 0.);
        }

        if i_lat < 0. || i_lat >= self.num_rows {
            return Coor4D::raw(f64::NAN, f64::NAN, 0., 0.);
        }

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
