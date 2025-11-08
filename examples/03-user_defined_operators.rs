// examples/03-user_defined_operators.rs

// See also 00-transformations.rs
// Run with:
// cargo run --example 03-user_defined_operators

// In this example we implement a user defined operator. To that end
// we need to access some of the lower level features of the Rust
// Geodesy library. Since they are mostly for library-internal use,
// they are wrapped up in this dedicated module
use geodesy::authoring::*;

// The functionality of the operator is straightforward: It simply
// adds 42 to the first element of any coordinate tuple thrown at it.
// It also implements the inverse operation, i.e. subtracting 42.

// Forward
fn add42(_op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let n = operands.len();
    for i in 0..n {
        let mut coord = operands.get_coord(i);
        coord[0] += 42.;
        operands.set_coord(i, &coord);
    }
    n
}

// Inverse
fn sub42(_op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let n = operands.len();
    for i in 0..n {
        let mut coord = operands.get_coord(i);
        coord[0] -= 42.;
        operands.set_coord(i, &coord);
    }
    n
}

// These are the parameters our 'add42'-operator are willing to respond to
pub const GAMUT: [OpParameter; 1] = [OpParameter::Flag { key: "inv" }];

// And this is the constructor, generating the object, the `Context` needs to instantiate an actual instance
pub fn add42_constructor(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    Op::basic(parameters, InnerOp(add42), Some(InnerOp(sub42)), &GAMUT)
}

fn main() -> anyhow::Result<()> {
    let mut prv = geodesy::prelude::Minimal::new();
    prv.register_op("add42", OpConstructor(add42_constructor));
    let add42 = prv.op("add42")?;

    // Same test coordinates as in example 00, but no conversion to radians.
    let cph = Coor2D::raw(12., 55.); // Copenhagen
    let osl = Coor2D::raw(10., 60.); // Oslo
    let sth = Coor2D::raw(59., 18.); // Stockholm
    let hel = Coor2D::raw(60., 25.); // Helsinki

    let mut data = [osl, cph, sth, hel];

    for coord in data {
        println!("    {:?}", coord);
    }

    // Now do the transformation
    assert_eq!(prv.apply(add42, Fwd, &mut data)?, 4);
    println!("add42 (fwd):");
    for coord in data {
        println!("    {:?}", coord);
    }

    // And go back...
    assert_eq!(prv.apply(add42, Inv, &mut data)?, 4);
    println!("add42 (inv):");
    for coord in data {
        println!("    {:?}", coord);
    }
    Ok(())
}
