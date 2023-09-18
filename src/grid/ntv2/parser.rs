use crate::{Coor2D, Error};

#[derive(Copy, Clone, Debug)]
pub(crate) enum Endianness {
    Be = 0,
    Le = 1,
}
const SEC_TO_RAD: f64 = 4.8481e-6;

// Both overview and sub grid headers have 11 fields of 16 bytes each.
pub(crate) const HEADER_SIZE: usize = 11 * 16;

// Buffer offsets for the NTv2 overview header
const HEAD_NUM_RECORDS: usize = 8; // (i32) number of records in the file

// Buffer offsets for the NTv2 subgrid header
const SUBGRID_NLAT: usize = 88; // (f64)
const SUBGRID_SLAT: usize = 72; // (f64)
const SUBGRID_ELON: usize = 104; // (f64)
const SUBGRID_WLON: usize = 120; // (f64)
const SUBGRID_DLAT: usize = 136; // (f64)
const SUBGRID_DLON: usize = 152; // (f64)
const SUBGRID_GSCOUNT: usize = 168; // (i32) grid node count

// Buffer offsets for the NTv2 grid nodes
const NODE_LON_CORRN: usize = 4; // (f32) correction to the longitude at this node point (secs)

const NODE_SIZE: usize = 16;

/// This NTv2 grid parser is based on the following documents:
/// - https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf
/// - http://mimaka.com/help/gs/html/004_NTV2%20Data%20Format.htm
/// - https://github.com/Esri/ntv2-file-routines/blob/master/README.md
///
/// And inspired by existing implementations in
/// - https://github.com/proj4js/proj4js/blob/master/lib/nadgrid.js
/// - https://github.com/3liz/proj4rs/blob/main/src/nadgrids/grid.rs
pub struct NTv2Parser {
    buf: Box<[u8]>,
    endian: Endianness,
}

impl NTv2Parser {
    pub fn new(buf: Box<[u8]>) -> Self {
        // A NTv2 header is expected to have 11 records
        let endian = if buf[HEAD_NUM_RECORDS] == 11 {
            Endianness::Le
        } else {
            Endianness::Be
        };

        Self { buf, endian }
    }

    pub fn get_f64(&self, offset: usize) -> f64 {
        match self.endian {
            Endianness::Be => f64::from_be_bytes(self.buf[offset..offset + 8].try_into().unwrap()),
            Endianness::Le => f64::from_le_bytes(self.buf[offset..offset + 8].try_into().unwrap()),
        }
    }

    pub fn get_f32(&self, offset: usize) -> f32 {
        match self.endian {
            Endianness::Be => f32::from_be_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
            Endianness::Le => f32::from_le_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
        }
    }

    pub fn get_u32(&self, offset: usize) -> u32 {
        match self.endian {
            Endianness::Be => u32::from_be_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
            Endianness::Le => u32::from_le_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
        }
    }

    pub fn get_str(&self, offset: usize, len: usize) -> Result<&str, Error> {
        std::str::from_utf8(&self.buf[offset..offset + len]).map_err(Error::from)
    }

    pub fn cmp_str(&self, offset: usize, s: &str) -> bool {
        self.get_str(offset, s.len())
            .map(|x| x == s)
            .unwrap_or(false)
    }
}

/// Parse a subgrid header for an NTv2 grid
/// Weird sign conventions like longitude being west positive are handled here.
pub fn parse_subgrid_header(
    parser: &NTv2Parser,
    offset: usize,
) -> Result<(f64, f64, f64, f64, f64, f64, f64, f64, f64), Error> {
    let nlat = parser.get_f64(offset + SUBGRID_NLAT);
    let slat = parser.get_f64(offset + SUBGRID_SLAT);
    let wlon = parser.get_f64(offset + SUBGRID_WLON);
    let elon = parser.get_f64(offset + SUBGRID_ELON);
    let dlat = parser.get_f64(offset + SUBGRID_DLAT);
    let dlon = parser.get_f64(offset + SUBGRID_DLON);

    // As defined by https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf (pg 30)
    let num_rows = (((slat - nlat) / dlat).abs() + 1.0).floor();
    let row_size = (((wlon - elon) / dlon).abs() + 1.0).floor();

    let num_nodes = parser.get_u32(offset + SUBGRID_GSCOUNT) as f64;
    if num_nodes != (num_rows * row_size) {
        return Err(Error::Invalid(
            "Number of nodes does not match the grid size".to_string(),
        ));
    }

    Ok((
        nlat * SEC_TO_RAD,
        slat * SEC_TO_RAD,
        // By default the longitude is positive west. By conventions east is positive.
        // This is likely because the Canadian makers of NTv2 are always west of Greenwich.
        -wlon * SEC_TO_RAD,
        -elon * SEC_TO_RAD,
        dlat * SEC_TO_RAD,
        dlon * SEC_TO_RAD,
        num_rows,
        row_size,
        num_nodes,
    ))
}

/// Parses the nodes of a sub grid into a vector of lon/lat shifts in radians
pub fn parse_subgrid_grid(
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

            Coor2D::raw(lon_corr * SEC_TO_RAD, lat_corr * SEC_TO_RAD)
        })
        .collect::<Vec<Coor2D>>();

    // Switch the row order so that the lower left is the SW corner
    for i in 0..num_rows {
        let offs = i * row_size;
        grid[offs..(offs + row_size)].reverse();
    }

    let grid_end_offset = grid_start + num_nodes * NODE_SIZE;

    if grid_end_offset > parser.buf.len() {
        return Err(Error::Invalid("Grid Too Short".to_string()));
    }

    Ok(grid)
}
