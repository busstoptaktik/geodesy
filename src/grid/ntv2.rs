//! This NTv2 grid parser is based on the following documents:
//! - https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf
//! - http://mimaka.com/help/gs/html/004_NTV2%20Data%20Format.htm
//! - https://github.com/Esri/ntv2-file-routines/blob/master/README.md
//!
//! And inspired by existing implementations in
//! - https://github.com/proj4js/proj4js/blob/master/lib/nadgrid.js
//! - https://github.com/3liz/proj4rs/blob/main/src/nadgrids/grid.rs
//!
//! It was originally written by [Sean Rennie](https://github.com/Rennzie),
//! and gradually modified by Thomas Knudsen, to fit more tightly into the
//! evolving versions of Rust Geodesy's general grid architecture

use super::{BaseGrid, GridHeader, GridSource};
use crate::Error;

// Both overview and sub grid headers have 11 fields of 16 bytes each.
const HEADER_SIZE: usize = 11 * 16;
// Buffer offsets for the NTv2 overview header
const HEAD_NUM_RECORDS: usize = 8; // (i32) number of records in the file

// Buffer offsets for the NTv2 (sub-)grid header
const NAME: usize = 8;
const PARENT: usize = 24;
const NLAT: usize = 88;
const SLAT: usize = 72;
const ELON: usize = 104;
const WLON: usize = 120;
const DLAT: usize = 136;
const DLON: usize = 152;
const GSCOUNT: usize = 168;

// Buffer offsets for the NTv2 grid nodes
const NODE_LAT_CORRECTION: usize = 0;
const NODE_LON_CORRECTION: usize = 4;
const NODE_SIZE: usize = 16;

// For now, we need crate visibiity here, because contexts may (and currently does)
// need to care about the file format. In the future, everything should be
// handled by the Grid module, and the visibility here will change to pub(super)
pub(crate) fn ntv2_grid(buf: &[u8]) -> Result<BaseGrid, Error> {
    let parser = NTv2Parser::new(buf);

    // NUM_OREC is the NTv2 signature, i.e. "magic bytes"
    if !parser.cmp_str(0, "NUM_OREC") {
        return Err(Error::Unsupported("Not a NTv2 file".to_string()));
    }

    // If the number of records in the overview record is not 11, then
    // we have misdetermined the endianness (i.e. the file is corrupt)
    let num_overview_records = parser.get_u32(8) as usize;
    if num_overview_records != 11 {
        return Err(Error::Unsupported("Bad header".to_string()));
    }

    if !parser.cmp_str(56, "SECONDS") {
        return Err(Error::Invalid("Not in seconds".to_string()));
    }

    let num_sub_grids = parser.get_u32(40) as usize;

    let mut grids = Vec::new();

    let mut offset = HEADER_SIZE;
    for _ in 0..num_sub_grids {
        let (name, parent, grid) = parse_ntv2_grid(&parser, offset)?;
        let GridSource::Internal { values } = &grid.grid else {
            return Err(Error::Invalid("Bad subgrid".to_string()));
        };
        offset += HEADER_SIZE + values.len() / 2 * NODE_SIZE;
        grids.push((name, parent, grid));
    }

    let mut top;
    let topname;
    // Extract the top level ("overview") grid
    if let Some(index) = grids.iter().position(|(_, parent, _)| parent == "NONE") {
        (topname, _, top) = grids.swap_remove(index);
    } else {
        return Err(Error::Invalid(
            "Invalid NTv2 file: Missing top level grid".into(),
        ));
    }

    // No subgrids - then we're done
    if grids.is_empty() {
        return Ok(top);
    }

    // We do not support files with more than one top level grid
    if grids.iter().any(|(_, parent, _)| parent == "NONE") {
        return Err(Error::Invalid(
            "Unsupported NTv2 file: More than one top level grid".into(),
        ));
    }

    // Make sure, we have a direct descendant of the top level grid in pole position
    if let Some(index) = grids.iter().position(|(_, parent, _)| *parent == topname) {
        grids.swap(0, index);
    } else {
        return Err(Error::Invalid("Invalid NTv2 grid: Missing subgrid".into()));
    }

    // The NTv2 spec does not guarantee the order of subgrids, so we sort them such that
    // in all cases parent comes before child(ren).
    // Yes - basically this is bubblesort. We cannot use the built-in vec.sort_by(...),
    // because the comparison function here is (probably) not compatible, as it is not transitive.
    // But since grids.len() will typically be less than 10, using an O(n*n) algorithm here will
    // (typically) not be a problem
    for i in 0..grids.len() {
        for j in i..grids.len() {
            // i <= j, so if name[i]==parent[j], we swap them, so parent comes before name
            if grids[i].0 == grids[j].1 {
                grids.swap(i, j);
            }
        }
    }

    grids.reverse();
    for grid in grids {
        top.subgrids.push(grid.2);
    }
    Ok(top)
}

// Parse a (sub-)grid header for an NTv2 grid
// Weird sign conventions like longitude being west positive are handled here.
// By default the longitude is positive west. By conventions east is positive.
// This is likely because the Canadian makers of NTv2 are always west of Greenwich.
fn parse_ntv2_header(
    parser: &NTv2Parser,
    offset: usize,
) -> Result<(String, String, GridHeader), Error> {
    let lat_n = parser.get_f64(offset + NLAT);
    let lat_s = parser.get_f64(offset + SLAT);
    let lon_w = -parser.get_f64(offset + WLON);
    let lon_e = -parser.get_f64(offset + ELON);

    let dlat = parser.get_f64(offset + DLAT);
    let dlon = parser.get_f64(offset + DLON);

    let rows = (((lat_n - lat_s) / dlat).abs() + 1.5).floor() as usize;
    let cols = (((lon_e - lon_w) / dlon).abs() + 1.5).floor() as usize;

    let num_nodes = parser.get_u32(offset + GSCOUNT) as usize;
    if num_nodes != rows * cols {
        return Err(Error::Invalid(
            "Number of nodes does not match the grid size".to_string(),
        ));
    }

    // dlat is negative in the normal case, where the first row is northernmost
    // dlon is positive in the normal case, where the first column is westernmost
    // Natively, NTv2 use the opposite of the normal cases, but since we parse
    // the grid nodes in last-to-first order, so we assign dlat and dlon the
    // normal case values
    let dlat = dlat.copysign(lat_s - lat_n);
    let dlon = dlon.copysign(lon_e - lon_w);

    // In NTv2, all angular header items are given in seconds of arc,
    // so we convert to radians to follow the `GridHeader` convention
    let radians_from_arcsec = 1f64.to_radians() / 3600f64;
    let header = GridHeader {
        lat_n: lat_n * radians_from_arcsec,
        lat_s: lat_s * radians_from_arcsec,
        lon_w: lon_w * radians_from_arcsec,
        lon_e: lon_e * radians_from_arcsec,
        dlat: dlat * radians_from_arcsec,
        dlon: dlon * radians_from_arcsec,
        rows,
        cols,
        bands: 2,
    };

    let name = parser.get_str(offset + NAME, 8)?.trim().to_string();
    let parent = parser.get_str(offset + PARENT, 8)?.trim().to_string();

    Ok((name, parent, header))
}

fn parse_ntv2_grid(
    parser: &NTv2Parser,
    head_offset: usize,
) -> Result<(String, String, BaseGrid), Error> {
    let (name, parent, header) = parse_ntv2_header(parser, head_offset)?;

    let grid_start = head_offset + HEADER_SIZE;
    let grid = parse_ntv2_grid_nodes(parser, grid_start, header.rows * header.cols)?;
    let base_grid = BaseGrid::new_new(
        &name,
        header,
        crate::grid::GridSource::Internal { values: grid },
    )?;
    Ok((name, parent, base_grid))
}

// Parse the nodes of a (sub-)grid into a vector of lon/lat shifts in radians
fn parse_ntv2_grid_nodes(
    parser: &NTv2Parser,
    grid_start: usize,
    num_nodes: usize,
) -> Result<Vec<f32>, Error> {
    let grid_end_offset = grid_start + num_nodes * NODE_SIZE;
    if grid_end_offset > parser.buffer().len() {
        return Err(Error::Invalid("Grid Too Short".to_string()));
    }

    let mut grid = Vec::with_capacity(2 * num_nodes);
    // Yes, the NTv2 format stores the grid node upside down, right to left,
    // so we iterate our way through the nodes in reverse
    for i in (0..num_nodes).rev() {
        let offset = grid_start + i * NODE_SIZE;
        let lat_offset = offset + NODE_LAT_CORRECTION;
        let lon_offset = offset + NODE_LON_CORRECTION;

        let lat_corr = parser.get_f32(lat_offset) as f64;
        let lon_corr = -parser.get_f32(lon_offset) as f64;
        grid.push(lon_corr as f32);
        grid.push(lat_corr as f32);
    }

    Ok(grid)
}

struct NTv2Parser<'a> {
    buf: &'a [u8],
    is_big_endian: bool,
}

impl<'a> NTv2Parser<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        // A NTv2 header is expected to have 11 records
        let is_big_endian = buf[HEAD_NUM_RECORDS] != 11;
        Self { buf, is_big_endian }
    }

    pub fn get_f64(&self, offset: usize) -> f64 {
        match self.is_big_endian {
            true => f64::from_be_bytes(self.buf[offset..offset + 8].try_into().unwrap()),
            false => f64::from_le_bytes(self.buf[offset..offset + 8].try_into().unwrap()),
        }
    }

    pub fn get_f32(&self, offset: usize) -> f32 {
        match self.is_big_endian {
            true => f32::from_be_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
            false => f32::from_le_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
        }
    }

    pub fn get_u32(&self, offset: usize) -> u32 {
        match self.is_big_endian {
            true => u32::from_be_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
            false => u32::from_le_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
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

    pub fn buffer(&self) -> &[u8] {
        self.buf
    }
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::{Coor4D, CoordinateTuple};
    use crate::grid::Grid;
    use float_eq::assert_float_eq;

    #[test]
    fn basic_ntv2_grid() -> Result<(), Error> {
        // 100800401.gsb is used in the ED50 to ETRS89 (14) transformation for Catalonia.
        // The corresponding transformation is EPSG:5661
        let grid_buff = std::fs::read("geodesy/gsb/100800401.gsb").unwrap();
        let basegrid = ntv2_grid(&grid_buff)?;

        let barc = Coor4D::geo(41.3874, 2.1686, 0.0, 0.0);
        let ldn = Coor4D::geo(51.505, -0.09, 0., 0.);
        let first = Coor4D::geo(40.0, 3.5, 0., 0.);
        let next = Coor4D::geo(40.0, 0., 0., 0.);

        assert_eq!(basegrid.subgrids.len(), 0);
        let GridSource::Internal { values } = &basegrid.grid else {
            return Err(Error::Invalid(
                "Missing internal values in NTv2 file".to_string(),
            ));
        };
        assert_eq!(values.len(), 1591 * 2);

        assert_eq!(basegrid.bands(), 2);
        assert!(basegrid.contains(&barc, 0.5, true));
        assert!(!basegrid.contains(&ldn, 0.5, true));

        // Interpolation to a point on the southern boundary
        // expected values from
        //     ntv2_cvt -f 100800401.gsb 40 0
        // Followed by
        //     eva (39.99882421665721-40)*3600
        //     eva (-0.001203127834531996)*3600
        let (dlon, dlat) = basegrid.at(None, &next, 0.0).unwrap().xy();
        assert_float_eq!(dlat, -4.2328200340, abs_all <= 1e-6);
        assert_float_eq!(dlon, -4.3312602043, abs_all <= 1e-6);

        // Interpolation to the south-eastern corner, i.e. the
        // set of corrections placed physically first in the
        // file
        let (dlon, dlat) = basegrid.at(None, &first, 1.0).unwrap().xy();
        assert_float_eq!(dlat, -4.1843700409, abs_all <= 1e-6);
        assert_float_eq!(dlon, -3.9602699280, abs_all <= 1e-6);

        Ok(())
    }

    #[test]
    fn ntv2_multi_subgrid() -> Result<(), Error> {
        let grid_buff = std::fs::read("geodesy/gsb/5458_with_subgrid.gsb").unwrap();
        let basegrid = ntv2_grid(&grid_buff)?;
        assert!(basegrid.subgrids.len() == 1);
        Ok(())
    }

    #[test]
    fn ntv2_multi_subgrid_find_grid() -> Result<(), Error> {
        let grid_buff = std::fs::read("geodesy/gsb/5458_with_subgrid.gsb").unwrap();
        let basegrid = ntv2_grid(&grid_buff)?;

        // A point within the densified subgrid is contained by it
        let within_densified_grid = Coor4D::geo(55.5, 13.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(within_densified_grid, 0.)
            .unwrap();
        assert_eq!(grid_id, "5556");

        // A point on the upper latitude of the densified subgrid falls outside that grid
        let on_densified_upper_lat = Coor4D::geo(56.0, 13.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_densified_upper_lat, 0.0)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the eastmost longitude of the densified subgrid falls outside that grid
        let on_densified_upper_lon = Coor4D::geo(55.5, 14.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_densified_upper_lon, 0.0)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the lower latitude of the densified subgrid is contained by it
        let on_densified_lower_lat = Coor4D::geo(55.0, 13.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_densified_lower_lat, 0.0)
            .unwrap();
        assert_eq!(grid_id, "5556");

        // A point on the westmost longitude of the densified subgrid is contained by it
        let on_densified_lower_lon = Coor4D::geo(55.5, 12.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_densified_lower_lon, 0.0)
            .unwrap();
        assert_eq!(grid_id, "5556");

        // A point on the upper latitude of the base grid is contained by it
        let on_root_upper_lat = Coor4D::geo(58.0, 12.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_root_upper_lat, 0.0)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the eastmost longitude of the base grid is contained by it
        let on_root_upper_lon = Coor4D::geo(55.5, 16.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_root_upper_lon, 0.0)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the lower latitude of the base grid is contained by it
        let on_root_lower_lat = Coor4D::geo(54.0, 12.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_root_lower_lat, 0.0)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the westmost longitude of the base grid is contained by it
        let on_root_lower_lon = Coor4D::geo(55.5, 8.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_root_lower_lon, 0.0)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point within the margin of the upper latitude of the base grid is contained by it
        let on_root_upper_lat = Coor4D::geo(58.25, 12.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_root_upper_lat, 0.5)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point within the margin of the upper longitude of the base grid is contained by it
        let on_root_upper_lon = Coor4D::geo(55.5, 16.25, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_root_upper_lon, 0.5)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point within the margin of the lower latitude of the base grid is contained by it
        let on_root_lower_lat = Coor4D::geo(53.75, 12.0, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_root_lower_lat, 0.5)
            .unwrap();
        assert_eq!(grid_id, "5458");

        // A point within the margin of the lower longitude of the base grid is contained by it
        let on_root_lower_lon = Coor4D::geo(55.5, 7.75, 0.0, 0.0);
        let grid_id = basegrid
            .which_subgrid_contains(on_root_lower_lon, 0.5)
            .unwrap();
        assert_eq!(grid_id, "5458");

        Ok(())
    }
}
