/// Geodesics
use crate::authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);

    let n = operands.len();
    let sliced = 0..n;

    let mut successes = 0_usize;
    for i in sliced {
        let args = operands.get_coord(i);
        let origin = Coor4D::geo(args[0], args[1], 0.0, 0.0);
        let azimuth = args[2].to_radians();
        let distance = args[3];

        let destination = ellps.geodesic_fwd(&origin, azimuth, distance).to_degrees();

        // No convergence?
        if destination[3] > 990.0 {
            operands.set_coord(i, &Coor4D::nan());
            continue;
        }

        let result = Coor4D([destination[1], destination[0], args[0], args[1]]);
        operands.set_coord(i, &result);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);
    let reversible = op.params.boolean("reversible");

    let n = operands.len();
    let sliced = 0..n;

    let mut successes = 0_usize;
    for i in sliced {
        let mut from = Coor4D::origin();
        let mut to = Coor4D::origin();

        let coord = operands.get_coord(i);
        from[0] = coord[1].to_radians();
        from[1] = coord[0].to_radians();
        to[0] = coord[3].to_radians();
        to[1] = coord[2].to_radians();

        let mut geodesic = ellps.geodesic_inv(&from, &to).to_degrees();

        // No convergence?
        if geodesic[3] > 990.0 {
            operands.set_coord(i, &Coor4D::nan());
            continue;
        }
        geodesic[3] = (geodesic[1] + 180.0) % 360.0;

        let distance = geodesic[2];
        let return_azi = geodesic[3];

        if reversible {
            operands.set_coord(i, &Coor4D::raw(coord[2], coord[3], return_azi, distance));
            continue;
        }

        operands.set_coord(i, &geodesic);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 3] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Flag { key: "reversible" },
    OpParameter::Text { key: "ellps", default: Some("GRS80") }
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let op = Op::plain(parameters, InnerOp(fwd), Some(InnerOp(inv)), &GAMUT, ctx)?;
    Ok(op)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geodesic() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        // Approximate coordinates of Copenhagen and Paris airports
        let cph_cdg = Coor4D::raw(55., 12., 49., 2.);

        // A geodesic from Copenhagen to Paris
        let op = ctx.op("geodesic")?;
        let mut operands = [cph_cdg];
        ctx.apply(op, Inv, &mut operands)?;

        let expected = Coor4D([
            -130.1540604203936,
            -138.05257941840648,
            956066.2319619625,
            41.94742058159352,
        ]);

        assert!((operands[0][0] - expected[0]).abs() < 1e-9);
        assert!((operands[0][1] - expected[1]).abs() < 1e-9);
        assert!((operands[0][2] - expected[2]).abs() < 1e-9);
        assert!((operands[0][3] - expected[3]).abs() < 1e-9);

        // A geodesic from Copenhagen to Paris in the "reversible" format
        let op = ctx.op("geodesic reversible")?;
        let mut operands = [cph_cdg];
        ctx.apply(op, Inv, &mut operands)?;

        let expected = Coor4D([49.0, 2.0, 41.94742058159352, 956066.2319619625]);

        assert!((operands[0][0] - expected[0]).abs() < 1e-9);
        assert!((operands[0][1] - expected[1]).abs() < 1e-9);
        assert!((operands[0][2] - expected[2]).abs() < 1e-9);
        assert!((operands[0][3] - expected[3]).abs() < 1e-9);

        // And back to Copenhagen!
        ctx.apply(op, Fwd, &mut operands)?;

        assert!((operands[0][0] - cph_cdg[0]).abs() < 1e-10);
        assert!((operands[0][1] - cph_cdg[1]).abs() < 1e-10);
        assert!((operands[0][2] - cph_cdg[2]).abs() < 1e-10);
        assert!((operands[0][3] - cph_cdg[3]).abs() < 1e-10);

        Ok(())
    }
}
