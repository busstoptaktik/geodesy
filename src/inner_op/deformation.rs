/// Kinematic datum shift using a 3D deformation model in ENU-space.
/// Based on Kristian Evers' implementation of the PROJ operator
/// `proj=deformation`.
///
/// The deformation operation takes cartesian coordinates as input and
/// yields cartesian coordinates as output.
///
/// The output is given by:
///
/// >  X' = X + (T1 - T0) * Vx
/// >  Y' = Y + (T1 - T0) * Vy
/// >  Z' = Z + (T1 - T0) * Vz
///
/// where:
///
/// - (X', Y', Z') is the result of the operation
///
/// - (X, Y, Z) is the input coordinate tuple
///
/// - T0 is the frame epoch of the dynamic reference frame associated
///   with the deformation model.
///
/// - T1 is the observation epoch of the input coordinate tuple (X, Y, Z)
///
/// - (Vx, Vy, Vz) is the deformation velocity vector (m/year)
///
/// Corrections in the gridded model are given in the east, north, up (ENU)
/// space. They are converted to the cartesian space before being applied
/// to the input coordinates.
use crate::operator_authoring::*;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grid = &op.params.grids["grid"];
    let mut successes = 0_usize;
    let n = operands.len();

    let dt = op.params.real("dt").unwrap();
    let epoch = op.params.real("t_epoch").unwrap();
    let ellps = op.params.ellps(0);

    // Datum shift
    for i in 0..n {
        let cart = operands.get_coord(i);
        let geo = ellps.geographic(&cart);

        // The deformation duration may be given either as a fixed duration or
        // as the difference between the frame epoch and the observation epoch
        let d = if dt.is_finite() { dt } else { epoch - geo[3] };

        // Interpolated deformation velocity
        let v = grid.interpolation(&geo, None);
        let deformation = rotate_and_integrate_velocity(v, geo[0], geo[1], d);

        // Outside of the grid? - stomp on the input coordinate and go on to the next
        if v[0].is_nan() {
            operands.set_coord(i, &Coor4D::nan());
            continue;
        }

        // Finally apply the deformation to the input coordinate
        operands.set_coord(i, &(cart + deformation));
        successes += 1;
    }
    successes
}

// ----- I N V E R S E --------------------------------------------------------------

//fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
//    let grid = &op.params.grids["grid"];
//    let mut successes = 0_usize;
//    let n = operands.len();
//
//    // Geoid
//    if grid.bands == 1 {
//        for i in 0..n {
//            let mut coord = operands.get_coord(i);
//            let t = grid.interpolation(&coord, None);
//            coord[2] += t[0];
//            operands.set_coord(i, &coord);
//            successes += 1;
//        }
//        return successes;
//    }
//
//    // Datum shift - here we need to iterate in the inverse case
//    for i in 0..n {
//        let coord = operands.get_coord(i);
//        let mut t = coord - grid.interpolation(&coord, None);
//
//        for _ in 0..10 {
//            let d = t - coord + grid.interpolation(&t, None);
//            t = t - d;
//            // i.e. d.dot(d).sqrt() < 1e-10
//            if d.dot(d) < 1e-20 {
//                break;
//            }
//        }
//
//        operands.set_coord(i, &t);
//        successes += 1;
//    }
//
//    successes
//}

// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 6] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "grids",   default: None },
    OpParameter::Real { key: "padding", default: Some(0.5) },
    OpParameter::Real { key: "dt",      default: Some(f64::NAN) },
    OpParameter::Real { key: "t_epoch", default: Some(f64::NAN) },
    OpParameter::Text { key: "ellps",   default: Some("GRS80") },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    if params.real("dt")?.is_nan() && params.real("t_epoch")?.is_nan() {
        return Err(Error::MissingParam(
            "- either t_epoch or dt must be given".to_string(),
        ));
    }

    let grid_file_name = params.text("grids")?;
    let buf = ctx.get_blob(&grid_file_name)?;

    let grid = Grid::gravsoft(&buf)?;
    let n = grid.bands;
    if n != 3 {
        return Err(Error::Unexpected {
            message: "Bad dimensionality of deformation model grid".to_string(),
            expected: "3".to_string(),
            found: n.to_string(),
        });
    }
    params.grids.insert("grid", grid);

    let fwd = InnerOp(fwd);
    // let inv = InnerOp(inv);
    let descriptor = OpDescriptor::new(def, fwd, None); //Some(inv));
    let steps = Vec::new();
    let id = OpHandle::new();

    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

// Rotate the deformation velocity from the ENU system to
// the geocentric cartesian system, and multiply by the
// deformation duration to obtain the total deformation
#[inline]
fn rotate_and_integrate_velocity(
    v: Coor4D,
    longitude: f64,
    latitude: f64,
    duration: f64,
) -> Coor4D {
    // First precompute the trigonometric constants
    let (slon, clon) = longitude.sin_cos();
    let (slat, clat) = latitude.sin_cos();

    // Then rotate the velocity vector and scale by the deformation
    // duration to obtain the total deformation
    Coor4D([
        duration * (-slat * clon * v[1] - slon * v[0] + clat * clon * v[2]),
        duration * (-slat * slon * v[1] + clon * v[0] + clat * slon * v[2]),
        duration * (clat * v[1] + slat * v[2]),
        0.0,
    ])
}

// ----- T E S T S ------------------------------------------------------------------

//#[cfg(with_plain)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deformation() -> Result<(), Error> {
        // Context and data
        let mut ctx = Minimal::default();
        let cph = Coor4D::geo(55., 12., 0., 0.);
        let test_deformation = include_str!("../../geodesy/deformation/test.deformation");

        // Check that grid registration works
        ctx.register_resource("test.deformation", test_deformation);

        let buf = ctx.get_blob("test.deformation")?;
        let grid = Grid::gravsoft(&buf)?;

        // Velocity in the ENU space
        let v = grid.interpolation(&cph, None);
        // Which we rotate into the XYZ space and integrate for 1000 years
        let deformation = rotate_and_integrate_velocity(v, cph[0], cph[1], 1000.);

        // Check that the length of the deformation correction, expressed as the
        // Euclidean norm, is identical in the XYZ and the ENU space
        let expected_length_of_correction = (55f64 * 55. + 12. * 12.).sqrt();
        let length_of_scaled_velocity = v.scale(1000.0).dot(v.scale(1000.0)).sqrt();
        let length_of_rotated_deformation = deformation.dot(deformation).sqrt();
        assert!((length_of_scaled_velocity - expected_length_of_correction).abs() < 1e-6);
        assert!((length_of_rotated_deformation - expected_length_of_correction).abs() < 1e-6);

        // Now do the same in the plain ol' way, checking that the operator
        // works identically to the hand held incantations above
        let op = ctx.op("deformation dt=1000 grids=test.deformation")?;
        // Create a test data point in the cartesian space
        let ellps = crate::Ellipsoid::default();
        let cph = ellps.cartesian(&cph);
        let mut data = [cph];

        ctx.apply(op, Fwd, &mut data)?;

        // Check the length of the correction
        let diff = data[0] - cph;
        let length_of_diff = diff.dot(diff).sqrt();
        assert!((length_of_diff - expected_length_of_correction).abs() < 1e-6);

        Ok(())
    }
}
