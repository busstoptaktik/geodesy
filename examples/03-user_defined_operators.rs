// examples/02-user_defined_macros.rs

// See also 00-transformations.rs
// Run with:
// cargo run --example 03-user_defined_operators

// In this example we implement a user defined operator. To that end
// we need to access some of the lower level features of the Rust
// Geodesy library: The Operator type, its definition argument type,
// OperatorArgs, and its core trait, OperatorCore. Since they are
// mostly for library-internal use, they are wrapped up in the dedicated
// module `operator_construction`.
use geodesy::{operator_construction::*, GeodesyError};
use geodesy::{Context, CoordinateTuple};

// The functionality of the operator is straightforward: It simply
// adds 42 to the first element of any coordinate tuple thrown at it.
// It also implements the inverse operation, i.e. subtracting 42.

pub struct Add42 {
    args: OperatorArgs,
    inverted: bool,
}

impl Add42 {
    fn new(args: &mut OperatorArgs) -> Result<Add42, GeodesyError> {
        let inverted = args.flag("inv");
        Ok(Add42 {
            args: args.clone(),
            inverted,
        })
    }

    // This is the interface to the Rust Geodesy library: Construct an Add42
    // element, and wrap it properly for consumption. It is 100% boilerplate.
    pub fn operator(args: &mut OperatorArgs, _ctx: &mut Context) -> Result<Operator, GeodesyError> {
        let op = Add42::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for Add42 {
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            coord[0] += 42.;
        }
        true
    }

    fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            coord[0] -= 42.;
        }
        true
    }

    fn name(&self) -> &'static str {
        "add42"
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }

    fn args(&self, _step: usize) -> &OperatorArgs {
        &self.args
    }
}

fn main() {
    let mut ctx = geodesy::Context::new();
    ctx.register_operator("add42", Add42::operator);

    let add42 = match ctx.operation("add42: {}") {
        Some(value) => value,
        None => {
            println!("Awful!");
            return;
        }
    };

    // Same test coordinates as in example 00, but no conversion to radians.
    let cph = CoordinateTuple::raw(12., 55., 0., 0.); // Copenhagen
    let osl = CoordinateTuple::raw(10., 60., 0., 0.); // Oslo
    let sth = CoordinateTuple::raw(59., 18., 0., 0.); // Stockholm
    let hel = CoordinateTuple::raw(60., 25., 0., 0.); // Helsinki

    let mut data = [osl, cph, sth, hel];

    for coord in data {
        println!("    {:?}", coord);
    }

    // Now do the transformation
    ctx.fwd(add42, &mut data);
    println!("add42 (fwd):");
    for coord in data {
        println!("    {:?}", coord);
    }

    // And go back...
    ctx.inv(add42, &mut data);
    println!("add42 (inv):");
    for coord in data {
        println!("    {:?}", coord);
    }
}
