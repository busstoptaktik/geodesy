/// Normal gravity
use crate::authoring::*;

fn welmec(operands: &mut dyn CoordinateSet, ellps: &Ellipsoid, zero_height: bool) -> usize {
    let number_of_operands = operands.len();
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        let latitude = coord[0].to_radians();
        let height = if zero_height { 0. } else { coord[1] };
        coord[0] = ellps.welmec(latitude, height);
        operands.set_coord(i, &coord);
    }
    number_of_operands
}

fn grs80(operands: &mut dyn CoordinateSet, ellps: &Ellipsoid, zero_height: bool) -> usize {
    let number_of_operands = operands.len();
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        let latitude = coord[0].to_radians();
        coord[0] = ellps.grs80_gravity(latitude);
        if !zero_height {
            coord[0] -= ellps.grs67_height_correction(latitude, coord[1]);
        }
        operands.set_coord(i, &coord);
    }
    number_of_operands
}

fn grs67(operands: &mut dyn CoordinateSet, ellps: &Ellipsoid, zero_height: bool) -> usize {
    let number_of_operands = operands.len();
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        let latitude = coord[0].to_radians();
        coord[0] = ellps.grs67_gravity(latitude);
        if !zero_height {
            coord[0] -= ellps.grs67_height_correction(latitude, coord[1]);
        }
        operands.set_coord(i, &coord);
    }
    number_of_operands
}

fn jeffreys(operands: &mut dyn CoordinateSet, ellps: &Ellipsoid, zero_height: bool) -> usize {
    let number_of_operands = operands.len();
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        let latitude = coord[0].to_radians();
        coord[0] = ellps.jeffreys_gravity_1948(latitude);
        if !zero_height {
            coord[0] -= ellps.cassinis_height_correction(coord[1], 2800.);
        }
        operands.set_coord(i, &coord);
    }
    number_of_operands
}

fn cassinis(operands: &mut dyn CoordinateSet, ellps: &Ellipsoid, zero_height: bool) -> usize {
    let number_of_operands = operands.len();
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        let latitude = coord[0].to_radians();
        coord[0] = ellps.cassinis_gravity_1930(latitude);
        if !zero_height {
            coord[0] -= ellps.cassinis_height_correction(coord[1], 2800.);
        }
        operands.set_coord(i, &coord);
    }
    number_of_operands
}

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);
    let zero_height = op.params.boolean("zero-height");
    dbg!(zero_height);
    let Some(action) = op.params.text.get("action") else {
        return 0;
    };

    match action.as_str() {
        "welmec" => welmec(operands, &ellps, zero_height),
        "grs80" => grs80(operands, &ellps, zero_height),
        "grs67" => grs67(operands, &ellps, zero_height),
        "jeffreys" => jeffreys(operands, &ellps, zero_height),
        "cassinis" => cassinis(operands, &ellps, zero_height),
        _ => 0,
    }
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 7] = [
    OpParameter::Flag { key: "cassinis" },
    OpParameter::Flag { key: "jeffreys" },
    OpParameter::Flag { key: "grs67" },
    OpParameter::Flag { key: "grs80" },
    OpParameter::Flag { key: "welmec" },
    OpParameter::Flag { key: "zero-height" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let mut op = Op::plain(parameters, InnerOp(fwd), None, &GAMUT, ctx)?;
    let valid = ["cassinis", "jeffreys", "grs67", "grs80", "welmec"];

    // Check that at most one normal gravity formula is specified
    let mut number_of_flags = 0_usize;
    for flag in &op.params.boolean {
        if valid.contains(flag) {
            number_of_flags += 1;
        }
    }
    if number_of_flags > 1 {
        return Err(Error::MissingParam(
            "gravity: must specify at most one of flags cassinis/jeffreys/grs67/grs80/welmec"
                .to_string(),
        ));
    }

    // The action defaults to grs80
    op.params.text.insert("action", "grs80".to_string());
    for flag in &op.params.boolean {
        if valid.contains(flag) {
            op.params.text.insert("action", flag.to_string());
            break;
        }
    }

    Ok(op)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_gravity() -> Result<(), Error> {
        let grs80: Ellipsoid = Ellipsoid::named("GRS80")?;
        let grs67: Ellipsoid = Ellipsoid::named("GRS67")?;
        let intl: Ellipsoid = Ellipsoid::named("intl")?;

        let mut ctx = Minimal::default();
        let lat = 45_f64.to_radians();

        let op = ctx.op("gravity cassinis ellps=intl")?;
        let mut poi = [Coor4D::raw(45., 100., 0., 0.)];
        ctx.apply(op, Fwd, &mut poi)?;
        assert_eq!(
            intl.cassinis_gravity_1930(lat) - intl.cassinis_height_correction(100., 2800.),
            poi[0][0]
        );

        let op = ctx.op("gravity jeffreys ellps=intl")?;
        let mut poi = [Coor4D::raw(45., 100., 0., 0.)];
        ctx.apply(op, Fwd, &mut poi)?;
        assert_eq!(
            intl.jeffreys_gravity_1948(lat) - intl.cassinis_height_correction(100., 2800.),
            poi[0][0]
        );

        let op = ctx.op("gravity grs67 ellps=GRS67")?;
        let mut poi = [Coor4D::raw(45., 100., 0., 0.)];
        ctx.apply(op, Fwd, &mut poi)?;
        assert_eq!(
            grs67.grs67_gravity(lat) - grs67.grs67_height_correction(lat, 100.),
            poi[0][0]
        );

        let op = ctx.op("gravity grs80 ellps=GRS80")?;
        let mut poi = [Coor4D::raw(45., 100., 0., 0.)];
        ctx.apply(op, Fwd, &mut poi)?;
        assert_eq!(
            grs80.grs80_gravity(lat) - grs80.grs67_height_correction(lat, 100.),
            poi[0][0]
        );

        Ok(())
    }
}
