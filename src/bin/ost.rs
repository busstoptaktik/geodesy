// use geodesy::Operand;
// use geodesy::Operator;
// use geodesy::OperatorArgs;
// use geodesy::OperatorCore;
// use yaml_rust::{Yaml, YamlLoader};
use minimal_yaml::{Yaml, parse};
/*
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;


use inline_python::python;

fn inline_main() {
    let who = "world";
    let n = 5;
    python! {
        for i in range('n):
            print(i, "Hello", 'who)
        print("Goodbye")
    }
}


fn salat() -> Result<(), ()> {
    Python::with_gil(|py| {
        salat_(py).map_err(|e| {
          // We can't display Python exceptions via std::fmt::Display,
          // so print the error here manually.
          e.print_and_set_sys_last_vars(py);
        })
    })
}

fn salat_(py: Python) -> PyResult<()> {
    let sys = py.import("sys")?;
    let version: String = sys.get("version")?.extract()?;
    let locals = [("os", py.import("os")?)].into_py_dict(py);
    let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
    let user: String = py.eval(code, None, Some(&locals))?.extract()?;
    println!("Hello {}, I'm Python {}", user, version);
    Ok(())
}
*/

fn main() {

    // Se https://docs.rs/yaml-rust/0.4.4/yaml_rust/yaml/enum.Yaml.html
    // let mut pap = OperatorArgs::new();
    let txt = std::fs::read_to_string("src/simple.yml").unwrap();
    //let docs = YamlLoader::load_from_str(&txt).unwrap();
    println!("{}", txt);
    let rep = parse(&txt).unwrap();
    println!("{:?}", rep);

    /*
    for dd in &rep {
        println!("dd: {:?}", dd);
    }
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
