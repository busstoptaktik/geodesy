use self::parser::HEADER_SIZE;
use crate::{Coor4D, Error, Grid};
use parser::NTv2Parser;
mod parser;
mod subgrid;
use super::BaseGrid;

/// Grid for using the NTv2 format.
#[derive(Debug, Default, Clone)]
pub struct Ntv2Grid {
    subgrids: Vec<BaseGrid>,
}

impl Ntv2Grid {
    pub fn new(buf: &[u8]) -> Result<Self, Error> {
        let parser = NTv2Parser::new(buf.into());

        let num_sub_grids = parser.get_u32(40) as usize;
        if num_sub_grids != 11 && parser.cmp_str(8, "NUM_OREC") {
            return Err(Error::Unsupported("Wrong header".to_string()));
        }

        if num_sub_grids != 1 {
            // TODO: Add support for subgrids
            return Err(Error::Unsupported(
                "Contains more than one subgrid".to_string(),
            ));
        }

        if !parser.cmp_str(56, "SECONDS") {
            return Err(Error::Invalid("Not in seconds".to_string()));
        }

        let mut subgrids = Vec::with_capacity(num_sub_grids);
        subgrids.push(subgrid::ntv2_subgrid(&parser, HEADER_SIZE)?);

        Ok(Self { subgrids })
    }
}

impl Grid for Ntv2Grid {
    fn bands(&self) -> usize {
        2
    }

    /// Checks if a `Coord4D` is within the grid limits +- `within` grid units
    fn contains(&self, position: &Coor4D, within: f64) -> bool {
        // Ntv2 spec does not allow grid extensions, so we only need to check the root grid
        // https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf (pg 27)
        self.subgrids[0].contains(position, within)
    }

    fn at(&self, coord: &Coor4D, within: f64) -> Option<Coor4D> {
        // NOTE: This may be naive as the spec suggests the order of subgrids is not guaranteed
        // It's ok for now because we're only supporting single subgrid grids
        for subgrid in self.subgrids.iter().rev() {
            if let Some(result) = subgrid.at(coord, within) {
                return Some(result);
            }
        }

        // If we get here the grid does not contain the coordinate
        None
    }
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntv2_grid() -> Result<(), Error> {
        // 100800401.gsb is used in the ED50 to ETRS89 (14) transformation for Catalonia.
        // The corresponding transformation is EPSG:5661
        let grid_buff = std::fs::read("geodesy/gsb/100800401.gsb").unwrap();
        let ntv2_grid = Ntv2Grid::new(&grid_buff)?;

        let barc = Coor4D::geo(41.3874, 2.1686, 0.0, 0.0);
        let ldn = Coor4D::geo(51.505, -0.09, 0., 0.);

        assert_eq!(ntv2_grid.subgrids.len(), 1);
        assert_eq!(ntv2_grid.subgrids[0].grid.len(), 1591 * 2);

        assert_eq!(ntv2_grid.bands(), 2);
        assert!(ntv2_grid.contains(&barc, 0.5));
        assert!(!ntv2_grid.contains(&ldn, 0.5));

        Ok(())
    }
}
