// examples/00-transformations.rs

// Using Rust Geodesy to transform geodata.
// Run with:
// cargo run --example 00-transformations

fn main() {
    // The CoordinateTuple type is much used, so
    // we give it a very brief abbreviation
    use geodesy::CoordinateTuple as C;

    // The context is the entry point to all transformation functionality:
    let mut ctx = geodesy::Context::new();
    // The concept of a "context data structure" will be well known to
    // PROJ users, where the context plays a somewhat free-flowing role,
    // and only becomes truly visible in multithreaded cases.
    // In Rust Geodesy, the context plays a much more visible role, as
    // most transformation functionality is implemented directly as
    // methods of the context data structure.

    // We need some coordinates to test the code. The convenience methods
    // `gis` and `geo` produces a 4D coordinate tuple and automatically handles
    // conversion of the angular parts to radians, and geographical coordinates
    // in latitude/longitude order, to the GIS convention of longitude/latitude.
    let cph = C::gis(12., 55., 0., 0.); // Copenhagen
    let osl = C::gis(10., 60., 0., 0.); // Oslo
    let sth = C::geo(59., 18., 0., 0.); // Stockholm
    let hel = C::geo(60., 25., 0., 0.); // Helsinki

    // `gis` and `geo` have a sibling `raw` which generates coordinate tuples
    // from raw numbers, in case your point coordinates are already given in
    // radians. But since a coordinate tuple is really just an array of 4
    // double precision numbers, you can also generate it directly using plain
    // Rust syntax:
    let cph_raw = C([12_f64.to_radians(), 55_f64.to_radians(), 0., 0.0]);

    // The two versions of Copenhagen coordinates should be identical.
    assert_eq!(cph, cph_raw);

    // The Rust Geodesy interface is based on transformation of *arrays* of
    // coordinate tuples, rather than single points. So let's make an array:
    let mut data = [osl, cph, sth, hel];
    // Since all operations are carried out in place, the array needs to
    // be mutable, hence `let mut`

    // Let's create a transformation element ("an operator"), turning
    // geographical coordinates into UTM zone 32 coordinates. Since
    // this may go wrong (e.g. due to syntax errors in the operator
    // definition), use the Rust `match` syntax to handle errors.
    let utm32 = match ctx.operator("utm: {zone: 32}") {
        Some(op) => op,
        None => return println!("{}", ctx.report()),
    };
    // Now, let's use the utm32-operator to transform some data
    ctx.fwd(utm32, &mut data);

    println!("utm32:");
    for coord in data {
        println!("    {:?}", coord);
    }

    // The inv() method takes us back to geographic coordinates
    ctx.inv(utm32, &mut data);

    // The output is in radians, so we use this convenience function:
    C::degrees_all(&mut data);

    println!("Roundtrip to geo:");
    for coord in data {
        println!("    {:?}", coord);
    }

    // Here's an example of handling bad syntax:
    println!("Bad syntax example:");
    if ctx.operator("aargh: {zone: 23}").is_none() {
        println!("Deliberate error - {}", ctx.report());
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
    let pipeline = "ed50_wgs84: {
        steps: [
            cart: {ellps: intl},
            helmert: {x: -87, y: -96, z: -120},
            cart: {inv: true, ellps: GRS80}
        ]
    }";

    let ed50_wgs84 = ctx.operator(pipeline);
    if ed50_wgs84.is_none() {
        println!("ERROR - {}", ctx.report());
        return;
    };
    let ed50_wgs84 = ed50_wgs84.unwrap();

    // Since the forward transformation goes *from* ed50 to wgs84, we use
    // the inverse method to take us the other way, back in time to ED50
    ctx.inv(ed50_wgs84, &mut data);
    // Convert the internal lon/lat-in-radians format to the more human
    // readable lat/lon-in-degrees - then print
    C::geo_all(&mut data);
    println!("ed50:");
    for coord in data {
        println!("    {:?}", coord);
    }
}
