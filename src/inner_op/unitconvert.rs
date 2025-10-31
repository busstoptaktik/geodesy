/// The unit conversion operator.
/// It has a subset of the conversions supported by PROJ.
/// NB: If no units are specified, the default is meters.
/// ...
/// Conversions are performed by means of a pivot unit.
/// For horizontal conversions, the pivot unit is meters for linear units and radians for angular units.
/// Vertical units always pivot around meters.
/// Unit_A => (meters || radians) => Unit_B
use super::units::{ANGULAR_UNITS, LINEAR_UNITS};
use crate::authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;

    let xy_in_to_pivot = op.params.real("xy_in_to_pivot").unwrap();
    let pivot_to_xy_out = op.params.real("pivot_to_xy_out").unwrap();
    let xy = xy_in_to_pivot * pivot_to_xy_out;

    let z_in_to_pivot = op.params.real("z_in_to_pivot").unwrap();
    let pivot_to_z_out = op.params.real("pivot_to_z_out").unwrap();
    let z = z_in_to_pivot * pivot_to_z_out;

    for i in 0..operands.len() {
        let mut coord = operands.get_coord(i);
        coord[0] *= xy;
        coord[1] *= xy;
        coord[2] *= z;
        operands.set_coord(i, &coord);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;

    let xy_in_to_pivot = op.params.real("xy_in_to_pivot").unwrap();
    let pivot_to_xy_out = op.params.real("pivot_to_xy_out").unwrap();
    let xy = xy_in_to_pivot * pivot_to_xy_out;

    let z_in_to_pivot = op.params.real("z_in_to_pivot").unwrap();
    let pivot_to_z_out = op.params.real("pivot_to_z_out").unwrap();
    let z = z_in_to_pivot * pivot_to_z_out;

    for i in 0..operands.len() {
        let mut coord = operands.get_coord(i);
        coord[0] /= xy;
        coord[1] /= xy;
        coord[2] /= z;
        operands.set_coord(i, &coord);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 5] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "xy_in", default: Some("m") },
    OpParameter::Text { key: "xy_out", default: Some("m") },
    OpParameter::Text { key: "z_in", default: Some("m") },
    OpParameter::Text { key: "z_out", default: Some("m") },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.instantiated_as;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    let xy_in = params.text("xy_in").unwrap();
    let xy_out = params.text("xy_out").unwrap();
    let z_in = params.text("z_in").unwrap();
    let z_out = params.text("z_out").unwrap();

    let Some(xy_in_to_pivot) = get_pivot_multiplier(xy_in.as_str()) else {
        return Err(Error::BadParam("xy_in".to_string(), xy_in));
    };
    let Some(xy_out_to_pivot) = get_pivot_multiplier(xy_out.as_str()) else {
        return Err(Error::BadParam("xy_out".to_string(), xy_out));
    };
    let Some(z_in_to_pivot) = get_pivot_multiplier(z_in.as_str()) else {
        return Err(Error::BadParam("z_in".to_string(), xy_in));
    };
    let Some(z_out_to_pivot) = get_pivot_multiplier(z_out.as_str()) else {
        return Err(Error::BadParam("z_out".to_string(), xy_out));
    };

    params.real.insert("xy_in_to_pivot", xy_in_to_pivot);
    params.real.insert("pivot_to_xy_out", 1. / xy_out_to_pivot);
    params.real.insert("z_in_to_pivot", z_in_to_pivot);
    params.real.insert("pivot_to_z_out", 1. / z_out_to_pivot);

    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));

    Ok(Op {
        descriptor,
        params,
        steps: None,
    })
}

fn get_pivot_multiplier(name: &str) -> Option<f64> {
    LINEAR_UNITS
        .iter()
        .chain(ANGULAR_UNITS.iter())
        .find(|u| u.name() == name)
        .map(|u| u.multiplier())
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn xyz_default_units() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("unitconvert", OpConstructor(new));
        let op = ctx.op("unitconvert xy_in=us-ft z_in=us-ft")?;

        let mut operands = [Coor4D::raw(5., 5., 5., 1.)];

        // Forward
        let successes = ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0][0], 1.524003048, abs_all <= 1e-9);
        assert_float_eq!(operands[0][1], 1.524003048, abs_all <= 1e-9);
        assert_float_eq!(operands[0][2], 1.524003048, abs_all <= 1e-9);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-9);

        assert_eq!(successes, 1);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][0], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][1], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][2], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-9);
        Ok(())
    }

    #[test]
    fn xy_us_ft_round_trip() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("unitconvert", OpConstructor(new));
        let op = ctx.op("unitconvert xy_in=us-ft xy_out=us-ft")?;

        let mut operands = [Coor4D::raw(5., 5., 1., 1.)];

        // Forward
        let successes = ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0][0], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][1], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][2], 1., abs_all <= 1e-9);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-9);

        assert_eq!(successes, 1);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][0], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][1], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][2], 1., abs_all <= 1e-9);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-9);
        Ok(())
    }

    #[test]
    fn xyz_us_ft_to_m() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("unitconvert", OpConstructor(new));
        let op = ctx.op("unitconvert xy_in=us-ft xy_out=m z_in=us-ft z_out=m")?;

        let mut operands = [Coor4D::raw(5., 5., 5., 1.)];

        // Forward
        let successes = ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0][0], 1.524003048, abs_all <= 1e-9);
        assert_float_eq!(operands[0][1], 1.524003048, abs_all <= 1e-9);
        assert_float_eq!(operands[0][2], 1.524003048, abs_all <= 1e-9);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-9);

        assert_eq!(successes, 1);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][0], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][1], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][2], 5., abs_all <= 1e-9);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-9);
        Ok(())
    }

    #[test]
    fn xy_yd_to_m() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("unitconvert", OpConstructor(new));
        let op = ctx.op("unitconvert xy_in=us-yd xy_out=m")?;

        let mut operands = [Coor4D::raw(1000., 1000., 500., 1.)];

        // Forward
        let successes = ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0][0], 914.40182880, abs_all <= 1e-5);
        assert_float_eq!(operands[0][1], 914.40182880, abs_all <= 1e-5);
        assert_float_eq!(operands[0][2], 500., abs_all <= 1e-5);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-5);

        assert_eq!(successes, 1);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][0], 1000.0, abs_all <= 1e-5);
        assert_float_eq!(operands[0][1], 1000.0, abs_all <= 1e-5);
        assert_float_eq!(operands[0][2], 500., abs_all <= 1e-9);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-9);
        Ok(())
    }
    #[test]
    fn xy_grad_to_deg() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("unitconvert", OpConstructor(new));
        let op = ctx.op("unitconvert xy_in=grad xy_out=deg")?;

        let mut operands = [Coor4D::raw(135.0, 40., 500., 1.)];

        // Forward
        let successes = ctx.apply(op, Fwd, &mut operands)?;
        assert_float_eq!(operands[0][0], 121.5, abs_all <= 1e-5);
        assert_float_eq!(operands[0][1], 36.0, abs_all <= 1e-5);
        assert_float_eq!(operands[0][2], 500., abs_all <= 1e-5);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-5);

        assert_eq!(successes, 1);

        // Inverse + roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert_float_eq!(operands[0][0], 135.0, abs_all <= 1e-5);
        assert_float_eq!(operands[0][1], 40.0, abs_all <= 1e-5);
        assert_float_eq!(operands[0][2], 500., abs_all <= 1e-9);
        assert_float_eq!(operands[0][3], 1., abs_all <= 1e-9);
        Ok(())
    }

    #[test]
    fn unknown_unit() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_op("unitconvert", OpConstructor(new));
        assert!(ctx.op("unitconvert xy_in=unknown xy_out=deg").is_err());
        Ok(())
    }
}
