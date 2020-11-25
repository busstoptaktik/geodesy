extern crate yaml_rust;
use yaml_rust::Yaml;
use std::collections::HashMap;
use crate::num;
use crate::inverted;
use crate::Coord;


pub fn helmert(args: &HashMap<&Yaml,&Yaml>)  -> impl Fn(&mut Coord, bool) -> &mut Coord {
    let dx = num(args, "dx", 0.);
    let dy = num(args, "dy", 0.);
    let dz = num(args, "dz", 0.);
    let dp = num(args, "dp", 64.);
    let inverse = inverted(args);

    println!("helmert.dx={}", dx);
    println!("helmert.dy={}", dy);
    println!("helmert.dz={}", dz);
    println!("helmert.dp={}", dp);
    println!("args = {:?}\n", args);

    return move |x: &mut Coord, mut fwd: bool| {
        if inverse {
            fwd = !fwd;
        }
        if fwd {
            x.first += dx;
            x.second += dy;
            x.third += dz;
        }
        else {
            x.first -= dx;
            x.second -= dy;
            x.third -= dz;
        }
        x
    }
}
