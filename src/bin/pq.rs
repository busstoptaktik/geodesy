/*! Plonketi Plonk! !*/
//! How to append a postscript to the help message generated.
use geodesy::{Provider, SearchLevel};
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
    operation: String,

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
    use geodesy::CoordinateTuple as C;
    let mut ctx = geodesy::Plain::new(SearchLevel::LocalPatches, false);
    trace!("trace message 2");
    debug!("debug message 2");

    let mut a = [0f64; 10];
    for (i, item) in [1f64, 2., 3.].into_iter().enumerate() {
        a[i] = item;
    }
    dbg!(a);

    let rp = geodesy::Plain::new(SearchLevel::LocalPatches, false);
    rp.expand_experiment("jeg kan | hoppe sagde | lille Yrsa: Hansen");

    // have_a_ball();

    if opt.debug {
        if let Some(dir) = dirs::data_local_dir() {
            eprintln!("data_local_dir: {}", dir.to_str().unwrap_or_default());
        }
    }

    let _oo = ctx.define_operation(&opt.operation)?;

    // A pipeline in Geodetic YAML Shorthand (GYS)
    let _gys =
        "geo | cart ellps:intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80 | geo inv";

    let copenhagen = C::raw(55., 12., 0., 0.);
    let stockholm = C::raw(59., 18., 0., 0.);
    let mut _gys_data = [copenhagen, stockholm];
    for coord in _gys_data {
        println!("    {:?}", coord);
    }
    Ok(())
}

/* Ultrasmall geodetic transformation system experiment

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

fn have_a_ball() {
    let mut ctx = geodesy::Context::new();

    let mut args: HashMap<String, String> = HashMap::new();
    args.insert("a".to_string(), "37".to_string());
    let using_a = use_use_a(args, &mut ctx).unwrap();
    assert!((using_a.b - 3.).abs() < f64::EPSILON);
    let mut coords = [CoordinateTuple::origin()];

    let n = using_a.operate(&mut ctx, &mut coords, FWD);
    assert_eq!(n, 1);
    assert!((37. - coords[0][0]).abs() < f64::EPSILON);
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
    let steps: Vec<_> = trimmed.split('|').collect();
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

fn value_of_key(
    key: &str,
    globals: &[(String, String)],
    locals: &[(String, String)],
) -> Result<String, GeodesyError> {
    // The haystack is a reverse iterator over both lists in series
    let mut haystack = globals.iter().chain(locals.iter()).rev();

    // Find the needle in the haystack, recursively chasing look-ups ('^')
    let key = key.trim();
    let mut needle = key;
    let mut chasing = false;
    loop {
        let found = haystack.find(|&x| x.0 == needle);
        if found.is_none() {
            if chasing {
                return Err(GeodesyError::Syntax(format!(
                    "Incomplete definition for '{}'",
                    key
                )));
            }
            return Err(GeodesyError::NotFound(String::from(key)));
        }
        let thevalue = found.unwrap().1.trim();

        // If the value is a(nother) lookup, we continue the search in the same iterator
        if thevalue.starts_with("^") {
            chasing = true;
            needle = &thevalue[1..];
            continue;
        }

        // Otherwise we have the proper result
        return Ok(String::from(thevalue.trim()));
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operator_args() -> Result<(), GeodesyError> {
        let globals: [(String, String); 6] = [
            (String::from("a"), String::from("a def")),
            (String::from("b"), String::from("b def")),
            (String::from("c"), String::from("c def")),
            (String::from("d"), String::from("d def")),
            (String::from("e"), String::from("e def")),
            (String::from("f"), String::from("f def")),
        ];

        let locals: [(String, String); 6] = [
            (String::from("a"), String::from("   ^b  ")),
            (String::from("b"), String::from("2 b def")),
            (String::from("c"), String::from("2 c def")),
            (String::from("d"), String::from("^2 d def")),
            (String::from("e"), String::from("    2 e def   ")),
            (String::from("f"), String::from("^a")),
        ];

        let f = value_of_key("  f  ", &globals, &locals)?;
        assert_eq!(f, globals[1].1);

        let e = value_of_key("  e  ", &globals, &locals)?;
        assert_eq!(e, "2 e def");

        if let Err(d) = value_of_key("  d  ", &globals, &locals) {
            println!("d: {:?}", d.to_string());
            assert!(d.to_string().starts_with("syntax error"));
        }
        let d = value_of_key("  d  ", &globals, &locals).unwrap_err();
        assert!(d.to_string().starts_with("syntax error"));

        let _d = value_of_key("  d  ", &globals, &locals).unwrap_or_else(|e| {
            if !e.to_string().starts_with("syntax error") {
                panic!("Expected syntax error here!");
            } else {
                String::default()
            }
        });

        Ok(())
    }
}
*/
