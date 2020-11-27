extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};
use std::collections::HashMap;
use geodesy::operators::helmert::helmert;
use geodesy::operators::hulmert::hulmert;
use geodesy::operators::Coord;
use geodesy::operators::Operation;
use geodesy::foundations::DMS;

fn main() {
    let helm = pain();
    let hulm = pulm();
    let mut v: Vec<Operation> = Vec::new();
    v.push(hulm);
    v.push(helm);
    v.push(pulm());

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
