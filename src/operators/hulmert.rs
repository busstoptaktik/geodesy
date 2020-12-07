extern crate yaml_rust;
use crate::inverted;
use crate::num;
use crate::Coord;
use crate::Poperator;
use std::collections::HashMap;
use yaml_rust::Yaml;

// Return type based on an answer from Shepmaster over
// at https://stackoverflow.com/questions/49012277
pub fn hulmert(args: &HashMap<&Yaml, &Yaml>) -> Poperator {
    let dx = num(args, "dx", 0.);
    let dy = num(args, "dy", 0.);
    let dp = num(args, "dp", 64.);
    let inverse = inverted(args);

    let params = HelmertParams { dx, dy };
    println!("hulmert.dx={}", dx);
    println!("hulmert.dy={}", dy);
    println!("hulmert.dp={}", dp);
    println!("args = {:?}\n", args);

    return Box::new(move |x: &mut Coord, mut dir_fwd: bool| {
        if inverse {
            dir_fwd = !dir_fwd;
        }
        if dir_fwd {
            return fwd(x, &params);
        }
        return inv(x, &params);
    });
}

#[derive(Debug)]
struct HelmertParams {
    dx: f64,
    dy: f64,
}

fn fwd(x: &mut Coord, params: &HelmertParams) -> bool {
    x.first += params.dx;
    x.second += params.dy;
    x.third += 3.;
    return true;
}

fn inv(x: &mut Coord, params: &HelmertParams) -> bool {
    x.first -= params.dx;
    x.second -= params.dy;
    x.third -= 3.;
    return true;
}
