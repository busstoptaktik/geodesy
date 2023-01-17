/*! Plonketi Plonk! !*/
//! How to append a postscript to the help message generated.
// use geodesy::preamble::*;
use clap::Parser;
use geodesy::preamble::*;
use log::{debug, trace};
use std::path::PathBuf;

/// PQ: The Rust Geodesy blablabla program is called pq in order to have
/// an alphabetically continuous source code file name "PQ.RS".
/// We encourage porting to other languages, and look forward to the C,
/// Fortran, Matlab, and Lex versions: "AB.C" and "DE.F", "KL.M", "JK.L",
/// and the obvious inverted ML version "ON.ML"
#[derive(Parser, Debug)]
#[clap(name = "pq")]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Inverse
    #[clap(short, long = "inv")]
    _inverse: bool,

    /// Activate debug mode
    #[clap(short, long)]
    debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    _verbose: u8,

    /// Output file, stdout if not present
    #[clap(short, long = "output", parse(from_os_str))]
    _output: Option<PathBuf>,

    /// Operation to apply
    #[clap(name = "ARGS", parse(from_str))]
    _operation: Vec<String>,
}
fn main() -> Result<(), anyhow::Error> {
    // Filter by setting RUST_LOG to one of {Error, Warn, Info, Debug, Trace}
    if std::env::var("RUST_LOG").is_err() {
        simple_logger::init_with_level(log::Level::Error)?;
    } else {
        simple_logger::init_with_env()?;
    }

    let opt = Cli::parse();
    println!("{:#?}", opt);
    debug!("debug message 1");
    trace!("trace message 1");

    // We use ::new() instead of ::default() in order to gain access to the
    // BUILTIN_ADAPTORS
    let mut ctx = geodesy::Minimal::new();
    trace!("trace message 2");
    debug!("debug message 2");

    let mut a = [0f64; 10];
    for (i, item) in [1f64, 2., 3.].into_iter().enumerate() {
        a[i] = item;
    }

    if opt.debug {
        if let Some(dir) = dirs::data_local_dir() {
            eprintln!("data_local_dir: {}", dir.to_str().unwrap_or_default());
        }
    }

    // let _oo = ctx.define_operation(&opt.operation)?;

    // A pipeline
    let pip =
        "geo:in | cart ellps:intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80 | geo:out";
    let pip = ctx.op(pip)?;

    let copenhagen = Coord::geo(55., 12., 0., 0.);
    let stockholm = Coord::geo(59., 18., 0., 0.);
    let mut data = [copenhagen, stockholm];
    for coord in data {
        println!("    {:?}", coord.to_geo());
    }

    ctx.apply(pip, Fwd, &mut data)?;
    for coord in &data {
        println!("    {:?}", coord.to_geo());
    }

    Ok(())
}
