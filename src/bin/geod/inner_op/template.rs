#![allow(non_snake_case)]
/// Template for implementation of operators
use super::*;

// ----- C O M M O N -------------------------------------------------------------------

fn template_common(
    op: &Op,
    prv: &dyn Provider,
    operands: &mut [CoordinateTuple],
    direction: Direction,
) -> usize {
    todo!();
}

// ----- F O R W A R D --------------------------------------------------------------

fn template_fwd(op: &Op, prv: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    todo!();
}

// ----- I N V E R S E --------------------------------------------------------------

fn template_inv(op: &Op, prv: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    todo!();
}

// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 19] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Real { key: "x", default: Some(0f64) },
    OpParameter::Text { key: "convention", default: Some("") },
];

pub fn new(parameters: &RawParameters, _provider: &dyn Provider) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    // Handle parameters here
    let x = params.real("x")?;

    let fwd = InnerOp(helmert_fwd);
    let inv = InnerOp(helmert_inv);
    let descriptor = OpDescriptor::new(def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    Ok(Op {
        descriptor,
        params,
        steps,
    })
}

// ----- A N C I L L A R Y   F U N C T I O N S   G O   H E R E -------------------------

fn rotation_matrix(r: Vec<f64>, exact: bool, position_vector: bool) -> [[f64; 3]; 3] {
    todo!()
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    const GDA94: CoordinateTuple =
        CoordinateTuple([-4052051.7643, 4212836.2017, -2545106.0245, 0.0]);
    const GDA2020A: CoordinateTuple =
        CoordinateTuple([-4052052.7379, 4212835.9897, -2545104.5898, 0.0]);
    const GDA2020B: CoordinateTuple =
        CoordinateTuple([-4052052.7373, 4212835.9835, -2545104.5867, 2020.0]);
    const ITRF2014: CoordinateTuple =
        CoordinateTuple([-4052052.6588, 4212835.9938, -2545104.6946, 2018.0]);

    #[test]
    fn translation() -> Result<(), Error> {
        let provider = Minimal::default();
        let op = Op::new("helmert x=-87 y=-96 z=-120", &provider)?;

        // EPSG:1134 - 3 parameter, ED50/WGS84, s = sqrt(27) m
        let mut operands = [CoordinateTuple::origin()];

        // Forward
        op.apply(&provider, &mut operands, Direction::Fwd);
        assert_eq!(operands[0].first(), -87.);
        assert_eq!(operands[0].second(), -96.);
        assert_eq!(operands[0].third(), -120.);

        // Inverse + roundtrip
        op.apply(&provider, &mut operands, Direction::Inv);
        assert_eq!(operands[0].first(), 0.);
        assert_eq!(operands[0].second(), 0.);
        assert_eq!(operands[0].third(), 0.);
        Ok(())
    }
}
