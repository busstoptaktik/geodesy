/*! Plonketi Plonk! !*/

/// Convert "Ghastly YAML Shorthand" to YAML
fn gys_to_yaml(gys: &str) -> String {
    // Appears to be YAML already - do nothing!
    if gys.contains('{') {
        return String::from(gys);
    }

    let mut yaml = String::new();
    let mut indent = "";
    let steps: Vec<&str> = gys.split('|').collect();
    let nsteps = steps.len();
    if nsteps > 1 {
        yaml += "pipeline_from_gys: {\n  steps: [\n";
        indent = "    ";
    }
    println!("GYS :\n{:?}", steps);
    for step in steps {
        let mut elements: Vec<&str> = step.split_whitespace().collect();
        let n = elements.len();
        if n == 0 {
            return String::from("Error: Empty step!");
        }

        // First the operator name
        yaml += indent;
        yaml += elements[0];
        yaml += ":";

        // linebreaks after the first step
        indent = ",\n    ";
        // No args? Then insert an empty argument list
        if n == 1 {
            yaml += " {}";
            continue;
        }

        // Handle args
        yaml += " {";

        for i in 1..n {
            // We constructed a key-value par in last iteration?
            if elements[i].is_empty() {
                continue;
            }
            let e = elements[i].to_string();
            if e.ends_with(':') {
                if i == n - 1 {
                    return String::from("Missing value for key '") + &e + "'";
                }
                yaml += &e;
                yaml += " ";
                yaml += elements[i + 1];
                if i + 2 < n {
                    yaml += ", ";
                }
                elements[i + 1] = "";
                continue;
            };

            // Ultra compact notation: key:value, no whitespace
            if e.contains(':') {
                yaml += &e.replace(":", ": ");
                if i + 1 < n {
                    yaml += ", ";
                }
                continue;
            }

            // Key with no value? provide "true"
            yaml += &e;
            yaml += ": true";
            if i + 1 < n {
                yaml += ", ";
            }
        }
        yaml += "}";
    }

    if nsteps > 1 {
        yaml += "\n  ]\n}";
    }

    yaml
}

/// Koordinatprocessering
fn main() {
    // use std::env;
    use geodesy::CoordinateTuple as C;
    let mut ctx = geodesy::Context::new();

    // A pipeline in YAML
    let pipeline = "ed50_etrs89: {
        steps: [
            cart: {ellps: intl},
            helmert: {x: -87, y: -96, z: -120},
            cart: {inv: true, ellps: GRS80}
        ]
    }";

    // The same pipeline in Ghastly YAML Shorthand (GYS)
    let gys = "cart ellps: intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80";

    let op_yaml = ctx.operation(pipeline).unwrap();
    let op_gys = ctx.operation(gys).unwrap();
    let copenhagen = C::geo(55., 12., 0., 0.);

    let stockholm = C::geo(59., 18., 0., 0.);
    let mut yaml_data = [copenhagen, stockholm];
    let mut gys_data = [copenhagen, stockholm];

    ctx.fwd(op_yaml, &mut yaml_data);
    ctx.fwd(op_gys, &mut gys_data);
    C::geo_all(&mut yaml_data);
    C::geo_all(&mut gys_data);

    println!("{:?}", yaml_data);
    println!("{:?}", gys_data);

    assert!(yaml_data[0].hypot3(&gys_data[0]) < 1e-16);
    assert!(yaml_data[1].hypot3(&gys_data[1]) < 1e-16);

    if let Some(dir) = dirs::data_local_dir() {
        println!("data_local_dir: {}", dir.to_str().unwrap_or_default());
    }

    // println!("YAML:\n{}", gys_to_yaml("pose |  a gurk: esalat og:rener beskidte|  banan med:æble line"));
    println!("YAML:\n{}", gys_to_yaml(gys));
    println!("YAML:\n{}", gys_to_yaml("cart ellps:intl"));
    println!("YAML:\n{}", gys_to_yaml("cart ellps:intl|cart"));
    println!("YAML:\n{}", gys_to_yaml("cart"));

    if let Some(utm32) = ctx.operation("utm: {zone: 32}") {
        let copenhagen = C::geo(55., 12., 0., 0.);
        let stockholm = C::geo(59., 18., 0., 0.);
        let mut data = [copenhagen, stockholm];

        ctx.fwd(utm32, &mut data);
        println!("{:?}", data);
    }

    let coo = C([1., 2., 3., 4.]);
    println!("coo: {:?}", coo);

    let geo = C::geo(55., 12., 0., 0.);
    let gis = C::gis(12., 55., 0., 0.);
    assert_eq!(geo, gis);
    println!("geo: {:?}", geo.to_geo());

    // Some Nordic/Baltic capitals
    let nuk = C::gis(-52., 64., 0., 0.); // Nuuk
    let tor = C::gis(-7., 62., 0., 0.); // Tórshavn
    let cph = C::gis(12., 55., 0., 0.); // Copenhagen
    let osl = C::gis(10., 60., 0., 0.); // Oslo
    let sth = C::gis(18., 59., 0., 0.); // Stockholm
    let mar = C::gis(20., 60., 0., 0.); // Mariehamn
    let hel = C::gis(25., 60., 0., 0.); // Helsinki
    let tal = C::gis(25., 59., 0., 0.); // Tallinn
    let rga = C::gis(24., 57., 0., 0.); // Riga
    let vil = C::gis(25., 55., 0., 0.); // Vilnius

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

    let utm32 = match ctx.operation("utm: {zone: 32}") {
        None => return println!("Awful error"),
        Some(op) => op,
    };

    ctx.fwd(utm32, &mut data_utm32);
    println!("utm32:");
    for coord in data_utm32 {
        println!("    {:?}", coord);
    }

    // Try to read predefined transformation from zip archive
    let pladder = match ctx.operation("ed50_etrs89") {
        None => return println!("Awful error"),
        Some(op) => op,
    };
    ctx.fwd(pladder, &mut data_all);
    println!("etrs89:");
    for coord in data_all {
        println!("    {:?}", coord.to_geo());
    }

    let pipeline = "ed50_etrs89: {
        steps: [
            cart: {ellps: intl},
            helmert: {x: -87, y: -96, z: -120},
            cart: {inv: true, ellps: GRS80}
        ]
    }";

    let ed50_etrs89 = match ctx.operation(pipeline) {
        None => return println!("Awful error"),
        Some(op) => op,
    };

    ctx.inv(ed50_etrs89, &mut data_all);
    println!("etrs89:");
    for coord in data_all {
        println!("    {:?}", coord.to_geo());
    }
}
