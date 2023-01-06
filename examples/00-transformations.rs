// examples/00-transformations.rs

// Using Rust Geodesy to transform geodata.
// Run with:
// cargo run --example 00-transformations

use geodesy::preamble::*;

// Use Anyhow for convenient error handling
fn main() -> anyhow::Result<()> {
    // The context provider is the entry point to all transformation functionality:
    let mut ctx = Minimal::default();
    // The concept of a "context data structure" will be well known to
    // PROJ users, where the context plays a somewhat free-flowing role,
    // and only becomes truly visible in multithreaded cases.
    // In Rust Geodesy, the context plays a much more visible role, as
    // most transformation functionality is implemented directly as
    // methods of the context data structure.

    // We need some coordinates to test the code. The convenience methods
    // `gis` and `geo` produce 4D coordinate tuples and automatically handle
    // conversion of the angular parts to radians, and geographical coordinates
    // in latitude/longitude order, to the GIS convention of longitude/latitude.

    // Here using the GIS convention - longitude before latitude
    let cph = Coord::gis(12., 55., 0., 0.); // Copenhagen
    let osl = Coord::gis(10., 60., 0., 0.); // Oslo

    // And here using the geodesy/navigation convention - latitude before longitude
    let sth = Coord::geo(59., 18., 0., 0.); // Stockholm
    let hel = Coord::geo(60., 25., 0., 0.); // Helsinki

    // `gis` and `geo` have a sibling `raw` which generates coordinate tuples
    // from raw numbers, in case your point coordinates are already given in
    // radians.
    let cph_raw = Coord::raw(12_f64.to_radians(), 55_f64.to_radians(), 0., 0.0);
    // But since a coordinate tuple is really just an array of 4 double
    // precision numbers, you may also generate it directly using plain
    // Rust syntax.
    let cph_direct = Coord([12_f64.to_radians(), 55_f64.to_radians(), 0., 0.0]);
    // The three versions of Copenhagen coordinates should be identical.
    assert_eq!(cph, cph_raw);
    assert_eq!(cph, cph_direct);

    // The Rust Geodesy interface is based on transformation of *arrays* of
    // coordinate tuples, rather than single points. So let's make an array:
    let mut data = [osl, cph, sth, hel];
    // Since all operations are carried out in place, the array needs to
    // be mutable, hence `let mut`

    // Let's obtain a handle to a transformation element ("an operation"),
    // turning geographical coordinates into UTM zone 32 coordinates.
    // Since this may go wrong (e.g. due to syntax errors in the operator
    // definition), use the Rust `?`-operator to handle errors.
    let utm32 = ctx.op("utm zone=32")?;
    // Now, let's use the utm32-operator to transform some coordinate data.
    // The data are transformed in place, so we pass them by mutable reference.
    // The `Fwd` constant indicates that the operation runs in the forward
    // direction
    ctx.apply(utm32, Fwd, &mut data)?;
    println!("utm32:");
    for coord in data {
        println!("    {:?}", coord);
    }

    // Specifying `Inv` rather than `Fwd` takes us on the inverse trip back
    // to geographic coordinates
    ctx.apply(utm32, Inv, &mut data)?;
    println!("Roundtrip to geo:");
    for coord in data {
        // Note the use of `to_geo`, which transforms lon/lat in radians
        // to lat/lon in degrees
        println!("    {:?}", Coord::to_geo(coord));
    }

    // To get rid of roundtrip-roundoff noise, let's make a fresh
    // version of the input data for the next example:
    let mut data = [osl, cph, sth, hel];

    // Now a slightly more complex case: Transforming the coordinates,
    // which we consider given in WGS84, back to the older ED50 datum.
    // The EPSG:1134 method handles that through a 3 parameter Helmert
    // transformation. But since the Helmert transformation works on
    // cartesian coordinates, rather than geographic, we need to add
    // pre- and post-processing steps, taking us from geographical
    // coordinates to cartesian, and back. Hence, we need a pipeline
    // of 3 steps:
    let pipeline = "cart ellps=intl | helmert x=-87 y=-96 z=-120 | cart inv=true ellps=GRS80";
    let ed50_wgs84 = ctx.op(pipeline)?;

    // Since the forward transformation goes *from* ed50 to wgs84, we use
    // the inverse method to take us the other way, back in time to ED50
    ctx.apply(ed50_wgs84, Inv, &mut data)?;
    println!("ed50:");
    for coord in data {
        // Again, use the `Coord::to_geo` method to get output following the
        // geodesy/navigation convention for angular unit/coordinate order
        println!("    {:?}", Coord::to_geo(coord));
    }

    // Finally an example of handling bad syntax:
    println!("Bad syntax example:");
    let op = ctx.op("aargh zone=23");
    if op.is_err() {
        println!("Deliberate error - {:?}", op);
    }

    // ------------------------------------------------------------------------------
    // Don't do this!
    // ------------------------------------------------------------------------------
    // As PROJ-conoisseurs know, the `Context` type in PROJ plays a somewhat more
    // passive role than the `Context` type in RG. But actually, the RG Context API
    // is built on top of a much more PROJ-like interface, where the operators take
    // center stage. In general, the use of this interface is *not* recommended, but
    // for the rare cases where it is preferable, we demonstrate its use below, by
    // repeating the exercises above, while swapping the roles of the `Op`/`OpHandle`
    // and the `Context`.

    // Create an `Op`, turning geographical coordinates into UTM zone 32 coordinates
    let utm32 = Op::new("utm zone=32", &ctx)?;
    // Now, let's use the utm32-operator to transform some data
    utm32.apply(&ctx, &mut data, Fwd)?;

    println!("utm32:");
    for coord in data {
        println!("    {:?}", coord);
    }

    // Take the inverse road back to geographic coordinates
    utm32.apply(&ctx, &mut data, Inv)?;

    println!("Roundtrip to geo:");
    for coord in data {
        println!("    {:?}", Coord::to_geo(coord));
    }

    // Make a fresh version of the input data for the next example:
    let mut data = [osl, cph, sth, hel];

    // EPSG:1134
    let ed50_wgs84 = Op::new(pipeline, &ctx)?;
    ed50_wgs84.apply(&ctx, &mut data, Inv)?;
    println!("ed50:");
    for coord in data {
        println!("    {:?}", Coord::to_geo(coord));
    }

    // Handling bad syntax:
    println!("Bad syntax example:");
    let op = Op::new("aargh zone=23", &ctx);
    //let op = ctx.define_operation("aargh zone: 23");
    if op.is_err() {
        println!("Deliberate error - {:?}", op);
    }

    Ok(())
}
