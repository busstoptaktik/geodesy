use log::{debug, trace};

fn main() -> Result<(), anyhow::Error> {
    // Filter by setting RUST_LOG to one of {Error, Warn, Info, Debug, Trace}
    if std::env::var("RUST_LOG").is_err() {
        simple_logger::init_with_level(log::Level::Error)?;
    } else {
        simple_logger::init_with_env()?;
    }

    trace!("Taking off");
    debug!("000");

    println!("Running the example from Rumination 000: Overall architecture and philosophy");
    rumination_000()?;

    Ok(())
}

fn rumination_000() -> Result<(), anyhow::Error> {
    // [0] Conventional shorthand for accessing the major functionality
    use geodesy::prelude::*;

    // [1] Build some context
    let mut ctx = Minimal::default();

    // [2] Obtain a handle to the utm-operator
    let utm32 = ctx.op("utm zone=32")?;

    // [3] Coordinates of some Scandinavian capitals
    let copenhagen = Coor2D::geo(55., 12.);
    let stockholm = Coor2D::geo(59., 18.);

    // [4] Put the coordinates into an array
    let mut data = [copenhagen, stockholm];

    // [5] Then do the forward conversion, i.e. geo -> utm
    ctx.apply(utm32, Fwd, &mut data)?;
    for coord in data {
        println!("{:?}", coord);
    }

    // [6] And go back, i.e. utm -> geo
    ctx.apply(utm32, Inv, &mut data)?;
    data.to_geo();
    for coord in data {
        println!("{:?}", coord);
    }

    Ok(())
}
