use crate::GeodesyError;

/// Gys representation of a (potentially singleton) pipeline with (potential)
/// documentation, split into steps, ready for further decomposition into `GysArgs`
#[derive(Debug, Default, Clone)]
pub struct GysResource {
    pub id: String,
    pub doc: String,
    pub steps: Vec<String>,
    pub globals: Vec<(String, String)>,
}

impl From<&str> for GysResource {
    fn from(definition: &str) -> Self {
        GysResource::new(
            definition,
            &[(String::from("ellps"), String::from("GRS80"))],
        )
    }
}

impl From<(&str, &[(String, String)])> for GysResource {
    fn from(definition_and_globals: (&str, &[(String, String)])) -> Self {
        GysResource::new(definition_and_globals.0, definition_and_globals.1)
    }
}

impl GysResource {
    pub fn new(definition: &str, globals: &[(String, String)]) -> GysResource {
        let all = definition.replace("\r\n", "\n").trim().to_string();
        let all = all.replace("\r", "\n").trim().to_string();

        let id = all
            .split_whitespace()
            .next()
            .unwrap_or("UNKNOWN")
            .to_string();

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

        // Generate trimmed steps with elements separated by a single space and
        // key-value pairs glued by ':' as in 'key_0:value_0 key_1:value_1' etc.
        let steps: Vec<_> = trimmed.split('|').collect();
        let mut trimmed_steps = Vec::<String>::new();
        for mut step in steps {
            // Ignore empty steps
            step = step.trim();
            if step.is_empty() {
                continue;
            }
            // Normalize flags and repeated args
            let normalized = GysArgs::normalize_step(step);
            trimmed_steps.push(normalized);
        }

        GysResource {
            id,
            doc: docstring,
            steps: trimmed_steps,
            globals: Vec::from(globals),
        }
    }

    pub fn to_args(&self, step: usize) -> Result<GysArgs, GeodesyError> {
        if self.steps.len() < step {
            return Err(GeodesyError::General(
                "Attempt to extract undefined step from GysResource",
            ));
        }
        Ok(GysArgs::new(&self.globals, &self.steps[step]))
    }
} // impl GysResource

/// The raw material for instantiation of Rust Geodesy objects
#[derive(Debug)]
pub struct GysArgs {
    pub globals: Vec<(String, String)>,
    pub locals: Vec<(String, String)>,
    pub used: Vec<(String, String)>,
}

impl GysArgs {
    pub fn new(globals: &[(String, String)], step: &str) -> GysArgs {
        let globals = Vec::from(globals);
        let locals = GysArgs::step_to_args(step);
        let mut used = Vec::<(String, String)>::new();
        for name in &locals {
            if name.0 == "name" {
                used.push((name.0.clone(), name.1.clone()));
            }
        }
        GysArgs {
            globals,
            locals,
            used,
        }
    }

    fn step_to_args(step: &str) -> Vec<(String, String)> {
        // Conflate contiguous whitespace, then turn ': ' into ':'
        let step = step.trim().to_string();
        let elements: Vec<_> = step.split_whitespace().collect();
        let step = elements.join(" ").replace(": ", ":");

        let mut args = Vec::<(String, String)>::new();
        let elements: Vec<_> = step.split_whitespace().collect();
        for element in elements {
            let mut parts: Vec<&str> = element.trim().split(':').collect();
            parts.push("");
            assert!(parts.len() > 1);

            // If the first arg is a key-without-value, it is the name of the operator
            if args.is_empty() && parts.len() == 2 {
                args.push((String::from("name"), String::from(parts[0])));
                continue;
            }

            // In case of repeated args, only retain the last one specified
            if let Some(index) = args.iter().position(|x| x.0 == parts[0]) {
                args.remove(index);
            }

            // Flag normalization 1: Leave out flags explicitly set to false
            if parts[1].to_lowercase() == "false" {
                continue;
            }

            // Flag normalization 2: Remove explicit "true" values from flags
            if parts[1].to_lowercase() == "true" {
                parts[1] = "";
            }

            args.push((String::from(parts[0]), String::from(parts[1])));
        }

        args
    }

    // Opposite of step_to_args
    fn args_to_step(args: &[(String, String)]) -> String {
        let mut joined = String::new();
        for element in args {
            if element.0.is_empty() {
                continue;
            }
            joined += &element.0;
            if element.1.is_empty() {
                joined += " ";
                continue;
            }
            joined += ":";
            joined += &element.1;
            joined += " ";
        }
        joined.trim().to_string()
    }

    fn normalize_step(step: &str) -> String {
        // Normalize flags and repeated args
        let normal = GysArgs::step_to_args(step);
        // Re-join the normalized elements
        let joined = GysArgs::args_to_step(&normal);
        joined.trim().to_string()
    }

    pub fn value(&mut self, key: &str) -> Result<Option<String>, GeodesyError> {
        // The haystack is a reverse iterator over both lists in series
        let mut haystack = self.globals.iter().chain(self.locals.iter()).rev();

        // Find the needle in the haystack, recursively chasing look-ups ('^')
        // and handling defaults ('*')
        let key = key.trim();
        if key.is_empty() {
            return Err(GeodesyError::Syntax(String::from("Empty key")));
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
                    return Err(GeodesyError::Syntax(format!(
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

        self.used.push((String::from(key), String::from(&value)));
        Ok(Some(value))
    }

    /// A flag is true if its value is empty or anything but 'false' (case ignored)
    pub fn flag(&mut self, key: &str) -> bool {
        let value = self.value(key);

        // If incompletely given (i.e. contains a lookup for a non-existing key),
        // the flag value is false
        if value.is_err() {
            return false;
        }

        let value = value.unwrap();
        // If the key is not given, the flag is false
        // If the key is given and its lowercased value is 'false', the flag is false
        // In any other case, the flag is true
        match value {
            None => false,
            Some(v) => !(v.to_lowercase() == "false"),
        }
    }

    pub fn string(&mut self, key: &str, default: &str) -> String {
        let value = self.value(key);
        if value.is_err() {
            return String::from(default);
        }
        value.unwrap().unwrap_or_else(|| String::from(default))
    }

    pub fn numeric(&mut self, key: &str, default: f64) -> Result<f64, GeodesyError> {
        if let Some(value) = self.value(key)? {
            // key given, value numeric: return value
            if let Ok(v) = value.parse::<f64>() {
                return Ok(v);
            }

            // Error: key given, but value not numeric
            return Err(GeodesyError::Syntax(format!(
                "Numeric value expected for '{}' - got [{}: {}].",
                key, key, value
            )));
        }
        Ok(default)
    }

    pub fn required_numeric(&mut self, key: &str) -> Result<f64, GeodesyError> {
        return match self.numeric(key, f64::NAN) {
            Err(result) => Err(result),
            Ok(result) => {
                if result.is_nan() {
                    Err(GeodesyError::NotFound(key.to_string()))
                } else {
                    Ok(result)
                }
            }
        };
    }
} // impl GysArgs

// --------------------------------------------------------------------------------

#[cfg(test)]
mod gys_tests {
    use super::*;

    // Testing GysArgs
    #[test]
    fn args() -> Result<(), GeodesyError> {
        let globals: [(String, String); 6] = [
            (String::from("a"), String::from("a_def")),
            (String::from("b"), String::from("b_def")),
            (String::from("c"), String::from("c_def")),
            (String::from("d"), String::from("d_def")),
            (String::from("e"), String::from("e_def")),
            (String::from("f"), String::from("f_def")),
        ];

        let step = String::from("foo a:    ^b  b:2_b_def c:*2 d:^2 e:    2   f:^a g:*default");
        let mut arg = GysArgs::new(&globals, &step);

        // Check plain lookup functionality
        let f = arg.value("f")?;
        assert_eq!(f.unwrap(), globals[1].1);

        let e = arg.value("e")?;
        assert_eq!(e.unwrap(), "2");

        // Check default value lookups
        let c = arg.value("  c  ")?;
        assert_eq!(c.unwrap(), "c_def");

        let g = arg.value("  g  ")?;
        assert_eq!(g.unwrap(), "default");

        if let Err(d) = arg.value("d") {
            if let GeodesyError::Syntax(ref e) = d {
                assert!(e.starts_with("Incomplete"));
            } else {
                panic!("Unexpected error variant");
            }
        } else {
            panic!("Expected error here");
        }

        if let GeodesyError::Syntax(d) = arg.value("  d  ").unwrap_err() {
            assert!(d.starts_with("Incomplete"));
        } else {
            panic!("Unexpected error variant");
        }

        // step_to_args - check the 'name'-magic
        let step = "a b:c d:e f g:h";
        let args = GysArgs::step_to_args(step);
        assert_eq!(args.len(), 5);
        assert_eq!(args[0].0, "name");
        assert_eq!(args[0].1, "a");

        let mut arg = GysArgs::new(
            &globals,
            "banana aa:^a bb:b c:*no cc:*yes 33:33 true:FaLsE tomato",
        );
        assert_eq!(arg.flag("tomato"), true);
        assert_eq!(arg.string("name", ""), "banana");

        assert_eq!(arg.string("c", ""), "c_def");
        assert_eq!(arg.string("cc", ""), "yes");

        assert_eq!(arg.string("33", "44"), "33");
        assert_eq!(arg.numeric("33", 44.)?, 33.);
        assert_eq!(arg.flag("33"), true);

        assert_eq!(arg.flag("true"), false);

        let mut arg = GysArgs::new(&globals, "a b:c b:d inv:true inv inv:false");
        assert_eq!(arg.locals.len(), 2);
        assert_eq!(arg.string("b", ""), "d");
        assert_eq!(arg.flag("inv"), false);

        Ok(())
    }

    // Testing GysResource
    #[test]
    fn resource() -> Result<(), GeodesyError> {
        let text = "\n # agurk \n en # agurk\r\n  ## Document all cucumbers \n##\n## agurker\n\ta b:c|  c   d: e    |f g:h|\t\th\n\n\n";
        let gys = GysResource::from(text);
        assert!(gys.doc.starts_with("Document all cucumbers"));
        assert!(gys.doc.ends_with("agurker"));
        assert_eq!(gys.steps.len(), 4);

        let gys = GysResource::from("");
        assert!(gys.doc.is_empty());
        assert_eq!(gys.steps.len(), 0);

        Ok(())
    }
}
