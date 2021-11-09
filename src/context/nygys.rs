use crate::GeodesyError;

#[allow(dead_code)]
pub fn gys_to_steps(gys: &str) -> (String, Vec<String>) {
    let all = gys.replace("\r", "\n").trim().to_string();

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

    // Generate trimmed steps with elements spearated by single ws and key-value pairs glued by ':' as in 'k:v'
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
    (docstring, trimmed_steps)
}

#[allow(dead_code)]
pub fn step_to_args(step: &str) -> Vec::<(String, String)> {
    let mut args = Vec::<(String, String)>::new();
    let elements: Vec<_> = step.split_whitespace().collect();
    for element in elements {
        let mut parts: Vec<&str> = element.trim().split(':').collect();
        parts.push("");
        assert!(parts.len() > 1);
        // If the first arg is a key-without-value, it is the name of the operator
        if args.len()==0 {
            if parts.len()==2 {
                args.push((String::from("op"), String::from(parts[0])));
                continue;
            }
        }
        args.push((String::from(parts[0]), String::from(parts[1])));
    }

    args
}

pub struct GysArgs {
    pub globals: Vec::<(String, String)>,
    pub locals: Vec::<(String, String)>,
    pub used: Vec::<(String, String)>
}

impl GysArgs {
    pub fn new(globals: &[(String, String)], step: &str) -> GysArgs {
        let globals = Vec::from(globals);
        let locals = step_to_args(step);
        let used = Vec::<(String, String)>::new();
        GysArgs{globals, locals, used}
    }

    pub fn value(&mut self, key: &str) -> Result<Option<String>, GeodesyError> {
        let value = value_of_key(key, &self.globals, &self.locals)?;
        if value.is_none() {
            return Ok(None)
        }
        let value = value.unwrap();
        let result = String::from(&value);
        self.used.push((String::from(key), String::from(value)));
        Ok(Some(result))
    }

    pub fn flag(&mut self, key: &str) -> Result<bool, GeodesyError> {
        if let Some(value) = self.value(key)? {
            if value.to_lowercase() != "false" {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn numeric(&mut self, key: &str, default: f64) -> Result<f64, GeodesyError> {
        if let Some(value) = self.value(key)? {
            // key given, value numeric: return value
            if let Ok(v) = value.parse::<f64>() {
                return Ok(v);
            }

            // key given, but not numeric: return error string
            return Err(GeodesyError::Syntax(format!(
                "Numeric value expected for '{}' - got [{}: {}].",
                key, key, value
            )))
        }
        Ok(default)
    }

    pub fn string(&mut self, key: &str, default: &str) -> Result<String, GeodesyError> {
        if let Some(value) = self.value(key)? {
            return Ok(String::from(value));

        }
        Ok(String::from(default))
    }
}

#[allow(dead_code)]
pub fn value_of_key(key: &str, globals: &[(String, String)], locals: &[(String, String)]) -> Result<Option<String>, GeodesyError> {
    // The haystack is a reverse iterator over both lists in series
    let mut haystack = globals.iter().chain(locals.iter()).rev();

    // Find the needle in the haystack, recursively chasing look-ups ('^')
    let key = key.trim();
    if key.is_empty() {
        return Err(GeodesyError::Syntax(String::from("Empty key")));
    }
    let mut default = "";
    let mut needle = key;
    let mut chasing = false;
    loop {
        let found = haystack.find(|&x| x.0 == needle);
        if found.is_none() {
            if default != "" {
                return Ok(Some(String::from(default)));
            }
            if chasing {
                return Err(GeodesyError::Syntax(format!("Incomplete definition for '{}'", key)));
            }
            return Ok(None);
        }
        let thevalue = found.unwrap().1.trim();

        // If the value is a(nother) lookup, we continue the search in the same iterator
        if thevalue.starts_with("^") {
            chasing = true;
            needle = &thevalue[1..];
            continue;
        }

        // If the value is a default, we continue the search using the same key
        if thevalue.starts_with("*") {
            chasing = true;
            needle = key;
            default = &thevalue[1..];
            continue;
        }

        // Otherwise we have the proper result
        return Ok(Some(String::from(thevalue.trim())));
    }
}


#[cfg(test)]
mod newtests {
    //use crate::GeodesyError;
    //use crate::context::nygys::*;
    use super::*;
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

        // Check plain lookup functionality
        let f = value_of_key("  f  ", &globals, &locals)?;
        assert_eq!(f.unwrap(), globals[1].1);

        let e = value_of_key("  e", &globals, &locals)?;
        assert_eq!(e.unwrap(), "2 e def");

        // Check default value lookups
        let c = value_of_key("c  ", &globals, &locals)?;
        assert_eq!(c.unwrap(), "c def");

        let g = value_of_key(" g ", &globals, &locals)?;
        assert_eq!(g.unwrap(), "default");

        if let Err(d) = value_of_key("d", &globals, &locals) {
            println!("d: {:?}", d.to_string());
            assert!(d.to_string().starts_with("syntax error"));
        }
        let d = value_of_key("  d  ", &globals, &locals).unwrap_err();
        assert!(d.to_string().starts_with("syntax error"));

        let _d = value_of_key("  d  ", &globals, &locals).unwrap_or_else(|e|
            if !e.to_string().starts_with("syntax error") {
                panic!("Expected syntax error here!");
            } else {Some(String::default())}
        );

        let mut arg = GysArgs::new(&globals, "banana tomato aa:^a bb:b c:*no cc:*yes 33:33 true:FaLsE");
        assert_eq!(arg.flag("tomato")?, true);
        assert_eq!(arg.string("op", "")?, "banana");

        assert_eq!(arg.string("c", "")?, "c def");
        assert_eq!(arg.string("cc", "")?, "yes");

        assert_eq!(arg.flag("33")?, true);
        assert_eq!(arg.string("33", "44")?, "33");
        assert_eq!(arg.numeric("33", 44.)?, 33.);

        assert_eq!(arg.flag("true")?, false);

        Ok(())
    }


    #[test]
    fn gys() -> Result<(), GeodesyError> {
        let gys = "\n # agurk \n en # agurk\r\n  ## Document all cucumbers \n##\n## agurker\n\ta b:c|  c   d: e    |f g:h|\t\th\n\n\n";
        let (doc, steps) = gys_to_steps(gys);
        assert!(doc.starts_with("Document all cucumbers"));
        assert!(doc.ends_with("agurker"));
        assert_eq!(steps.len(), 4);

        let (doc, steps) = gys_to_steps("");
        assert!(doc.is_empty());
        assert_eq!(steps.len(), 0);

        Ok(())
    }


    #[test]
    fn steps() -> Result<(), GeodesyError> {
        let step = "a b:c d:e f g:h";
        let args = step_to_args(step);
        assert_eq!(args.len(), 5);
        assert_eq!(args[0].0, "op");
        assert_eq!(args[0].1, "a");

        Ok(())
    }

}
