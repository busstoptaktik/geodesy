#![allow(dead_code, unused_variables)]

mod geod;
pub use geod::preamble::*;
pub use log::info;
/*
use geodesy::CoordinateTuple as C;
use geod::etc;
use geod::parsed_parameters::ParsedParameters;
use geod::provider;
use geod::raw_parameters::RawParameters;
use geod::*;
*/

// -----------------------------------------------------------------------------
// UG: An experiment with an *U*ltrasmall *G*eodetic transformation system
// -----------------------------------------------------------------------------

fn main() -> Result<(), anyhow::Error> {
    // Filter by setting RUST_LOG to one of {Error, Warn, Info, Debug, Trace}
    if std::env::var("RUST_LOG").is_err() {
        simple_logger::init_with_level(log::Level::Info)?;
        let yes = "fino";
        info!("Logging at info level! - {yes}");
    } else {
        simple_logger::init_with_env()?;
    }

    let size = std::mem::size_of::<Op>();
    dbg!(size);
    Ok(())
}
