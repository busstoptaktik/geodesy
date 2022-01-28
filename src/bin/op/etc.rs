/// Generally useful free functions, not tied to any specific type or trait,
/// but mostly related to the interpretation of operator parameters.
use crate::internal::*;

pub fn is_pipeline(definition: &str) -> bool {
    definition.contains('|')
}

pub fn is_resource_name(definition: &str) -> bool {
    operator_name(definition, "").contains(":")
}

pub fn operator_name(definition: &str, default: &str) -> String {
    if is_pipeline(definition) {
        return default.to_string();
    }
    split_into_parameters(&definition)
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
        .replace(", ", ",");

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
        // if parts[1].to_lowercase() == "false" {
        //     continue;
        // }

        // Flag normalization 2: Remove explicit "true" values from flags
        // if parts[1].to_lowercase() == "true" {
        //     parts[1] = "";
        // }

        params.insert(String::from(parts[0]), String::from(parts[1]));
    }

    dbg!(&params);
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



pub fn chase(globals: &BTreeMap<String, String>, locals: &BTreeMap<String, String>, key: &str) -> Result<Option<String>, Error> {
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



#[cfg(test)]
pub fn some_basic_coordinates() -> [CoordinateTuple; 2] {
    let copenhagen = CoordinateTuple::raw(55., 12., 0., 0.);
    let stockholm = CoordinateTuple::raw(59., 18., 0., 0.);
    [copenhagen, stockholm]
}
