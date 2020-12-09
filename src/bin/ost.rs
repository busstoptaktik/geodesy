extern crate yaml_rust;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

// ----------------- TYPES -------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct CoordinateTuple(f64, f64, f64, f64);
impl CoordinateTuple {
    pub fn first(&self) -> f64 {self.0}
    pub fn second(&self) -> f64 {self.1}
    pub fn third(&self) -> f64 {self.2}
    pub fn fourth(&self) -> f64 {self.3}
}


#[derive(Debug)]
pub struct OperatorWorkSpace {
    pub coord: CoordinateTuple,
    pub stack: Vec<f64>,
    pub coordinate_stack: Vec<CoordinateTuple>,
    pub last_failing_operation: &'static str,
}

impl OperatorWorkSpace {
    fn new() -> OperatorWorkSpace {
        OperatorWorkSpace {
            coord: CoordinateTuple(0.,0.,0.,0.),
            stack: vec![],
            coordinate_stack: vec![],
            last_failing_operation: "",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CoordType {

}


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
struct OperatorArgs {
    args: HashMap<String, String>,
    used: HashMap<String, String>,
    all_used: HashMap<String, String>,
}

impl OperatorArgs{
    pub fn new() -> OperatorArgs {
        OperatorArgs{
            args: HashMap::new(),
            used: HashMap::new(),
            all_used: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.args.insert(key.to_string(), value.to_string());
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




// ----------------- HELM -------------------------------------------------
struct Helm {
    dx: f64,
    dy: f64,
    dz: f64,
    // pi: Pipeline,
}

impl Helm {
    pub fn new(args: &mut OperatorArgs) -> Helm {
        Helm {
            dx: args.numeric_value("dx", 0.0),
            dy: args.numeric_value("dy", 0.0),
            dz: args.numeric_value("dz", 0.0),
            // pi: vec![],
        }
    }
}


impl OperatorCore for Helm {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> bool {
        ws.coord.0 += self.dx;
        ws.coord.1 += self.dy;
        ws.coord.2 += self.dz;
        return true;
    }
    fn inv(&self, ws: &mut OperatorWorkSpace) -> bool {
        ws.coord.0 -= self.dx;
        ws.coord.1 -= self.dy;
        ws.coord.2 -= self.dz;
        return true;
    }
    fn name(&self) -> &'static str {
        return "HELM";
    }
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



    #[test]
    fn helm() {
        use super::*;
        let mut o = OperatorWorkSpace::new();
        let mut args = OperatorArgs::new();
        args.insert("dx", "1");
        args.insert("dy", "2");
        args.insert("dz", "3");
        println!("\nargs: {:?}\n", args);
        let h = operator_factory("helm", &mut args);
        h.fwd(&mut o);
        assert_eq!(o.coord.first(), 1.);

        h.inv(&mut o);
        assert_eq!(o.coord.first(), 0.);
        assert_eq!(o.coord.second(), 0.);
        assert_eq!(o.coord.third(), 0.);
    }


}






// ----------------- CART -------------------------------------------------
struct Cart {
    dx: f64,
    dy: f64,
    dz: f64,
}

impl Cart {
    pub fn new(args: &mut OperatorArgs) -> Cart {
        let dx = args.numeric_value("dx", 0.0);
        let dy = args.numeric_value("dy", 0.0);
        let dz = args.numeric_value("dz", 0.0);
        let cart = Cart {
            dx: dx,
            dy: dy,
            dz: dz,
        };
        return cart;
    }
}

impl OperatorCore for Cart {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> bool {
        ws.coord.0 += self.dx;
        ws.coord.1 += self.dy;
        ws.coord.2 += self.dz;
        return true;
    }
    fn inv(&self, ws: &mut OperatorWorkSpace) -> bool {
        ws.coord.0 -= self.dx;
        ws.coord.1 -= self.dy;
        ws.coord.2 -= self.dz;
        return true;
    }
    fn name(&self) -> &'static str {
        return "CART";
    }
}


fn operator_factory(name: &str, args: &mut OperatorArgs) -> Operator {
    if name == "cart" {
        return Box::new(Cart::new(args));
    }
    if name == "helm" {
        return Box::new(Helm::new(args));
    }
    return Box::new(Helm::new(args));
}


fn generic_experiment() -> Pipeline {
    // Se https://docs.rs/yaml-rust/0.4.4/yaml_rust/yaml/enum.Yaml.html
    let mut pap = OperatorArgs::new();
    let txt = std::fs::read_to_string("src/transformations.yml").unwrap();
    let docs = YamlLoader::load_from_str(&txt).unwrap();
    //println!("OOOOOOOOOOOOOOOOOOOOooooooo {}", docs[0]["main"].as_hash().unwrap().iter().len());
    let steps = docs[0]["main"]["steps"].as_vec().unwrap();
    for _s in steps {
        //println!("OOOOOOOOOOOOOOOOOOOOooooooo {:#?}", _s);
    }
    let globals = docs[0]["main"]["globals"].as_hash().unwrap();
    let iter = globals.iter();
    println!("\nGlobals: {:?}\n", globals);
    for (arg, val) in iter {
        if arg.as_str().unwrap() != "dir" {
            let vall = match val {
                Yaml::Integer(val) => val.to_string(),
                Yaml::Real(val) => val.as_str().to_string(),
                Yaml::String(val) => val.to_string(),
                Yaml::Boolean(val) => val.to_string(),
                _ => "".to_string(),
            };
            pap.insert(arg.as_str().unwrap(), &vall);
        }
    }
    println!("\nPap: {:?}\n", pap);


    println!("GENERIC *****************************");
    let mut o = OperatorWorkSpace::new();
    let mut args = OperatorArgs::new();
    operator_factory("cart", &mut args);
    args.insert("dx", "1");
    args.insert("dy", "2");
    args.insert("dz", "3");
    println!("\nargs: {:?}\n", args);
    let c = operator_factory("cart", &mut args);
    let h = operator_factory("helm", &mut args);

    c.fwd(&mut o);
    println!("{:?}", o);
    c.inv(&mut o);
    println!("{:?}", o);

    println!("{}", c.name());
    println!("{}", h.name());

    let mut pipeline: Pipeline = Vec::new();
    pipeline.push(c);
    pipeline.push(h);
    for x in &pipeline {
        println!("{}", x.name());
    }
    println!("{:?}", o);

    return pipeline;
}


fn main() {

    let pipeline = generic_experiment();
    println!("MAIN*****************************");
    for x in &pipeline {
        println!("{}", x.name());
    }
}
