//! Very rudimentary maintenance tool for Unigrids. Supports only 5 subcommands:
//! - `add`: Add a new grid to an existing Unigrid (or creates it if not exiting)
//! - `list`: List the elementary grids comprising a unigrid
//! - `paths`: Show unigrid search paths
//! - `vacuum`: Placeholder for an upcomming clean up functionality
//! - `help` show help text
//!
use byteorder::{LittleEndian, WriteBytesExt};
use geodesy::authoring::*;

use core::f64;
use log::{info, trace};
use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::path::Path;
use std::time; // debug, error, warn: not used

const HEADER_SIZE: usize = 6 * size_of::<f64>() + 2 * size_of::<u64>();

use std::path::PathBuf;

use clap::{Command, arg};

fn cli() -> Command {
    Command::new("unigrid")
        .about("Handling Rust Geodesy Unigrids")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(false)
        .subcommand(
            Command::new("add")
                .about("Add grids to geodesy/unigrid.grids")
                .arg_required_else_help(true)
                .arg(arg!(<PATH> ... "Grids to add"))
                .arg(arg!(--force "Let new grids shadow older ones with the same name")),
        )
        .subcommand(
            Command::new("list")
                .about("List contents of unigrid in ./geodesy")
                .arg(arg!(--verbose "Show additional details")),
        )
        .subcommand(Command::new("paths").about("Show unigrid search paths"))
        .subcommand(Command::new("vacuum").about("Remove shadowed gridfiles (unimplemented)"))
}

fn main() -> Result<(), anyhow::Error> {
    // set RUST_LOG={error|info|debug|trace}
    env_logger::init();
    log::trace!("This is geodesy-grids");

    let matches = cli().get_matches();
    let ctx = Plain::new();

    match matches.subcommand() {
        Some(("list", sub_matches)) => {
            let index: PathBuf = "geodesy".into();
            let index = geodesy::grd::read_unigrid_index(&[index])?;
            let grids = index[0].keys();
            let verbose = sub_matches.get_flag("verbose");
            for grid in grids {
                let subgrids = &index[0][grid].subgrids;
                let n = subgrids.len();
                let header = &index[0][grid].header;
                println!("{grid} [{n}]");
                if verbose {
                    println!("    {header:?}");
                    for subgrid in subgrids {
                        let header = &subgrid.header;
                        println!("        {header:?}");
                    }
                    println!();
                }
            }
            Ok(())
        }
        Some(("paths", _sub_matches)) => {
            let paths = ctx.get_paths();
            println!("{paths:?}");
            Ok(())
        }
        Some(("vacuum", _sub_matches)) => {
            println!("Vacuuming, i.e. removing shadowed files, is not implemented yet");
            Ok(())
        }
        Some(("add", sub_matches)) => {
            let paths = sub_matches
                .get_many::<String>("PATH")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            let force = sub_matches.get_flag("force");
            add_grid_files_to_unigrid(&paths, force)?;
            Ok(())
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
    }

    // Continued program logic goes here...
}

fn add_grid_files_to_unigrid(args: &[&String], force: bool) -> Result<(), Error> {
    let args = args.iter();
    let unifile = File::options()
        .append(true)
        .create(true)
        .open("geodesy/unigrid.grids")?;
    let mut uniwriter = BufWriter::new(unifile);
    let mut uniindex = File::options()
        .append(true)
        .create(true)
        .open("geodesy/unigrid.index")?;

    let index: PathBuf = "geodesy".into();
    let index = geodesy::grd::read_unigrid_index(&[index])?;
    let grids: Vec<_> = index[0].keys().cloned().collect();

    'arg: for arg in args {
        info!("Handling input file `{arg}`");
        let start = time::Instant::now();

        let grid_path = Path::new(arg);

        // The `grid_id` is the bare filename, stripped of path and extension
        let grid_id = grid_path.file_stem().unwrap().to_str().unwrap().to_owned();
        if grids.contains(&grid_id) && !force {
            println!(
                "Adding {grid_id} would shadow existing element. Specify '--force' if this is intended."
            );
            continue;
        }

        let grid = geodesy::grd::read_grid(arg)?;
        let header = &grid.header;
        let records = header.rows * header.cols * header.bands;
        let duration = start.elapsed();
        trace!("Read {records} records in {duration:?}");

        // Check validity of subgrids
        for subgrid in &grid.subgrids {
            if subgrid.subgrids.is_empty() {
                continue;
            }
            let sub_id = subgrid.name.as_str();
            eprintln!(
                "Skipping the file {grid_path:?}: Its subgrid {sub_id:?}, contains recursive subgrids, which is not supported"
            );
            continue 'arg;
        }

        // Write basegrid into unigrid, and update uniindex
        let offset = write_single_grid_to_unigrid(&mut uniwriter, &grid)?;
        writeln!(uniindex, "{grid_id} 0 {offset} {grid_path:?}")?;

        // Repeat for all subgrids
        for (i, subgrid) in grid.subgrids.iter().enumerate() {
            let offset = write_single_grid_to_unigrid(&mut uniwriter, subgrid)?;
            // The basegrid holds id=0. Subgrid numbering starts at 1
            let corrected_index = i + 1;
            writeln!(
                uniindex,
                "{grid_id} {corrected_index} {offset} {grid_path:?}"
            )?;
        }

        let duration = start.elapsed();
        trace!("Done in {duration:?}");
    }
    Ok(())
}

// Returns the offset of the binary header (not of the grid)
fn write_single_grid_to_unigrid(
    uniwriter: &mut BufWriter<File>,
    grid: &BaseGrid,
) -> Result<u64, Error> {
    let offset = uniwriter.seek(std::io::SeekFrom::End(0))?;
    let header = &grid.header;
    let records = header.rows * header.cols * header.bands;

    // Write the header
    uniwriter.write_f64::<LittleEndian>(header.lat_n)?;
    uniwriter.write_f64::<LittleEndian>(header.lat_s)?;
    uniwriter.write_f64::<LittleEndian>(header.lon_w)?;
    uniwriter.write_f64::<LittleEndian>(header.lon_e)?;
    uniwriter.write_f64::<LittleEndian>(header.dlat)?;
    uniwriter.write_f64::<LittleEndian>(header.dlon)?;
    uniwriter.write_u64::<LittleEndian>(header.bands as u64)?;

    // Offset of the grid, not of the header: Vacuuming collects all headers
    // at the start of the file, for faster reading, so we need to keep the
    // physical start of the grid in the header, since it does not immediately
    // follow the binary header
    uniwriter.write_u64::<LittleEndian>(offset + HEADER_SIZE as u64)?;

    if let GridSource::Internal { values } = &grid.grid {
        let len = values.len();
        let rows = header.rows;
        let cols = header.cols;
        let bands = header.bands;
        let grid_id = &grid.name;
        if records != len {
            info!("{grid_id}, {records}, {len}, {rows}, {cols}, {bands}");
            return Err(Error::General("Mismatch between header info and grid size"));
        }
        // We do not know the size of the grid at compile time, so we need to write it element-by-element
        for element in values.iter() {
            uniwriter.write_f32::<LittleEndian>(*element)?;
        }
    } else {
        return Err(Error::General("Unexpected grid type"));
    }
    Ok(offset)
}
