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

    let params = HelmertParams{dx, dy, dz};
    println!("helmert.dx={}", dx);
    println!("helmert.dy={}", dy);
    println!("helmert.dz={}", dz);
    println!("helmert.dp={}", dp);
    println!("args = {:?}\n", args);

    return move |x: &mut Coord, mut dir_fwd: bool| {
        if inverse {
            dir_fwd = !dir_fwd;
        }
        if dir_fwd {
            fwd(x, &params);
            return x
        }
        return inv(x, dx, dy, dz);
    }
}

// #[derive(Debug)]
struct HelmertParams {
    dx: f64,
    dy: f64,
    dz: f64
}


fn fwd(x: &mut Coord, params: &HelmertParams) -> bool {
    x.first += params.dx;
    x.second += params.dy;
    x.third += params.dz;
    return true;
}


fn inv(x: &mut Coord, dx: f64, dy: f64, dz: f64) -> &mut Coord {
    x.first -= dx;
    x.second -= dy;
    x.third -= dz;
    return x;
}
