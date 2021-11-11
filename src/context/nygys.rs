#![allow(dead_code)]

use crate::operator_construction as oa;
use crate::operator_construction::Operator;
use crate::CoordinateTuple;
use crate::GeodesyError;
use std::collections::HashMap;

/// A pipeline with documentation, split into steps, ready for further
/// decomposition by `GysArgs`
#[derive(Debug)]
pub struct GysResource {
    doc: String,
    steps: Vec<String>,
    globals: Vec<(String, String)>,
}

impl From<&str> for GysResource {
    fn from(definition: &str) -> GysResource {
        GysResource::new(
            definition,
            &[(String::from("ellps"), String::from("GRS80"))],
        )
    }
}

impl GysResource {
    pub fn new(definition: &str, globals: &[(String, String)]) -> GysResource {
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
            doc: docstring,
            steps: trimmed_steps,
            globals: Vec::from(globals),
        }
    }
} // impl GysResource

/// The raw material for instantiation of Rust Geodesy objects
pub struct GysArgs {
    pub globals: Vec<(String, String)>,
    pub locals: Vec<(String, String)>,
    pub used: Vec<(String, String)>,
}

impl GysArgs {
    pub fn new(globals: &[(String, String)], step: &str) -> GysArgs {
        let globals = Vec::from(globals);
        let locals = GysArgs::step_to_local_args(step);
        let used = Vec::<(String, String)>::new();
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
            )));
        }
        Ok(default)
    }

    pub fn string(&mut self, key: &str, default: &str) -> Result<String, GeodesyError> {
        if let Some(value) = self.value(key)? {
            return Ok(value);
        }
        Ok(String::from(default))
    }
} // impl GysArgs

#[cfg(test)]
mod ny_gys_tests {
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
        assert_eq!(arg.flag("tomato")?, true);
        assert_eq!(arg.string("name", "")?, "banana");

        assert_eq!(arg.string("c", "")?, "c def");
        assert_eq!(arg.string("cc", "")?, "yes");

        assert_eq!(arg.flag("33")?, true);
        assert_eq!(arg.string("33", "44")?, "33");
        assert_eq!(arg.numeric("33", 44.)?, 33.);

        assert_eq!(arg.flag("true")?, false);

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

//---------------------------------------------------------------------------------
// Enter the land of the ResourceProviders
//---------------------------------------------------------------------------------

use crate::Ellipsoid;
use enum_iterator::IntoEnumIterator;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use uuid::Uuid;

// Preliminary scaffolding
pub trait Opera {
    fn operator_operate(&self, operands: &mut [CoordinateTuple], forward: bool) -> bool;
}
impl Opera for Operator {
    fn operator_operate(&self, _operands: &mut [CoordinateTuple], _forward: bool) -> bool {
        false
    }
}

pub struct PlainResourceProvider {
    searchlevel: ResourceProviderSearchLevel,
    check_builtins_first: bool,
    // pile: memmap::Mmap
    user_defined_operators: HashMap<String, oa::OperatorConstructor>,
    user_defined_macros: HashMap<String, String>,
    operations: HashMap<Uuid, oa::Operator>,
    globals: Vec<(String, String)>,
}

impl PlainResourceProvider {
    pub fn new(
        searchlevel: ResourceProviderSearchLevel,
        check_builtins_first: bool,
    ) -> PlainResourceProvider {
        let user_defined_operators = HashMap::new();
        let user_defined_macros = HashMap::new();
        let operations = HashMap::new();
        let globals = Vec::from([("ellps".to_string(), "GRS80".to_string())]);

        PlainResourceProvider {
            searchlevel,
            check_builtins_first,
            user_defined_operators,
            user_defined_macros,
            operations,
            globals,
        }
    }

    pub fn expand_experiment(&self, definition: &str) {
        let first = GysResource::new(definition, &self.globals);
        dbg!(first);
    }
}

impl ResourceProvider for PlainResourceProvider {
    fn searchlevel(&self) -> ResourceProviderSearchLevel {
        self.searchlevel
    }
    fn check_builtins_first(&self) -> bool {
        self.check_builtins_first
    }

    fn operate(&self, operation: Uuid, operands: &mut [CoordinateTuple], forward: bool) -> bool {
        if !self.operations.contains_key(&operation) {
            return false;
        }
        let op = &self.operations[&operation];
        op.operator_operate(operands, forward);
        false
    }

    /// Forward operation.
    fn fwd(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, true)
    }

    /// Inverse operation.
    fn inv(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }

    /// workhorse for gys_definition
    fn get_gys_definition_from_level(
        level: ResourceProviderSearchLevel,
        branch: &str,
        name: &str,
    ) -> Option<String> {
        let filename: PathBuf = match level {
            ResourceProviderSearchLevel::GlobalPatches => {
                // $HOME/share/geodesy/branch/name.gys
                let mut d = dirs::data_local_dir().unwrap_or_default();
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", name));
                d
            }
            ResourceProviderSearchLevel::Globals => {
                // $HOME/share/geodesy/branch/branch.gys
                let mut d = dirs::data_local_dir().unwrap_or_default();
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", branch));
                d
            }
            ResourceProviderSearchLevel::LocalPatches => {
                // ./geodesy/branch/name.gys
                let mut d = PathBuf::from(".");
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", name));
                d
            }
            ResourceProviderSearchLevel::Locals => {
                // ./geodesy/branch/branch.gys
                let mut d = PathBuf::from(".");
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", branch));
                d
            }
            _ => return None,
        };

        let file = std::fs::File::open(filename);
        if file.is_err() {
            return None;
        }
        let mut file = file.unwrap();

        // Patches
        if level == ResourceProviderSearchLevel::LocalPatches
            || level == ResourceProviderSearchLevel::GlobalPatches
        {
            let mut definition = String::new();
            if file.read_to_string(&mut definition).is_ok() {
                return Some(definition);
            }
            return None;
        }

        // For non-patches, we search for the *last* occurence of a section with
        // the name we want, since updates are appended to the existing file
        let mut definition = String::new();
        let target = format!("<{}>", name);
        let mut skipping = true;

        let lines = BufReader::new(file).lines();
        for line in lines {
            if line.is_err() {
                continue;
            }
            let text = line.ok()?;
            if skipping && text.trim() == target {
                skipping = false;
                definition.clear();
                continue;
            }
            if skipping {
                continue;
            }
            if text.trim().starts_with('<') {
                if text.trim() != target {
                    skipping = true;
                    continue;
                }
                // Another instance of the same target
                definition.clear();
                continue;
            }
            definition += &text;
            definition += "\n";
        }

            if !definition.is_empty() {
            return Some(definition);
        }
        None
    }
}

// let f = File::open("numbers.bin").expect("Error: 'numbers.bin' not found");
// Create the memory mapped buffer
// let mmap = unsafe { memmap::Mmap::map(&f).expect("Error mapping 'numbers.bin'") };

#[derive(Debug, IntoEnumIterator, Clone, Copy, PartialEq, PartialOrd)]
pub enum ResourceProviderSearchLevel {
    LocalPatches,
    Locals,
    GlobalPatches,
    Globals,
    Builtins,
}

pub trait ResourceProvider {
    fn searchlevel(&self) -> ResourceProviderSearchLevel {
        ResourceProviderSearchLevel::Builtins
    }
    fn check_builtins_first(&self) -> bool {
        true
    }

    fn gys_resource(
        &self,
        branch: &str,
        name: &str,
        globals: Vec<(String, String)>,
    ) -> Result<GysResource, GeodesyError> {
        match self.gys_definition(branch, name) {
            Ok(definition) => Ok(GysResource::new(&definition, &globals)),
            Err(err) => Err(err)
        }
    }

    fn gys_definition(&self, branch: &str, name: &str) -> Result<String, GeodesyError> {
        for i in ResourceProviderSearchLevel::into_enum_iter() {
            if i < self.searchlevel() && i != ResourceProviderSearchLevel::Builtins {
                continue;
            }
            if let Some(definition) = Self::get_gys_definition_from_level(i, branch, name) {
                return Ok(definition.trim().to_string());
            }
        }
        Err(GeodesyError::NotFound(format!("{}({})", branch, name)))
    }

    #[allow(unused_variables)]
    fn get_gys_definition_from_level(
        level: ResourceProviderSearchLevel,
        branch: &str,
        name: &str,
    ) -> Option<String> {
        None
    }

    fn operate(&self, operation: Uuid, operands: &mut [CoordinateTuple], forward: bool) -> bool;

    /// Forward operation.
    fn fwd(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, true)
    }

    /// Inverse operation.
    fn inv(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }

    fn ellipsoid(&self, name: &str) -> Result<Ellipsoid, GeodesyError> {
        if name == "GRS80" {
            return Ok(Ellipsoid::default());
        }
        Err(GeodesyError::NotFound(String::from(name)))
    }

    // fn grid_descriptor(&self, name: &str) -> Result<GridDescriptor, GeodesyError> {
    //     Err(GeodesyError::NotFound(String::from(name)))
    // }
}

#[cfg(test)]
mod resourceprovidertests {
    //use crate::GeodesyError;
    //use crate::context::nygys::*;
    use super::*;
    #[test]
    fn gys() -> Result<(), GeodesyError> {
        let foo = PlainResourceProvider::get_gys_definition_from_level(
            ResourceProviderSearchLevel::LocalPatches,
            "macros",
            "foo",
        )
        .unwrap();
        assert_eq!(foo.trim(), "bar");

        let foo = PlainResourceProvider::get_gys_definition_from_level(
            ResourceProviderSearchLevel::Locals,
            "macros",
            "foo",
        )
        .unwrap();
        assert_eq!(foo.trim(), "baz");

        let rp_patch = PlainResourceProvider::new(ResourceProviderSearchLevel::LocalPatches, false);
        let foo = rp_patch.gys_definition("macros", "foo")?;
        assert_eq!(foo, "bar");
        let rp_local = PlainResourceProvider::new(ResourceProviderSearchLevel::Locals, false);
        let foo = rp_local.gys_definition("macros", "foo")?;
        assert_eq!(foo, "baz");

        Ok(())
    }
}
