// use core::slice::SlicePattern;

use slice_of_array::prelude::*;

//use geodesy::CoordinateTuple;
// use geodesy::Gas;


trait CoordinatePrimitives {
    fn new(x: f64, y: f64, z: f64, t: f64) -> Coo ;
    fn shift(cc: &mut [Coo]);
    fn deg(x: f64, y: f64, z: f64, t: f64) -> Coo ;
    fn to_degrees(self) -> Coo;
    fn to_radians(self) -> Coo;
}

type Coo = [f64;4];
impl CoordinatePrimitives for Coo {
    fn new(x: f64, y: f64, z: f64, t: f64) -> Coo {
        [x, y, z, t]
    }

    fn shift(cc: &mut [Coo]) {
        for c in cc {
            c[1] += 33.;
        }
    }

    #[must_use]
    fn deg(x: f64, y: f64, z: f64, t: f64) -> Coo {
        Coo::new(x.to_radians(),  y.to_radians(), z, t)
    }

    #[must_use]
    fn to_degrees(self) -> Coo {
        Coo::new(self[0].to_degrees(), self[1].to_degrees(), self[2], self[3])
    }

    #[must_use]
    fn to_radians(self) -> Coo {
        Coo::new(self[0].to_radians(), self[1].to_radians(), self[2], self[3])
    }

}

#[allow(non_snake_case)]
fn Coo(x: f64, y: f64, z: f64, t: f64) -> Coo {
 [x, y, z, t]
}

fn shift(cc: &mut [Coo]) {
    for c in cc {
        c[1] += 33.;
    }
}

fn pille(cc: &mut [f64]) {
    for c in cc {
        *c += 3.;
    }
}



fn main() {
    let y = Coo(1., 2., 3., 4.);
    let x = Coo::new(1., 2., 3., 4.);
    println!("{:?}", Coo(8.5, 55.00, 0., 0.));
    assert_eq!(x[1], 2.);
    assert_eq!(x, y);

    let mut bok = [x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,];
    let mut vok = vec![x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,x,y,];
    //let b: Vec<f64> = bok.iter().flatten();
    //let bok = &mut bok[..];
    let golf = vok.flat_mut();
    pille(golf);
    println!("vok {:?}", vok);

    shift(&mut bok);
    shift(&mut vok);

    let b = bok.flat();
    println!("flat {:?}", b);
    println!("bok {:?}", bok);
    println!("boklen {:?}", bok.len());

    let b = bok.iter().flatten().collect::<Vec<_>>();
    println!("flatten {:?}", b);

    //println!("Hello from kp!");
    //let g = Gas::new("tests/geo.gas").unwrap();
    //println!("{:?}", g);
    //println!("{:?}", g.value(CoordinateTuple(8.5, 55.00, 0., 0.)));
}
