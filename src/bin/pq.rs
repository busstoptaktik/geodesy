/*! Plonketi Plonk! !*/
//! How to append a postscript to the help message generated.
use anyhow::{Context, Result};
use log::{debug, trace};
use std::{collections::HashMap, path::PathBuf};
use structopt::StructOpt;

use geodesy::CoordinateTuple;

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

    hold_fest();

    if opt.debug {
        if let Some(dir) = dirs::data_local_dir() {
            eprintln!("data_local_dir: {}", dir.to_str().unwrap_or_default());
        }
    }

    let _oo = ctx.operation(&opt.operation)?;

    // A pipeline in YAML
    let pipeline = "none: {
        steps: [
            adapt: {from: neut_deg},
            cart: {ellps: intl},
            helmert: {x: -87, y: -96, z: -120},
            cart: {inv: true, ellps: GRS80},
            adapt: {to: neut_deg}
        ]
    }";

    // The same pipeline in Geodetic YAML Shorthand (GYS)
    let gys = "geo | cart ellps:intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80 | geo inv";

    let op_gys = ctx.operation(gys)?;
    let op_yaml = ctx.operation(pipeline)?;

    let copenhagen = C::raw(55., 12., 0., 0.);
    let stockholm = C::raw(59., 18., 0., 0.);
    let mut yaml_data = [copenhagen, stockholm];
    let mut gys_data = [copenhagen, stockholm];
    for coord in yaml_data {
        println!("    {:?}", coord);
    }
    for coord in gys_data {
        println!("    {:?}", coord);
    }

    ctx.operate(op_yaml, &mut yaml_data, FWD);
    ctx.operate(op_gys, &mut gys_data, FWD);

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

use log::error;
use std::io;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum OpError {
    #[error("i/o error")]
    Io(#[from] io::Error),
    #[error("error: {0}")]
    General(&'static str),
    #[error("syntax error: {0}")]
    Syntax(String),
    #[error("{0}: {1}")]
    Operator(&'static str, &'static str),

    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("{message:?} (expected {expected:?}, found {found:?})")]
    Unexpected {
        message: String,
        expected: String,
        found: String,
    },
    #[error("operator {0} not found")]
    NotFound(String),
    #[error("too deep recursion for {0}")]
    Recursion(String),
    #[error("unknown error")]
    Unknown,
}

/// Indicate that a two-way operator, function, or method, should run in the *forward* direction.
pub const FWD: bool = true;
/// Indicate that a two-way operator, function, or method, should run in the *inverse* direction.
pub const INV: bool = false;

pub type OpConstructor =
    fn(args: &mut HashMap<String, String>, ctx: &mut geodesy::Context) -> Result<Op, OpError>;

pub type OpOperator =
    fn(op: &Op, ctx: &mut geodesy::Context, operands: &mut [CoordinateTuple]) -> usize;

pub fn noop_placeholder(
    _op: &Op,
    _ctx: &mut geodesy::Context,
    _operands: &mut [CoordinateTuple],
) -> usize {
    0
}

pub struct ConOp {
    pub invertible: bool,
    pub inverted: bool,
    pub definition: String,
    steps: Vec<Op>,
}

pub struct Op {
    fwd: OpOperator,
    inv: OpOperator,
    pub invertible: bool,
    pub inverted: bool,
    pub definition: String,
    pub args: HashMap<String, String>,
    a: f64,
    b: f64,
}

impl ConOp {
    /// The equivalent of the PROJ `proj_create()` function: Create an operator object
    /// from a text string.
    pub fn new(definition: &str, _ctx: &mut geodesy::Context) -> Result<ConOp, OpError> {
        let definition = definition.to_string(); //Context::gys_to_yaml(definition);
        let invertible = true;
        let inverted = false;
        // let mut args: HashMap<String, String> = HashMap::new();
        // oa.populate(&definition, "");
        // operator_factory(&mut oa, ctx, 0)
        let steps: Vec<Op> = Vec::new();
        Ok(ConOp {
            invertible,
            inverted,
            definition,
            steps,
        })
    }

    fn fwd(&self, _ctx: &mut geodesy::Context, operands: &mut [CoordinateTuple]) -> usize {
        let mut n = usize::MAX;
        for step in &self.steps {
            n = n.min(step.operate(_ctx, operands, FWD))
        }
        n
    }

    fn inv(&self, _ctx: &mut geodesy::Context, operands: &mut [CoordinateTuple]) -> usize {
        let mut n = usize::MAX;
        for step in self.steps.iter().rev() {
            n = n.min(step.operate(_ctx, operands, INV))
        }
        n
    }

    // operate fwd/inv, taking operator inversion into account.
    pub fn operate(
        &self,
        ctx: &mut geodesy::Context,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> usize {
        // Short form of (inverted && !forward) || (forward && !inverted)
        if self.inverted != forward {
            return self.fwd(ctx, operands);
        }
        // We do not need to check for self.invertible() here, since non-invertible
        // operators will return zero counts from fn noop_placeholder().
        self.inv(ctx, operands)
    }
}

impl Op {
    // operate fwd/inv, taking operator inversion into account.
    fn operate(
        &self,
        ctx: &mut geodesy::Context,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> usize {
        // Short form of (inverted && !forward) || (forward && !inverted)
        if self.inverted != forward {
            return (self.fwd)(self, ctx, operands);
        }
        // We do not need to check for self.invertible() here, since non-invertible
        // operators will no-op per default.
        (self.inv)(self, ctx, operands)
    }
}

use core::fmt::Debug;
impl Debug for Op {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Operator {{{}}}", self.definition)
    }
}

// ------------------------------------------------------------------------------------------
// USE_A
// ------------------------------------------------------------------------------------------
fn add_a(op: &Op, _ctx: &mut geodesy::Context, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = usize::MIN;
    for o in operands {
        o[0] += op.a;
        n += 1;
    }
    n
}

fn sub_a(op: &Op, _ctx: &mut geodesy::Context, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = usize::MIN;
    for o in operands {
        o[0] -= op.a;
        n += 1;
    }
    n
}

fn use_a(args: HashMap<String, String>, _ctx: &mut geodesy::Context) -> Result<Op, OpError> {
    let mut a = 42_f64;
    let b = 0_f64;
    if let Some(val) = args.get("a") {
        if let Ok(theval) = val.parse::<f64>() {
            a = theval;
        } else {
            return Err(OpError::Unexpected {
                message: "Bad value for a".to_string(),
                expected: "number".to_string(),
                found: val.to_string(),
            });
        }
    }
    let definition = format!("{:?}", args);
    Ok(Op {
        fwd: add_a,
        inv: sub_a,
        invertible: true,
        inverted: false,
        definition,
        args,
        a,
        b,
    })
}

fn use_use_a(args: HashMap<String, String>, ctx: &mut geodesy::Context) -> Result<Op, OpError> {
    let mut using_a = use_a(args, ctx).unwrap();
    using_a.b = 3.;
    Ok(using_a)
}

fn hold_fest() {
    let mut ctx = geodesy::Context::new();

    let mut args: HashMap<String, String> = HashMap::new();
    args.insert("a".to_string(), "37".to_string());
    let using_a = use_use_a(args, &mut ctx).unwrap();
    #[allow(clippy::float_cmp)]
    assert_eq!(using_a.b, 3.);
    let mut coords = [CoordinateTuple::origin()];

    let n = using_a.operate(&mut ctx, &mut coords, FWD);
    assert_eq!(n, 1);
    assert_eq!(37., coords[0][0]);
    println!("Vi fik {}", coords[0][0]);
    split_det_op();
}

fn split_det_op() {
    let all = "\n # agurk \n en # agurk\r\n  ## Arbejd med agurker \n##\n## agurker\n\ta b:c|  c   d: e    |f g:h|\t\th\n\n\n";
    let all = all.replace("\r", "\n").trim().to_string();
    println!("all = {}", all);

    // Collect docstrings and remove plain comments
    let mut trimmed = Vec::<String>::new();
    let mut docstring = Vec::<String>::new();
    println!("woot: '{}'", docstring.join("\n"));
    for line in all.lines() {
        let line = line.trim();

        // Collect docstrings
        if line.starts_with("##") {
            docstring.push((line.to_string() + "    ")[3..].trim_end().to_string());
            continue;
        }

        // Remove comments
        let line: Vec<&str> = line.trim().split('#').collect();
        if line[0].starts_with('#') {
            continue;
        }
        trimmed.push(line[0].trim().to_string());
    }

    // Finalize the docstring
    let docstring = docstring.join("\n").trim().to_string();

    // Remove superfluous newlines in the comment-trimmed text
    let trimmed = trimmed.join(" ").replace("\n", " ");
    println!("trimmed = '{}'", trimmed);

    // Generate trimmed steps with elements spearated by single ws and key-value pairs glued by ':' as in 'k:v'
    let steps: Vec<_> = trimmed.split("|").collect();
    let mut trimmed_steps = Vec::<String>::new();
    for mut step in steps {
        step = step.trim();
        let elements: Vec<_> = step.split_whitespace().collect();
        let joined = elements.join(" ").replace(": ", ":");
        trimmed_steps.push(joined);
    }

    println!("trimmed steps = {:#?}", trimmed_steps);
    println!("docstring = '{}'", docstring);
}
