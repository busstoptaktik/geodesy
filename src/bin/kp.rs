use clap::Parser;
use geodesy::authoring::Jacobian;
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

    /// Specify a base ellipsoid for evaluation of deformation factors,
    /// based on the jacobian
    #[clap(long)]
    factors: Option<String>,

    /// Specify a fixed height for all coordinates
    #[clap(short = 'z', long)]
    height: Option<f64>,

    /// Specify a fixed observation time for all coordinates
    #[clap(short = 't', long)]
    time: Option<f64>,

    /// Number of decimals in output
    #[clap(short = 'd', long)]
    decimals: Option<usize>,

    /// Output dimensionality - default: Estimate from input
    #[clap(short = 'D', long)]
    dimension: Option<usize>,

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
    let mut number_of_dimensions_in_input = 0;
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

            number_of_dimensions_in_input = number_of_dimensions_in_input.max(n);

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
                number_of_operands_succesfully_transformed += transform(
                    &options,
                    op,
                    number_of_dimensions_in_input,
                    &mut operands,
                    &ctx,
                )?;
                operands.truncate(0);
            }
        }
    }

    // Transform the remaining coordinates
    number_of_operands_succesfully_transformed += transform(
        &options,
        op,
        number_of_dimensions_in_input,
        &mut operands,
        &ctx,
    )?;

    let duration = start.elapsed();
    info!("Read {number_of_operands_read} coordinates and succesfully transformed {number_of_operands_succesfully_transformed} in {duration:?}");

    Ok(())
}

// Transformation - this is the actual geodetic content
fn transform(
    options: &Cli,
    op: OpHandle,
    number_of_dimensions_in_input: usize,
    operands: &mut Vec<Coor4D>,
    ctx: &Plain,
) -> Result<usize, geodesy::Error> {
    let output_dimension = options.dimension.unwrap_or(number_of_dimensions_in_input);

    // When roundtripping, or computing deformation factors,
    // we must keep a copy of the input to be able to compute
    // the roundtrip differences/factors from the Jacobian
    let mut buffer = Vec::new();
    buffer.clone_from(operands);

    let factors = options.factors.is_some();
    let ellps = if factors {
        Ellipsoid::named(options.factors.as_ref().unwrap())?
    } else {
        Ellipsoid::named("GRS80")?
    };

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
    for (index, coord) in operands.iter().enumerate() {
        match output_dimension {
            0 | 4 => print!(
                "{1:.0$} {2:.0$} {3:.0$} {4:.0$} ",
                decimals, coord[0], coord[1], coord[2], coord[3]
            ),
            1 => print!("{1:.0$} ", decimals, coord[0]),
            2 => print!("{1:.0$} {2:.0$} ", decimals, coord[0], coord[1]),
            3 => print!(
                "{1:.0$} {2:.0$} {3:.0$} ",
                decimals, coord[0], coord[1], coord[2]
            ),
            _ => print!(
                "{1:.0$} {2:.0$} {3:.0$} {4:.0$} ",
                decimals, coord[0], coord[1], coord[2], coord[3]
            ),
        }

        if factors {
            let scale = [1., 1.];
            let swap = [true, true];
            let at = Coor2D([buffer[index][0], buffer[index][1]]);
            let f = Jacobian::new(ctx, op, scale, swap, ellps, at)?.factors();
            if options.verbose.is_present() {
                // Full output, as with the proj "-V" option
                println!();
                println!("{f:#?}");
            } else {
                // Inline output, as with the proj "-S" option
                println!(
                    "  < {0:.10} {1:.10} {2:.10} | {3:.5} {4:.5} {5:.5} >",
                    f.meridional_scale,
                    f.parallel_scale,
                    f.areal_scale,
                    f.angular_distortion,
                    f.meridian_parallel_angle,
                    f.meridian_convergence
                );
            }
        } else {
            println!();
        }
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
