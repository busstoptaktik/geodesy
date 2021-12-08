// examples/02-user_defined_macros.rs

// See also 00-transformations.rs
// Run with:
// cargo run --example 02-user_defined_macros

// The CoordinateTuple type is much used, so we give it a very short alias
use geodesy::CoordinateTuple as C;

// Let Anyhow and GeodesyError play together for convenient error handling
use anyhow::Context;
use geodesy::Provider;

fn main() -> anyhow::Result<()> {
    let mut ctx = geodesy::Plain::new(geodesy::SearchLevel::LocalPatches, false);

    // Same test coordinates as in example 00.
    let cph = C::gis(12., 55., 0., 0.); // Copenhagen
    let osl = C::gis(10., 60., 0., 0.); // Oslo
    let sth = C::geo(59., 18., 0., 0.); // Stockholm
    let hel = C::geo(60., 25., 0., 0.); // Helsinki

    let mut data = [osl, cph, sth, hel];

    // In example 00, we instantiated a pipeline comprising a Helmert
    // transformation sandwiched between conversions between geodetic/
    // cartesian coordinates.
    // Since this cartesian|helmert|geodetic triplet is quite useful in
    // its own right, then why not create a macro, making it immediately
    // available under the name `geohelmert`?
    let geohelmert_macro_text = "cart ellps:^left | helmert | cart inv ellps:^right";
    // Note the 'hats' (^). The hat points upward, and is known as
    // "the look up operator". Within a macro, it looks up and
    // captures values set in the calling environment, as will become
    // clear in a moment...

    // First we need to register our macro in the resource provider ("context")
    ctx.register_macro("geohelmert", geohelmert_macro_text)
        .context("Macro registration failed")?;
    // if !ctx.register_macro("geohelmert", geohelmert_macro_text) {
    //         return Err(Error::General(
    //         "Awful error: Couldn't register macro 'geohelmert'",
    //     ));
    // };

    // Now let's see whether it works - instantiate the macro, using the same
    // parameters as used in example 00.
    let ed50_wgs84 = ctx
        .define_operation("geohelmert left:intl right:GRS80 x:-87 y:-96 z:-120")
        .context("Macro not found")?;
    // ... and do the same transformation as in example 00
    ctx.inv(ed50_wgs84, &mut data);

    // geo_all(data) transforms all elements in data from the internal GIS
    // format (lon/lat in radians) to lat/lon in degrees.
    C::geo_all(&mut data);
    println!("ed50:");
    for coord in data {
        println!("    {:?}", coord);
    }

    Ok(())
}
