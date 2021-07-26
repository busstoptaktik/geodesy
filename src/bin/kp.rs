use slice_of_array::prelude::*;

use geodesy::CoordinateTuple;
use geodesy::CoordinatePrimitives;

fn forty_two(cc: &mut CoordinateTuple) {
    cc[1] = 42.;
}

// fn pille(cc: &mut [f64]) {
//     for c in cc {
//         *c += 3.;
//     }
// }


pub struct FlatIndexer {
    offset: [usize; 4],
    stride: [usize; 4],
    channels: usize,
}

impl FlatIndexer {
    pub fn new(offset: [usize; 4], stride: [usize; 4], channels: usize) -> FlatIndexer {
        FlatIndexer{offset, stride, channels}
    }

    pub fn get(&self, index: usize, data: &mut [f64]) -> CoordinateTuple {
        let mut element = CoordinateTuple::nan();
        for i in 0..self.channels {
            element[i] = data[self.offset[i] + index*self.stride[i]];
        }
        element
    }

    pub fn set(&self, index: usize, data: &mut [f64], result: CoordinateTuple) {
        for i in 0..self.channels {
            data[self.offset[i] + index*self.stride[i]] = result[i];
        }
    }
}



fn main() {
    let y = CoordinateTuple::new(1., 2., 3., 4.);
    let x = CoordinateTuple::new(1., 2., 3., 4.);
    assert_eq!(x[1], 2.);
    assert_eq!(x, y);

    // `FlatIndexer` for a slice of `CoordinateTuple`s
    let f = FlatIndexer::new([0,1,2,3], [4,4,4,4], 4);
    let z = CoordinateTuple::new(0.,0.,0.,0.);
    let mut data = [z,z,z,z,z,z];
    let n_data = data.len();
    let flat = data.flat_mut();
    println!("n_data: {}, n_flat: {}", n_data, flat.len());

    for i in 0..n_data {
        let mut e = f.get(i, flat);
        e[0] = i as f64;
        forty_two(&mut e);
        f.set(i, flat, e);
    }

    for i in 0..data.len() {
        println!("woohoo: {:?}", data[i]);
    }
}
