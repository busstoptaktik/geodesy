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
}

#[derive(Clone, Copy, Debug)]
pub struct CoordType {}


pub type Operator = Box<dyn IsOperator>;
pub type Pipeline = Vec<Operator>;
pub trait IsOperator {
    fn fwd(&self, ws: &mut OperatorWorkSpace) -> i32;
    fn inv(&self, ws: &mut OperatorWorkSpace) -> i32;
    fn print_name(&self) {
        println!("*** {} ***", self.name());
    }
    //fn name(&self) -> &'static str;
    fn name(&self) -> &'static str {
        return "UNKNOWN";
    }
    //fn msg(&self, errcode: i32) -> &'static str;
    //fn left(&self) -> CoordType;
    //fn right(&self) -> CoordType;
}

pub type OperatorArgs = HashMap<String, String>;
pub trait IsOperatorArgs {
    fn num_val(&self, key: &str, default: f64) -> f64;
}
impl IsOperatorArgs for OperatorArgs {
    fn num_val(&self, key: &str, default: f64) -> f64 {
        let arg = self.get(key);
        let arg = match arg {
            Some(arg) => arg,
            None => return default,
        };
        if arg.starts_with("^") {
            let mut v = arg.clone();
            v.remove(0);
            return self.num_val(v.as_str(), default);
        }
        return arg.parse::<f64>().unwrap_or(default);
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

fn get_cart(args: &OperatorArgs) -> Operator {
    let mut s = Cart {
        dx: 1f64,
        dy: 2f64,
        dz: 3f64,
    };
    s.dx = args.num_val("dx", 0.0);
    s.dy = args.num_val("dy", 0.0);
    s.dz = args.num_val("dz", 0.0);
    return Box::new(s);
}



fn generic_experiment() -> Pipeline {
    println!("GENERIC *****************************");
    let mut o = OperatorWorkSpace {
        first: 1.0,
        second: 2.0,
        third: 3.0,
        fourth: 4.0,
    };
    let mut args: OperatorArgs = HashMap::new();
    args.insert("dx".to_string(), "^thedxvalue".to_string());
    args.insert("thedxvalue".to_string(), "^thehiddendxvalue".to_string());
    args.insert("thehiddendxvalue".to_string(), "1".to_string());

    args.insert("dy".to_string(), "2".to_string());
    args.insert("dz".to_string(), "3".to_string());
    assert_eq!(1.0, args.num_val("dx", 0.0));

    let c = get_cart(&args);
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
