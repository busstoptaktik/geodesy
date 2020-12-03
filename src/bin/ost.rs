extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};
use std::collections::HashMap;
use geodesy::operators::helmert::helmert;
use geodesy::operators::hulmert::hulmert;
use geodesy::Coord;
use geodesy::Operator;
use geodesy::foundations::DMS;

/*
struct Op<T>{x: T}

struct Cart {a: f64, f: f64}
struct Helm {dx: f64, dy: f64, dz: f64}

trait HasArea {
    fn area(&self) -> f64;
}

impl HasArea for Rectangle {
    fn area(&self) -> f64 { self.length * self.height }
}


impl HasArea for Triangle {
    fn area(&self) -> f64 { self.length * self.height / 2.0 }
}

#[derive(Debug)]
struct Rectangle { length: f64, height: f64 }
#[derive(Debug)]
struct Triangle  { length: f64, height: f64 }

// `T` must implement `HasArea`. Any type which meets
// the bound can access `HasArea`'s function `area`.
struct Shp<T: HasArea>(T);

fn area<T: HasArea>(t: &T) -> f64 { t.area() }



// lånt fra https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=c11f8f2b3e5481faafb6367226de8c1e
#[derive(PartialEq)]
struct Material {}
#[derive(PartialEq)]
struct Matrix {}

trait Shape {
    fn material(&self) -> &Material;
    fn transformation(&self) -> &Matrix;
}

#[derive(PartialEq)]
struct Sphere {
    transformation: Matrix,
    material: Material
}

impl Shape for Sphere {
    fn material(&self) -> &Material {
        &self.material
    }
    fn transformation(&self) -> &Matrix {
        &self.transformation
    }
}

#[derive(PartialEq)]
struct Trapezoid {
    transformation: Matrix,
    material: Material,
    height: f64
}


impl Shape for Trapezoid {
    fn material(&self) -> &Material {
        &self.material
    }
    fn transformation(&self) -> &Matrix {
        &self.transformation
    }
}


impl<'a> PartialEq for dyn Shape + 'a {
    fn eq(&self, other: &Self) -> bool {
        self.material() == other.material() && self.transformation() == other.transformation()
    }
}

fn do_stuff(objects: &Vec<&dyn Shape>) {
    let obj1 = objects[0];
    let obj2 = objects[1];
    if obj1 == obj2 {
        println!("Equal");
    }
}


fn generic_experiment() -> Vec(&dyn Shape) {
    println!("************************** GENERIC ****************************");
    let s = Sphere{ transformation: Matrix{}, material: Material {} };
    let t = &Trapezoid{ transformation: Matrix{}, material: Material {}, height: 42. } as &dyn Shape;
    let s_shape = &s as &dyn Shape;
//    let t_shape = &t as &dyn Shape;
    let mut v = vec![ s_shape, t ];
    println!("{}", s_shape == s_shape);
    println!("{}", s_shape == t);

    do_stuff(&v);
    v;
}
*/






// SE https://stackoverflow.com/questions/41301239/how-to-unbox-elements-contained-in-polymorphic-vectors

trait HasArea {
    fn area(&self) -> f64;
    fn urea(&self) -> f64;
    fn aurea(&self, dir: i32) -> f64 {
        if dir > 0 {
            return self.area();
        }
        return self.urea();
    }
    fn print_area(&self) {
        println!("This shape has area of {}", self.area());
    }
}

struct Circle {
    x: f64,
    y: f64,
    radius: f64,
}

impl HasArea for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * (self.radius * self.radius)
    }
    fn urea(&self) -> f64 {
        self.radius
    }
}

struct Square {
    x: f64,
    y: f64,
    side: f64,
}

impl HasArea for Square {
    fn area(&self) -> f64 {
        self.side * self.side
    }
    fn urea(&self) -> f64 {
        self.side
    }
}


pub type Poperator = Box<dyn HasArea>;
pub type Pipeline = Vec<Poperator>;

fn get_circle() -> Poperator {
    let c = Circle {
        x: 0.0f64,
        y: 0.0f64,
        radius: 1.0f64,
    };
    return Box::new(c);
}

fn get_square() -> Poperator {
    let s = Square {
        x: 0.0f64,
        y: 0.0f64,
        side: 1.0f64,
    };
    return Box::new(s);
}

fn generic_experiment() ->  Pipeline {
    println!("GENERIC *****************************");

    /*
    let c = Circle {
        x: 0.0f64,
        y: 0.0f64,
        radius: 1.0f64,
    };
    */
    let c = get_circle();
    let s = get_square();


    c.print_area();
    s.print_area();

    let mut pipeline: Pipeline = Vec::new();
    pipeline.push(c);
    pipeline.push(s);
    for x in &pipeline {
        x.print_area();
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
        x.print_area();
        println!("{}",x.aurea(1));
        println!("{}",x.aurea(-1));
    }


    /*
    let mut x = Coord{first: 1., second: 2., third: 3., fourth: 4.};
    v[1](&mut x, true);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 2.0);
    v[0](&mut x, false);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 1.0);
    v[2](&mut x, true);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 2.0);

    let dms = DMS::new(60, 24, 36.);
    assert_eq!(dms.d, 60);
    assert_eq!(dms.m, 24);
    assert_eq!(dms.s, 36.);
    let d = dms.to_deg();
    assert_eq!(d, 60.41);
    */
}

fn pain() -> Box<dyn Fn(&mut Coord, bool) -> bool>  {
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
    let mut x = Coord{first: 1., second: 2., third: 3., fourth: 4.};
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



fn pulm() -> Box<dyn Fn(&mut Coord, bool) -> bool>  {
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
    let mut x = Coord{first: 1., second: 2., third: 3., fourth: 4.};
    hulm(&mut x, true);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 2.0);
    hulm(&mut x, false);
    println!("x:  {:?}", x);
    assert_eq!(x.first, 1.0);
    return hulm;
}
