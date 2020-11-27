extern crate yaml_rust;
use yaml_rust::Yaml;
use std::collections::HashMap;



// Næste skridt:
// HashMap<String, String> til defaults/definitioner for operationer
// Plain string til definitioner
// implementer pipeline (steps)
// operationer skal ikke operere på en Coord, men på en OpArg
// Operationer skal returnere en struct med lidt metadata OG closure

pub mod foundations;

pub mod operators;
pub type Operation = Box<dyn Fn(&mut Coord, bool) -> bool>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


fn num(args: &HashMap<&yaml_rust::Yaml,&yaml_rust::Yaml>, key: &str, default: f64) -> f64 {
    let k = Yaml::from_str(key);
    let arg = args.get(&k);
    match arg {
        Some(arg) => arg,
        None => return default
    };

    let val = arg.unwrap();
    if let Yaml::Integer(value) = val {
        return *value as f64;
    }
    return val.as_f64().unwrap_or(default);
}


fn inverted(args: &HashMap<&yaml_rust::Yaml,&yaml_rust::Yaml>) -> bool {
    let k = Yaml::from_str("dir");
    let arg = args.get(&k);
    if let Some(val) = arg {
        if val.as_str().unwrap_or("fwd") == "inv" {
            return true;
        }
    }
    return false;
}


#[derive(Debug)]
pub struct Coord {
    pub first: f64,
    pub second: f64,
    pub third: f64,
    pub fourth: f64,
}
