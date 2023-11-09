use super::parser::{NTv2Parser, HEADER_SIZE};
use crate::{grid::BaseGrid, Error};

pub(super) fn ntv2_subgrid(parser: &NTv2Parser, head_offset: usize) -> Result<BaseGrid, Error> {
    let head = SubGridHeader::new(parser, head_offset)?;

    let grid_start = head_offset + HEADER_SIZE;
    let grid = parse_subgrid_grid(parser, grid_start, head.num_nodes as usize)?;
    let header = [
        head.slat, head.nlat, head.elon, head.wlon, head.dlat, head.dlon, 2.0,
    ];
    BaseGrid::plain(&header, Some(&grid), Some(0))
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
    pub num_nodes: u64,
    pub nlat: f64,
    pub slat: f64,
    pub wlon: f64,
    pub elon: f64,
    pub dlat: f64,
    pub dlon: f64,
}

impl SubGridHeader {
    // Parse a subgrid header for an NTv2 grid
    // Weird sign conventions like longitude being west positive are handled here.
    pub(crate) fn new(parser: &NTv2Parser, offset: usize) -> Result<Self, Error> {
        let nlat = parser.get_f64(offset + NLAT);
        let slat = parser.get_f64(offset + SLAT);
        let wlon = parser.get_f64(offset + WLON);
        let elon = parser.get_f64(offset + ELON);
        let dlat = parser.get_f64(offset + DLAT);
        let dlon = parser.get_f64(offset + DLON);

        let num_rows = (((slat - nlat) / dlat).abs() + 1.0).floor() as u64;
        let row_size = (((wlon - elon) / dlon).abs() + 1.0).floor() as u64;

        let num_nodes = parser.get_u32(offset + GSCOUNT) as u64;
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
) -> Result<Vec<f32>, Error> {
    let grid_end_offset = grid_start + num_nodes * NODE_SIZE;
    if grid_end_offset > parser.buffer().len() {
        return Err(Error::Invalid("Grid Too Short".to_string()));
    }

    let mut grid = Vec::with_capacity(num_nodes);
    for i in 0..num_nodes {
        let offset = grid_start + i * NODE_SIZE;
        let lon_offset = offset + NODE_LON_CORRN;

        let lat_corr = parser.get_f32(offset).to_radians() / 3600.;
        let lon_corr = -parser.get_f32(lon_offset).to_radians() / 3600.;
        grid.push(lon_corr);
        grid.push(lat_corr);
    }

    Ok(grid)
}
