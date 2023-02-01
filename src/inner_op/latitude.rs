/// Auxiliary latitudes
use crate::operator_authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> usize {
    let mut successes = 0_usize;
    let ellps = op.params.ellps[0];

    if op.params.boolean("geocentric") {
        for coord in operands {
            coord[1] = ellps.latitude_geographic_to_geocentric(coord[1]);
            successes += 1;
        }
    } else if op.params.boolean("reduced") {
        for coord in operands {
            coord[1] = ellps.latitude_geographic_to_reduced(coord[1]);
            successes += 1;
        }
    } else if op.params.boolean("conformal") {
        let Some(coefficients) = op.params.fourier_coefficients.get("coefficients") else {
            return 0;
        };

        for coord in operands {
            coord[1] = ellps.latitude_geographic_to_conformal(coord[1], coefficients);
            successes += 1;
        }
    } else if op.params.boolean("rectifying") {
        let Some(coefficients) = op.params.fourier_coefficients.get("coefficients") else {
            return 0;
        };

        for coord in operands {
            coord[1] = ellps.latitude_geographic_to_rectifying(coord[1], coefficients);
            successes += 1;
        }
    } else if op.params.boolean("authalic") {
        let Some(coefficients) = op.params.fourier_coefficients.get("coefficients") else {
            return 0;
        };

        for coord in operands {
            coord[1] = ellps.latitude_geographic_to_authalic(coord[1], coefficients);
            successes += 1;
        }
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut [Coord]) -> usize {
    let mut successes = 0_usize;
    let ellps = op.params.ellps[0];

    if op.params.boolean("geocentric") {
        for coord in operands {
            coord[1] = ellps.latitude_geocentric_to_geographic(coord[1]);
            successes += 1;
        }
    } else if op.params.boolean("reduced") {
        for coord in operands {
            coord[1] = ellps.latitude_reduced_to_geographic(coord[1]);
            successes += 1;
        }
    } else if op.params.boolean("conformal") {
        let Some(coefficients) = op.params.fourier_coefficients.get("coefficients") else {
            return 0;
        };

        for coord in operands {
            coord[1] = ellps.latitude_conformal_to_geographic(coord[1], coefficients);
            successes += 1;
        }
    } else if op.params.boolean("rectifying") {
        let Some(coefficients) = op.params.fourier_coefficients.get("coefficients") else {
            return 0;
        };

        for coord in operands {
            coord[1] = ellps.latitude_rectifying_to_geographic(coord[1], coefficients);
            successes += 1;
        }
    } else if op.params.boolean("authalic") {
        let Some(coefficients) = op.params.fourier_coefficients.get("coefficients") else {
            return 0;
        };

        for coord in operands {
            coord[1] = ellps.latitude_authalic_to_geographic(coord[1], coefficients);
            successes += 1;
        }
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 7] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Flag { key: "geocentric" },
    OpParameter::Flag { key: "reduced" },
    OpParameter::Flag { key: "conformal" },
    OpParameter::Flag { key: "authalic" },
    OpParameter::Flag { key: "rectifying" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") }
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let mut op = Op::plain(parameters, InnerOp(fwd), InnerOp(inv), &GAMUT, ctx)?;
    let ellps = op.params.ellps[0];

    let mut number_of_flags = 0_usize;
    if op.params.boolean("geocentric") {
        number_of_flags += 1;
    }
    if op.params.boolean("reduced") || op.params.boolean("parametric") {
        number_of_flags += 1;
    }
    if op.params.boolean("conformal") {
        let coefficients = ellps.coefficients_for_conformal_latitude_computations();
        op.params
            .fourier_coefficients
            .insert("coefficients", coefficients);
        number_of_flags += 1;
    }
    if op.params.boolean("authalic") {
        let coefficients = ellps.coefficients_for_authalic_latitude_computations();
        op.params
            .fourier_coefficients
            .insert("coefficients", coefficients);
        number_of_flags += 1;
    }
    if op.params.boolean("rectifying") {
        let coefficients = ellps.coefficients_for_rectifying_latitude_computations();
        op.params
            .fourier_coefficients
            .insert("coefficients", coefficients);
        number_of_flags += 1;
    }
    if number_of_flags != 1 {
        return Err(Error::MissingParam("latitude: must specify exactly one of flags authalic/conformal/geocentric/rectifying/reduced/parametric".to_string()));
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
        let mut operands = [Coord::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.818_973_308_324_5).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // Reduced (alias parametric)
        let op = ctx.op("latitude reduced ellps=GRS80")?;
        let mut operands = [Coord::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.909_538_187_092_245).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // Conformal
        let op = ctx.op("latitude conformal ellps=GRS80")?;
        let mut operands = [Coord::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.819_109_023_689_023_275).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // Rectifying
        let op = ctx.op("latitude rectifying ellps=GRS80")?;
        let mut operands = [Coord::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.772_351_809_646_84).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        // Authalic
        let op = ctx.op("latitude authalic ellps=GRS80")?;
        let mut operands = [Coord::geo(55., 12., 0., 0.)];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 54.879_361_594_517_796).abs() < 1e-12);
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][1].to_degrees() - 55.).abs() < 1e-12);

        Ok(())
    }
}
