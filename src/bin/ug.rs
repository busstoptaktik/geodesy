#![allow(dead_code, unused_variables)]

/*

key, ns, name: [A-Za-z0-9]*
value: ^[:^|]

id: :name, ns:name
parameter: key=value


*/

mod op;
use geodesy::CoordinateTuple as C;
use log::info;
use op::etc;
use op::parsed_parameters::ParsedParameters;
use op::provider;
use op::raw_parameters::RawParameters;
use op::*;

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

    let provider = provider::Minimal::default();
    let op = Op::new("addone", &provider)?;
    let copenhagen = C::raw(55., 12., 0., 0.);
    let stockholm = C::raw(59., 18., 0., 0.);
    let mut data = [copenhagen, stockholm];
    dbg!(data);
    op.operate(&provider, &mut data, Direction::Fwd);
    dbg!(data);

    let op = an_operator_constructor()?;
    let mut op = op;
    op.name = "Ost".to_string();
    // dbg!(&op);
    println!("{:#?}", op.ignored());

    let one = "one two three"
        .split_whitespace()
        .next()
        .unwrap_or("unknown");
    dbg!(one);
    let size = std::mem::size_of::<Op>();
    dbg!(size);
    Ok(())
}

// -----------------------------------------------------------------------------

/// TODO: Ned som test!
use op::parameter::OpParameter::*;
fn an_operator_constructor() -> Result<ParsedParameters, Error> {
    #[rustfmt::skip]
    let gamut = [
        Flag    {key: "flag" },
        Natural {key: "natural_default",  default: Some(42)},
        Natural {key: "natural_required", default: None},
        Integer {key: "integer_default",  default: Some(-42)},
        Integer {key: "integer_required", default: None},
        Real    {key: "real_default",     default: Some(-42.)},
        Real    {key: "real_required",    default: None},
        Text    {key: "text_default",     default: Some("GRS80")},
        Text    {key: "text_required",    default: None}
    ];

    let definition = "
        operatorname
        two_of_these
        two_of_these
        flag flag_ignored
        natural_default=44 natural_required=42 natural_ignored=22
        integer_required=42 integer_ignored=11
        real_required=42
        text_required=banana, hønsefedt
    ";

    // Test the recursive call functionality of `OpResource`
    let globals = etc::split_into_parameters("globals inv ellps=GRS80");
    let macro_invocation = "translation cheese=ost salt=salt soup=suppe";
    let first = RawParameters::new(definition, &globals);
    dbg!(&first);
    let next = first.next(definition);
    dbg!(&next);

    ParsedParameters::new(&next, &gamut)
}

// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operator_args() -> Result<(), Error> {
        Ok(())
    }
}
