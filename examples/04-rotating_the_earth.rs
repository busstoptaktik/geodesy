// examples/04-rotating_the_earth

// See also 03-user_defined_operators.rs
// Run with:
// cargo run --example 04-rotating_the_earth

// In this example we implement a user defined operator. In contrast
// to the previous example in 03-user_defined_operators.rs, this one
// actually carries out something marginally geodetically useful: It
// formally shifts the geodetic reference frame, such that a certain
// surface point `A` ends where another point `B` used to be. And it
// does so using the **distance and bearing** between A and B, so
// while it may sound somewhat like a Helmert transformation, it really
// isnt: This operator moves all points the same amount, and in the same
// direction, **as measured on the *surface* of the earth**, i.e. taking
// care of variations in local ellipsoidal curvature. The Helmert
// transformation on the other hand, works in cartesian coordinates, and
// generally moves points on the earth's surface away from the surface
// when the system rotates.
//
// The technique implemented here is not in general geodetic use,
// so consider it more of a geodetic pun in the pursuit of a reasonable
// answer to the oft-occurring question "If we shift Copenhagen to Vienna,
// where would Helsinki end up?".
//
// Also note that since the return-bearing depends on the destination,
// this operator is **not** directly invertible (although an iterative
// solution is feasible)

use geodesy::operator_construction::*;
use geodesy::{Context, CoordinateTuple, Ellipsoid};

pub struct GeodesicShift {
    args: OperatorArgs,
    inverted: bool,

    ellps: Ellipsoid,

    bearing: f64,
    distance: f64,
}

impl GeodesicShift {
    fn new(args: &mut OperatorArgs) -> Result<GeodesicShift, &'static str> {
        let ellps = Ellipsoid::named(&args.value("ellps", "GRS80"));
        let inverted = args.flag("inv");

        // Coordinate of the origin
        let lat_0 = args.numeric_value("lat_0", std::f64::NAN)?;
        let lon_0 = args.numeric_value("lon_0", std::f64::NAN)?;

        // Coordinate of the target
        let lat_1 = args.numeric_value("lat_1", std::f64::NAN)?;
        let lon_1 = args.numeric_value("lon_1", std::f64::NAN)?;

        if [lat_0, lon_0, lat_1, lon_1].iter().any(|&f| f.is_nan()) {
            return Err("Missing lat_0, lon_0, lat_1 or lon_1");
        }

        // Now find the distance and bearing between the origin and the target
        let origin = CoordinateTuple::geo(lat_0, lon_0, 0., 0.);
        let target = CoordinateTuple::geo(lat_1, lon_1, 0., 0.);

        let d = ellps.geodesic_inv(&origin, &target);
        let bearing = d[0];
        let distance = d[2];

        Ok(GeodesicShift {
            args: args.clone(),
            ellps,
            inverted,
            bearing,
            distance,
        })
    }

    // This is the interface to the Rust Geodesy library: Construct a
    // GeodesicShift element, and wrap it properly for consumption.
    pub fn operator(args: &mut OperatorArgs, _ctx: &mut Context) -> Result<Operator, &'static str> {
        let op = GeodesicShift::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for GeodesicShift {
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        for coord in operands {
            let res = self.ellps.geodesic_fwd(&coord, self.bearing, self.distance);
            coord[0] = res[0];
            coord[1] = res[1];
        }
        true
    }

    // This operator is not invertible (because the return azimuth depends
    // on the destination), so we implement `invertible()` as false, let the
    // empty default implementation from the trait take care of `inv()`, and
    // leave it for a rainy day to implement an iterative inverse solution.
    fn invertible(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "geodesic_shift"
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
    ctx.register_operator("geodesic_shift", GeodesicShift::operator);
    let op = "geodesic_shift: {lat_0: 55, lon_0: 12, lat_1: 48, lon_1: 16.}";

    let cph_to_vie = match ctx.operator(op) {
        Some(value) => value,
        None => {
            println!("Awful!");
            return;
        }
    };

    // Same test coordinates as in example 00, but no conversion to radians.
    let cph = CoordinateTuple::geo(55., 12., 0., 0.); // Copenhagen
    let osl = CoordinateTuple::geo(60., 10., 0., 0.); // Oslo
    let sth = CoordinateTuple::geo(59., 18., 0., 0.); // Stockholm
    let hel = CoordinateTuple::geo(60., 25., 0., 0.); // Helsinki

    let mut data = [osl, cph, sth, hel];

    // Now do the transformation
    ctx.fwd(cph_to_vie, &mut data);
    println!("cph_to_vie (fwd):");
    let mut result = data.clone();
    CoordinateTuple::geo_all(&mut result);
    for coord in result {
        println!("    {:?}", coord);
    }

    // And assert there is no way back...
    assert_eq!(false, ctx.inv(cph_to_vie, &mut data));
}
