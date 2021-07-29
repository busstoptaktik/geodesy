/// Koordinatprocessering
fn main() {
    use geodesy::CoordinateTuple as C;
    let mut ctx = geodesy::Context::new();

    let coo = C([1.,2.,3.,4.]);
    println!("coo: {:?}", coo);

    let geo = C::geo(55., 12.,0.,0.);
    let gis = C::gis(12., 55.,0.,0.);
    assert_eq!(geo, gis);
    println!("geo: {:?}", geo.to_geo());

    // Some Nordic/Baltic capitals
    let nuk = ctx.coordeg(-52., 64., 0., 0.); // Nuuk
    let tor = ctx.coordeg(-7., 62., 0., 0.); // Tórshavn
    let cph = ctx.coordeg(12., 55., 0., 0.); // Copenhagen
    let osl = ctx.coordeg(10., 60., 0., 0.); // Oslo
    let sth = ctx.coordeg(18., 59., 0., 0.); // Stockholm
    let mar = ctx.coordeg(20., 60., 0., 0.); // Mariehamn
    let hel = ctx.coordeg(25., 60., 0., 0.); // Helsinki
    let tal = ctx.coordeg(25., 59., 0., 0.); // Tallinn
    let rga = ctx.coordeg(24., 57., 0., 0.); // Riga
    let vil = ctx.coordeg(25., 55., 0., 0.); // Vilnius

    // Gothenburg is not a capital, but it is strategically placed
    // approximately equidistant from OSL, CPH and STH, so it
    // deserves special treatment by getting its coordinate
    // from direct inline construction, which is perfectly
    // possible: A coordinate is just an array of four double
    // precision floats
    let got = C::geo(58., 12., 0., 0.0);

    let mut data_all = [nuk, tor, osl, cph, sth, mar, hel, tal, rga, vil];
    let mut data_utm32 = [osl, cph, got];

    // We loop over the full dataset, and add some arbitrary time information
    for (i, dimser) in data_all.iter_mut().enumerate() {
        dimser[3] = i as f64;
    }

    let utm32 = match ctx.operator("utm: {zone: 32}") {
        Err(e) => return println!("Awful error: {}", e),
        Ok(op) => op,
    };

    ctx.fwd(utm32, &mut data_utm32);
    println!("utm32:");
    for coord in data_utm32 {
        println!("    {:?}", coord);
    }

    let pipeline = "ed50_etrs89: {
        steps: [
            cart: {ellps: intl},
            helmert: {dx: -87, dy: -96, dz: -120},
            cart: {inv: true, ellps: GRS80}
        ]
    }";

    let ed50_etrs89 = match ctx.operator(pipeline) {
        Err(e) => return println!("Awful error: {}", e),
        Ok(op) => op,
    };

    ctx.fwd(ed50_etrs89, &mut data_all);
    ctx.to_degrees(&mut data_all);
    println!("etrs89:");
    for coord in data_all {
        println!("    {:?}", coord);
    }
}

/*
 Documenting dirs v3.0.2
 Documenting geodesy v0.1.0 (C:\Users\B004330\Documents\2021\Projects\geodesy)
warning: unresolved link to `self::coordinates::CoordinateTuple::hypot2`
   --> src\ellipsoids.rs:588:20
    |
588 |     /// [`hypot2`](crate::coordinates::CoordinateTuple::hypot2),
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no item named `CoordinateTuple` in module `coordinates`
    |
    = note: `#[warn(rustdoc::broken_intra_doc_links)]` on by default

warning: unresolved link to `self::coordinates::CoordinateTuple::hypot3`
   --> src\ellipsoids.rs:589:20
    |
589 |     /// [`hypot3`](crate::coordinates::CoordinateTuple::hypot3)
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no item named `CoordinateTuple` in module `coordinates`

warning: unresolved link to `self::coordinates::CoordinateTuple::hypot3`
  --> src\coordinates.rs:58:20
   |
58 |     /// [`hypot3`](crate::coordinates::CoordinateTuple::hypot3),
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no item named `CoordinateTuple` in module `coordinates`

warning: unresolved link to `self::coordinates::CoordinateTuple::hypot2`
  --> src\coordinates.rs:87:20
   |
87 |     /// [`hypot2`](crate::coordinates::CoordinateTuple::hypot2),
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no item named `CoordinateTuple` in module `coordinates`

warning: 4 warnings emitted


*/
