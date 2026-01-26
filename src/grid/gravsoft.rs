use super::{BaseGrid, GridHeader, GridSource};
use crate::Error;
use std::io::BufRead;

pub fn gravsoft(name: &str, buf: &[u8]) -> Result<BaseGrid, Error> {
    let mut grids = gravsoft_grid_reader(name, buf)?;
    if grids.is_empty() {
        return Err(Error::Invalid("Empty grid: {name}".to_string()));
    }
    let mut main = grids.remove(0);
    grids.reverse();
    main.subgrids = grids;
    Ok(main)
}

// Read a gravsoft grid. Discard '#'-style comments
pub fn gravsoft_grid_reader(name: &str, buf: &[u8]) -> Result<Vec<BaseGrid>, Error> {
    let all = std::io::BufReader::new(buf);
    let mut grid_values = Vec::<f32>::new();
    let mut header_values = Vec::<f64>::new();
    let basename = name.to_string();
    let mut previous_name = name.to_string();
    let mut next_name;
    let mut grids = Vec::<BaseGrid>::new();
    const SUBGRID_DELIMITER: &str = "# ---";

    for line in all.lines() {
        let line = line?;

        // Beginning of a new subgrid? - construct the previous grid
        if line.starts_with(SUBGRID_DELIMITER) {
            // Is the subgrid name given as part of the delimiter line?
            next_name = line
                .strip_prefix(SUBGRID_DELIMITER)
                .unwrap_or("")
                .trim()
                .to_string();

            // If no name is given, we use the basename to constuct unambiguous names
            if next_name.is_empty() {
                previous_name = basename.clone() + "[" + &grids.len().to_string() + "]";
            }

            let previous_grid =
                gravsoft_grid_interpreter(&previous_name, &header_values, &mut grid_values)?;
            grids.push(previous_grid);

            previous_name = next_name;
            grid_values.clear();
            header_values.clear();
            continue;
        }

        // Remove comments
        let line = line.split('#').collect::<Vec<_>>()[0];
        // Convert to f64
        for item in line.split_whitespace() {
            let value = item.parse::<f64>().unwrap_or(f64::NAN);
            // In Gravsoft grids, the header is the first 6 numbers of the file
            if header_values.len() < 6 {
                header_values.push(value);
            } else {
                // In classic Gravsoft grids, NODATA==9999
                grid_values.push(if value == 9999. {
                    f32::NAN
                } else {
                    value as f32
                });
            }
        }
    }

    // Construct the last grid
    let name = if grids.is_empty() {
        basename.clone()
    } else if previous_name.is_empty() {
        basename + "[" + &grids.len().to_string() + "]"
    } else {
        previous_name.clone()
    };
    let last_grid = gravsoft_grid_interpreter(&name, &header_values, &mut grid_values)?;
    grids.push(last_grid);

    Ok(grids)
}

// transform raw gravsoft data into a proper BaseGrid
pub fn gravsoft_grid_interpreter(
    name: &str,
    header: &[f64],
    grid: &mut [f32],
) -> Result<BaseGrid, Error> {
    // The Gravsoft header has lat_s before lat_n
    let lat_n = header[1];
    let lat_s = header[0];
    let lon_w = header[2];
    let lon_e = header[3];

    // The Gravsoft header has inverted sign for dlat. We force
    // the two deltas to have signs compatible with the grid
    // organization
    let dlat = header[4].copysign(lat_s - lat_n);
    let dlon = header[5].copysign(lon_e - lon_w);
    let rows = (((lat_s - lat_n) / dlat).abs() + 1.5).floor() as usize;
    let cols = (((lon_e - lon_w) / dlon).abs() + 1.5).floor() as usize;
    let bands = grid.len() / (rows * cols);
    if (rows * cols * bands) > grid.len() || bands < 1 {
        return Err(Error::General("Incomplete Gravsoft grid"));
    }

    if (rows * cols * bands) != grid.len() {
        return Err(Error::General(
            "Unrecognized material at end of Gravsoft grid",
        ));
    }

    if bands > 3 {
        return Err(Error::General(
            "Unsupported number of bands in Gravsoft grid",
        ));
    }

    // Swap datum shift corrections into longitude/latitude order
    if bands == 2 {
        for i in 0..rows * cols {
            grid.swap(2 * i, 2 * i + 1);
        }
    }

    let header = GridHeader {
        lat_n: lat_n.to_radians(),
        lat_s: lat_s.to_radians(),
        lon_w: lon_w.to_radians(),
        lon_e: lon_e.to_radians(),
        dlat: dlat.to_radians(),
        dlon: dlon.to_radians(),
        rows,
        cols,
        bands,
    };

    let grid = GridSource::Internal {
        values: grid.into(),
    };
    let subgrids = Vec::new();
    let name = name.to_string();

    Ok(BaseGrid {
        name,
        header,
        grid,
        subgrids,
    })
}

#[cfg(test)]
mod tests {
    use crate::authoring::*;

    #[test]
    fn gravsoft_datum() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=test.datum")?;
        let cph = Coor4D::geo(55., 12., 0., 0.);
        let mut data = [cph];

        ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!((res[0] - 55.015278).abs() < 1e-6);
        assert!((res[1] - 12.003333).abs() < 1e-6);

        ctx.apply(op, Inv, &mut data)?;
        assert!((data[0][0] - cph[0]).abs() < 1e-10);
        assert!((data[0][1] - cph[1]).abs() < 1e-10);

        Ok(())
    }

    #[test]
    fn gravsoft_geoid_with_subgrid() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("gridshift grids=test_with_subgrid.geoid")?;

        // Inside the subgrid (Copenhagen - truncated to integers)
        let cph = Coor3D::geo(55., 12., 0.);
        let mut data = [cph];

        ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!(res[2] + 55.12001 < 1e-6);

        ctx.apply(op, Inv, &mut data)?;
        assert!((data[0][2] - cph[2]).abs() < 1e-10);

        // Outside of the subgrid (Gedser)
        let ged = Coor3D::geo(54., 12., 0.);
        let mut data = [ged];

        ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        dbg!(res[2]);
        assert!(res[2] as f32 + 54.12f32 < 1e-6);

        ctx.apply(op, Inv, &mut data)?;
        assert!((data[0][2] - ged[2]).abs() < 1e-10);

        // Raw characteristics
        let buf = include_bytes!("../../geodesy/geoid/test_with_subgrid.geoid");
        let grid = crate::grid::gravsoft::gravsoft("test_with_subgrid", buf)?;
        assert_eq!(grid.name, "test_with_subgrid");
        assert_eq!(grid.subgrids.len(), 1);
        assert_eq!(grid.subgrids[0].name, "the_subgrid");
        assert_eq!(grid.header.bands, 1);
        assert_eq!(grid.header.lat_n, 58f64.to_radians());
        assert_eq!(grid.subgrids[0].header.bands, 1);
        assert_eq!(grid.subgrids[0].header.lat_n, 57f64.to_radians());

        Ok(())
    }
}

// Additional relevant tests in
//     - src/grid/mod.rs,
//     - src/inner_op/gridshift.rs
//     - src/inner_op/deformation.rs
