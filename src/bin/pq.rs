/*! Plonketi Plonk! !*/
//! How to append a postscript to the help message generated.
use anyhow::{Context, Result};
use log::{debug, trace};
use std::path::PathBuf;
use structopt::StructOpt;

/// PQ: The Rust Geodesy blablabla program is called pq in order to have
/// an alphabetically continuous source code file name "PQ.RS". We
/// encourage porting to other languages, and look forward to the C and
/// Fortran versions: "AB.C" and "DE.F".
#[derive(StructOpt, Debug)]
#[structopt(name = "pq")]
struct Opt {
    /// Inverse
    #[structopt(short, long = "inv")]
    inverse: bool,

    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    // Set speed
    //#[structopt(short, long, default_value = "42")]
    //speed: f64,
    /// Output file, stdout if not present
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    // the long option will be translated by default to kebab case,
    // i.e. `--nb-cars`.
    // Number of cars
    // /#[structopt(short = "c", long)]
    //nb_cars: Option<i32>,
    /// Operation to apply
    #[structopt(name = "OPERATION", parse(from_str))]
    operation: String,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}
fn main() -> Result<()> {
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
    use geodesy::CoordinateTuple as C;
    let mut ctx = geodesy::Context::new();
    trace!("trace message 2");
    debug!("debug message 2");

    if opt.debug {
        if let Some(dir) = dirs::data_local_dir() {
            eprintln!("data_local_dir: {}", dir.to_str().unwrap_or_default());
        }
    }

    let _oo = ctx.operation(&opt.operation)?;

    // A pipeline in YAML
    let pipeline = "ed50_etrs89: {
        steps: [
            adapt: {from: neut_deg},
            cart: {ellps: intl},
            helmert: {x: -87, y: -96, z: -120},
            cart: {inv: true, ellps: GRS80},
            adapt: {from: neut_deg}
        ]
    }";

    // The same pipeline in Geodetic YAML Shorthand (GYS)
    let gys = "geo | cart ellps:intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80 | geo inv";

    let op_yaml = ctx.operation(pipeline)?;
    let op_gys = ctx.operation(gys)?;

    let copenhagen = C::raw(55., 12., 0., 0.);
    let stockholm = C::raw(59., 18., 0., 0.);
    let mut yaml_data = [copenhagen, stockholm];
    let mut gys_data = [copenhagen, stockholm];

    ctx.fwd(op_yaml, &mut yaml_data);
    ctx.fwd(op_gys, &mut gys_data);

    println!("{:?}", yaml_data);
    println!("{:?}", gys_data);

    assert!(yaml_data[0].hypot3(&gys_data[0]) < 1e-16);
    assert!(yaml_data[1].hypot3(&gys_data[1]) < 1e-16);

    if false {
        if let Ok(utm32) = ctx.operation("utm: {zone: 32}") {
            let copenhagen = C::geo(55., 12., 0., 0.);
            let stockholm = C::geo(59., 18., 0., 0.);
            let mut data = [copenhagen, stockholm];

            ctx.fwd(utm32, &mut data);
            println!("{:?}", data);
        }

        let coo = C([1., 2., 3., 4.]);
        println!("coo: {:?}", coo);

        let geo = C::geo(55., 12., 0., 0.);
        let gis = C::gis(12., 55., 0., 0.);
        assert_eq!(geo, gis);
        println!("geo: {:?}", geo.to_geo());

        // Some Nordic/Baltic capitals
        let nuk = C::gis(-52., 64., 0., 0.); // Nuuk
        let tor = C::gis(-7., 62., 0., 0.); // Tórshavn
        let cph = C::gis(12., 55., 0., 0.); // Copenhagen
        let osl = C::gis(10., 60., 0., 0.); // Oslo
        let sth = C::gis(18., 59., 0., 0.); // Stockholm
        let mar = C::gis(20., 60., 0., 0.); // Mariehamn
        let hel = C::gis(25., 60., 0., 0.); // Helsinki
        let tal = C::gis(25., 59., 0., 0.); // Tallinn
        let rga = C::gis(24., 57., 0., 0.); // Riga
        let vil = C::gis(25., 55., 0., 0.); // Vilnius

        // Gothenburg is not a capital, but it is strategically placed
        // approximately equidistant from OSL, CPH and STH, so it
        // deserves special treatment by getting its coordinate
        // from direct inline construction, which is perfectly
        // possible: A coordinate is just an array of four double
        // precision floats
        let got = C::geo(58., 12., 0., 0.0);

        let mut data_all = [nuk, tor, osl, cph, sth, mar, hel, tal, rga, vil];
        let mut data_utm32 = [osl, cph, got];

        // We loop over the full dataset, and add some arbitrary time information
        for (i, dimser) in data_all.iter_mut().enumerate() {
            dimser[3] = i as f64;
        }

        let utm32 = ctx
            .operation("utm: {zone: 32}")
            .context("Awful UTM error")?;

        ctx.fwd(utm32, &mut data_utm32);
        println!("utm32:");
        for coord in data_utm32 {
            println!("    {:?}", coord);
        }

        // Try to read predefined transformation from zip archive
        let pladder = ctx.operation("ed50_etrs89").context("Awful ED50 error")?;
        ctx.fwd(pladder, &mut data_all);
        println!("etrs89:");
        for coord in data_all {
            println!("    {:?}", coord.to_geo());
        }

        let pipeline = "ed50_etrs89: {
        steps: [
            cart: {ellps: intl},
            helmert: {x: -87, y: -96, z: -120},
            cart: {inv: true, ellps: GRS80}
        ]
    }";

        let ed50_etrs89 = ctx.operation(pipeline).context("Awful repeated error")?;
        ctx.inv(ed50_etrs89, &mut data_all);
        println!("etrs89:");
        for coord in data_all {
            println!("    {:?}", coord.to_geo());
        }
    }
    Ok(())
}
