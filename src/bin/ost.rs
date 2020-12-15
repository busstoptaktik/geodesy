extern crate yaml_rust;
use geodesy::operator_factory;
use geodesy::OperatorArgs;
use geodesy::OperatorWorkSpace;
use geodesy::Pipeline;
use yaml_rust::{Yaml, YamlLoader};

fn generic_experiment() -> Pipeline {
    // Se https://docs.rs/yaml-rust/0.4.4/yaml_rust/yaml/enum.Yaml.html
    let mut pap = OperatorArgs::new();
    let txt = std::fs::read_to_string("src/transformations.yml").unwrap();
    let docs = YamlLoader::load_from_str(&txt).unwrap();
    //println!("OOOOOOOOOOOOOOOOOOOOooooooo {}", docs[0]["main"].as_hash().unwrap().iter().len());
    let steps = docs[0]["recipe"]["steps"].as_vec().unwrap();
    for _s in steps {
        //println!("OOOOOOOOOOOOOOOOOOOOooooooo {:#?}", _s);
    }
    let globals = docs[0]["recipe"]["globals"].as_hash().unwrap();
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

    pipeline
}

fn main() {
    let mut global_globals = OperatorArgs::new();
    global_globals.insert("ellps", "GRS80");

    println!("Global_globals: {:?}", global_globals);
    let (_steps, globals) = geodesy::steps_and_globals("recipe");
    println!("Globals: {:?}", globals);
    global_globals.append(&globals);
    println!("Global_globals: {:?}", global_globals);

    let pipeline = generic_experiment();
    println!("MAIN*****************************");
    for x in &pipeline {
        println!("{}", x.name());
    }
}
