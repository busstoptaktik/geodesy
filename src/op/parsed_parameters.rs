//! The `InnerOp` specific representation of the operator arguments

use crate::math::angular;
use crate::math::FourierCoefficients;
use std::collections::BTreeSet;
use std::sync::Arc;

use super::*;

#[rustfmt::skip]
const ZERO_VALUED_IMPLICIT_GAMUT_ELEMENTS: [&str; 16] = [
    "x_0", "x_1", "x_2", "x_3",
    "y_0", "y_1", "y_2", "y_3",
    "lat_0", "lat_1", "lat_2", "lat_3",
    "lon_0", "lon_1", "lon_2", "lon_3"
];

#[rustfmt::skip]
const UNIT_VALUED_IMPLICIT_GAMUT_ELEMENTS: [&str; 4] = [
    "k_0", "k_1", "k_2", "k_3"
];

/// The [InnerOp](crate::inner_op::InnerOp) specific
/// representation of the operator arguments.
///
/// The [InnerOp](crate::inner_op::InnerOp)-constructor asks
/// `ParsedParameters::new(...)` to interpret the
/// [RawParameters](super::RawParameters)-representation
/// according to the `GAMUT` of the `InnerOp` (i.e. the args it is willing
/// to interpret and use).
///
/// Also, the `InnerOp` constructor is free to pre-compute
/// derived parameters and store them in the `ParsedParameters`
/// struct, ready for use at run time.
#[derive(Debug, Clone)]
pub struct ParsedParameters {
    pub name: String,

    // Op-specific options are stored in B-Trees
    pub boolean: BTreeSet<&'static str>,
    pub natural: BTreeMap<&'static str, usize>,
    pub integer: BTreeMap<&'static str, i64>,
    pub real: BTreeMap<&'static str, f64>,
    pub series: BTreeMap<&'static str, Vec<f64>>,
    pub text: BTreeMap<&'static str, String>,
    pub texts: BTreeMap<&'static str, Vec<String>>,
    pub uuid: BTreeMap<&'static str, uuid::Uuid>,
    pub fourier_coefficients: BTreeMap<&'static str, FourierCoefficients>,
    pub ignored: Vec<String>,
    pub given: BTreeMap<String, String>,

    // Pointers to the grids required by the operator
    // They should be inserted in the order they appear in the definition
    pub grids: Vec<Arc<dyn Grid>>,
}

// Accessors
impl ParsedParameters {
    pub fn boolean(&self, key: &str) -> bool {
        self.boolean.contains(key)
    }
    pub fn natural(&self, key: &str) -> Result<usize, Error> {
        if let Some(value) = self.natural.get(key) {
            return Ok(*value);
        }
        Err(Error::MissingParam(key.to_string()))
    }
    pub fn integer(&self, key: &str) -> Result<i64, Error> {
        if let Some(value) = self.integer.get(key) {
            return Ok(*value);
        }
        Err(Error::MissingParam(key.to_string()))
    }
    pub fn real(&self, key: &str) -> Result<f64, Error> {
        if let Some(value) = self.real.get(key) {
            return Ok(*value);
        }
        Err(Error::MissingParam(key.to_string()))
    }
    pub fn series(&self, key: &str) -> Result<&[f64], Error> {
        if let Some(value) = self.series.get(key) {
            return Ok(value);
        }
        Err(Error::MissingParam(key.to_string()))
    }
    pub fn text(&self, key: &str) -> Result<String, Error> {
        if let Some(value) = self.text.get(key) {
            return Ok(value.to_string());
        }
        Err(Error::MissingParam(key.to_string()))
    }
    pub fn texts(&self, key: &str) -> Result<&Vec<String>, Error> {
        if let Some(value) = self.texts.get(key) {
            return Ok(value);
        }
        Err(Error::MissingParam(key.to_string()))
    }
    pub fn uuid(&self, key: &str) -> Result<uuid::Uuid, Error> {
        if let Some(value) = self.uuid.get(key) {
            return Ok(*value);
        }
        Err(Error::MissingParam(key.to_string()))
    }
    pub fn fourier_coefficients(&self, key: &str) -> Result<FourierCoefficients, Error> {
        if let Some(value) = self.fourier_coefficients.get(key) {
            return Ok(*value);
        }
        Err(Error::MissingParam(key.to_string()))
    }
    pub fn ignored(&self) -> Vec<String> {
        self.ignored.clone()
    }
    pub fn ellps(&self, index: usize) -> Ellipsoid {
        // if 'ellps' was explicitly given, it will override 'ellps_0'
        if index == 0 {
            if let Some(e) = self.text.get("ellps") {
                return Ellipsoid::named(e).unwrap();
            }
        }
        let key = format!("ellps_{index}");
        if let Some(e) = self.text.get(&key[..]) {
            return Ellipsoid::named(e).unwrap();
        }
        // If none of them existed, i.e. no defaults were given, we return the general default
        Ellipsoid::default()
    }
    pub fn k(&self, index: usize) -> f64 {
        *(self.real.get(&format!("k_{index}")[..]).unwrap_or(&1.))
    }
    pub fn x(&self, index: usize) -> f64 {
        *(self.real.get(&format!("x_{index}")[..]).unwrap_or(&0.))
    }
    pub fn y(&self, index: usize) -> f64 {
        *(self.real.get(&format!("y_{index}")[..]).unwrap_or(&0.))
    }
    pub fn lat(&self, index: usize) -> f64 {
        *self.real.get(&format!("lat_{index}")[..]).unwrap_or(&0.)
    }
    pub fn lon(&self, index: usize) -> f64 {
        *self.real.get(&format!("lon_{index}")[..]).unwrap_or(&0.)
    }
}

impl ParsedParameters {
    pub fn new(
        parameters: &RawParameters,
        gamut: &[OpParameter],
    ) -> Result<ParsedParameters, Error> {
        let locals = parameters.definition.split_into_parameters();
        let globals = &parameters.globals;
        let mut boolean = BTreeSet::<&'static str>::new();
        let mut natural = BTreeMap::<&'static str, usize>::new();
        let mut integer = BTreeMap::<&'static str, i64>::new();
        let mut real = BTreeMap::<&'static str, f64>::new();
        let mut series = BTreeMap::<&'static str, Vec<f64>>::new();
        let mut text = BTreeMap::<&'static str, String>::new();
        let mut texts = BTreeMap::<&'static str, Vec<String>>::new();
        let grids = Vec::new();
        #[allow(unused_mut)]
        let mut uuid = BTreeMap::<&'static str, uuid::Uuid>::new();
        let fourier_coefficients = BTreeMap::<&'static str, FourierCoefficients>::new();

        // Try to locate all accepted parameters, type check, and place them into
        // their proper bins
        for p in gamut {
            match *p {
                OpParameter::Flag { key } => {
                    if let Some(value) = chase(globals, &locals, key)? {
                        if value.is_empty() || value.to_lowercase() == "true" {
                            boolean.insert(key);
                            continue;
                        }
                        warn!("Cannot parse {key}:{value} as a boolean constant!");
                        return Err(Error::BadParam(key.to_string(), value));
                    }
                    // If we're here, the key was not found, and we're done, since
                    // flags are always optional (i.e. implicitly false when not given)
                    continue;
                }

                OpParameter::Natural { key, default } => {
                    if let Some(value) = chase(globals, &locals, key)? {
                        if let Ok(v) = value.parse::<usize>() {
                            natural.insert(key, v);
                            continue;
                        }
                        warn!("Cannot parse {key}:{value} as a natural number!");
                        return Err(Error::BadParam(key.to_string(), value));
                    }

                    // Key not found - default given?
                    if let Some(value) = default {
                        natural.insert(key, value);
                        continue;
                    }

                    error!("Missing required parameter '{key}'");
                    return Err(Error::MissingParam(key.to_string()));
                }

                OpParameter::Integer { key, default } => {
                    if let Some(value) = chase(globals, &locals, key)? {
                        if let Ok(v) = value.parse::<i64>() {
                            integer.insert(key, v);
                            continue;
                        }
                        warn!("Cannot parse {key}:{value} as an integer!");
                        return Err(Error::BadParam(key.to_string(), value));
                    }

                    // If we're here, the key was not found

                    // Default given?
                    if let Some(value) = default {
                        integer.insert(key, value);
                        continue;
                    }

                    // Missing a required parameter
                    error!("Missing required parameter '{key}'");
                    return Err(Error::MissingParam(key.to_string()));
                }

                OpParameter::Real { key, default } => {
                    if let Some(value) = chase(globals, &locals, key)? {
                        let v = angular::parse_sexagesimal(&value);
                        if v.is_nan() {
                            return Err(Error::BadParam(key.to_string(), value));
                        }
                        real.insert(key, v);
                        continue;
                    }

                    // If we're here, the key was not found

                    // Default given?
                    if let Some(value) = default {
                        real.insert(key, value);
                        continue;
                    }

                    // Missing a required parameter
                    error!("Missing required parameter '{key}'");
                    return Err(Error::MissingParam(key.to_string()));
                }

                OpParameter::Series { key, default } => {
                    let mut elements = Vec::<f64>::new();
                    if let Some(value) = chase(globals, &locals, key)? {
                        for element in value.split(',') {
                            let v = angular::parse_sexagesimal(element);
                            if v.is_nan() {
                                warn!("Cannot parse {key}:{value} as a series");
                                return Err(Error::BadParam(key.to_string(), value.to_string()));
                            }
                            elements.push(v);
                            continue;
                        }
                        series.insert(key, elements);
                        continue;
                    }

                    // If we're here, the key was not found

                    // Default given?
                    if let Some(value) = default {
                        // Defaults to nothing, so we just continue with the next parameter
                        if value.is_empty() {
                            continue;
                        }
                        for element in value.split(',') {
                            let v = angular::parse_sexagesimal(element);
                            if v.is_nan() {
                                warn!("Cannot parse {key}:{value} as a series");
                                return Err(Error::BadParam(key.to_string(), value.to_string()));
                            }
                            elements.push(v);
                            continue;
                        }
                        series.insert(key, elements);
                        continue;
                    }

                    // Missing a required parameter
                    error!("Missing required parameter '{key}'");
                    return Err(Error::MissingParam(key.to_string()));
                }

                OpParameter::Text { key, default } => {
                    if let Some(value) = chase(globals, &locals, key)? {
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
                    return Err(Error::MissingParam(key.to_string()));
                }

                OpParameter::Texts { key, default } => {
                    if let Some(value) = chase(globals, &locals, key)? {
                        let elements: Vec<String> =
                            value.split(',').map(|x| x.trim().to_string()).collect();
                        texts.insert(key, elements);
                        continue;
                    }

                    // If we're here, the key was not found

                    // Default given?
                    if let Some(value) = default {
                        // Defaults to nothing, so we just continue with the next parameter
                        if value.is_empty() {
                            continue;
                        }
                        let elements: Vec<String> =
                            value.split(',').map(|x| x.trim().to_string()).collect();
                        texts.insert(key, elements);
                        continue;
                    }

                    // Missing a required parameter
                    error!("Missing required parameter '{key}'");
                    return Err(Error::MissingParam(key.to_string()));
                }
            };
        }

        // Default gamut elements - traditionally supported for all operators

        // omit_fwd and omit_inv are implicitly valid for all ops
        if let Some(value) = chase(globals, &locals, "omit_fwd")? {
            if value.is_empty() || value.to_lowercase() == "true" {
                boolean.insert("omit_fwd");
            }
        }
        if let Some(value) = chase(globals, &locals, "omit_inv")? {
            if value.is_empty() || value.to_lowercase() == "true" {
                boolean.insert("omit_inv");
            }
        }

        for k in ZERO_VALUED_IMPLICIT_GAMUT_ELEMENTS {
            if !real.contains_key(k) {
                real.insert(k, 0.);
            }
        }

        for k in UNIT_VALUED_IMPLICIT_GAMUT_ELEMENTS {
            if !real.contains_key(k) {
                real.insert(k, 1.);
            }
        }

        let name = locals
            .get("_name")
            .unwrap_or(&"unknown".to_string())
            .to_string();

        // TODO:
        // Params explicitly set to the default value
        // let mut redundant = BTreeSet::<String>::new();
        // Params specified, but not used
        let given = locals.clone();
        let ignored: Vec<String> = locals.into_keys().collect();
        Ok(ParsedParameters {
            name,
            boolean,
            natural,
            integer,
            real,
            series,
            grids,
            text,
            texts,
            uuid,
            fourier_coefficients,
            ignored,
            given,
        })
    }
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

pub fn chase(
    globals: &BTreeMap<String, String>,
    locals: &BTreeMap<String, String>,
    key: &str,
) -> Result<Option<String>, Error> {
    // The haystack is a reverse iterator over both lists in series
    let mut haystack = globals.iter().chain(locals.iter()).rev();

    // Find the needle in the haystack, recursively chasing look-ups ('$')
    // and handling defaults ('*')
    let key = key.trim();
    if key.is_empty() {
        return Err(Error::Syntax(String::from("Empty key")));
    }

    let mut default = "";
    let mut needle = key;
    let mut chasing = false;
    let value;

    loop {
        let found = haystack.find(|&x| x.0 == needle);
        if found.is_none() {
            if !default.is_empty() {
                return Ok(Some(String::from(default)));
            }
            if chasing {
                return Err(Error::Syntax(format!(
                    "Incomplete definition for '{key}' ('{needle}' not found)"
                )));
            }
            return Ok(None);
        }
        let thevalue = found.unwrap().1.trim();

        // If the value is a(nother) lookup, we continue the search in the same iterator,
        // now using a *new search key*, as specified by the current value
        if let Some(stripped) = thevalue.strip_prefix('$') {
            let mut parts: Vec<_> = stripped
                .trim()
                .split(&['(', ')'][..])
                .filter(|x| !x.trim().is_empty())
                .collect();
            if ![1, 2].contains(&parts.len()) {
                return Err(Error::Syntax(format!(
                    "Bad format for optional default in  '{thevalue}'"
                )));
            }

            // Do we have a default value?, i.e. $arg_name(defualt_value)
            if parts.len() == 2 && !chasing {
                default = parts.pop().unwrap();
            }
            chasing = true;
            needle = parts.pop().unwrap();
            continue;
        }

        // If the value is a provided default, we continue the search using the *same key*,
        // in case a proper value is provided.
        // cf. the test `macro_expansion_with_defaults_provided_in_parenthesis()` in `./mod.rs`
        if let Some(stripped) = thevalue.strip_prefix('(') {
            chasing = true;
            needle = key;
            default = stripped.trim_end_matches(')');
            continue;
        }

        // Otherwise we have the proper result
        value = String::from(thevalue.trim());
        break;
    }
    Ok(Some(value))
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    const GAMUT: [OpParameter; 11] = [
        OpParameter::Flag    { key: "flag" },
        OpParameter::Natural { key: "natural",     default: Some(0) },
        OpParameter::Integer { key: "integer",     default: Some(-1)},
        OpParameter::Real    { key: "real",        default: Some(1.25) },
        OpParameter::Real    { key: "sexagesimal", default: Some(1.25) },
        OpParameter::Series  { key: "series",      default: Some("1,2,3,4") },
        OpParameter::Series  { key: "bad_series",  default: Some("1:2:3,2:3:4") },
        OpParameter::Text    { key: "text",        default: Some("text") },
        OpParameter::Texts   { key: "names",       default: Some("foo, bar") },
        OpParameter::Texts   { key: "foo",         default: Some("   bar   ") },
        OpParameter::Text    { key: "ellps_0",     default: Some("6400000, 300") },
    ];

    #[test]
    fn basic() -> Result<(), Error> {
        let mut globals = BTreeMap::<String, String>::new();
        globals.insert("indirection".to_string(), "123".to_string());

        let invocation = String::from(
            "cucumber flag ellps_0=123 , 456 natural=$indirection sexagesimal=1:30:36 names=alice, bob",
        );
        let raw = RawParameters::new(&invocation, &globals);
        let p = ParsedParameters::new(&raw, &GAMUT)?;

        // Booleans correctly parsed?
        assert!(
            p.boolean.get("flag").is_some(),
            "`flag` not in registered booleans: {:#?}",
            p.boolean
        );
        assert!(
            p.boolean.get("galf").is_none(),
            "`galf` not in registered booleans: {:?}",
            p.boolean
        );

        // Series correctly parsed?
        let series = p.series.get("series").unwrap();
        assert_eq!(series.len(), 4);
        assert_eq!(series[0], 1.);
        assert_eq!(series[3], 4.);

        // Texts correctly parsed?
        let texts = p.texts.get("names").unwrap();
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0], "alice");
        assert_eq!(texts[1], "bob");
        let texts = p.texts.get("foo").unwrap();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0], "bar");

        // Etc.
        assert_eq!(*p.real.get("sexagesimal").unwrap(), 1.51);
        assert_eq!(*p.natural.get("natural").unwrap(), 123_usize);
        assert_eq!(*p.integer.get("integer").unwrap(), -1);
        assert_eq!(*p.text.get("text").unwrap(), "text");

        assert_eq!(
            p.ellps(0).semimajor_axis(),
            Ellipsoid::new(123., 1. / 456.).semimajor_axis()
        );

        // Mismatching series format
        let invocation = String::from("cucumber bad_series=no, numbers, here");
        let raw = RawParameters::new(&invocation, &globals);
        assert!(matches!(
            ParsedParameters::new(&raw, &GAMUT),
            Err(Error::BadParam(_, _))
        ));

        // Invalid indirection (i.e. missing macro argument)
        let invocation = String::from("cucumber integer=$not_given");
        let raw = RawParameters::new(&invocation, &globals);
        assert!(matches!(
            ParsedParameters::new(&raw, &GAMUT),
            Err(Error::Syntax(_))
        ));

        // Valid indirection, because we combine the arg with a default
        let invocation = String::from("cucumber integer=$not_given_but_defaults_to_42(42)");
        let raw = RawParameters::new(&invocation, &globals);
        assert_eq!(
            *ParsedParameters::new(&raw, &GAMUT)
                .unwrap()
                .integer
                .get("integer")
                .unwrap(),
            42
        );

        // Valid indirection, because we actually gave the arg at the call point
        globals.insert("given_and_is_set_to_43".to_string(), "43".to_string());
        let invocation = String::from("cucumber integer=$given_and_is_set_to_43(42)");
        let raw = RawParameters::new(&invocation, &globals);
        assert_eq!(
            *ParsedParameters::new(&raw, &GAMUT)
                .unwrap()
                .integer
                .get("integer")
                .unwrap(),
            43
        );

        Ok(())
    }
}
