mod parser;
mod subgrid;

use super::BaseGrid;
use crate::Error;
use parser::{HEADER_SIZE, NTv2Parser};
use subgrid::NODE_SIZE;

#[allow(dead_code)]
pub(crate) fn ntv2_basegrid(buf: &[u8]) -> Result<BaseGrid, Error> {
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

    let mut grids = Vec::new();

    let mut offset = HEADER_SIZE;
    for _ in 0..num_sub_grids {
        let (name, parent, grid) = subgrid::ntv2_subgrid(&parser, offset)?;
        offset += HEADER_SIZE + grid.grid.len() / 2 * NODE_SIZE;
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

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::Coor4D;
    use crate::grid::Grid;
    use float_eq::assert_float_eq;

    #[test]
    fn ntv2_grid() -> Result<(), Error> {
        // 100800401.gsb is used in the ED50 to ETRS89 (14) transformation for Catalonia.
        // The corresponding transformation is EPSG:5661
        let grid_buff = std::fs::read("geodesy/gsb/100800401.gsb").unwrap();
        let basegrid = ntv2_basegrid(&grid_buff)?;

        let barc = Coor4D::geo(41.3874, 2.1686, 0.0, 0.0);
        let ldn = Coor4D::geo(51.505, -0.09, 0., 0.);
        let first = Coor4D::geo(40.0, 3.5, 0., 0.);
        let next = Coor4D::geo(40.0, 0., 0., 0.);

        assert_eq!(basegrid.subgrids.len(), 0);
        assert_eq!(basegrid.grid.len(), 1591 * 2);

        assert_eq!(basegrid.bands(), 2);
        assert!(basegrid.contains(&barc, 0.5, true));
        assert!(!basegrid.contains(&ldn, 0.5, true));

        // Interpolation to a point on the southern boundary
        // expected values from
        //     ntv2_cvt -f 100800401.gsb 40 0
        // Followed by
        //     eva (39.99882421665721-40)*3600
        //     eva (-0.001203127834531996)*3600
        let v = basegrid.at(None, &next, 0.0).unwrap();
        let dlon = v[0].to_degrees() * 3600.0;
        let dlat = v[1].to_degrees() * 3600.0;
        assert_float_eq!(dlat, -4.2328200340, abs_all <= 1e-6);
        assert_float_eq!(dlon, -4.3312602043, abs_all <= 1e-6);

        // Interpolation to the south-eastern corner, i.e. the
        // set of corrections placed physically first in the
        // file
        let v = basegrid.at(None, &first, 1.0).unwrap();
        let dlon = v[0].to_degrees() * 3600.0;
        let dlat = v[1].to_degrees() * 3600.0;
        assert_float_eq!(dlat, -4.1843700409, abs_all <= 1e-6);
        assert_float_eq!(dlon, -3.9602699280, abs_all <= 1e-6);

        Ok(())
    }

    #[test]
    fn ntv2_multi_subgrid() -> Result<(), Error> {
        let grid_buff = std::fs::read("geodesy/gsb/5458_with_subgrid.gsb").unwrap();
        let basegrid = ntv2_basegrid(&grid_buff)?;
        assert!(basegrid.subgrids.len() == 1);
        Ok(())
    }

    #[test]
    fn ntv2_multi_subgrid_find_grid() -> Result<(), Error> {
        let grid_buff = std::fs::read("geodesy/gsb/5458_with_subgrid.gsb").unwrap();
        let basegrid = ntv2_basegrid(&grid_buff)?;

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
