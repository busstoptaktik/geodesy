// `ParsedParameters is the `InnerOp` specific representation of the operator
// arguments. The `InnerOp`-constructor asks ParsedParameters::new(...) to
// interpret the `RawParameters`-representation according to the `GAMUT` of
// the `InnerOp`(i.e. the args it is willing to interpret and use).
// Also, the constructor is free to compute up front derived parameters and
// store them in the `ParsedParameters` struct, ready for use at run time.

#![allow(clippy::needless_range_loop)]
use crate::math::FourierCoefficients;
use std::collections::BTreeSet;

use super::*;

#[derive(Debug, Clone)]
pub struct ParsedParameters {
    pub name: String,

    // Commonly used options have hard-coded slots
    pub ellps: [Ellipsoid; 2],
    pub lat: [f64; 4],
    pub lon: [f64; 4],
    pub x: [f64; 4],
    pub y: [f64; 4],
    pub k: [f64; 4],

    // Op-specific options are stored in B-Trees
    pub boolean: BTreeSet<&'static str>,
    pub natural: BTreeMap<&'static str, usize>,
    pub integer: BTreeMap<&'static str, i64>,
    pub real: BTreeMap<&'static str, f64>,
    pub series: BTreeMap<&'static str, Vec<f64>>,
    pub grids: BTreeMap<&'static str, Grid>,
    pub text: BTreeMap<&'static str, String>,
    pub texts: BTreeMap<&'static str, Vec<String>>,
    pub uuid: BTreeMap<&'static str, uuid::Uuid>,
    pub fourier_coefficients: BTreeMap<&'static str, FourierCoefficients>,
    pub ignored: Vec<String>,
    pub given: BTreeMap<String, String>,
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

impl ParsedParameters {
    pub fn new(
        parameters: &RawParameters,
        gamut: &[OpParameter],
    ) -> Result<ParsedParameters, Error> {
        let locals = super::split_into_parameters(&parameters.definition);
        let globals = &parameters.globals;
        let mut boolean = BTreeSet::<&'static str>::new();
        let mut natural = BTreeMap::<&'static str, usize>::new();
        let mut integer = BTreeMap::<&'static str, i64>::new();
        let mut real = BTreeMap::<&'static str, f64>::new();
        let mut series = BTreeMap::<&'static str, Vec<f64>>::new();
        let mut text = BTreeMap::<&'static str, String>::new();
        let mut texts = BTreeMap::<&'static str, Vec<String>>::new();
        let grids = BTreeMap::<&'static str, Grid>::new();
        #[allow(unused_mut)]
        let mut uuid = BTreeMap::<&'static str, uuid::Uuid>::new();
        let fourier_coefficients = BTreeMap::<&'static str, FourierCoefficients>::new();

        // 'omit_fwd'/'omit_inv' are implicitly valid for all operators (including pipelines),
        // so we append them to the gamut
        let mut gamutt = Vec::new();
        for g in gamut {
            gamutt.push(g);
        }
        gamutt.push(&OpParameter::Flag { key: "omit_fwd" });
        gamutt.push(&OpParameter::Flag { key: "omit_inv" });

        // Try to locate all accepted parameters, type check, and place them into
        // their proper bins
        for p in gamutt {
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
                        let v = crate::math::parse_sexagesimal(&value);
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
                            let v = crate::math::parse_sexagesimal(element);
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
                            let v = crate::math::parse_sexagesimal(element);
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

        // Now handle the commonly used options with the hard-coded slots

        let mut ellps = [Ellipsoid::default(), Ellipsoid::default()];
        let mut lat = [0.; 4];
        let mut lon = [0.; 4];
        let mut x = [0.; 4];
        let mut y = [0.; 4];
        let mut k = [1.; 4];

        // ellps_{n}
        for i in 0..2 {
            let key = format!("ellps_{i}");
            if let Some(e) = text.get(&key[..]) {
                ellps[i] = Ellipsoid::named(e)?;
            }
        }
        // But `ellps` trumps `ellps_0`
        if let Some(e) = text.get("ellps") {
            ellps[0] = Ellipsoid::named(e)?;
        }

        // lat_{n}
        for i in 0..4 {
            let key = format!("lat_{i}");
            lat[i] = (*real.get(&key[..]).unwrap_or(&0.)).to_radians();
        }

        // lon_{n}
        for i in 0..4 {
            let key = format!("lon_{i}");
            lon[i] = (*real.get(&key[..]).unwrap_or(&0.)).to_radians();
        }

        // x_{n}
        for i in 0..4 {
            let key = format!("x_{i}");
            x[i] = *real.get(&key[..]).unwrap_or(&0.);
        }

        // y_{n}
        for i in 0..4 {
            let key = format!("y_{i}");
            y[i] = *real.get(&key[..]).unwrap_or(&0.);
        }

        // k_{n}
        for i in 0..4 {
            let key = format!("k_{i}");
            k[i] = *real.get(&key[..]).unwrap_or(&0.);
        }

        let name = locals
            .get("name")
            .unwrap_or(&"unknown".to_string())
            .to_string();

        // TODO:
        // Params explicitly set to the default value
        // let mut redundant = BTreeSet::<String>::new();
        // Params specified, but not used
        let given = locals.clone();
        let ignored: Vec<String> = locals.into_keys().collect();
        Ok(ParsedParameters {
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
                return Err(Error::Syntax(format!("Incomplete definition for '{key}'")));
            }
            return Ok(None);
        }
        let thevalue = found.unwrap().1.trim();

        // If the value is a(nother) lookup, we continue the search in the same iterator,
        // now using a *new search key*, as specified by the current value
        if let Some(stripped) = thevalue.strip_prefix('$') {
            chasing = true;
            needle = stripped;
            continue;
        }

        // If the value is a provided default, we continue the search using the *same key*,
        // in case a proper value is provided.
        // cf. the test `macro_expansion_with_defaults_provided()` in `./mod.rs`
        if let Some(stripped) = thevalue.strip_prefix('*') {
            chasing = true;
            needle = key;
            default = stripped;
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
        let invocation = String::from(
            "cucumber flag ellps_0=123 , 456 natural=$indirection sexagesimal=1:30:36 names=alice, bob",
        );
        let mut globals = BTreeMap::<String, String>::new();
        globals.insert("indirection".to_string(), "123".to_string());
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
            p.ellps[0].semimajor_axis(),
            Ellipsoid::new(123., 1. / 456.).semimajor_axis()
        );

        let invocation = String::from("cucumber bad_series=no, numbers, here");
        let raw = RawParameters::new(&invocation, &globals);
        assert!(matches!(
            ParsedParameters::new(&raw, &GAMUT),
            Err(Error::BadParam(_, _))
        ));

        Ok(())
    }
}
