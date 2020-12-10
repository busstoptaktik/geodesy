extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};


use std::collections::HashMap;
pub mod foundations;
pub mod operators;

// ----------------- TYPES -------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct CoordinateTuple(f64, f64, f64, f64);
impl CoordinateTuple {
    pub fn first(&self) -> f64 {
        self.0
    }
    pub fn second(&self) -> f64 {
        self.1
    }
    pub fn third(&self) -> f64 {
        self.2
    }
    pub fn fourth(&self) -> f64 {
        self.3
    }
}

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

#[derive(Clone, Copy, Debug)]
pub struct CoordType {}

pub type Operator = Box<dyn OperatorCore>;
pub type Pipeline = Vec<Operator>;

pub trait OperatorCore {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> bool;

    // implementations must override at least one of {inv, invertible}
    fn inv(&self, _ws: &mut OperatorWorkSpace) -> bool {
        return false;
    }
    fn invertible(&self) -> bool {
        return true;
    }

    fn name(&self) -> &'static str {
        return "UNKNOWN";
    }

    fn error_message(&self) -> &'static str {
        return "";
    }
    //fn is_inverted(&self) -> bool .. return self.inverted
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
        return arg;
    }

    /// Return the arg for a given key; maintain usage info.
    pub fn value(&mut self, key: &str, default: &str) -> String {
        let arg = self.value_recursive_search(key, default);
        if arg != default {
            self.used.insert(key.to_string(), arg.to_string());
        }
        return arg;
    }

    pub fn numeric_value(&mut self, key: &str, default: f64) -> f64 {
        let arg = self.value(key, "");
        // key not given: return default
        if arg == "" {
            return default;
        }
        // key given, but not numeric: return NaN
        return arg.parse().unwrap_or(f64::NAN);
    }
}

pub fn operator_factory(name: &str, args: &mut OperatorArgs) -> Operator {
    use crate::operators as co;
    if name == "cart" {
        return Box::new(co::cart::Cart::new(args));
    }
    if name == "helm" {
        return Box::new(co::helmert::Helm::new(args));
    }
    return Box::new(co::helmert::Helm::new(args));
}


pub fn steps_and_globals(name: &str) -> (Vec<Yaml>, OperatorArgs) {
    // Read YAML-document, locate "name", extract steps and globals
    let txt = std::fs::read_to_string("src/transformations.yml").unwrap();
    let docs = YamlLoader::load_from_str(&txt).unwrap();
    let steps = docs[0][name]["steps"].as_vec().unwrap();
    let globals = docs[0][name]["globals"].as_hash().unwrap();

    // Loop over all globals, create corresponding OperartorArgs object
    let mut args = OperatorArgs::new();
    let iter = globals.iter();
    for (arg, val) in iter {
        if arg.as_str().unwrap() != "inv" {
            let vall = match val {
                Yaml::Integer(val) => val.to_string(),
                Yaml::Real(val) => val.as_str().to_string(),
                Yaml::String(val) => val.to_string(),
                Yaml::Boolean(val) => val.to_string(),
                _ => "".to_string(),
            };
            args.insert(arg.as_str().unwrap(), &vall);
        }
    }

    return (steps.to_vec(), args);
}


#[cfg(test)]
mod tests {
    #[test]
    fn operator_args() {
        use super::*;
        let mut pargs = OperatorArgs::new();

        // dx and dy are straightforward
        pargs.insert("dx", "1");
        pargs.insert("dy", "2");

        // But we hide dz behind two levels of indirection
        pargs.insert("dz", "^ddz");
        pargs.insert("ddz", "^dddz");
        pargs.insert("dddz", "3");
        println!("pargs: {:?}", pargs);

        assert_eq!("1", pargs.value("dx", ""));
        println!("used: {:?}", &pargs.used);

        assert_eq!("2", pargs.value("dy", ""));
        println!("used: {:?}", &pargs.used);

        assert_eq!("3", pargs.value("dz", ""));
        assert_eq!(3.0, pargs.numeric_value("dz", 42.0));
        println!("used: {:?}", &pargs.used);
        println!("all_used: {:?}", &pargs.all_used);

        // Finally one for testing NAN returned for non-numerics
        pargs.insert("ds", "foo");
        assert!(pargs.numeric_value("ds", 0.0).is_nan());
    }
}
