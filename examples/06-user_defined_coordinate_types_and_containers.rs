use geodesy::prelude::*;
use log::{debug, trace};

// This is out one-dimensional user defined coordinate type "Abscissa"
#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone)]
struct Abscissa(f64);

// From<Coord> and Into<Coord> are useful when implementing
// the CoordinateSet trait below
impl From<Coor4D> for Abscissa {
    fn from(xyzt: Coor4D) -> Self {
        Abscissa(xyzt[0])
    }
}

// This gives us Into<Coord> for Abscissa for free
impl From<Abscissa> for Coor4D {
    fn from(a: Abscissa) -> Self {
        Coor4D([a.0, 0.0, 0.0, 0.0])
    }
}

// The AbscissaCollection type contains 4 Abscissae, indexed in an
// utterly idiotic way. It is implemented like this only in order
// to show that the CoordinateSet may be implemented for even
// utterly odd collection types
#[derive(Debug, Default, PartialEq, PartialOrd, Clone)]
struct AbscissaCollection {
    first_abscissa: Abscissa,
    second_abscissa: Abscissa,
    third_abscissa: Abscissa,
    fourth_abscissa: Abscissa,
}

// We simplify the implementation of the CoordinateSet trait
// by implementing the Index and IndexMut traits
use std::ops::Index;
use std::ops::IndexMut;

impl Index<usize> for AbscissaCollection {
    type Output = Abscissa;
    fn index(&self, i: usize) -> &Abscissa {
        match i {
            0 => &self.first_abscissa,
            1 => &self.second_abscissa,
            2 => &self.third_abscissa,
            3 => &self.fourth_abscissa,
            _ => panic!(),
        }
    }
}

impl IndexMut<usize> for AbscissaCollection {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.first_abscissa,
            1 => &mut self.second_abscissa,
            2 => &mut self.third_abscissa,
            3 => &mut self.fourth_abscissa,
            _ => panic!(),
        }
    }
}

// Having the Index & IndexMut traits implemented for AbscissaCollection
// and the From<Coord> and Into<Coord> implemented for Abscissa, it is
// next to trivial to implement the CoordinateSet trait
impl CoordinateSet for AbscissaCollection {
    fn get_coord(&self, index: usize) -> Coor4D {
        self[index].into()
    }
    fn set_coord(&mut self, index: usize, value: &Coor4D) {
        self[index] = (*value).into();
    }
    fn len(&self) -> usize {
        4
    }
    fn dim(&self) -> usize {
        1
    }
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    debug!("User defined coordinate types and containers");

    // We use ::new() instead of ::default() in order to gain access to the
    // BUILTIN_ADAPTORS (geo:in, geo:out etc.)
    let mut ctx = geodesy::Minimal::new();
    trace!("have context");

    let copenhagen = Coor4D([55., 12., 0., 0.]);

    let a = Abscissa(3.0);
    let b: Abscissa = copenhagen.into(); // Coord to Abscissa
    let c: Coor4D = b.into(); // Abscissa to Coord
    let d: Abscissa = c.into(); // ... and back to Abscissa
    let e = a;

    let mut collection = AbscissaCollection {
        first_abscissa: a,
        second_abscissa: b,
        third_abscissa: d,
        fourth_abscissa: e,
    };

    for index in 0..collection.len() {
        println!("{:?}", collection[index]);
    }

    collection[2] = Abscissa(44.0);
    assert_eq!(collection[2].0, 44.0);

    trace!("Instantiating 'addone' operator");
    let add_one = ctx.op("addone")?;

    trace!("Adding one");
    ctx.apply(add_one, Fwd, &mut collection)?;

    for i in 0..collection.len() {
        println!("{:?}", collection[i]);
    }
    assert_eq!(collection[2].0, 45.0);

    Ok(())
}
