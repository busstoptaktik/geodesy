use anyhow::bail;
use clap::Parser;
use geodesy::prelude::*;
use simple_logger::SimpleLogger;
use std::io::BufRead;
use std::path::PathBuf;
use std::time;

/// KP: The Rust Geodesy "Coordinate Processing" program. Called `kp` in honor
/// of Knud Poder (1925-2019), the nestor of computational geodesy, who would
/// have found it amusing to know that he provides a reasonable abbreviation
/// for something that would otherwise have collided with the name of the
/// Unix file copying program `cp`.

#[derive(Parser, Debug)]
#[clap(name = "kp")]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Inverse.
    /// Use of `inverse` mode excludes the use of `roundtrip` mode.
    #[clap(short, long = "inv")]
    inverse: bool,

    /// Activate debug mode
    #[clap(short, long)]
    debug: bool,

    /// Roundtrip mode - a signature feature of Knud Poder's programs:
    /// Evaluate the accuracy of the transformation by comparing the
    /// input argument with its supposedly identical alter ego after
    /// a forward+inverse transformation pair.
    /// Use of `roundtrip` mode excludes the use of `inverse` mode.
    #[clap(short, long)]
    roundtrip: bool,

    /// Echo input to output
    #[clap(short, long)]
    echo: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Output file, stdout if not present
    #[clap(short, long, parse(from_os_str))]
    _output: Option<PathBuf>,

    /// First argument is the operation to apply, the remaining the files to operate on
    args: Vec<String>,
}

fn main() -> Result<(), anyhow::Error> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .env()
        .init()
        .unwrap();
    log::trace!("This is KP");

    let opt = Cli::parse();
    println!("args: {:?}", opt.args);

    let mut ctx = Minimal::new();

    if opt.inverse && opt.roundtrip {
        bail!("Options `inverse` and `roundtrip` are mutually exclusive");
    }

    if opt.debug {
        if let Some(dir) = dirs::data_local_dir() {
            eprintln!("data_local_dir: {}", dir.to_str().unwrap_or_default());
        }
        eprintln!("opt: {opt:#?}");
    }

    if opt.args.is_empty() {
        return Ok(());
    }

    let start = time::Instant::now();
    let op = ctx.op(&opt.args[0])?;
    if opt.verbose > 2 {
        let duration = start.elapsed();
        println!("Created operation in: {duration:?}");
        println!("{op:#?}");
    }

    let start = time::Instant::now();
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        let line = line.trim();

        let mut args: Vec<&str> = line.split_whitespace().collect();
        let n = args.len();

        // Empty line
        if n < 1 {
            continue;
        }

        // Convert text to CoordinateTuple
        args.extend(["0"; 4]);
        let mut b: Vec<f64> = vec![];
        for e in args {
            b.push(e.parse().unwrap_or(std::f64::NAN))
        }
        let coord = Coord::raw(b[0], b[1], b[2], b[3]);
        let mut data = [coord];

        // Transformation - this is the actual geodetic content
        if opt.inverse {
            ctx.apply(op, Inv, &mut data)?;
            if opt.roundtrip {
                ctx.apply(op, Fwd, &mut data)?;
            }
        } else {
            ctx.apply(op, Fwd, &mut data)?;
            if opt.roundtrip {
                ctx.apply(op, Inv, &mut data)?;
            }
        }

        if opt.roundtrip {
            let d = roundtrip_distance(&opt.args[0], n, coord, data[0]);
            println!("{}:  d = {:.2} mm", line, 1000. * d);
            continue;
        }
        // Print output
        if opt.echo {
            println!("#  {line}");
        }
        if data[0][0] > 1000. {
            // Projected or cartesian coordinates
            println!(
                "{:.5} {:.5} {:.5} {:.5}",
                data[0][0], data[0][1], data[0][2], data[0][3]
            );
        } else {
            // Angular coordinates
            println!(
                "{:.10} {:.10} {:.5} {:.5}",
                data[0][0], data[0][1], data[0][2], data[0][3]
            );
        }
    }
    if opt.verbose > 1 {
        let duration = start.elapsed();
        println!("Transformed in: {duration:?}");
    }

    Ok(())
}

/// Distance between input and output after a forward-inverse roundtrip
fn roundtrip_distance(op: &str, dim: usize, mut input: Coord, mut result: Coord) -> f64 {
    // Try to figure out what kind of coordinates we're working with
    if op.starts_with("geo") {
        // Latitude, longitude...
        input = Coord::geo(input[0], input[1], input[2], input[3]);
        result = Coord::geo(result[0], result[1], result[2], result[3]);
    } else if op.starts_with("gis") {
        // Longitude, latitude...
        input = Coord::gis(input[0], input[1], input[2], input[3]);
        result = Coord::gis(result[0], result[1], result[2], result[3]);
    } else if dim < 3 {
        // 2D linear
        return input.hypot2(&result);
    } else {
        // 3D linear
        return input.hypot3(&result);
    }

    // 2D angular: geodesic distance
    if dim < 2 {
        return input.default_ellps_dist(&result);
    }

    // 3D angular: cartesian distance as-if-on GRS80
    input.default_ellps_3d_dist(&result)
}
