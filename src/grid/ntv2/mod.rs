mod parser;
mod subgrid;

use std::collections::BTreeMap;

use self::subgrid::NODE_SIZE;

use super::BaseGrid;
use crate::{Coor4D, Error, Grid};
use parser::{NTv2Parser, HEADER_SIZE};

/// Grid for using the NTv2 format.
#[derive(Debug, Default, Clone)]
pub struct Ntv2Grid {
    subgrids: BTreeMap<String, BaseGrid>,
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
            offset += HEADER_SIZE + grid.grid.len() * NODE_SIZE;

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
}

impl Grid for Ntv2Grid {
    fn bands(&self) -> usize {
        2
    }

    /// Checks if a `Coord4D` is margin the grid limits +- `margin` grid units
    fn contains(&self, position: &Coor4D, margin: f64) -> bool {
        // Ntv2 spec does not allow grid extensions, so we only need to check the root grid
        // https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf (pg 27)
        self.subgrids
            .get("0INT2GRS")
            .unwrap()
            .contains(position, margin)
    }

    fn at(&self, coord: &Coor4D, margin: f64) -> Option<Coor4D> {
        return self.subgrids.get("0INT2GRS").unwrap().at(coord, margin);

        // If we get here the grid does not contain the coordinate
        // None
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
