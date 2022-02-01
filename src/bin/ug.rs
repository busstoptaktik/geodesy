pub use geodesy;
pub use log::info;

// -------------------------------------------------------------------------------------
// UG: An experiment with an *U*ltrasmall *G*eodetic transformation system
// -------------------------------------------------------------------------------------

fn main() -> Result<(), anyhow::Error> {
    // Filter by setting RUST_LOG to one of {Error, Warn, Info, Debug, Trace}
    if std::env::var("RUST_LOG").is_err() {
        simple_logger::init_with_level(log::Level::Info)?;
        let yes = "fino";
        info!("Logging at info level! - {yes}");
    } else {
        simple_logger::init_with_env()?;
    }

    let size = std::mem::size_of::<geodesy::Op>();
    dbg!(size);
    Ok(())
}
