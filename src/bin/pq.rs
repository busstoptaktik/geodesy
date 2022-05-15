/*! Plonketi Plonk! !*/
//! How to append a postscript to the help message generated.
// use geodesy::preamble::*;
use log::{debug, trace};
use std::path::PathBuf;
use structopt::StructOpt;

/// PQ: The Rust Geodesy blablabla program is called pq in order to have
/// an alphabetically continuous source code file name "PQ.RS".
/// We encourage porting to other languages, and look forward to the C,
/// Fortran, Matlab, and Lex versions: "AB.C" and "DE.F", "KL.M", "JK.L",
/// and the obvious inverted ML version "ON.ML"
#[derive(StructOpt, Debug)]
#[structopt(name = "pq")]
struct Opt {
    /// Inverse
    #[structopt(short, long = "inv")]
    _inverse: bool,

    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    _verbose: u8,

    // Set speed
    //#[structopt(short, long, default_value = "42")]
    //speed: f64,
    /// Output file, stdout if not present
    #[structopt(short, long, parse(from_os_str))]
    _output: Option<PathBuf>,

    // the long option will be translated by default to kebab case,
    // i.e. `--nb-cars`.
    // Number of cars
    // /#[structopt(short = "c", long)]
    //nb_cars: Option<i32>,
    /// Operation to apply
    #[structopt(name = "OPERATION", parse(from_str))]
    _operation: String,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    _files: Vec<PathBuf>,
}
fn main() -> Result<(), anyhow::Error> {
    // Filter by setting RUST_LOG to one of {Error, Warn, Info, Debug, Trace}
    if std::env::var("RUST_LOG").is_err() {
        simple_logger::init_with_level(log::Level::Error)?;
    } else {
        simple_logger::init_with_env()?;
    }

    let opt = Opt::from_args();
    println!("{:#?}", opt);
    debug!("debug message 1");
    trace!("trace message 1");

    // use std::env;
    use geodesy::Coord as C;
    let _ctx = geodesy::Minimal::default();
    trace!("trace message 2");
    debug!("debug message 2");

    let mut a = [0f64; 10];
    for (i, item) in [1f64, 2., 3.].into_iter().enumerate() {
        a[i] = item;
    }

    // let rp = geodesy::Plain::new(SearchLevel::LocalPatches, false);
    // rp.expand_experiment("jeg kan | hoppe sagde | lille Yrsa: Hansen");

    if opt.debug {
        if let Some(dir) = dirs::data_local_dir() {
            eprintln!("data_local_dir: {}", dir.to_str().unwrap_or_default());
        }
    }

    // let _oo = ctx.define_operation(&opt.operation)?;

    // A pipeline in Geodetic YAML Shorthand (GYS)
    let _gys =
        "geo | cart ellps:intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80 | geo inv";

    let copenhagen = C::raw(55., 12., 0., 0.);
    let stockholm = C::raw(59., 18., 0., 0.);
    let gys_data = [copenhagen, stockholm];
    for coord in gys_data {
        println!("    {:?}", coord);
    }
    Ok(())
}
