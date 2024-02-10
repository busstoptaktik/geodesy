/// Normal gravity
use crate::authoring::*;

fn welmec(operands: &mut dyn CoordinateSet, ellps: &Ellipsoid, zero_height: bool) -> usize {
    let number_of_operands = operands.len();
    for i in 0..number_of_operands {
        let mut coord = operands.get_coord(i);
        let latitude = coord[0].to_radians();
        let height = if zero_height {0.} else {coord[1]};
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
    let zero_height = op.params.boolean("zero-height"); dbg!(zero_height);
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

    // Check that at most one normal gravity formula is specified
    let mut number_of_flags = 0_usize;
    for flag in &op.params.boolean {
        if ["cassinis", "grs67", "grs80", "welmec"].contains(&flag) {
            number_of_flags += 1;
        }
    }
    if number_of_flags > 1 {
        return Err(Error::MissingParam("gravity: must specify at most one of flags cassinis/grs67/grs80/welmec".to_string()));
    }

    // The action defaults to grs80
    op.params.text.insert("action", "grs80".to_string());
    for flag in &op.params.boolean {
        if ["cassinis", "grs67", "grs80", "welmec"].contains(&flag) {
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
    fn latitude() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        // Geocentric
        let op = ctx.op("latitude geocentric ellps=GRS80")?;
        let mut operands = [Coor4D::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.818_973_308_324_5).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // Reduced (alias parametric)
        let op = ctx.op("latitude reduced ellps=GRS80")?;
        let mut operands = [Coor4D::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.909_538_187_092_245).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // And vice versa: Parametric (alias Reduced)
        let op = ctx.op("latitude parametric ellps=GRS80")?;
        let mut operands = [Coor4D::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.909_538_187_092_245).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // Conformal
        let op = ctx.op("latitude conformal ellps=GRS80")?;
        let mut operands = [Coor4D::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.819_109_023_689_02).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // Rectifying
        let op = ctx.op("latitude rectifying ellps=GRS80")?;
        let mut operands = [Coor4D::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.772_351_809_646_84).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // Authalic
        let op = ctx.op("latitude authalic ellps=GRS80")?;
        let mut operands = [Coor4D::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.879_361_594_517_796).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        Ok(())
    }
}
