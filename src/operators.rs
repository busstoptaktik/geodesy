use std::collections::HashMap;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};
use crate::CoordinateTuple;

// Renovering af Poder/Engsager tmerc i B:\2019\Projects\FIRE\tramp\tramp\tramp.c
// Detaljer i C:\Users\B004330\Downloads\2.1.2 A HIGHLY ACCURATE WORLD WIDE ALGORITHM FOR THE TRANSVE (1).doc

mod badvalue;
mod cart;
mod helmert;
mod noop;
mod pipeline;

pub type Operator = Box<dyn OperatorCore>;

#[derive(Debug)]
pub struct Operand {
    pub coord: CoordinateTuple,
    pub stack: Vec<f64>,
    pub coordinate_stack: Vec<CoordinateTuple>,
    pub last_failing_operation: &'static str,
    pub cause: &'static str,
}

impl Operand {
    pub fn new() -> Operand {
        Operand {
            coord: CoordinateTuple(0., 0., 0., 0.),
            stack: vec![],
            coordinate_stack: vec![],
            last_failing_operation: "",
            cause: "",
        }
    }
}


pub trait OperatorCore {
    fn fwd(&self, ws: &mut Operand) -> bool;

    // implementations must override at least one of {inv, invertible}
    fn inv(&self, operand: &mut Operand) -> bool {
        operand.last_failing_operation = self.name();
        operand.cause = "Operator not invertible";
        false
    }

    fn invertible(&self) -> bool {
        true
    }

    // operate fwd/inv, taking operator inversion into account
    fn operate(&self, operand: &mut Operand, forward: bool) -> bool {
        // Short form of (inverted && !forward) || (forward && !inverted)
        if self.is_inverted() != forward {
            return self.fwd(operand)
        }
        // We do not need to check for self.invertible() here, since non-invertible
        // operators will return false as per the default-defined fn inv() above.
        self.inv(operand)
    }

    // number of steps. 1 unless the operator is a pipeline
    fn steps(&self) -> usize {
        1 as usize
    }

    fn args(&self, step: usize) -> &OperatorArgs;

    fn name(&self) -> &'static str {
        "UNKNOWN"
    }

    fn error_message(&self) -> &'static str {
        "Generic error"
    }

    fn is_inverted(&self) -> bool;

    fn is_noop(&self) -> bool {
        false
    }

    fn is_badvalue(&self) -> bool {
        false
    }

    //fn left(&self) -> CoordType;
    //fn right(&self) -> CoordType;
}


#[derive(Debug, Clone)]
pub struct OperatorArgs {
    name: String,
    args: HashMap<String, String>,
    used: HashMap<String, String>,
    all_used: HashMap<String, String>,
}

impl OperatorArgs {
    pub fn new() -> OperatorArgs {
        OperatorArgs {
            name: String::new(),
            args: HashMap::new(),
            used: HashMap::new(),
            all_used: HashMap::new(),
        }
    }

    /// Provides an OperatorArgs object, populated by the global defaults (`ellps: GRS80`)
    pub fn global_defaults() -> OperatorArgs {
        let mut op = OperatorArgs {
            name: String::new(),
            args: HashMap::new(),
            used: HashMap::new(),
            all_used: HashMap::new(),
        };
        op.insert("ellps", "GRS80");
        op
    }


    /// Provides an OperatorArgs object, populated by the defaults from an existing
    /// OperatorArgs, combined with a new object definition.
    pub fn with_globals_from(existing: &OperatorArgs, definition: &str, which: &str) -> OperatorArgs {
        let mut oa = OperatorArgs::new();
        for (arg, val) in &existing.args {
            if arg.starts_with("_") || (arg == "inv") {
                continue
            }
            oa.insert(arg, val);
        }
        oa.populate(definition, which);
        oa
    }

    ///
    /// Insert PROJ style operator definition arguments, converted from a YAML
    /// setup string.
    ///
    /// If `which` is set to the empty string, we first look for a pipeline
    /// definition. If that is not found, and there is only one list element
    /// in the setup string, we assert that this is the element to handle.
    ///
    /// If `which` is not the empty string, we look for a list element with
    /// that name, and handle that either as a pipeline definition, or as a
    /// single operator definition.
    ///
    /// # Returns
    ///
    /// `true` on success, `false` on sseccus.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geodesy::operators::OperatorArgs;
    ///
    /// let mut args = OperatorArgs::global_defaults();
    /// let txt = std::fs::read_to_string("tests/tests.yml").unwrap_or_default();
    ///
    /// assert!(args.populate(&txt, "pipeline"));
    /// assert_eq!(&args.value("_step_0", "")[0..4], "cart");
    /// ```
    ///
    ///
    pub fn populate(&mut self, definition: &str, which: &str) -> bool {
        // First, we copy the full text in the args, to enable recursive definitions
        self.insert("_definition", definition);

        // Read the entire YAML-document and extract the first sub-document
        let docs = YamlLoader::load_from_str(definition).unwrap();
        let main = &docs[0].as_hash();
        if main.is_none() {
            return self.badvalue("Cannot parse definition");
        }
        let main = main.unwrap();

        // Is it conforming?
        let mut main_entry_name = which;
        if main_entry_name == "" {
            for (arg, val) in main {
                if val.is_badvalue() {
                    return self.badvalue("Cannot parse definition");
                }
                let name = &arg.as_str().unwrap();
                if name.starts_with("_") {
                    continue;
                }
                if main_entry_name != "" {
                    return self.badvalue("Too many items in definition root");
                }
                main_entry_name = name;
            }
        }
        self.name = main_entry_name.to_string();

        // Grab the sub-tree defining the 'main_entry_name'
        let main_entry = &docs[0][main_entry_name];
        if main_entry.is_badvalue() {
            return self.badvalue("Cannot locate definition");
        }

        // Loop over all globals and create the corresponding OperatorArgs entries
        let globals = main_entry["globals"].as_hash();
        if globals.is_some() {
            let globals = globals.unwrap();
            for (arg, val) in globals {
                let thearg = arg.as_str().unwrap();
                if thearg != "inv" {
                    let theval = match val {
                        Yaml::Integer(val) => val.to_string(),
                        Yaml::Real(val) => val.as_str().to_string(),
                        Yaml::String(val) => val.to_string(),
                        Yaml::Boolean(val) => val.to_string(),
                        _ => "".to_string(),
                    };
                    if theval != "" {
                        self.insert(thearg, &theval);
                    }
                }
            }
        }

        // Try to locate the step definitions, to determine whether we
        // are handling a pipeline or a plain operator definition
        let steps = main_entry["steps"].as_vec();

        // Not a pipeline? Just insert the operator args and return
        if steps.is_none() {
            let args = main_entry.as_hash();
            if args.is_none() {
                return self.badvalue("Cannot read args");
            }
            let args = args.unwrap();
            for (arg, val) in args {
                let thearg = arg.as_str().unwrap();
                let theval = match val {
                    Yaml::Integer(val) => val.to_string(),
                    Yaml::Real(val) => val.as_str().to_string(),
                    Yaml::String(val) => val.to_string(),
                    Yaml::Boolean(val) => val.to_string(),
                    _ => "".to_string(),
                };
                if theval != "" {
                    self.insert(thearg, &theval);
                }
            }
            return true;
        }

        // It's a pipeline - insert the number of steps into the argument list.
        let steps = steps.unwrap();
        let nsteps = steps.len();
        self.insert("_nsteps", &nsteps.to_string());

        // Insert each step into the argument list, formatted as YAML.
        // Unfortunately the compact mode does not work.
        for (index, step) in steps.iter().enumerate() {
            // Write the step definition to a new string
            let mut step_definition = String::new();
            let mut emitter = YamlEmitter::new(&mut step_definition);
            emitter.compact(true);
            assert_eq!(emitter.is_compact(), true);
            emitter.dump(step).unwrap();

            // Remove the initial doc separator "---\n"
            let stripped_definition = step_definition.trim_start_matches("---\n");
            let step_key = format!("_step_{}", index);
            self.insert(&step_key, stripped_definition);
        }

        true
    }

    fn badvalue(&mut self, cause: &str) -> bool {
        self.name = "badvalue".to_string();
        self.insert("cause", cause);
        false
    }



    pub fn name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.args.insert(key.to_string(), value.to_string());
    }

    pub fn append(&mut self, additional: &OperatorArgs) {
        let iter = additional.args.iter();
        for (key, val) in iter {
            self.insert(key, val);
        }
    }

    // Workhorse for ::value - this indirection is needed in order to keep the
    // original key available, when traversing an indirect definition.
    fn value_recursive_search(&mut self, key: &str, default: &str) -> String {
        let arg = self.args.get(key);
        let arg = match arg {
            Some(arg) => arg.to_string(),
            None => return default.to_string(),
        };
        // all_used includes intermediate steps in indirect definitions
        self.all_used.insert(key.to_string(), arg.to_string());
        if arg.starts_with("^") {
            let arg = &arg[1..];
            return self.value_recursive_search(arg, default);
        }
        arg
    }

    /// Return the arg for a given key; maintain usage info.
    pub fn value(&mut self, key: &str, default: &str) -> String {
        let arg = self.value_recursive_search(key, default);
        if arg != default {
            self.used.insert(key.to_string(), arg.to_string());
        }
        arg
    }

    pub fn numeric_value(&mut self, key: &str, default: f64) -> f64 {
        let arg = self.value(key, "");
        // key not given: return default
        if arg == "" {
            return default;
        }
        // key given, but not numeric: return NaN
        arg.parse().unwrap_or(f64::NAN)
    }

    // If key is given, and value != false: true; else: false
    pub fn flag(&mut self, key: &str) -> bool {
        self.value(key, "false") != "false"
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn operator_args() {
        use super::*;
        let mut args = OperatorArgs::new();

        // dx and dy are straightforward
        args.insert("dx", "1");
        args.insert("dy", "2");

        // But we hide dz behind two levels of indirection
        args.insert("dz", "^ddz");
        args.insert("ddz", "^dddz");
        args.insert("dddz", "3");
        // println!("args: {:?}", args);

        assert_eq!("1", args.value("dx", ""));
        assert_eq!("2", args.value("dy", ""));
        assert_eq!(args.used.len(), 2);

        assert_eq!("3", args.value("dz", ""));
        assert_eq!(3.0, args.numeric_value("dz", 42.0));

        assert_eq!(args.used.len(), 3);
        assert_eq!(args.all_used.len(), 5);

        // println!("used: {:?}", &args.used);
        // println!("all_used: {:?}", &args.all_used);

        assert_eq!("", args.value("abcdefg", ""));

        // Finally one for testing NAN returned for non-numerics
        args.insert("ds", "foo");
        assert!(args.numeric_value("ds", 0.0).is_nan());
    }

    #[test]
    fn preparing_args() {
        use super::*;
        let mut args = OperatorArgs::global_defaults();

        // Explicitly stating the name of the pipeline
        let txt = std::fs::read_to_string("tests/tests.yml").unwrap_or_default();
        assert!(args.populate(&txt, "pipeline"));
        assert_eq!(&args.value("_step_0", "    ")[0..4], "cart");

        // Let populate() figure out what we want
        let mut args = OperatorArgs::global_defaults();
        assert!(args.populate(&txt, ""));
        assert_eq!(&args.value("_step_0", "    ")[0..4], "cart");

        // When op is not a pipeline
        let mut args = OperatorArgs::global_defaults();
        assert!(args.populate("cart: {ellps: intl}", ""));
        assert_eq!(args.name, "cart");
        assert_eq!(&args.value("ellps", ""), "intl");
    }

    #[test]
    fn bad_value() {
        use super::*;
        let v = Yaml::BadValue;
        assert!(v.is_badvalue());
        let v = Yaml::Null;
        assert!(v.is_null());
        let v = Yaml::Integer(77);
        assert!(v == Yaml::Integer(77));
    }
}


pub fn operator_factory(args: &mut OperatorArgs) -> Operator {
    use crate::operators as co;

    // Pipelines do not need to be named "pipeline": They are characterized simply
    // by containing steps.
    if args.name == "pipeline" || args.numeric_value("_nsteps", 0.0) as i64 > 0 {
        return Box::new(co::pipeline::Pipeline::new(args));
    }
    if args.name == "badvalue" {
        return Box::new(co::badvalue::BadValue::new(args))
    }
    if args.name == "cart" {
        return Box::new(co::cart::Cart::new(args));
    }
    if args.name == "helmert" {
        return Box::new(co::helmert::Helmert::new(args));
    }
    if args.name == "noop" {
        return Box::new(co::noop::Noop::new(args));
    }

    // Herefter: Søg efter 'name' i filbøtten
    Box::new(co::badvalue::BadValue::new(args))
}
