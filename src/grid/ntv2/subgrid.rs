use super::*;

pub(super) fn ntv2_subgrid(
    parser: &NTv2Parser,
    head_offset: usize,
) -> Result<(String, String, BaseGrid), Error> {
    let head = SubGridHeader::new(parser, head_offset)?;
    let name = head.name.clone();
    let parent = head.parent.clone();

    let grid_start = head_offset + HEADER_SIZE;
    let grid = parse_subgrid_grid(parser, grid_start, head.num_nodes as usize)?;
    let header = [
        //head.slat, head.nlat, head.elon, head.wlon, head.dlat, head.dlon, 2.0,
        head.nlat, head.slat, head.wlon, head.elon, head.dlat, head.dlon, 2.0,
    ];
    Ok(SubGrid {
        name: head.name,
        parent: head.parent,
        grid: BaseGrid::plain(&header, Some(&grid), Some(0))?,
    })
}

// Buffer offsets for the NTv2 subgrid header
const NAME: usize = 8;
const PARENT: usize = 24;
const NLAT: usize = 88;
const SLAT: usize = 72;
const ELON: usize = 104;
const WLON: usize = 120;
const DLAT: usize = 136;
const DLON: usize = 152;
const GSCOUNT: usize = 168;

struct SubGridHeader {
    pub name: String,
    pub parent: String,
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
    fn new(parser: &NTv2Parser, offset: usize) -> Result<Self, Error> {
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
            name: parser.get_str(offset + NAME, 8)?.trim().to_string(),
            parent: parser.get_str(offset + PARENT, 8)?.trim().to_string(),
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

    fn into_header(self) -> [f64; 7] {
        [
            self.slat, self.nlat, self.elon, self.wlon, self.dlat, self.dlon, 2.0,
        ]
    }
}

// Buffer offsets for the NTv2 grid nodes
const NODE_LAT_CORRECTION: usize = 0;
const NODE_LON_CORRECTION: usize = 4;
pub(super) const NODE_SIZE: usize = 16;

// Parse the nodes of a sub grid into a vector of lon/lat shifts in radians
fn parse_subgrid_grid(
    parser: &NTv2Parser,
    grid_start: usize,
    num_nodes: usize,
) -> Result<Vec<f32>, Error> {
    let grid_end_offset = grid_start + num_nodes * NODE_SIZE;
    if grid_end_offset > parser.buffer().len() {
        return Err(Error::Invalid("Grid Too Short".to_string()));
    }

    let mut grid = Vec::with_capacity(2 * num_nodes);
    for i in 0..num_nodes {
        let offset = grid_start + i * NODE_SIZE;
        let lat_offset = offset + NODE_LAT_CORRECTION;
        let lon_offset = offset + NODE_LON_CORRECTION;

        let mut lat_corr = parser.get_f32(lat_offset) as f64;
        let mut lon_corr = -parser.get_f32(lon_offset) as f64;
        lat_corr = (lat_corr / 3600.).to_radians();
        lon_corr = (lon_corr / 3600.).to_radians();
        grid.push(lat_corr as f32);
        grid.push(lon_corr as f32);
    }
    grid.reverse();

    Ok(grid)
}
