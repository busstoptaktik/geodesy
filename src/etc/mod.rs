//! Generally useful free functions, not tied to any specific type or trait,
//! but mostly related to the interpretation of operator parameters.
use super::internal::*;

pub fn is_pipeline(definition: &str) -> bool {
    definition.contains('|')
}

pub fn is_resource_name(definition: &str) -> bool {
    operator_name(definition, "").contains(':')
}

pub fn operator_name(definition: &str, default: &str) -> String {
    if is_pipeline(definition) {
        return default.to_string();
    }
    split_into_parameters(definition)
        .get("name")
        .unwrap_or(&default.to_string())
        .to_string()
}

pub fn split_into_parameters(step: &str) -> BTreeMap<String, String> {
    // Conflate contiguous whitespace, then remove whitespace after {"=",  ":",  ","}
    let step = step.trim().to_string();
    let elements: Vec<_> = step.split_whitespace().collect();
    let step = elements
        .join(" ")
        .replace("= ", "=")
        .replace(": ", ":")
        .replace(", ", ",")
        .replace(" =", "=")
        .replace(" :", ":")
        .replace(" ,", ",");

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

        params.insert(String::from(parts[0]), String::from(parts[1]));
    }

    params
}

pub fn split_into_steps(definition: &str) -> (Vec<String>, String) {
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

pub fn chase(
    globals: &BTreeMap<String, String>,
    locals: &BTreeMap<String, String>,
    key: &str,
) -> Result<Option<String>, Error> {
    // The haystack is a reverse iterator over both lists in series
    let mut haystack = globals.iter().chain(locals.iter()).rev();

    // Find the needle in the haystack, recursively chasing look-ups ('^')
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
                    "Incomplete definition for '{}'",
                    key
                )));
            }
            return Ok(None);
        }
        let thevalue = found.unwrap().1.trim();

        // If the value is a(nother) lookup, we continue the search in the same iterator,
        // now using a *new search key*, as specified by the current value
        if let Some(stripped) = thevalue.strip_prefix('^') {
            chasing = true;
            needle = stripped;
            continue;
        }

        // If the value is a default, we continue the search using the *same key*
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

// Rust Geodesy internals - i.e. functions that are needed in more
// than one module and hence belongs naturally in neither of them.

// pj_tsfn is the equivalent of Charles Karney's PROJ function of the
// same name, which determines the function ts(phi) as defined in
// Snyder (1987), Eq. (7-10)
//
// ts is the exponential of the negated isometric latitude, i.e.
// exp(-𝜓), but evaluated in a numerically more stable way than
// the naive ellps.isometric_latitude(...).exp()
//
// This version is essentially identical to Charles Karney's PROJ
// version, including the majority of the comments.
//
// Inputs:
//   (sin phi, cos phi) = trigs of geographic latitude
//   e = eccentricity of the ellipsoid
// Output:
//   ts = exp(-psi) where psi is the isometric latitude (dimensionless)
//      = 1 / (tan(chi) + sec(chi))
// Here isometric latitude is defined by
//   psi = log( tan(pi/4 + phi/2) *
//              ( (1 - e*sin(phi)) / (1 + e*sin(phi)) )^(e/2) )
//       = asinh(tan(phi)) - e * atanh(e * sin(phi))
//       = asinh(tan(chi))
//   chi = conformal latitude
pub(crate) fn pj_tsfn(sincos: (f64, f64), e: f64) -> f64 {
    // exp(-asinh(tan(phi)))
    //    = 1 / (tan(phi) + sec(phi))
    //    = cos(phi) / (1 + sin(phi))  good for phi > 0
    //    = (1 - sin(phi)) / cos(phi)  good for phi < 0
    let factor = if sincos.0 > 0. {
        sincos.1 / (1. + sincos.0)
    } else {
        (1. - sincos.0) / sincos.1
    };
    (e * (e * sincos.0).atanh()).exp() * factor
}

// Snyder (1982) eq. 12-15, PROJ's pj_msfn()
pub(crate) fn pj_msfn(sincos: (f64, f64), es: f64) -> f64 {
    sincos.1 / (1. - sincos.0 * sincos.0 * es).sqrt()
}

// Equivalent to the PROJ pj_phi2 function
pub(crate) fn pj_phi2(ts0: f64, e: f64) -> f64 {
    sinhpsi_to_tanphi((1. / ts0 - ts0) / 2., e).atan()
}

// Ancillary function for computing the inverse isometric latitude.
// Follows [Karney, 2011](crate::Bibliography::Kar11), and the PROJ
// implementation in proj/src/phi2.cpp
pub(crate) fn sinhpsi_to_tanphi(taup: f64, e: f64) -> f64 {
    // min iterations = 1, max iterations = 2; mean = 1.954
    const MAX_ITER: usize = 5;

    // rooteps, tol and tmax are compile time constants, but currently
    // Rust cannot const-evaluate powers and roots, so we must either
    // evaluate these "constants" as lazy_statics, or just swallow the
    // penalty of an extra sqrt and two divisions on each call.
    // If this shows unbearable, we can just also assume IEEE-64 bit
    // arithmetic, and set rooteps = 0.000000014901161193847656
    let rooteps: f64 = f64::EPSILON.sqrt();
    let tol: f64 = rooteps / 10.; // the criterion for Newton's method
    let tmax: f64 = 2. / rooteps; // threshold for large arg limit exact

    let e2m = 1. - e * e;
    let stol = tol * taup.abs().max(1.0);

    // The initial guess.  70 corresponds to chi = 89.18 deg
    let mut tau = if taup.abs() > 70. {
        taup * (e * e.atanh()).exp()
    } else {
        taup / e2m
    };

    // Handle +/-inf, nan, and e = 1
    if (tau.abs() >= tmax) || tau.is_nan() {
        return tau;
    }

    for _ in 0..MAX_ITER {
        let tau1 = (1. + tau * tau).sqrt();
        let sig = (e * (e * tau / tau1).atanh()).sinh();
        let taupa = (1. + sig * sig).sqrt() * tau - sig * tau1;
        let dtau =
            (taup - taupa) * (1. + e2m * (tau * tau)) / (e2m * tau1 * (1. + taupa * taupa).sqrt());
        tau += dtau;

        if (dtau.abs() < stol) || tau.is_nan() {
            return tau;
        }
    }
    f64::NAN
}

#[cfg(test)]
pub fn some_basic_coordinates() -> [CoordinateTuple; 2] {
    let copenhagen = CoordinateTuple::raw(55., 12., 0., 0.);
    let stockholm = CoordinateTuple::raw(59., 18., 0., 0.);
    [copenhagen, stockholm]
}
