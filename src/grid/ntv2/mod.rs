mod parser;
mod subgrid;

use self::subgrid::NODE_SIZE;
use super::BaseGrid;
use crate::{coord::Coor4D, grid::Grid, Error};
use parser::{NTv2Parser, HEADER_SIZE};
use std::collections::BTreeMap;

/// Grid for using the NTv2 format.
#[derive(Debug, Default, Clone)]
pub struct Ntv2Grid {
    // Subgrids stored by their `SUBNAME` property
    subgrids: BTreeMap<String, BaseGrid>,

    // Lookup table for finding subgrids by their `PARENT` property
    // The key is the `PARENT` property and the value is a vector of `SUBNAME` properties
    // It's expected that root subgrids have a `PARENT` property of `NONE`
    lookup_table: BTreeMap<String, Vec<String>>,
}

impl Ntv2Grid {
    pub fn new(buf: &[u8]) -> Result<Self, Error> {
        let parser = NTv2Parser::new(buf.into());

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

        let mut subgrids = BTreeMap::new();
        let mut lookup_table = BTreeMap::new();

        let mut offset = HEADER_SIZE;
        for _ in 0..num_sub_grids {
            let (name, parent, grid) = subgrid::ntv2_subgrid(&parser, offset)?;
            offset += HEADER_SIZE + grid.grid.len() / 2 * NODE_SIZE;

            // The NTv2 spec does not guarantee the order of subgrids, so we must create
            // a lookup table from parent to children to make it possible for `find_grid` to
            // have a start point for working out which subgrid, if any, contains the point
            subgrids.insert(name.clone(), grid);
            lookup_table
                .entry(parent)
                .or_insert_with(Vec::new)
                .push(name);
        }

        Ok(Self {
            subgrids,
            lookup_table,
        })
    }

    // As defined by the FGRID subroutine in the NTv2 [spec](https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf) (page 42)
    fn find_grid(&self, coord: &Coor4D, margin: f64) -> Option<(String, &BaseGrid)> {
        // Start with the base grids whose parent id is `NONE`
        let mut current_grid_id: String = "NONE".to_string();
        let mut queue = self.lookup_table.get(&current_grid_id).unwrap().clone();

        while let Some(grid_id) = queue.pop() {
            // Unwrapping is safe because a panic means we didn't
            // properly populate the `lookup_table` & `subgrids` properties
            let current_grid = self.subgrids.get(&grid_id).unwrap();

            // We do a strict margin check on the first pass because grids cannot overlap in the NTv2 spec
            if current_grid.contains(coord, 1e-6) {
                // BaseGrid considers on the line to be in but by NTv2 standards points on
                // either upper latitude or longitude are considered outside the grid.
                // We explicitly check for this case here and keep trying if it happens.
                let (lat_n, lon_e) = (current_grid.lat_n, current_grid.lon_e);
                if (coord[0] - lon_e).abs() < 1e-6 || (coord[1] - lat_n).abs() < 1e-6 {
                    continue;
                }

                current_grid_id = grid_id.clone();

                if let Some(children) = self.lookup_table.get(&current_grid_id) {
                    queue = children.clone();
                } else {
                    // If we get here it means the current_parent_id has no children and we've found the grid
                    break;
                }
            }
        }

        if let Some(grid) = self.subgrids.get(&current_grid_id) {
            return Some((current_grid_id, grid));
        }

        // There's a chance the point fell on the upper boundary of one of the base grids,
        // or it's within the specified margin. If this happens we re-evaluate the
        // base grids, this time using the specified margin.
        // At this point we've evaluated all the internal boundaries between grids and found no
        // match. That means the only possible option is that one of the base grids contains the point
        // within it's outer margin.
        if current_grid_id == "NONE" {
            // Find the first base grid which contain the point +- the margin, if at all.
            for base_grid_id in self.lookup_table.get(&current_grid_id).unwrap() {
                if let Some(base_grid) = self.subgrids.get(base_grid_id) {
                    if base_grid.contains(coord, margin) {
                        return Some((base_grid_id.clone(), base_grid));
                    }
                }
            }
        }

        // None of the subgrids contain the point
        None
    }
}

impl Grid for Ntv2Grid {
    fn bands(&self) -> usize {
        2
    }

    /// Checks if a `Coord4D` is within the grid limits +- `margin` grid units
    fn contains(&self, position: &Coor4D, margin: f64) -> bool {
        self.find_grid(position, margin).is_some()
    }

    fn at(&self, coord: &Coor4D, margin: f64) -> Option<Coor4D> {
        self.find_grid(coord, margin)
            .and_then(|grid| grid.1.at(coord, margin))
    }
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn ntv2_grid() -> Result<(), Error> {
        // 100800401.gsb is used in the ED50 to ETRS89 (14) transformation for Catalonia.
        // The corresponding transformation is EPSG:5661
        let grid_buff = std::fs::read("geodesy/gsb/100800401.gsb").unwrap();
        let ntv2_grid = Ntv2Grid::new(&grid_buff)?;

        let barc = Coor4D::geo(41.3874, 2.1686, 0.0, 0.0);
        let ldn = Coor4D::geo(51.505, -0.09, 0., 0.);
        let first = Coor4D::geo(40.0, 3.5, 0., 0.);
        let next = Coor4D::geo(40.0, 0., 0., 0.);

        assert_eq!(ntv2_grid.subgrids.len(), 1);
        assert_eq!(
            ntv2_grid.subgrids.get("0INT2GRS").unwrap().grid.len(),
            1591 * 2
        );

        assert_eq!(ntv2_grid.bands(), 2);
        assert!(ntv2_grid.contains(&barc, 0.5));
        assert!(!ntv2_grid.contains(&ldn, 0.5));

        // Interpolation to a point on the southern boundary
        // expected values from
        //     ntv2_cvt -f 100800401.gsb 40 0
        // Followed by
        //     eva (39.99882421665721-40)*3600
        //     eva (-0.001203127834531996)*3600
        let v = ntv2_grid.at(&next, 0.0).unwrap();
        let dlon = v[0].to_degrees() * 3600.0;
        let dlat = v[1].to_degrees() * 3600.0;
        dbg!((dlon, dlat));
        assert_float_eq!(dlat, -4.2328200340, abs_all <= 1e-6);
        assert_float_eq!(dlon, -4.3312602043, abs_all <= 1e-6);

        // Interpolation to the south-eastern corner, i.e. the
        // set of corrections placed physically first in the
        // file
        let v = ntv2_grid.at(&first, 1.0).unwrap();
        let dlon = v[0].to_degrees() * 3600.0;
        let dlat = v[1].to_degrees() * 3600.0;
        dbg!((dlon, dlat));
        assert_float_eq!(dlat, -4.1843700409, abs_all <= 1e-6);
        assert_float_eq!(dlon, -3.9602699280, abs_all <= 1e-6);
        Ok(())
    }

    #[test]
    fn ntv2_multi_subgrid() -> Result<(), Error> {
        let grid_buff = std::fs::read("geodesy/gsb/5458_with_subgrid.gsb").unwrap();
        let ntv2_grid = Ntv2Grid::new(&grid_buff)?;

        assert!(ntv2_grid.subgrids.len() == 2);
        assert!(!ntv2_grid.lookup_table.get("NONE").unwrap().is_empty());
        assert!(ntv2_grid
            .lookup_table
            .get("NONE")
            .unwrap()
            .contains(&"5458".to_string()));

        // Grids with no children do not appear in the lookup table
        assert!(ntv2_grid.lookup_table.get("5556").is_none());

        Ok(())
    }

    #[test]
    fn ntv2_multi_subgrid_find_grid() -> Result<(), Error> {
        let grid_buff = std::fs::read("geodesy/gsb/5458_with_subgrid.gsb").unwrap();
        let ntv2_grid = Ntv2Grid::new(&grid_buff)?;

        // A point within the densified subgrid is contained by it
        let within_densified_grid = Coor4D::geo(55.5, 13.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&within_densified_grid, 1e-6).unwrap();
        assert_eq!(grid_id, "5556");

        // A point on the upper latitude of the densified subgrid falls outside that grid
        let on_densified_upper_lat = Coor4D::geo(56.0, 13.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_densified_upper_lat, 1e-6).unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the upper longitude of the densified subgrid falls outside that grid
        let on_densified_upper_lon = Coor4D::geo(55.5, 14.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_densified_upper_lon, 1e-6).unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the lower latitude of the densified subgrid is contained by it
        let on_densified_lower_lat = Coor4D::geo(55.0, 13.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_densified_lower_lat, 1e-6).unwrap();
        assert_eq!(grid_id, "5556");

        // A point on the lower longitude of the densified subgrid is contained by it
        let on_densified_lower_lon = Coor4D::geo(55.5, 12.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_densified_lower_lon, 1e-6).unwrap();
        assert_eq!(grid_id, "5556");

        // A point on the upper latitude of the base grid is contained by it
        let on_root_upper_lat = Coor4D::geo(58.0, 12.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_root_upper_lat, 1e-6).unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the upper longitude of the base grid is contained by it
        let on_root_upper_lon = Coor4D::geo(55.5, 16.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_root_upper_lon, 1e-6).unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the lower latitude of the base grid is contained by it
        let on_root_lower_lat = Coor4D::geo(54.0, 12.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_root_lower_lat, 1e-6).unwrap();
        assert_eq!(grid_id, "5458");

        // A point on the lower longitude of the base grid is contained by it
        let on_root_lower_lon = Coor4D::geo(55.5, 8.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_root_lower_lon, 1e-6).unwrap();
        assert_eq!(grid_id, "5458");

        // A point within the margin of the upper latitude of the base grid is contained by it
        let on_root_upper_lat = Coor4D::geo(58.25, 12.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_root_upper_lat, 0.5).unwrap();
        assert_eq!(grid_id, "5458");

        // A point within the margin of the upper longitude of the base grid is contained by it
        let on_root_upper_lon = Coor4D::geo(55.5, 16.25, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_root_upper_lon, 0.5).unwrap();
        assert_eq!(grid_id, "5458");

        // A point within the margin of the lower latitude of the base grid is contained by it
        let on_root_lower_lat = Coor4D::geo(53.75, 12.0, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_root_lower_lat, 0.5).unwrap();
        assert_eq!(grid_id, "5458");

        // A point within the margin of the lower longitude of the base grid is contained by it
        let on_root_lower_lon = Coor4D::geo(55.5, 7.75, 0.0, 0.0);
        let (grid_id, _) = ntv2_grid.find_grid(&on_root_lower_lon, 0.5).unwrap();
        assert_eq!(grid_id, "5458");

        Ok(())
    }
}
