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
            step = step.trim();
            if step.is_empty() {
                continue;
            }
            // Conflate contiguous whitespace, then turn ': ' into ':'
            let elements: Vec<_> = step.split_whitespace().collect();
            let joined = elements.join(" ").replace(": ", ":");
            trimmed_steps.push(joined);
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
        Ok(GysArgs::new(&self.globals, &self.steps[0]))
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
        let locals = GysArgs::step_to_local_args(step);
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

    pub fn new_symmetric(globals: &[(String, String)], locals: &[(String, String)]) -> GysArgs {
        let globals = Vec::from(globals);
        let locals = Vec::from(locals);
        let used = Vec::<(String, String)>::new();
        GysArgs {
            globals,
            locals,
            used,
        }
    }

    fn step_to_local_args(step: &str) -> Vec<(String, String)> {
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
            args.push((String::from(parts[0]), String::from(parts[1])));
        }

        args
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

            // If the value is a(nother) lookup, we continue the search in the same iterator
            if let Some(stripped) = thevalue.strip_prefix('^') {
                chasing = true;
                needle = stripped;
                continue;
            }

            // If the value is a default, we continue the search using the same *key*
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

        // If incompletely given, the flag value is false
        if value.is_err() {
            return false;
        }
        let value = value.unwrap();
        // If the key is not given, the flag is false
        // If the key is given and the value is 'false', the flag is false
        // If the key is given without any value, the flag is true
        // If the key is given with any other value than 'false', the flag is true
        match value {
            None => return false,
            Some(v) => {
                return if v.to_lowercase() == "false" {
                    false
                } else {
                    v.is_empty()
                }
            }
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

            // Error: key given, but not numeric
            return Err(GeodesyError::Syntax(format!(
                "Numeric value expected for '{}' - got [{}: {}].",
                key, key, value
            )));
        }
        Ok(default)
    }
} // impl GysArgs

#[cfg(test)]
mod new_gys_tests {
    //use crate::GeodesyError;
    //use crate::context::nygys::*;
    use super::*;

    // Testing GysArgs
    #[test]
    fn args() -> Result<(), GeodesyError> {
        let globals: [(String, String); 6] = [
            (String::from("a"), String::from("a def")),
            (String::from("b"), String::from("b def")),
            (String::from("c"), String::from("c def")),
            (String::from("d"), String::from("d def")),
            (String::from("e"), String::from("e def")),
            (String::from("f"), String::from("f def")),
        ];

        let locals: [(String, String); 7] = [
            (String::from("a"), String::from("   ^b  ")),
            (String::from("b"), String::from("2 b def")),
            (String::from("c"), String::from("*2 c def")),
            (String::from("d"), String::from("^2 d def")),
            (String::from("e"), String::from("    2 e def   ")),
            (String::from("f"), String::from("^a")),
            (String::from("g"), String::from("*default")),
        ];

        let mut arg = GysArgs::new_symmetric(&globals, &locals);

        // Check plain lookup functionality
        let f = arg.value("  f  ")?;
        assert_eq!(f.unwrap(), globals[1].1);

        let e = arg.value("  e  ")?;
        assert_eq!(e.unwrap(), "2 e def");

        // Check default value lookups
        let c = arg.value("  c  ")?;
        assert_eq!(c.unwrap(), "c def");

        let g = arg.value("  g  ")?;
        assert_eq!(g.unwrap(), "default");

        if let Err(d) = arg.value("d") {
            println!("d: {:?}", d.to_string());
            assert!(d.to_string().starts_with("syntax error"));
        }
        let d = arg.value("  d  ").unwrap_err();
        assert!(d.to_string().starts_with("syntax error"));

        let _d = arg.value("  d  ").unwrap_or_else(|e| {
            if !e.to_string().starts_with("syntax error") {
                panic!("Expected syntax error here!");
            } else {
                Some(String::default())
            }
        });

        // step_to_local_args - check the 'name'-magic
        let step = "a b:c d:e f g:h";
        let args = GysArgs::step_to_local_args(step);
        assert_eq!(args.len(), 5);
        assert_eq!(args[0].0, "name");
        assert_eq!(args[0].1, "a");

        let mut arg = GysArgs::new(
            &globals,
            "banana tomato aa:^a bb:b c:*no cc:*yes 33:33 true:FaLsE",
        );
        assert_eq!(arg.flag("tomato"), true);
        assert_eq!(arg.string("name", ""), "banana");

        assert_eq!(arg.string("c", ""), "c def");
        assert_eq!(arg.string("cc", ""), "yes");

        assert_eq!(arg.flag("33"), true);
        assert_eq!(arg.string("33", "44"), "33");
        assert_eq!(arg.numeric("33", 44.)?, 33.);

        assert_eq!(arg.flag("true"), false);

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
