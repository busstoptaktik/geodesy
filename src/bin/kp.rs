//! KP: The Rust Geodesy "Coordinate Processing" program is called kp rather
//! than the straightforward cp. Because cp is the Unix copy-command,
//! and because kp was the login name and email address of the late
//! Knud Poder (1925-2019), among colleagues and collaborators
//! rightfully considered the Nestor of computational
//! geodesy.

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "kp")]
struct Opt {
    /// Inverse
    #[structopt(short, long = "inv")]
    inverse: bool,

    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Echo input to output
    #[structopt(short, long)]
    echo: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Output file, stdout if not present
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Operation to apply
    #[structopt(name = "OPERATION", parse(from_str))]
    operation: String,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

use geodesy::CoordinateTuple as Coord;
use std::io::{self, BufRead};

fn main() {
    let opt = Opt::from_args();

    let mut ctx = geodesy::Context::new();

    if opt.debug {
        if let Some(dir) = dirs::data_local_dir() {
            eprintln!("data_local_dir: {}", dir.to_str().unwrap_or_default());
        }
        eprintln!("{:#?}", opt);
    }

    let op = ctx.operation(&opt.operation).unwrap();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let line = line.trim();

        let mut args: Vec<&str> = line.split_whitespace().collect();
        let n = args.len();

        // Empty line
        if n < 1 {
            continue;
        }

        // Convert test to CoordinateTuple
        args.extend(["0"; 4]);
        let mut b: Vec<f64> = vec![];
        for e in args {
            b.push(e.parse().unwrap_or(std::f64::NAN))
        }
        let coord = Coord::raw(b[0], b[1], b[2], b[3]);
        let mut data = [coord];

        // Transformation - this is the actual geodetic content
        if opt.inverse {
            ctx.inv(op, &mut data);
        } else {
            ctx.fwd(op, &mut data);
        }

        // Print output
        if opt.echo {
            println!("#  {}", line);
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
}
