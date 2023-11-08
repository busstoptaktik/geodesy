use super::parser::{NTv2Parser, HEADER_SIZE};
use crate::{Coor2D, Coor4D, Error, Grid};

#[derive(Debug, Clone)]
pub(crate) struct Ntv2SubGrid {
    pub(crate) head: SubGridHeader,
    pub(crate) grid: Vec<Coor2D>,
}

impl Ntv2SubGrid {
    pub(crate) fn new(parser: &NTv2Parser, head_offset: usize) -> Result<Self, Error> {
        let head = SubGridHeader::new(parser, head_offset)?;

        let grid_offset = head_offset + HEADER_SIZE;
        let grid = parse_subgrid_grid(
            parser,
            grid_offset,
            head.num_nodes as usize,
            head.num_rows as usize,
            head.row_size as usize,
        )?;

        Ok(Self { head, grid })
    }
}

impl Grid for Ntv2SubGrid {
    fn bands(&self) -> usize {
        2
    }

    // Checks if a `Coor4D` is within the grid limits +- `within` grid units
    fn contains(&self, position: &Coor4D, within: f64) -> bool {
        let lon = position[0];
        let lat = position[1];

        let grace = within * self.head.dlat;
        if lat != lat.clamp(self.head.slat - grace, self.head.nlat + grace) {
            return false;
        }

        let grace = within * self.head.dlon;
        if lon != lon.clamp(self.head.wlon - grace, self.head.elon + grace) {
            return false;
        }

        // If we fall through to here we're within the grid
        true
    }

    // Implementation adapted from [projrs](https://github.com/3liz/proj4rs/blob/8b5eb762c6be65eed0ca0baea33f8c70d1cd56cb/src/nadgrids/grid.rs#L206C1-L252C6) && [proj4js](https://github.com/proj4js/proj4js/blob/d9faf9f93ebeccac4b79fa80f3e9ad8a7032828b/lib/datum_transform.js#L167)
    fn at(&self, coord: &Coor4D, within: f64) -> Option<Coor4D> {
        if !self.contains(coord, within) {
            return None;
        }

        // Normalise to the grid origin which is the SW corner
        let rlon = coord[0] - self.head.wlon;
        let rlat = coord[1] - self.head.slat;

        let (t_lon, t_lat) = (rlon / self.head.dlon, rlat / self.head.dlat);
        let (i_lon, i_lat) = (t_lon.floor(), t_lat.floor());
        let (f_lon, f_lat) = (t_lon - 1.0 * i_lon, t_lat - 1.0 * i_lat);

        let mut index = (i_lat * self.head.row_size + i_lon) as usize;
        let f00 = &self.grid[index];
        let f10 = &self.grid[index + 1];
        index += self.head.row_size as usize;
        let f01 = &self.grid[index];
        let f11 = &self.grid[index + 1];

        let m00 = (1. - f_lon) * (1. - f_lat);
        let m01 = (1. - f_lon) * f_lat;
        let m10 = f_lon * (1. - f_lat);
        let m11 = f_lon * f_lat;

        let mut result = Coor4D::origin();
        result[0] = -(m00 * f00[0] + m10 * f10[0] + m01 * f01[0] + m11 * f11[0]); // lon
        result[1] = m00 * f00[1] + m10 * f10[1] + m01 * f01[1] + m11 * f11[1]; // lat

        Some(result)
    }
}

// Buffer offsets for the NTv2 subgrid header
const NLAT: usize = 88;
const SLAT: usize = 72;
const ELON: usize = 104;
const WLON: usize = 120;
const DLAT: usize = 136;
const DLON: usize = 152;
const GSCOUNT: usize = 168;

#[derive(Debug, Clone)]
pub(crate) struct SubGridHeader {
    pub num_nodes: f64,
    pub nlat: f64,
    pub slat: f64,
    pub wlon: f64,
    pub elon: f64,
    pub dlat: f64,
    pub dlon: f64,
    pub num_rows: f64,
    pub row_size: f64,
}

impl SubGridHeader {
    // Parse a subgrid header for an NTv2 grid
    // Weird sign conventions like longitude being west positive are handled here.
    fn new(parser: &NTv2Parser, offset: usize) -> Result<Self, Error> {
        let nlat = parser.get_f64(offset + NLAT);
        let slat = parser.get_f64(offset + SLAT);
        let wlon = parser.get_f64(offset + WLON);
        let elon = parser.get_f64(offset + ELON);
        let dlat = parser.get_f64(offset + DLAT);
        let dlon = parser.get_f64(offset + DLON);

        // As defined by https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf (pg 30)
        let num_rows = (((slat - nlat) / dlat).abs() + 1.0).floor();
        let row_size = (((wlon - elon) / dlon).abs() + 1.0).floor();

        let num_nodes = parser.get_u32(offset + GSCOUNT) as f64;
        if num_nodes != (num_rows * row_size) {
            return Err(Error::Invalid(
                "Number of nodes does not match the grid size".to_string(),
            ));
        }

        Ok(Self {
            nlat: nlat.to_radians() / 3600.,
            slat: slat.to_radians() / 3600.,
            // By default the longitude is positive west. By conventions east is positive.
            // This is likely because the Canadian makers of NTv2 are always west of Greenwich.
            wlon: -wlon.to_radians() / 3600.,
            elon: -elon.to_radians() / 3600.,
            dlat: dlat.to_radians() / 3600.,
            dlon: dlon.to_radians() / 3600.,
            num_rows,
            row_size,
            num_nodes,
        })
    }
}

// Buffer offsets for the NTv2 grid nodes
const NODE_LON_CORRN: usize = 4; // (f32) correction to the longitude at this node point (secs)
const NODE_SIZE: usize = 16;

// Parses the nodes of a sub grid into a vector of lon/lat shifts in radians
fn parse_subgrid_grid(
    parser: &NTv2Parser,
    grid_start: usize,
    num_nodes: usize,
    num_rows: usize,
    row_size: usize,
) -> Result<Vec<Coor2D>, Error> {
    let mut grid = (0..num_nodes as usize)
        .map(|i| {
            let offset = grid_start + i * NODE_SIZE;
            let lon_offset = offset + NODE_LON_CORRN;
            let lat_corr = parser.get_f32(offset) as f64;
            let lon_corr = parser.get_f32(lon_offset) as f64;

            Coor2D::arcsec(lon_corr, lat_corr)
        })
        .collect::<Vec<Coor2D>>();

    // Switch the row order so that the lower left is the SW corner
    for i in 0..num_rows {
        let offs = i * row_size;
        grid[offs..(offs + row_size)].reverse();
    }

    let grid_end_offset = grid_start + num_nodes * NODE_SIZE;

    if grid_end_offset > parser.buffer().len() {
        return Err(Error::Invalid("Grid Too Short".to_string()));
    }

    Ok(grid)
}
