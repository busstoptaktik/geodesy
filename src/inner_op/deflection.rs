/// Estimate deflection of the vertical from a geoid model.
/// Mostly for manual look-ups, so it takes input in degrees and conventional
/// nautical latitude-longitude order, and provides output in arcsec in the
/// corresponding (ξ, η) order.
///
/// Note that this is mostly for order-of-magnitude considerations:
/// Typically observations of deflections of the vertical are input
/// data for geoid determination, not the other way round, as here.
use crate::authoring::*;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grids = &op.params.grids;
    let ellps = op.params.ellps(0);

    let mut successes = 0_usize;
    let n = operands.len();

    // Nothing to do?
    if grids.is_empty() {
        return n;
    }

    for i in 0..n {
        let mut coord = operands.get_coord(i);
        let lat = coord[0].to_radians();
        let lon = coord[1].to_radians();
        coord[0] = lon;
        coord[1] = lat;

        // The latitude step corresponding to a 1 m linear step along the local meridian
        let lat_dist = ellps.meridian_latitude_to_distance(lat);
        let dlat = ellps.meridian_distance_to_latitude(lat_dist + 1.0) - lat;

        // The longitude step corresponding to a 1 m linear step along the local parallel
        let dlon = (lat.cos() * ellps.prime_vertical_radius_of_curvature(lat)).recip();

        let Some(origin) = grids_at(grids, &coord, false) else {
            operands.set_coord(i, &Coor4D::nan());
            continue;
        };

        coord[1] += dlat;
        let Some(lat_1) = grids_at(grids, &coord, false) else {
            operands.set_coord(i, &Coor4D::nan());
            continue;
        };
        coord[1] = lat;
        coord[0] += dlon;
        let Some(lon_1) = grids_at(grids, &coord, false) else {
            operands.set_coord(i, &Coor4D::nan());
            continue;
        };

        coord[0] = (lat_1[0] - origin[0]).atan2(1.0); // xi
        coord[1] = (lon_1[0] - origin[0]).atan2(1.0); // eta
        operands.set_coord(i, &coord.to_arcsec());
        successes += 1;
    }
    successes
}

// ----- C O N S T R U C T O R ------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Texts { key: "grids", default: None },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    for mut grid_name in params.texts("grids")?.clone() {
        let optional = grid_name.starts_with('@');
        if optional {
            grid_name = grid_name.trim_start_matches('@').to_string();
        }

        if grid_name == "null" {
            params.boolean.insert("null_grid");
            break; // ignore any additional grids after a null grid
        }

        match ctx.get_grid(&grid_name) {
            Ok(grid) => params.grids.push(grid),
            Err(e) => {
                if !optional {
                    return Err(e);
                }
            }
        }
    }

    let fwd = InnerOp(fwd);
    let descriptor = OpDescriptor::new(def, fwd, None);
    let steps = Vec::new();
    let id = OpHandle::new();

    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// ----- T E S T S ------------------------------------------------------------------

//#[cfg(with_plain)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deflection() -> Result<(), Error> {
        let mut ctx = Plain::default();
        let op = ctx.op("deflection grids=test.geoid")?;
        let cph = Coor4D::raw(55., 12., 0., 0.);
        let mut data = [cph];

        ctx.apply(op, Fwd, &mut data)?;
        assert!((data[0][0] - 1.8527755901425906).abs() < 1e-6);
        assert!((data[0][1] - 0.032238719594433175).abs() < 1e-6);
        Ok(())
    }
}

// See additional tests in src/grid/mod.rs
