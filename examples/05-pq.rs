/*! Plonketi Plonk! !*/
//! How to append a postscript to the help message generated.
// use geodesy::prelude::*;
use clap::Parser;
use geodesy::prelude::*;
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
    let options = Cli::parse();
    env_logger::Builder::new()
        .filter_level(options.verbose.log_level_filter())
        .init();

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
    let pip = "cart ellps=intl | helmert x=-87 y=-96 z=-120 | cart inv ellps=GRS80 | geo:out";
    let pip = ctx.op(pip)?;

    let copenhagen = Coor4D::geo(55., 12., 0., 0.);
    let stockholm = Coor4D::geo(59., 18., 0., 0.);
    let mut data = [copenhagen, stockholm];
    for coord in data {
        println!("    {:?}", coord.to_geo());
    }

    ctx.apply(pip, Fwd, &mut data)?;
    for coord in &data {
        println!("    {:?}", coord);
    }

    Ok(())
}
