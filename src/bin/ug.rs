#![allow(dead_code, unused_variables)]

/*

key, ns, name: [A-Za-z0-9]*
value: ^[:^|]

id: :name, ns:name
parameter: key=value


*/

use geodesy::{CoordinateTuple as Coord, Ellipsoid};
use std::collections::{BTreeMap, BTreeSet};

use log::{error, info, warn};
use std::io;
use thiserror::Error;

// -----------------------------------------------------------------------------
// UG: An experiment with an *U*ltrasmall *G*eodetic transformation system
// -----------------------------------------------------------------------------

fn main() -> Result<(), anyhow::Error> {
    // Filter by setting RUST_LOG to one of {Error, Warn, Info, Debug, Trace}
    if std::env::var("RUST_LOG").is_err() {
        simple_logger::init_with_level(log::Level::Info)?;
        let yes = "fino";
        info!("Logging at info level! - {yes}");
    } else {
        simple_logger::init_with_env()?;
    }

    let op = an_operator_constructor()?;
    // dbg!(&op);
    let mut op = op;
    op.name = "Ost".to_string();
    // dbg!(&op);
    println!("{:#?}", op.ignored());

    let one = "one two three"
        .split_whitespace()
        .next()
        .unwrap_or("unknown");
    dbg!(one);
    let size = std::mem::size_of::<Op>();
    dbg!(size);
    Ok(())
}

// -----------------------------------------------------------------------------

/// TODO: Ned som test!
use OpParameter::*;
fn an_operator_constructor() -> Result<OpParsedParameters, OpError> {
    #[rustfmt::skip]
    let gamut = [
        Flag    {key: "flag" },
        Natural {key: "natural_default",  default: Some(42)},
        Natural {key: "natural_required", default: None},
        Integer {key: "integer_default",  default: Some(-42)},
        Integer {key: "integer_required", default: None},
        Real    {key: "real_default",     default: Some(-42.)},
        Real    {key: "real_required",    default: None},
        Text    {key: "text_default",     default: Some("GRS80")},
        Text    {key: "text_required",    default: None}
    ];

    let definition = "
        operatorname
        two_of_these
        two_of_these
        flag flag_ignored
        natural_default=44 natural_required=42 natural_ignored=22
        integer_required=42 integer_ignored=11
        real_required=42
        text_required=banana, hønsefedt
    ";

    // Test the recursive call functionality of `OpResource`
    let globals = split_step_into_parameters("globals inv ellps: GRS80");
    let macro_invocation = "translation cheese: ost salt: salt soup: suppe";
    let first = OpResource::new(definition, &globals);
    dbg!(&first);
    let next = OpResource::next(definition, &first);
    dbg!(&next);

    OpParsedParameters::new(definition, &gamut)
}

// -----------------------------------------------------------------------------

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
    #[error("operator {0} not found{1}")]
    NotFound(String, String),
    #[error("too deep recursion for {0}")]
    Recursion(String),
    #[error("missing required parameter {0}")]
    MissingParam(String),
    #[error("malformed value for parameter {0}: {1}")]
    BadParam(String, String),
    #[error("unknown error")]
    Unknown,
}

/// `Fwd`: Indicate that a two-way operator, function, or method,
/// should run in the *forward* direction.
/// `Inv`: Indicate that a two-way operator, function, or method,
/// should run in the *inverse* direction.
#[derive(Debug)]
pub enum OpDirection {
    Fwd,
    Inv,
}

// -----------------------------------------------------------------------------

/// Blueprint for the overall instantiation of an operator
pub struct OpConstructor(fn(args: &OpResource, ctx: &dyn OpProvider) -> Result<Op, OpError>);

/// Blueprint for the functions actually doing the transformation work
pub struct OpOperator(fn(op: &Op, ctx: &dyn OpProvider, operands: &mut [Coord]) -> usize);

// Cannot autoderive Debug trait here, for some reason
impl core::fmt::Debug for OpOperator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Operator")
    }
}

// Defaults to no_op
impl Default for OpOperator {
    fn default() -> OpOperator {
        OpOperator(noop_placeholder)
    }
}

fn noop_placeholder(_op: &Op, _provider: &dyn OpProvider, _operands: &mut [Coord]) -> usize {
    0
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct OpResource {
    invocation: String,
    definition: String,
    globals: BTreeMap<String, String>,
    recursion_level: usize,
    inv_state: bool
}

impl OpResource {
    pub fn new(invocation: &str, globals: &BTreeMap<String, String>) -> OpResource {
        let mut inv_state = false;
        let recursion_level = 0;
        let mut globals = globals.clone();
        let invocation = invocation.to_string();
        let definition = invocation.clone();

        // Direct invocation of a pipeline: "foo | bar baz | bonk"
        // invocation == definition
        if invocation.is_pipeline() {
            return OpResource {
                invocation,
                definition,
                globals,
                recursion_level,
                inv_state
            }
        }

        // Direct invocation of a primitive operation: "foo bar=baz bonk"
        // invocation == definition
        if !invocation.is_macro() {
            return OpResource {
                invocation,
                definition,
                globals,
                recursion_level,
                inv_state
            }
        }

        // TODO: clean up this comment!
        // Macro expansion initialization
        // definition == ""
        // invocation == macro name and macro arguments.
        // The tough parameter handling is carried out by
        // the `next()` method in the upcomming step.
        // If the previous step was the initialization of a macro expansion,
        // the "invocation" slot contains the macro name and macro arguments.
        // We continue by saving the name and inv parameters under disguise
        // as "_invoked_as_name" and "_invoked_as_inv", to avoid having them
        // wreak havoc during the expansion step(s). The remaining macro
        // arguments are stored as-is.
        let mut definition = "".to_string();
        let mut params = invocation.split_step_into_parameters();
        if params.contains_key("name") {
            let name = params["name"].clone();
            definition = name.clone();
            params.insert("_invoked_as_name".to_string(), name);
        }
        if params.contains_key("inv") {
            inv_state = !inv_state;
            if inv_state {               // TODO: Double check this
                definition += " inv";
            }
            params.insert("_invoked_as_inv".to_string(), "".to_string());
        }
        params.remove("name");
        params.remove("inv");
        globals.extend(invocation.split_step_into_parameters().into_iter());

        OpResource {
            invocation,
            definition,
            globals,
            recursion_level,
            inv_state
        }

    }

    // If the next step is a macro (i.e. potentially an embedded pipeline), we
    // get the arguments from the invocation and bring them into the globals.
    // Otherwise, we just copy the globals from the previous step, and
    // update the recursion counter.
    pub fn next(definition: &str, previous: &OpResource) -> OpResource {
        let invocation = previous.invocation.clone();
        let mut definition = definition.to_string();
        let mut globals = previous.globals.clone();
        let recursion_level = previous.recursion_level + 1;
        let mut inv_state = previous.inv_state;



        // globals.remove("name");
        // globals.remove("inv");
        OpResource {
            invocation,
            definition,
            globals,
            recursion_level,
            inv_state
        }
    }
}

// -----------------------------------------------------------------------------

/// The `OpParameter` enumeration is used to represent which defining parameters
/// are valid for a given `Op`eration.
///
/// The individual `Op`eration implementations use these to define the types of
/// the parameters accepted, and whether they are *required* (in which case the
/// provided default value is set to `None`), or *optional* (in which
/// case, a default value of the proper type is provided). The odd man out here
/// is the `Flag` type: Since a flag is a boolean which is true if present and
/// false if not, it does not make much sense to provide a default in this case.
///
/// Any other parameters given should be ignored, but warned about.
///
/// For a given operation, the union of the sets of its required and optional
/// parameters is called the *gamut* of the operation.
pub enum OpParameter {
    /// A flag is a boolean that is true if present, false if not
    Flag { key: &'static str },
    /// The natural numbers + zero (𝐍₀ or 𝐖 in math terms)
    Natural {
        key: &'static str,
        default: Option<usize>,
    },
    /// Integers (𝐙 in math terms)
    Integer {
        key: &'static str,
        default: Option<i64>,
    },
    /// Reals (𝐑 in math terms)
    Real {
        key: &'static str,
        default: Option<f64>,
    },
    /// A series of reals (𝐑ⁿ in math terms)
    Series {
        key: &'static str
    },
    /// Any kind of text
    Text {
        key: &'static str,
        default: Option<&'static str>,
    },
}

pub trait OpProvider {
    fn globals(&self) -> BTreeMap<String, String>;
    fn get_user_defined_constructor(&self, name: &str) -> Result<OpConstructor, OpError>;
    fn get_resource_definition(&self, name: &str) -> Result<String, OpError>;
}

pub struct OpMinimalProvider {}
impl OpProvider for OpMinimalProvider {
    fn globals(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("ellps".to_string(), "GRS80".to_string())])
    }

    fn get_user_defined_constructor(&self, name: &str) -> Result<OpConstructor, OpError> {
        Err(OpError::NotFound(
            name.to_string(),
            ": User defined operators not supported by 'Minimal' provider".to_string(),
        ))
    }

    fn get_resource_definition(&self, name: &str) -> Result<String, OpError> {
        Err(OpError::NotFound(
            name.to_string(),
            ": User defined operators not supported by 'Minimal' provider".to_string(),
        ))
    }

}



/// Slightly-more-than-minimal provider, for mocking/testing
pub struct OpMockProvider {}
impl OpProvider for OpMockProvider {
    fn globals(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("ellps".to_string(), "GRS80".to_string())])
    }

    fn get_user_defined_constructor(&self, name: &str) -> Result<OpConstructor, OpError> {
        if name == "mock" {
            return Ok(OpConstructor(mock_constructor));
        }
        Err(OpError::NotFound(
            name.to_string(),
            ": User defined operators not supported by 'Mock' provider".to_string(),
        ))
    }

    fn get_resource_definition(&self, name: &str) -> Result<String, OpError> {
        let (section, item) = name.split_once(':').ok_or(OpError::Syntax(name.to_string()))?;
        if section == "mock" && item == "mock" {
            return Ok("mock inv a=b c=d e=f".to_string());
        }
        Err(OpError::NotFound(
            name.to_string(),
            ": User defined resources not supported by 'Mock' provider".to_string(),
        ))
    }
}

// -----------------------------------------------------------------------------

// Sample operator implementation for testing purposes
fn mock_fwd(op: &Op, provider: &dyn OpProvider, operands: &mut [Coord]) -> usize {
    let mut n = 0;
    for o in operands {
        o[0] += 1.;
        n += 1;
    }
    n
}

fn mock_inv(op: &Op, provider: &dyn OpProvider, operands: &mut [Coord]) -> usize {
    let mut n = 0;
    for o in operands {
        o[0] -= 1.;
        n += 1;
    }
    n
}

// Fake user defined operator (for structural testing)
fn mock_constructor(definition: &OpResource, provider: &dyn OpProvider) -> Result<Op, OpError> {
    let gamut = [Flag { key: "inv" }];

    let def = &definition.definition;
    let fwd = OpOperator(mock_fwd);
    let inv = OpOperator(mock_inv);
    let params = OpParsedParameters::new(def, &gamut)?;
    let base = OpBase::new(&def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    Ok(Op {
        base,
        steps,
        params,
    })
}

// Demo built-in operator
fn builtin_mock_constructor(definition: &OpResource, provider: &dyn OpProvider) -> Result<Op, OpError> {
    let gamut = [Flag { key: "inv" }];

    let def = &definition.definition;
    // Yes - swapped
    let fwd = OpOperator(mock_inv);
    let inv = OpOperator(mock_fwd);
    let params = OpParsedParameters::new(def, &gamut)?;
    let base = OpBase::new(&def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    Ok(Op {
        base,
        steps,
        params,
    })
}

// A BTreeMap would have been a better choice here, except for the
// annoying fact that it cannot be compile-time const-constructed
#[rustfmt::skip]
const BUILTIN_OPERATORS: [(&str, OpConstructor); 1] = [
    ("builtin_mock", OpConstructor(builtin_mock_constructor))
];

/// Handle instantiation of built-in operators, as defined in
/// `BUILTIN_OPERATORS` above.
pub fn builtin(name: &str) -> Result<OpConstructor, OpError> {
    for p in BUILTIN_OPERATORS {
        if p.0 == name {
            return Ok(p.1);
        }
    }
    Err(OpError::NotFound(name.to_string(), String::default()))
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Op {
    base: OpBase,
    steps: Vec<Op>,
    params: OpParsedParameters,
}

impl Op {
    pub fn new(definition: &str, provider: &dyn OpProvider) -> Result<Op, OpError> {
        let globals = provider.globals();
        let definition = OpResource::new(definition, &globals);
        Self::op(definition, provider)
    }

    fn pipeline(definition: OpResource, provider: &dyn OpProvider) -> Result<Op, OpError> {
        todo!()
    }

    // (builtins og userdefineds)
    fn op(definition: OpResource, provider: &dyn OpProvider) -> Result<Op, OpError> {
        // If it looks like a pipeline, we call the pipeline constructor
        if definition.definition.is_pipeline() {
            return Self::pipeline(definition, provider);
        }

        let name = definition
            .definition
            .split_whitespace()
            .next()
            .unwrap_or_default();

        // A user defined operator?
        if let Ok(constructor) = provider.get_user_defined_constructor(name) {
            return constructor.0(&definition, provider);
        }

        todo!()
    }

    fn parameters(&self) -> &OpParsedParameters {
        &self.params
    }
}

// -----------------------------------------------------------------------------

/// The fundamental elements of an operator (i.e. everything but the args)
#[derive(Debug)]
pub struct OpBase {
    invocation: String, // e.g. geohelmert ellps_0=GRS80 x=1 y=2 z=3 ellps_1=intl
    definition: String, // e.g. cart ellps=^ellps_0 | helmert | cart inv ellps=^ellps_1
    invertible: bool,
    inverted: bool,
    fwd: OpOperator,
    inv: OpOperator,
    uuid: uuid::Uuid,
}

impl OpBase {
    pub fn new(definition: &str, fwd: OpOperator, inv: Option<OpOperator>) -> OpBase {
        let definition = definition.to_string();
        let invertible = inv.is_some();
        let inverted = false; // TODO
        let invocation = "".to_string(); // TODO
        let inv = inv.unwrap_or_default();
        let uuid = uuid::Uuid::new_v4();
        OpBase {
            invocation,
            definition,
            invertible,
            inverted,
            fwd,
            inv,
            uuid,
        }
    }
}

// -----------------------------------------------------------------------------

/// TODO:
/// Handle globals properly
/// impl Chase
/// .get() -> .chase()
#[derive(Debug)]
pub struct OpParsedParameters {
    name: String,
    // pub inverted: bool,

    // Commonly used options have hard-coded slots
    ellps: [Ellipsoid; 2],
    lat: [f64; 4],
    lon: [f64; 4],
    x: [f64; 4],
    y: [f64; 4],
    k: [f64; 4],

    // Op-specific options are stored in B-Trees
    boolean: BTreeSet<&'static str>,
    natural: BTreeMap<&'static str, usize>,
    integer: BTreeMap<&'static str, i64>,
    real: BTreeMap<&'static str, f64>,
    series: BTreeMap<&'static str, Vec<f64>>,
    text: BTreeMap<&'static str, String>,
    uuid: BTreeMap<&'static str, uuid::Uuid>,
    ignored: Vec<String>,
}

// Accessors
impl OpParsedParameters {
    pub fn boolean(&self, key: &str) -> bool {
        self.boolean.contains(key)
    }
    pub fn natural(&self, key: &str) -> Result<usize, OpError> {
        if let Some(value) = self.natural.get(key) {
            return Ok(*value);
        }
        Err(OpError::MissingParam(key.to_string()))
    }
    pub fn integer(&self, key: &str) -> Result<i64, OpError> {
        if let Some(value) = self.integer.get(key) {
            return Ok(*value);
        }
        Err(OpError::MissingParam(key.to_string()))
    }
    pub fn real(&self, key: &str) -> Result<f64, OpError> {
        if let Some(value) = self.real.get(key) {
            return Ok(*value);
        }
        Err(OpError::MissingParam(key.to_string()))
    }
    pub fn text(&self, key: &str) -> Result<String, OpError> {
        if let Some(value) = self.text.get(key) {
            return Ok(value.to_string());
        }
        Err(OpError::MissingParam(key.to_string()))
    }
    pub fn uuid(&self, key: &str) -> Result<uuid::Uuid, OpError> {
        if let Some(value) = self.uuid.get(key) {
            return Ok(*value);
        }
        Err(OpError::MissingParam(key.to_string()))
    }
    pub fn ignored(&self) -> Vec<String> {
        self.ignored.clone()
    }
    pub fn ellps(&self, index: usize) -> &Ellipsoid {
        &self.ellps[index]
    }
    pub fn x(&self, index: usize) -> f64 {
        self.x[index]
    }
    pub fn y(&self, index: usize) -> f64 {
        self.y[index]
    }
    pub fn lat(&self, index: usize) -> f64 {
        self.lat[index]
    }
    pub fn lon(&self, index: usize) -> f64 {
        self.lon[index]
    }
    pub fn k(&self, index: usize) -> f64 {
        self.k[index]
    }
}

impl OpParsedParameters {
    pub fn new(
        definition: &str,
        parameter_gamut: &[OpParameter],
    ) -> Result<OpParsedParameters, OpError> {
        let parameters = split_step_into_parameters(definition);

        let mut boolean = BTreeSet::<&'static str>::new();
        let mut natural = BTreeMap::<&'static str, usize>::new();
        let mut integer = BTreeMap::<&'static str, i64>::new();
        let mut real = BTreeMap::<&'static str, f64>::new();
        let mut series = BTreeMap::<&'static str, Vec<f64>>::new();
        let mut text = BTreeMap::<&'static str, String>::new();
        let mut uuid = BTreeMap::<&'static str, uuid::Uuid>::new();

        // Try to locate all accepted parameters, type check, and place them into
        // their proper bins
        for p in parameter_gamut {
            match *p {
                OpParameter::Flag { key } => {
                    dbg!(key);
                    if let Some(value) = parameters.get(key) {
                        // should chase!
                        dbg!(value);
                        if value.is_empty() || value.to_lowercase() == "true" {
                            boolean.insert(key);
                            continue;
                        }
                        warn!("Cannot parse {key}:{value} as a boolean constant!");
                        return Err(OpError::BadParam(key.to_string(), value.to_string()));
                    }
                    // If we're here, the key was not found, and we're done, since
                    // flags are always optional (i.e. implicitly false when not given)
                    continue;
                }

                OpParameter::Natural { key, default } => {
                    if let Some(value) = parameters.get(key) {
                        // should chase!
                        if let Ok(v) = value.parse::<usize>() {
                            natural.insert(key, v);
                            continue;
                        }
                        warn!("Cannot parse {key}:{value} as a natural number!");
                        return Err(OpError::BadParam(key.to_string(), value.to_string()));
                    }

                    // Key not found - default given?
                    if let Some(value) = default {
                        natural.insert(key, value);
                        continue;
                    }

                    error!("Missing required parameter '{key}'");
                    return Err(OpError::MissingParam(key.to_string()));
                }

                OpParameter::Integer { key, default } => {
                    if let Some(value) = parameters.get(key) {
                        // should chase!
                        if let Ok(v) = value.parse::<i64>() {
                            integer.insert(key, v);
                            continue;
                        }
                        warn!("Cannot parse {key}:{value} as an integer!");
                        return Err(OpError::BadParam(key.to_string(), value.to_string()));
                    }

                    // If we're here, the key was not found

                    // Default given?
                    if let Some(value) = default {
                        integer.insert(key, value);
                        continue;
                    }

                    // Missing a required parameter
                    error!("Missing required parameter '{key}'");
                    return Err(OpError::MissingParam(key.to_string()));
                }

                OpParameter::Real { key, default } => {
                    if let Some(value) = parameters.get(key) {
                        // should chase!
                        if let Ok(v) = value.parse::<f64>() {
                            real.insert(key, v);
                            continue;
                        }
                        warn!("Cannot parse {key}:{value} as a real number - ignoring");
                        return Err(OpError::BadParam(key.to_string(), value.to_string()));
                    }

                    // If we're here, the key was not found

                    // Default given?
                    if let Some(value) = default {
                        real.insert(key, value);
                        continue;
                    }

                    // Missing a required parameter
                    error!("Missing required parameter '{key}'");
                    return Err(OpError::MissingParam(key.to_string()));
                }

                // TODO! (only reads first element of the series, and puts it into the Real store)
                OpParameter::Series { key } => {
                    if let Some(value) = parameters.get(key) {
                        // should chase!
                        if let Ok(v) = value.parse::<f64>() {
                            real.insert(key, v);
                            continue;
                        }
                        warn!("Cannot parse {key}:{value} as a real number - ignoring");
                        return Err(OpError::BadParam(key.to_string(), value.to_string()));
                    }

                    // If we're here, the key was not found

                    // Missing a required parameter
                    error!("Missing required parameter '{key}'");
                    return Err(OpError::MissingParam(key.to_string()));
                }
                OpParameter::Text { key, default } => {
                    if let Some(value) = parameters.get(key) {
                        // should chase!
                        text.insert(key, value.to_string());
                        continue;
                    }

                    // If we're here, the key was not found

                    // Default given?
                    if let Some(value) = default {
                        text.insert(key, value.to_string());
                        continue;
                    }

                    // Missing a required parameter
                    error!("Missing required parameter '{key}'");
                    return Err(OpError::MissingParam(key.to_string()));
                }
            };
        }

        let ellps = [Ellipsoid::default(), Ellipsoid::default()];
        let lat = [0.; 4];
        let lon = [0.; 4];
        let x = [0.; 4];
        let y = [0.; 4];
        let k = [0.; 4];

        let name = parameters
            .get("name")
            .unwrap_or(&"unknown".to_string())
            .to_string();

        // Params explicitly set to the default value
        // let mut redundant = BTreeSet::<String>::new();
        // Params specified, but not used
        let ignored: Vec<String> = parameters.clone().into_keys().collect();
        Ok(OpParsedParameters {
            ellps,
            lat,
            lon,
            x,
            y,
            k,
            name,
            boolean,
            natural,
            integer,
            real,
            series,
            text,
            uuid,
            ignored,
        })
    }
}

// -----------------------------------------------------------------------------

pub trait OpInvocationHelpers {
    // An expression is a pipeline, if it contains a '|'
    fn is_pipeline(&self) -> bool;
    // An expression is a macro if its name contains a ':'
    fn is_macro(&self) -> bool;
    fn split_step_into_parameters(&self) -> BTreeMap<String, String>;
    fn split_definition_into_steps(&self) -> (Vec<String>, String);
}

impl OpInvocationHelpers for String {
    fn is_pipeline(&self) -> bool {
        self.contains('|')
    }
    fn is_macro(&self) -> bool {
        if self.is_pipeline() {
            return false
        }
        let params = self.split_step_into_parameters();
        params.get("name").unwrap_or(&String::default()).to_string().contains(':')
    }

    fn split_definition_into_steps(&self) -> (Vec<String>, String) {
        split_definition_into_steps(&self)
    }

    fn split_step_into_parameters(&self) -> BTreeMap<String, String> {
        split_step_into_parameters(&self)
    }

}


fn split_step_into_parameters(step: &str) -> BTreeMap<String, String> {
    // Conflate contiguous whitespace, then remove whitespace after {= and ,}
    let step = step.trim().to_string();
    let elements: Vec<_> = step.split_whitespace().collect();
    let step = elements.join(" ").replace("= ", "=").replace(", ", ",");

    let mut params = BTreeMap::new();
    let elements: Vec<_> = step.split_whitespace().collect();
    for element in elements {
        // Split a key=value-pair into key and value parts
        let mut parts: Vec<&str> = element.trim().split('=').collect();
        // Add an empty part, to make sure we have a value, even for flags
        parts.push("");
        assert!(parts.len() > 1);

        // If the first arg is a key-without-value, it is the name of the operator
        if params.is_empty() && parts.len() == 2 {
            params.insert(String::from("name"), String::from(parts[0]));
            continue;
        }

        // Flag normalization 1: Leave out flags explicitly set to false
        if parts[1].to_lowercase() == "false" {
            continue;
        }

        // Flag normalization 2: Remove explicit "true" values from flags
        if parts[1].to_lowercase() == "true" {
            parts[1] = "";
        }

        params.insert(String::from(parts[0]), String::from(parts[1]));
    }

    dbg!(&params);
    params
}

// -----------------------------------------------------------------------------

fn split_definition_into_steps(definition: &str) -> (Vec<String>, String) {
    let all = definition.replace("\r", "\n").trim().to_string();

    // Collect docstrings and remove plain comments
    let mut trimmed = Vec::<String>::new();
    let mut docstring = Vec::<String>::new();
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

    // Generate trimmed steps with elements spearated by a single space,
    // and key-value pairs glued by '=' as in 'key=value'
    let steps: Vec<_> = trimmed.split('|').collect();
    let mut trimmed_steps = Vec::<String>::new();
    for mut step in steps {
        step = step.trim();
        let elements: Vec<_> = step.split_whitespace().collect();
        let joined = elements.join(" ").replace("= ", "=");
        trimmed_steps.push(joined);
    }
    let trimmed_steps = trimmed_steps;
    (trimmed_steps, docstring)
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operator_args() -> Result<(), OpError> {
        Ok(())
    }
}
