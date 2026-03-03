use super::{BaseGrid, GridHeader, GridSource};
use crate::Error;

use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek};
use std::sync::Arc;

type UnigridIndex = BTreeMap<String, Arc<BaseGrid>>;

pub fn read_unigrid_index(paths: &[std::path::PathBuf]) -> Result<Vec<UnigridIndex>, Error> {
    let mut index = Vec::new();
    // We can have unigrids in multiple locations. Typically local, user, group,
    // and global conventional directories, so we loop over all levels, and
    // accept that not all levels need to be populated
    for (level, path) in paths.iter().enumerate() {
        let mut grids = BTreeMap::new();
        // Open the index file, with buffering
        let uniindex_path = path.join("unigrid.index");
        let Ok(uniindex) = File::options().read(true).open(uniindex_path) else {
            index.push(grids);
            continue;
        };
        let indexreader = BufReader::new(uniindex);

        // Open the grid file with buffering
        let unifile_path = path.join("unigrid.grids");
        let Ok(unifile) = File::options().read(true).open(unifile_path) else {
            index.push(grids);
            continue;
        };
        let mut gridreader = BufReader::new(unifile);

        // Each line is an index record, but we ignore blank lines and comments
        for line in indexreader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let args = line.split_whitespace().collect::<Vec<_>>();
            if args[0] == "#" {
                continue;
            }
            if args.len() != 4 {
                return Err(Error::Invalid(
                    "Cannot interpret `{line:#?}` in `{uniindex_path}` as a unigrid index record"
                        .into(),
                ));
            }

            // Parse the unigrid index record
            let grid_id = args[0].to_string();
            let grid_index = args[1].parse::<usize>()?;
            let hdr_offset = args[2].parse::<u64>()?;

            // Locate the header for the current grid_id
            gridreader.seek(std::io::SeekFrom::Start(hdr_offset))?;

            // And read the header
            let lat_n = gridreader.read_f64::<LittleEndian>()?;
            let lat_s = gridreader.read_f64::<LittleEndian>()?;
            let lon_w = gridreader.read_f64::<LittleEndian>()?;
            let lon_e = gridreader.read_f64::<LittleEndian>()?;

            let dlat = gridreader.read_f64::<LittleEndian>()?;
            let dlon = gridreader.read_f64::<LittleEndian>()?;

            let bands = gridreader.read_u64::<LittleEndian>()? as usize;
            let offset = gridreader.read_u64::<LittleEndian>()? as usize;

            let grid = GridSource::External { level, offset };

            // The BaseGrid constructor takes input as a Gravsoft style header
            let header = GridHeader::new(lat_n, lat_s, lon_w, lon_e, dlat, dlon, bands)?;
            let name = format!("{grid_id}[{grid_index}]");
            let grid = BaseGrid::new(&name, header, grid)?;

            // Parent grids (index==0) go into the grid collection, while subgrids
            // go into the `subgrids` vector of their parent
            if grid_index == 0 {
                grids.insert(grid_id, Arc::new(grid));
            } else {
                let Some(parent) = grids.get_mut(&grid_id) else {
                    return Err(Error::Invalid(
                        "Parent grid not found for subgrid {index} of {grid_id:?}".into(),
                    ));
                };
                Arc::get_mut(parent).unwrap().subgrids.push(grid);
            }
        }
        index.push(grids);
    }

    Ok(index)
}
