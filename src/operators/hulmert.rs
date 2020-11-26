extern crate yaml_rust;
use yaml_rust::Yaml;
use std::collections::HashMap;
use crate::num;
use crate::inverted;
use crate::Coord;


pub fn hulmert(args: &HashMap<&Yaml,&Yaml>)  -> impl Fn(&mut Coord, bool) -> bool {
    let dx = num(args, "dx", 0.);
    let dy = num(args, "dy", 0.);
    let dz = num(args, "dz", 0.);
    let dp = num(args, "dp", 64.);
    let inverse = inverted(args);

    let params = HelmertParams{dx, dy, dz};
    println!("hulmert.dx={}", dx);
    println!("hulmert.dy={}", dy);
    println!("hulmert.dz={}", dz);
    println!("hulmert.dp={}", dp);
    println!("args = {:?}\n", args);

    return move |x: &mut Coord, mut dir_fwd: bool| {
        if inverse {
            dir_fwd = !dir_fwd;
        }
        if dir_fwd {
            return fwd(x, &params);
        }
        return inv(x, &params);
    }
}

// #[derive(Debug)]
struct HelmertParams {
    dx: f64,
    dy: f64,
}


fn fwd(x: &mut Coord, params: &HelmertParams) -> bool {
    x.first += params.dx;
    x.second += params.dy;
    x.third += 3;
    return true;
}


fn inv(x: &mut Coord, params: &HelmertParams) -> bool {
    x.first -= params.dx;
    x.second -= params.dy;
    x.third -= 3;
    return true;
}
