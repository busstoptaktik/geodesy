use clap::Parser;
use geodesy::prelude::*;
use log::{info, trace}; // debug, error, warn: not used
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time;

/// KP: The Rust Geodesy "Coordinate Processing" program. Called `kp` in honor
/// of Knud Poder (1925-2019), the nestor of computational geodesy, who would
/// have found it amusing to know that he provides a reasonable abbreviation
/// for something that would otherwise have collided with the name of the
/// Unix file copying program `cp`.
#[derive(Parser, Debug)]
#[command(name = "kp")]
#[command(author, version, about = "KP: The Rust Geodesy 'Coordinate Processing' program", long_about = None)]
struct Cli {
    /// The operation to carry out e.g. 'kp "utm zone=32"'
    operation: String,

    /// Inverse operation
    #[clap(long = "inv")]
    inverse: bool,

    /// Specify a fixed height for all coordinates
    #[clap(short = 'z', long)]
    height: Option<f64>,

    /// Specify a fixed observation time for all coordinates
    #[clap(short = 't', long)]
    time: Option<f64>,

    #[clap(short = 'd', long)]
    decimals: Option<usize>,

    /// Activate debug mode
    #[clap(long)]
    debug: bool,

    /// Report fwd-inv roundtrip deviation
    #[clap(short, long)]
    roundtrip: bool,

    /// Echo input to output
    #[clap(short, long)]
    echo: bool,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    /// Output file, stdout if not present
    #[clap(short, long)]
    _output: Option<PathBuf>,

    /// The files to operate on
    args: Vec<String>,
}

fn main() -> Result<(), anyhow::Error> {
    let mut options = Cli::parse();
    env_logger::Builder::new()
        .filter_level(options.verbose.log_level_filter())
        .init();

    log::trace!("This is KP");

    if options.debug {
        eprintln!("args: {:?}", options.args);
        if let Some(dir) = dirs::data_local_dir() {
            eprintln!("data_local_dir: {}", dir.to_str().unwrap_or_default());
        }
        eprintln!("options: {options:#?}");
    }

    // A dash, '-', given as file name indicates stdin
    if options.args.is_empty() {
        options.args.push("-".to_string());
    }

    // Create context and operator
    let start = time::Instant::now();
    let mut ctx = Plain::new();
    let duration = start.elapsed();
    trace!("Created context in: {duration:?}");
    let op = ctx.op(&options.operation)?;
    let duration = start.elapsed();
    trace!("Created operation in: {duration:?}");
    trace!("{op:#?}");

    // Get ready to read and transform input data
    let mut number_of_operands_read = 0_usize;
    let mut number_of_operands_succesfully_transformed = 0_usize;
    let mut operands = Vec::new();
    let start = time::Instant::now();

    // Now loop over all input files (of which stdin may be one)
    for arg in &options.args {
        let reader: Box<dyn BufRead> = if arg == "-" {
            Box::new(BufReader::new(std::io::stdin().lock()))
        } else {
            Box::new(BufReader::new(File::open(arg)?))
        };
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            let mut args: Vec<&str> = line.split_whitespace().collect();

            // Remove comments
            for (n, arg) in args.iter().enumerate() {
                if arg.starts_with('#') {
                    args.truncate(n);
                    break;
                }
            }
            let n = args.len();

            // Empty line
            if n < 1 {
                continue;
            }

            // Convert the text representation to a Coor4D
            args.extend(&(["0", "0", "0", "NaN", "0"][args.len()..]));
            let mut b: Vec<f64> = vec![];
            for e in args {
                b.push(angular::parse_sexagesimal(e));
            }
            b[2] = options.height.unwrap_or(b[2]);
            b[3] = options.time.unwrap_or(b[3]);

            let coord = Coor4D([b[0], b[1], b[2], b[3]]);
            number_of_operands_read += 1;
            operands.push(coord);

            // To avoid unlimited buffer growth, we send material
            // on to the transformation factory every time, we have
            // 25000 operands to operate on
            if operands.len() == 25000 {
                number_of_operands_succesfully_transformed +=
                    transform(&options, op, &mut operands, &ctx)?;
                operands.truncate(0);
            }
        }
    }

    // Transform the remaining coordinates
    number_of_operands_succesfully_transformed += transform(&options, op, &mut operands, &ctx)?;

    let duration = start.elapsed();
    info!("Read {number_of_operands_read} coordinates and succesfully transformed {number_of_operands_succesfully_transformed} in {duration:?}");

    Ok(())
}

fn transform(
    options: &Cli,
    op: OpHandle,
    operands: &mut Vec<Coor4D>,
    ctx: &Plain,
) -> Result<usize, geodesy::Error> {
    // Transformation - this is the actual geodetic content

    // When roundtripping, we must keep a copy of the input to be able
    // to compute the roundtrip differences
    let mut buffer = Vec::new();
    if options.roundtrip {
        buffer = operands.clone();
    }

    let mut n = if options.inverse {
        ctx.apply(op, Inv, operands)?
    } else {
        ctx.apply(op, Fwd, operands)?
    };

    // Roundtrip
    let m = if options.roundtrip {
        let m = if options.inverse {
            ctx.apply(op, Fwd, operands)?
        } else {
            ctx.apply(op, Inv, operands)?
        };
        if m != n {
            return Err(Error::General(
                "Roundtrip - mismatch between number of Fwd and Inv results",
            ));
        }

        for index in 0..n {
            operands[index] = operands[index] - buffer[index];
        }

        m
    } else {
        n
    };

    n = n.min(m);

    // If the number of output decimals are not given as option "-d",
    // we try guess a reasonable value, using the heuristic that if
    // the first coordinate is larger than 1000, the output is most
    // probably not in degrees. Hence give 5 decimals for linear units,
    // 10 for angular
    let decimals = options
        .decimals
        .unwrap_or(if operands[0][0] > 1000. { 5 } else { 10 });

    // Finally output the transformed coordinates
    for coord in operands {
        println!(
            "{1:.0$} {2:.0$} {3:.0$} {4:.0$} ",
            decimals, coord[0], coord[1], coord[2], coord[3]
        );
    }
    Ok(n)
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    fn some_basic_coordinates() -> [Coor4D; 2] {
        let copenhagen = Coor4D::raw(55., 12., 0., 0.);
        let stockholm = Coor4D::raw(59., 18., 0., 0.);
        [copenhagen, stockholm]
    }

    #[test]
    fn introspection() -> Result<(), Error> {
        let mut ctx = Minimal::new();

        let op = ctx.op("geo:in | utm zone=32 | neu:out")?;

        let mut data = some_basic_coordinates();
        let expected = [6098907.825005002, 691875.6321396609, 0., 0.];

        ctx.apply(op, Fwd, &mut data)?;
        assert_float_eq!(data[0].0, expected, abs_all <= 1e-9);

        // The text definitions of each step
        let steps = ctx.steps(op)?;
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0], "geo:in");
        assert_eq!(steps[1], "utm zone=32");
        assert_eq!(steps[2], "neu:out");

        // Behind the curtains, the two i/o-macros are just calls to the 'adapt' operator
        assert_eq!("adapt", ctx.params(op, 0)?.name);
        assert_eq!("adapt", ctx.params(op, 2)?.name);

        // While the utm step really is the 'utm' operator, not 'tmerc'-with-extras
        assert_eq!("utm", ctx.params(op, 1)?.name);

        // All the 'common' elements (lat_?, lon_?, x_?, y_? etc.) defaults to 0,
        // while ellps_? defaults to GRS80 - so they are there even though we havent
        // set them
        let params = ctx.params(op, 1)?;
        let ellps = params.ellps(0);
        assert_eq!(ellps.semimajor_axis(), 6378137.);
        assert_eq!(0., ctx.params(op, 1)?.lat(0));

        // The zone id is found among the natural numbers (which here includes 0)
        let zone = ctx.params(op, 1)?.natural("zone")?;
        assert_eq!(zone, 32);

        // Taking a look at the internals is not too hard either
        // let params = ctx.params(op, 0)?;
        // dbg!(params);

        Ok(())
    }
}
