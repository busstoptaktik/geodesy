// examples/02-user_defined_macros.rs

// See also 00-transformations.rs
// Run with:
// cargo run --example 02-user_defined_macros
use geodesy::preamble::*;

fn main() -> anyhow::Result<()> {
    let mut ctx = Minimal::default();

    // Same test coordinates as in example 00.
    let cph = Coord::gis(12., 55., 0., 0.); // Copenhagen
    let osl = Coord::gis(10., 60., 0., 0.); // Oslo
    let sth = Coord::geo(59., 18., 0., 0.); // Stockholm
    let hel = Coord::geo(60., 25., 0., 0.); // Helsinki

    let mut data = [osl, cph, sth, hel];

    // In example 00, we instantiated a pipeline comprising a Helmert
    // transformation sandwiched between conversions between geodetic/
    // cartesian coordinates.
    // Since this cartesian|helmert|geodetic triplet is quite useful in
    // its own right, then why not create a macro, making it immediately
    // available under the name `geohelmert`?
    let geohelmert_macro_text = "cart ellps=^left | helmert | cart inv ellps=^right";
    // Note the 'hats' (^). The hat points upward, and is known as
    // "the look up operator". Within a macro, it looks up and
    // captures values set in the calling environment, as will become
    // clear in a moment...

    // First we need to register our macro in the resource provider ("context")
    ctx.register_resource("geo:helmert", geohelmert_macro_text);

    // Now let's see whether it works - instantiate the macro, using the same
    // parameters as used in example 00. The ':' in the operator name invokes
    // the macro expansion machinery.
    let ed50_wgs84 = ctx.op("geo:helmert left=intl right=GRS80 x=-87 y=-96 z=-120")?;
    // ... and do the same transformation as in example 00
    ctx.apply(ed50_wgs84, Fwd, &mut data)?;
    ctx.apply(ed50_wgs84, Inv, &mut data)?;

    // geo_all(data) transforms all elements in data from the internal GIS
    // format (lon/lat in radians) to lat/lon in degrees.
    let mut etrs89 = data.clone();
    Coord::geo_all(&mut etrs89);
    println!("etrs89:");
    for coord in data {
        println!("    {:?}", coord);
    }

    ctx.apply(ed50_wgs84, Inv, &mut data)?;

    // geo_all(data) transforms all elements in data from the internal GIS
    // format (lon/lat in radians) to lat/lon in degrees.
    Coord::geo_all(&mut data);
    println!("Back to ed50:");
    for coord in data {
        println!("    {:?}", coord);
    }

    Ok(())
}
