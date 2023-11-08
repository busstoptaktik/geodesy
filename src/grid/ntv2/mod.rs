use self::parser::HEADER_SIZE;
use crate::{Coor4D, Error, Grid};
use parser::NTv2Parser;
mod parser;
mod subgrid;
use self::subgrid::Ntv2SubGrid;

/// Grid for using the NTv2 format.
/// Interpolation has been adapted from a few sources
/// - [proj4rs](https://github.com/3liz/proj4rs/blob/8b5eb762c6be65eed0ca0baea33f8c70d1cd56cb/src/nadgrids/grid.rs#L206C1-L252C6)
/// - [NTv2 Archived Spec](https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf)
/// - [ESRI](https://github.com/Esri/ntv2-file-routines/blob/master/README.md)
/// to work with Rust Geodesy
#[derive(Debug, Default, Clone)]
pub struct Ntv2Grid {
    subgrids: Vec<Ntv2SubGrid>,
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
        subgrids.push(Ntv2SubGrid::new(&parser, HEADER_SIZE)?);

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

    /// Implementation adapted from [projrs](https://github.com/3liz/proj4rs/blob/8b5eb762c6be65eed0ca0baea33f8c70d1cd56cb/src/nadgrids/grid.rs#L206C1-L252C6) && [proj4js](https://github.com/proj4js/proj4js/blob/d9faf9f93ebeccac4b79fa80f3e9ad8a7032828b/lib/datum_transform.js#L167)
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
        // 100800401.gsb is used in the ED50 to ETRS89 (14) transformation for Catalonia - transformation is EPSG:5661
        let grid_buff = std::fs::read("tests/fixtures/100800401.gsb").unwrap();
        let ntv2_grid = Ntv2Grid::new(&grid_buff)?;

        let barc = Coor4D::geo(41.3874, 2.1686, 0.0, 0.0);
        let ldn = Coor4D::geo(51.505, -0.09, 0., 0.);

        assert_eq!(ntv2_grid.subgrids.len(), 1);
        assert_eq!(ntv2_grid.subgrids[0].grid.len(), 1591);

        assert_eq!(ntv2_grid.bands(), 2);
        assert!(ntv2_grid.contains(&barc, 0.5));
        assert!(!ntv2_grid.contains(&ldn, 0.5));

        Ok(())
    }
}
