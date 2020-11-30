extern crate yaml_rust;
use yaml_rust::Yaml;
use std::collections::HashMap;
use crate::num;
use crate::inverted;
use crate::Coord;
use crate::Operator;

/*





*/

pub fn helmert(args: &HashMap<&Yaml,&Yaml>) -> Operator {
    let dx = num(args, "dx", 0.);
    let dy = num(args, "dy", 0.);
    let dz = num(args, "dz", 0.);
    let dp = num(args, "dp", 64.);
    let inverse = inverted(args);

    let params = HelmertParams{dx, dy, dz};
    println!("helmert.dx={}", dx);
    println!("helmert.dy={}", dy);
    println!("helmert.dz={}", dz);
    println!("args = {:?}\n", args);

    return Box::new(move |x: &mut Coord, mut dir_fwd: bool| {
        if inverse {
            dir_fwd = !dir_fwd;
        }
        if dir_fwd {
            return fwd(x, &params);
        }
        return inv(x, &params);
    })
}

#[derive(Debug)]
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


fn inv(x: &mut Coord, params: &HelmertParams) -> bool {
    x.first -= params.dx;
    x.second -= params.dy;
    x.third -= params.dz;
    return true;
}

mod tests {
    use super::*;

    #[test]
    fn helmert() {
        let mut x = Coord{first: 1., second: 2., third: 3., fourth: 4.};
        let params = HelmertParams{dx: 1., dy: 2., dz: 3.};
            fwd(&mut x, &params);
            assert_eq!(x.first, 2.);

            inv(&mut x, &params);
            assert_eq!(x.first, 1.);
            assert_eq!(x.second, 2.);
            assert_eq!(x.third, 3.);
    }
}
