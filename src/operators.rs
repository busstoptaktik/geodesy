use std::collections::HashMap;

use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use crate::CoordinateTuple;

mod badvalue;
mod cart;
mod helmert;
mod noop;
mod pipeline;

pub type Operator = Box<dyn OperatorCore>;
pub type Steps = Vec<Operator>;


#[derive(Debug)]
pub struct OperatorWorkSpace {
    pub coord: CoordinateTuple,
    pub stack: Vec<f64>,
    pub coordinate_stack: Vec<CoordinateTuple>,
    pub last_failing_operation: &'static str,
}

impl OperatorWorkSpace {
    pub fn new() -> OperatorWorkSpace {
        OperatorWorkSpace {
            coord: CoordinateTuple(0., 0., 0., 0.),
            stack: vec![],
            coordinate_stack: vec![],
            last_failing_operation: "",
        }
    }
}



pub trait OperatorCore {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> bool;

    // implementations must override at least one of {inv, invertible}
    fn inv(&self, _ws: &mut OperatorWorkSpace) -> bool {
        false
    }
    fn invertible(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "UNKNOWN"
    }

    fn error_message(&self) -> &'static str {
        "Unknown error"
    }

    fn is_inverted(&self) -> bool;

    fn is_noop(&self) -> bool {
        false
    }

    fn is_badvalue(&self) -> bool {
        false
    }

    //fn operate(&self, dir: bool) .. if inverted dir=!dir if dir fwd else inv
    //fn left(&self) -> CoordType;
    //fn right(&self) -> CoordType;
}



#[derive(Debug)]
pub struct OperatorArgs {
    args: HashMap<String, String>,
    used: HashMap<String, String>,
    all_used: HashMap<String, String>,
}

impl OperatorArgs {
    pub fn new() -> OperatorArgs {
        OperatorArgs {
            args: HashMap::new(),
            used: HashMap::new(),
            all_used: HashMap::new(),
        }
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
    pub fn boolean_value(&mut self, key: &str) -> bool {
        self.value(key, "false") != "false"
    }
}


pub fn steps_and_globals(full_text: String, name: &str, globals: &mut OperatorArgs) {
    // Read YAML-document, locate "name", extract steps and globals
    let docs = YamlLoader::load_from_str(&full_text).unwrap();

    // Loop over all globals and create corresponding OperartorArgs entries
    let moreglobals = &docs[0][name]["globals"].as_hash().unwrap();
    for (arg, val) in moreglobals.iter() {
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
                globals.insert(thearg, &theval);
            }
        }
    }


    if docs[0][name]["steps"].as_vec().is_some() {
        println!("************************** EMPTY ****************************");
    }

    // Locate the step definitions, and insert the number of steps
    // into the argument list
    let steps = docs[0][name]["steps"].as_vec().unwrap();
    let nsteps = steps.len();
    globals.insert("nsteps", &nsteps.to_string());

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
        let step_key = format!("step_{}", index);
        globals.insert(&step_key, stripped_definition);
    }


    // Finally, we provide the full text to enable recursive definitions
    globals.insert("full_text", &full_text);
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
        println!("args: {:?}", args);

        assert_eq!("1", args.value("dx", ""));
        assert_eq!("2", args.value("dy", ""));
        assert_eq!(args.used.len(), 2);

        assert_eq!("3", args.value("dz", ""));
        assert_eq!(3.0, args.numeric_value("dz", 42.0));

        assert_eq!(args.used.len(), 3);
        assert_eq!(args.all_used.len(), 5);

        println!("used: {:?}", &args.used);
        println!("all_used: {:?}", &args.all_used);

        assert_eq!("", args.value("abcdefg", ""));

        // Finally one for testing NAN returned for non-numerics
        args.insert("ds", "foo");
        assert!(args.numeric_value("ds", 0.0).is_nan());
    }

    #[test]
    fn parsing_steps() {
        use super::*;
        let mut globals = OperatorArgs::new();
        // Read YAML-document, locate "name", extract steps and globals
        let txt = std::fs::read_to_string("tests/tests.yml").unwrap();

        globals.insert("ellps", "GRS80");
        steps_and_globals(txt, "pipeline", &mut globals);
        println!("GLOBALS****: {:?}", globals);
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


pub fn operator_factory(name: &str, args: &mut OperatorArgs) -> Operator {
    use crate::operators as co;
    if name == "badvalue" {
        return Box::new(co::badvalue::BadValue::new(args))
    }
    if name == "cart" {
        return Box::new(co::cart::Cart::new(args));
    }
    if name == "helmert" {
        return Box::new(co::helmert::Helmert::new(args));
    }
    if name == "noop" {
        return Box::new(co::noop::Noop::new(args));
    }
    if name == "pipeline" {
        return Box::new(co::pipeline::Pipeline::new(args));
    }

    // Herefter: Søg efter 'name' i filbøtten
    Box::new(co::badvalue::BadValue::new(args))
}
