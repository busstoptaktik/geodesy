//! Web Mercator
use crate::authoring::*;
use std::f64::consts::FRAC_PI_2;
use std::f64::consts::FRAC_PI_4;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);
    let a = ellps.semimajor_axis();

    let mut successes = 0_usize;
    for i in 0..operands.len() {
        let (lon, lat) = operands.xy(i);

        let easting = lon * a;
        let northing = a * (FRAC_PI_4 + lat / 2.0).tan().ln();

        operands.set_xy(i, easting, northing);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);
    let a = ellps.semimajor_axis();

    let mut successes = 0_usize;
    for i in 0..operands.len() {
        let (easting, northing) = operands.xy(i);

        // Easting -> Longitude
        let longitude = easting / a;

        // Northing -> Latitude
        let latitude = FRAC_PI_2 - 2.0 * (-northing / a).exp().atan();

        operands.set_xy(i, longitude, latitude);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 2] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "ellps",  default: Some("WGS84") },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.instantiated_as;
    let params = ParsedParameters::new(parameters, &GAMUT)?;

    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));

    Ok(Op {
        descriptor,
        params,
        steps: None,
        id: OpHandle::new(),
    })
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn webmerc() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "webmerc";
        let op = ctx.op(definition)?;

        // Validation value from PROJ: echo 12 55 0 0 | cct -d18 +proj=webmerc
        // followed by quadrant tests from PROJ builtins.gie
        let geo = [Coor4D::geo(55., 12., 0., 0.)];

        let projected = [Coor4D::raw(
            1_335_833.889_519_282_8,
            7_361_866.113_051_188,
            0.,
            0.,
        )];

        // Forward
        let mut operands = geo;
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert_float_eq!(operands[i].0, projected[i].0, abs_all <= 1e-8);
        }

        // Roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert_float_eq!(operands[i].0, geo[i].0, abs_all <= 2e-9);
        }

        Ok(())
    }
}
