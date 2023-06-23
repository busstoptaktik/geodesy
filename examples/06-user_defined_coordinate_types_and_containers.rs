use geodesy::prelude::*;
use log::{debug, trace};

fn main() -> Result<(), anyhow::Error> {
    // Filter by setting RUST_LOG to one of {Error, Warn, Info, Debug, Trace}
    if std::env::var("RUST_LOG").is_err() {
        simple_logger::init_with_level(log::Level::Error)?;
    } else {
        simple_logger::init_with_env()?;
    }

    debug!("debug message 1");
    trace!("trace message 1");

    // We use ::new() instead of ::default() in order to gain access to the
    // BUILTIN_ADAPTORS
    let mut ctx = geodesy::Minimal::new();
    trace!("trace message 2");
    debug!("debug message 2");

    let mut a = [0f64; 10];
    for (i, item) in [1f64, 2., 3.].into_iter().enumerate() {
        a[i] = item;
    }

    // let _oo = ctx.define_operation(&opt.operation)?;

    // A pipeline
    let pip =
        "geo:in | cart ellps:intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80 | geo:out";
    let pip = ctx.op(pip)?;

    let copenhagen = Coord::geo(55., 12., 0., 0.);
    let stockholm = Coord::geo(59., 18., 0., 0.);
    let mut data = [copenhagen, stockholm];
    for coord in data {
        println!("    {:?}", coord.to_geo());
    }

    ctx.apply(pip, Fwd, &mut data)?;
    for coord in &data {
        println!("    {:?}", coord.to_geo());
    }

    let a = Abscissa{the_abscissa: 3.0};
    let b: Abscissa = copenhagen.into();
    let c: Coord = b.into();
    let d: Abscissa = c.into();
    let e = a;

    let mut collection = AbscissaCollection{
        first_abscissa: a,
        second_abscissa: b,
        third_abscissa: d,
        fourth_abscissa: e
    };

    for index in 0..collection.len() {
        dbg!(collection[index]);
    }

    collection[2] = Abscissa{the_abscissa: 44.0};
    assert_eq!(collection[2].the_abscissa, 44.0);

    Ok(())
}



pub trait CoordinateItem: From<Coord> + Into<Coord> {}


#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone)]
struct Abscissa {
    the_abscissa: f64
}

impl From<Coord> for Abscissa {
    fn from(xyzt: Coord) -> Self {
        Abscissa{the_abscissa: xyzt[0]}
    }
}

impl Into<Coord> for Abscissa {
    fn into(self) -> Coord {
        Coord([self.the_abscissa, 0.0, 0.0, 0.0])
    }
}


pub trait AltCoordinateSet<T,C>: CoordinateMetadata + IndexMut<usize> {}


use std::ops::IndexMut;
use std::ops::Index;


#[derive(Debug, Default, PartialEq, PartialOrd, Clone)]
struct AbscissaCollection {
    first_abscissa: Abscissa,
    second_abscissa: Abscissa,
    third_abscissa: Abscissa,
    fourth_abscissa: Abscissa,
}

impl Index<usize> for AbscissaCollection {
    type Output = Abscissa;
    fn index(&self, i: usize) -> &Abscissa {
        match i {
            0 => &self.first_abscissa,
            1 => &self.second_abscissa,
            2 => &self.third_abscissa,
            3 => &self.fourth_abscissa,
            _ => panic!()
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
            _ => panic!()
        }
    }
}

impl CoordinateSet for AbscissaCollection {
    fn get_coord(&self, index: usize) -> Coord {
        self[index].into()
    }
    fn set_coord(&mut self, index: usize, value: &Coord) {
        self[index] = (*value).into();
    }
    fn len(&self) -> usize {
        4
    }

}
