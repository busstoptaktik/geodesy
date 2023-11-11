mod parser;
mod subgrid;

use self::subgrid::NODE_SIZE;
use super::BaseGrid;
use crate::{Coor4D, Error, Grid};
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
    fn find_grid(&self, coord: &Coor4D, margin: f64) -> Option<&BaseGrid> {
        // Start with grids whose parent grid id is `NONE`
        let mut current_parent_id: String = "NONE".to_string();
        let mut queue = self.lookup_table.get(&current_parent_id).unwrap().clone();

        while let Some(child_id) = queue.pop() {
            // Unwrappping is safe as a panic means we didn't
            // properly populate the `lookup_table` & `subgrids` properties
            let current_grid = self.subgrids.get(&child_id).unwrap();

            // The NTv2 spec has a myriad of different options for handling coordinates
            // that fall on the boundaries of a grid. We've chosen to ignore them for now
            // and return most dense AND first grid that contains the coordinate.
            // This should be relatively safe given then NTv2 spec does ensure that grids cannot overlap.
            // See the FGRID subroutine in the NTv2 spec linked above for more details.
            // NOTE: We may want to consider enforcing a margin of 0.0 for inner grids.
            if current_grid.contains(coord, margin) {
                current_parent_id = child_id.clone();

                if let Some(children) = self.lookup_table.get(&current_parent_id) {
                    queue = children.clone();
                    continue;
                }
                // If we get here it means the current_parent_id has no children and we have found the grid
                break;
            }
        }

        self.subgrids.get(&current_parent_id)
    }
}

impl Grid for Ntv2Grid {
    fn bands(&self) -> usize {
        2
    }

    /// Checks if a `Coord4D` is within the grid limits +- `margin` grid units
    fn contains(&self, position: &Coor4D, margin: f64) -> bool {
        if self.find_grid(position, margin).is_some() {
            return true;
        }

        false
    }

    fn at(&self, coord: &Coor4D, margin: f64) -> Option<Coor4D> {
        if let Some(grid) = self.find_grid(coord, margin) {
            return grid.at(coord, margin);
        }

        // If we get here the grid does not contain the coordinate
        None
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
}
