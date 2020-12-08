extern crate yaml_rust;
use geodesy::operators::helmert::helmert;
use geodesy::operators::hulmert::hulmert;
use geodesy::Coord;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

// SE https://stackoverflow.com/questions/41301239/how-to-unbox-elements-contained-in-polymorphic-vectors

//const MESSAGES: [&'static str; 20] = [
//    "OK",
//    "Warning 1",
//];
//const OPERATOR_NAME: &'static str = "cart";
//const OPERATOR_DESC: &'static str = "Convert between cartesian and geographical coordinates";


// ----------------- TYPES -------------------------------------------------

#[derive(Debug)]
pub struct OperatorWorkSpace {
    pub first: f64,
    pub second: f64,
    pub third: f64,
    pub fourth: f64,
    pub last_failing_operation: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct CoordType {

}


pub type Operator = Box<dyn IsOperator>;
pub type Pipeline = Vec<Operator>;

pub trait IsOperator {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> i32;
    fn inv(&self, ws: &mut OperatorWorkSpace) -> i32;
    fn print_name(&self) {
        println!("*** {} ***", self.name());
    }
    fn name(&self) -> &'static str {
        return "UNKNOWN";
    }
    //fn is_inverted(&self) -> bool .. return self.inverted
    //fn operate(&self, dir: bool) .. if inverted dir=!dir if dir fwd else inv
    //fn msg(&self, errcode: i32) -> &'static str;
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


#[cfg(test)]
mod tests {
    #[test]
    fn pargs() {
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




// ----------------- HELM -------------------------------------------------
struct Helm {
    dx: f64,
    dy: f64,
    dz: f64,
}

impl IsOperator for Helm {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> i32 {
        ws.first += self.dx;
        ws.second += self.dy;
        ws.third += self.dz;
        return 0;
    }
    fn inv(&self, ws: &mut OperatorWorkSpace) -> i32 {
        ws.first -= self.dx;
        ws.second -= self.dy;
        ws.third -= self.dz;
        return 0;
    }
    fn name(&self) -> &'static str {
        return "HELM";
    }
}


fn get_helm() -> Operator {
    let s = Helm {
        dx: 1f64,
        dy: 2f64,
        dz: 3f64,
    };
    return Box::new(s);
}



// ----------------- CART -------------------------------------------------
struct Cart {
    dx: f64,
    dy: f64,
    dz: f64,
}

impl IsOperator for Cart {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> i32 {
        ws.first += self.dx;
        ws.second += self.dy;
        ws.third += self.dz;
        return 0;
    }
    fn inv(&self, ws: &mut OperatorWorkSpace) -> i32 {
        ws.first -= self.dx;
        ws.second -= self.dy;
        ws.third -= self.dz;
        return 0;
    }
    fn print_name(&self) {
        println!("*** Cart ***");
    }
    fn name(&self) -> &'static str {
        return "CART";
    }
}


fn get_cart(args: &mut OperatorArgs) -> Operator {
    let mut s = Cart {
        dx: 1f64,
        dy: 2f64,
        dz: 3f64,
    };
    s.dx = args.numeric_value("dx", 0.0);
    s.dy = args.numeric_value("dy", 0.0);
    s.dz = args.numeric_value("dz", 0.0);
    return Box::new(s);
}


fn operator_factory(name: &str, args: &mut OperatorArgs) -> Operator {
    if name == "cart" {
        return get_cart(args);
    }
    if name == "helm" {
        return get_helm();
    }
    return get_cart(args);
}

fn generic_experiment() -> Pipeline {
    println!("GENERIC *****************************");
    let mut o = OperatorWorkSpace {
        first: 1.0,
        second: 2.0,
        third: 3.0,
        fourth: 4.0,
        last_failing_operation: "",
    };
    let mut args = OperatorArgs::new();
    operator_factory("cart", &mut args);
    args.insert("dx", "1");
    args.insert("dy", "2");
    args.insert("dz", "3");
    let c = get_cart(&mut args);
    let h = get_helm();

    c.fwd(&mut o);
    println!("{:?}", o);
    c.inv(&mut o);
    println!("{:?}", o);

    c.print_name();
    h.print_name();

    let mut pipeline: Pipeline = Vec::new();
    pipeline.push(c);
    pipeline.push(h);
    for x in &pipeline {
        x.print_name();
        println!("{}", x.name());
    }

    return pipeline;
}

fn main() {
    /*
    let helm = pain();
    let hulm = pulm();
    let mut v: Vec<Operator> = Vec::new();
    v.push(hulm);
    v.push(helm);
    v.push(pulm());

    let mut v: Vec<Op> = Vec::new();
    let cart = Cart{a: 6378137.0, f: 1.0/298.257};
    let hehe = Helm{dx: 1., dy: 2., dz: 3.};
    v.push(cart);
    v.push(hehe);
    */

    let pipeline = generic_experiment();
    println!("MAIN*****************************");
    for x in &pipeline {
        x.print_name();
    }
}

fn pain() -> Box<dyn Fn(&mut Coord, bool) -> bool> {
    let mut pap = HashMap::new();

    let txt = std::fs::read_to_string("src/transformations.yml").unwrap();
    let docs = YamlLoader::load_from_str(&txt).unwrap();
    let globals = docs[0]["main"]["globals"].as_hash().unwrap();
    let iter = globals.iter();
    println!("\nGlobals: {:?}\n", globals);
    for (arg, val) in iter {
        if arg.as_str().unwrap() != "dir" {
            pap.insert(arg, val);
        }
    }

    println!("\nPAP: {:?}\n", pap);
    println!("\nkeys: {:?}\n", pap.keys());
    let hule = Yaml::from_str("hule");
    let ellps = Yaml::from_str("ellps");
    let bopbop = Yaml::Integer(33);
    pap.insert(&hule, &bopbop);
    pap.insert(&ellps, &bopbop);
    if let Yaml::Integer(c) = pap[&hule] {
        println!("PAPC: {}", *c as f64);
    }

    // Multi document support, doc is a yaml::Yaml
    let doc = docs[0].as_hash().unwrap();
    let iter = doc.iter();
    println!("\n{:?}\n", doc.len());

    for item in iter {
        println!("{}", &item.0.as_str().unwrap_or("~"));
    }

    let mut par = HashMap::new();
    let k = Yaml::from_str("dx");
    let v = Yaml::Real(1.to_string());
    par.insert(&k, &v);
    let k = Yaml::from_str("dy");
    let v = Yaml::Real(2.to_string());
    par.insert(&k, &v);
    let k = Yaml::from_str("dz");
    let v = Yaml::Real(3.to_string());
    let v = Yaml::Real("^dx".to_string());
    par.insert(&k, &v);
    let k = Yaml::from_str("dp");
    let v = Yaml::from_str("dp");
    par.insert(&k, &v);
    println!("PAR: {:?}", par);

    let helm = helmert(&par);
    let mut x = Coord {
        first: 1.,
        second: 2.,
        third: 3.,
        fourth: 4.,
    };
    helm(&mut x, true);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 2.0);
    helm(&mut x, false);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 1.0);

    // Det er sådan det skal se ud fra en operationsimplementerings synspunkt
    let mut pax = HashMap::new();
    pax.insert(String::from("pap"), String::from("pop"));
    println!("PAX: {:?}", pax);
    return helm;
}

fn pulm() -> Box<dyn Fn(&mut Coord, bool) -> bool> {
    let mut par = HashMap::new();
    let k = Yaml::from_str("dx");
    let v = Yaml::Real(1.to_string());
    par.insert(&k, &v);
    let k = Yaml::from_str("dy");
    let v = Yaml::Real(2.to_string());
    par.insert(&k, &v);
    let k = Yaml::from_str("dz");
    let v = Yaml::Real(3.to_string());
    par.insert(&k, &v);
    let k = Yaml::from_str("dp");
    let v = Yaml::from_str("dp");
    par.insert(&k, &v);
    println!("PAR: {:?}", par);

    let hulm = hulmert(&par);
    let mut x = Coord {
        first: 1.,
        second: 2.,
        third: 3.,
        fourth: 4.,
    };
    hulm(&mut x, true);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 2.0);
    hulm(&mut x, false);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 1.0);
    return hulm;
}
