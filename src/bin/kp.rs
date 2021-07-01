// use geodesy::Operand;
// use geodesy::Operator;
// use geodesy::OperatorArgs;
// use geodesy::OperatorCore;
// use yaml_rust::{Yaml, YamlLoader};

fn main() {
    println!("Hello from kp!")
    /*
    // Se https://docs.rs/yaml-rust/0.4.4/yaml_rust/yaml/enum.Yaml.html
    // let mut pap = OperatorArgs::new();
    let txt = std::fs::read_to_string("src/simple.yml").unwrap();
    //let docs = YamlLoader::load_from_str(&txt).unwrap();
    println!("{}", txt);
    let rep = parse(&txt).unwrap();
    match &rep {
        Yaml::Scalar(value) => println!("Aaaaargh!"),
        Yaml::Mapping(map) => {
            for entry in map.iter() {
                println!("{}: {:?}", entry.key, entry.value)
            }
        }
        Yaml::Sequence(ref seq) => println!("Aaaaargh!"),
    }
    println!("{:?}", rep);

    // let dd = rep.get(0);
    // println!("dd: {:?}", dd);
*/
    /*
    let steps = docs["recipe"]["steps"].as_vec().unwrap();
    for _s in steps {
        println!("OOOOOOOOOOOOOOOOOOOOooooooo {:#?}", _s);
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
    let c = Operator::new("cart: {}").unwrap();
    let h = Operator::new("helmert: {dx: 1, dy: 2, dz: 3}").unwrap();
    let mut o = Operand::new();

    c.fwd(&mut o);
    println!("{:?}", o);
    c.inv(&mut o);
    println!("{:?}", o);

    println!("{}", c.name());
    println!("{}", h.name());
    */
}
