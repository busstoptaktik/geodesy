/// Kinematic datum shift using a 3D deformation model in ENU-space.
///
/// Based on Kristian Evers' implementation of the
/// [corresponding PROJ operator](https://github.com/OSGeo/PROJ/blob/effac63ae5360e737790defa5bdc3d070d19a49b/src/transformations/deformation.cpp).
///
/// The deformation operation takes cartesian coordinates as input and
/// yields cartesian coordinates as output. The deformation model is
/// assumed to come from a 3 channel grid of deformation velocities,
/// with the grid georeference given as geographical coordinates in a
/// compatible frame.
///
/// #### The Deformation
///
/// The deformation expressed by the grid is given in the local
/// east-north-up (ENU) frame. It is converted to the cartesian XYZ
/// frame when applied to the input coordinates.
///
/// The total deformation at the position P: (X, Y, Z), at the time T1 is
/// given by:
///
/// |         DX(X, Y, Z) = (T1 - T0) * Vx(φ, λ)
/// |   (1)   DY(X, Y, Z) = (T1 - T0) * Vy(φ, λ)
/// |         DZ(X, Y, Z) = (T1 - T0) * Vz(φ, λ)
///
/// where:
///
/// - (X, Y, Z) is the cartesian coordinates of P
///
/// - (DX, DY, DZ) is the deformation along the cartesian earth centered
///   axes of the input frame
///
/// - (Vx, Vy, Vz) is the deformation velocity vector (m/year), obtained
///   from interpolation in the model grid, and converted from the local
///   ENU frame, to the global, cartesian XYZ frame
///
/// - (φ, λ) is the latitude and longitude, i.e. the grid coordinates,
///   of P, computed from its cartesian coordinates (X, Y, Z)
///
/// - T0 is the frame epoch of the kinematic reference frame associated
///   with the deformation model.
///
/// - T1 is the observation epoch of the input coordinate tuple (X, Y, Z)
///
/// #### The transformation
///
/// While you may obtain the deformation vector and its Euclidean norm
/// by specifying the `raw` option, that is not the primary use case for
/// the `deformation` operator. Rather, the primary use case is to *apply*
/// the deformation to the input coordinates and return the deformed
/// coordinates. Naively, but incorrectly, we may write this as
///
/// |         X'   =   X + DX   =   X + (T1 - T0) * Vx(φ, λ)
/// |   (2)   Y'   =   Y + DY   =   Y + (T1 - T0) * Vy(φ, λ)
/// |         Z'   =   Z + DZ   =   Z + (T1 - T0) * Vz(φ, λ)
///
/// Where (X, Y, Z) is the *observed* coordinate tuple, and (X', Y', Z')
/// is the same tuple after applying the deformation. While formally
/// correct, this is not the operation we intend to carry out. Neither
/// are the names used for the two types of coordinates fully useful
/// for understanding what goes on.
///
/// Rather, when we transform a set of observations, we want to obtain the
/// position of P at the time T0, i.e. at the *epoch* of the deforming
/// frame. In other words, we want to remove the deformation effect such
/// that *no matter when* we go and re-survey a given point, we will always
/// obtain the same coordinate tuple, after transforming the observed
/// coordinates back in time to the frame epoch. Hence, for the forward
/// transformation we must *remove* the effect of the deformation by negating
/// the sign of the deformation terms in eq. 2:
///
/// |         X'   =   X - DX   =   X - (T1 - T0) * Vx(φ, λ)
/// |   (3)   Y'   =   Y - DY   =   Y - (T1 - T0) * Vy(φ, λ)
/// |         Z'   =   Z - DZ   =   Z - (T1 - T0) * Vz(φ, λ)
///
/// In order to be able to discuss the remaining intricacies of the task, we
/// now introduce the designations *observed coordinates* for (X, Y, Z), and
/// *canonical coordinates* for (X', Y', Z').
///
/// What we want to do is to compute the canonical coordinates given the
/// observed ones, by applying a correction based on the deformation grid.
///
/// The deformation grid is georeferenced with respect to the *canonical system*
/// (this is necessary, since the deforming system changes as time goes).
///
/// But we cannot *observe* anything with respect to the canonical system:
/// It represents the world as it was at the epoch of the system. So the observed
/// coordinates are given in a system slightly different from the canonical.
///
/// The deformation model makes it possible to *predict* the coordinates we will
/// observe at any given time, for any given point that was originally observed
/// at the epoch of the system.
///
/// But we are really more interested in the opposite: To look back in time and
/// figure out "what were the coordinates at time T0, of the point P, which we
/// *actually observed at time T1*".
///
/// But since the georefererence of the deformation grid is given in the canonical
/// system, we actually need to know the canonical coordinates already in order to
/// look up the deformation needed to convert the observed coordinates to the
/// canonical, leaving us with a circular dependency ("to understand recursion, we
/// must first understand recursion").
///
/// To solve this, we do not actually need recursion - there is a perfectly
/// fine solution based on iteration, which is widely used in the inverse case
/// of plain 2D grid based datum shifts (whereas here, we need it in the forward
/// case).
///
/// There is however an even simpler solution to the problem - simply to ignore it.
/// The deformations are typically so small compared to the grid node distance,
/// that the iterative correction is way below the accuracy of the transformation
/// grid information, so we may simply look up in the grid using the observed
/// coordinates, and correct the same coordinates with the correction obtained
/// from the grid.
///
/// For now, this is the solution implemented here.
use crate::authoring::*;

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grids = &op.params.grids;
    let mut successes = 0_usize;
    let n = operands.len();

    let dt = op.params.real("dt").unwrap();
    let epoch = op.params.real("t_epoch").unwrap();
    let ellps = op.params.ellps(0);
    let raw = op.params.boolean("raw");
    let use_null_grid = op.params.boolean("null_grid");

    // Datum shift
    'points: for i in 0..n {
        let cart = operands.get_coord(i);
        let geo = ellps.geographic(&cart);
        for margin in [0.0, 0.5] {
            for grid in grids.iter() {
                // Interpolated deformation velocity
                if let Some(v) = grid.at(None, &geo, margin) {
                    // The deformation duration may be given either as a fixed duration or
                    // as the difference between the frame epoch and the observation epoch
                    let d = if dt.is_finite() { dt } else { epoch - geo[3] };

                    let deformation =
                        rotate_and_integrate_velocity(v.scale(-1.), geo[0], geo[1], d);

                    // Finally apply the deformation to the input coordinate - or just
                    // provide the raw correction if that was what was requested
                    if raw {
                        let mut deformation_with_length = deformation;
                        deformation_with_length[3] = deformation.dot(deformation).sqrt();
                        operands.set_coord(i, &deformation_with_length);
                    } else {
                        operands.set_coord(i, &(cart + deformation));
                    }
                    successes += 1;

                    // We've found the grid that contains the point, so we can move on
                    continue 'points;
                }
            }
        }

        if use_null_grid {
            successes += 1;
            continue;
        }

        // No grid found so we stomp on the coordinate
        operands.set_coord(i, &Coor4D::nan());
    }
    successes
}

// ----- I N V E R S E --------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let grids = &op.params.grids;
    let mut successes = 0_usize;
    let n = operands.len();

    let dt = op.params.real("dt").unwrap();
    let epoch = op.params.real("t_epoch").unwrap();
    let ellps = op.params.ellps(0);
    let raw = op.params.boolean("raw");
    let use_null_grid = op.params.boolean("null_grid");

    // Datum shift
    'points: for i in 0..n {
        let cart = operands.get_coord(i);
        let geo = ellps.geographic(&cart);
        for margin in [0.0, 0.5] {
            for grid in grids.iter() {
                // Interpolated deformation velocity
                if let Some(v) = grid.at(None, &geo, margin) {
                    // The deformation duration may be given either as a fixed duration or
                    // as the difference between the frame epoch and the observation epoch
                    let d = if dt.is_finite() { dt } else { epoch - geo[3] };

                    let deformation = rotate_and_integrate_velocity(v, geo[0], geo[1], d);

                    // Finally apply the deformation to the input coordinate - or just
                    // provide the raw correction if that was what was requested
                    if raw {
                        let mut deformation_with_length = deformation;
                        deformation_with_length[3] = deformation.dot(deformation).sqrt();
                        operands.set_coord(i, &deformation_with_length);
                    } else {
                        operands.set_coord(i, &(cart + deformation));
                    }
                    successes += 1;

                    // We've found the grid that contains the point, so we can move on
                    continue 'points;
                }
            }
        }

        if use_null_grid {
            successes += 1;
            continue;
        }

        // No grid found so we stomp on the coordinate
        operands.set_coord(i, &Coor4D::nan());
    }
    successes
}

// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 7] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Flag { key: "raw" },
    OpParameter::Texts { key: "grids",   default: None },
    OpParameter::Real { key: "padding", default: Some(0.5) },
    OpParameter::Real { key: "dt",      default: Some(f64::NAN) },
    OpParameter::Real { key: "t_epoch", default: Some(f64::NAN) },
    OpParameter::Text { key: "ellps",   default: Some("GRS80") },
];

pub fn new(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.instantiated_as;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    if params.real("dt")?.is_nan() && params.real("t_epoch")?.is_nan() {
        return Err(Error::MissingParam(
            "- either t_epoch or dt must be given".to_string(),
        ));
    }

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
            Ok(grid) => {
                let n = grid.bands();
                if n != 3 {
                    return Err(Error::Unexpected {
                        message: "Bad dimensionality of deformation model grid".to_string(),
                        expected: "3".to_string(),
                        found: n.to_string(),
                    });
                }
                params.grids.push(grid);
            }

            Err(e) => {
                if !optional {
                    return Err(e);
                }
            }
        }
    }

    let fwd = InnerOp(fwd);
    let inv = InnerOp(inv);
    let descriptor = OpDescriptor::new(def, fwd, Some(inv));

    Ok(Op {
        descriptor,
        params,
        steps: None,
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
        let mut ctx = Plain::default();
        let cph = Coor4D::geo(55., 12., 0., 0.);
        let test_deformation = include_str!("../../geodesy/deformation/test.deformation");
        let another_test_deformation =
            include_str!("../../geodesy/deformation/another_test.deformation");

        // Check that grid registration works
        ctx.register_resource("test.deformation", test_deformation);
        ctx.register_resource("another_test.deformation", another_test_deformation);

        let buf = ctx.get_blob("test.deformation")?;
        let grid = BaseGrid::gravsoft("test_deformation", &buf)?;

        // Velocity in the ENU space
        let v = grid.at(None, &cph, 0.0).unwrap();
        // Which we rotate into the XYZ space and integrate for 1000 years - and mutate into Coor3D,
        // to remove the NaN from the 4th dimension, which make the tests crash, because NaN!=NaN
        let deformation = rotate_and_integrate_velocity(v, cph[0], cph[1], 1000.);

        // Check that the length of the deformation correction, expressed as the
        // Euclidean norm, is identical in the XYZ and the ENU space
        let expected_length_of_correction = (55f64 * 55. + 12. * 12.).sqrt();
        let length_of_scaled_velocity = v.scale(1000.0).dot(v.scale(1000.0)).sqrt();
        let length_of_rotated_deformation = deformation.dot(deformation).sqrt();
        dbg!(length_of_rotated_deformation);
        dbg!(expected_length_of_correction);
        dbg!(length_of_scaled_velocity);
        dbg!(deformation);
        dbg!(v);
        assert!((length_of_scaled_velocity - expected_length_of_correction).abs() < 1e-6);
        assert!((length_of_rotated_deformation - expected_length_of_correction).abs() < 1e-6);

        // Now do the same in the plain ol' way, checking that the operator
        // works identically to the hand held incantations above
        let op = ctx.op("deformation dt=1000 grids=test.deformation")?;
        // Create a test data point in the cartesian space
        let ellps = Ellipsoid::default();
        let cph = ellps.cartesian(&cph);

        // Check the length of the correction after a forward step
        let mut data = [cph];
        ctx.apply(op, Fwd, &mut data)?;
        let diff = data[0] - cph;
        let length_of_diff = diff.dot(diff).sqrt();
        dbg!(length_of_diff);
        assert!((length_of_diff - expected_length_of_correction).abs() < 1e-6);

        // Check the length of the correction after an inverse step
        let mut data = [cph];
        ctx.apply(op, Inv, &mut data)?;
        let diff = data[0] - cph;
        let length_of_diff = diff.dot(diff).sqrt();
        dbg!(length_of_diff);
        dbg!(expected_length_of_correction);
        dbg!(data[0]);
        dbg!(cph);
        assert!((length_of_diff - expected_length_of_correction).abs() < 1e-6);

        // Check the accuracy of a roundtrip step. Consider improving the accuracy by
        // implementing iterative lookup in the forward direction.
        let mut data = [cph];
        ctx.apply(op, Fwd, &mut data)?;
        ctx.apply(op, Inv, &mut data)?;
        dbg!(cph);
        dbg!(data[0]);
        assert!(cph.hypot3(&data[0]) < 1e-3);

        // Check the "raw" functionality
        let op =
            ctx.op("deformation raw dt=1000 grids=@another_test.deformation,test.deformation")?;

        // Forward direction
        let mut data = [cph];
        ctx.apply(op, Fwd, &mut data)?;
        let fwd = data[0];
        dbg!(fwd);
        assert!((fwd[3] - expected_length_of_correction) < 0.001);

        // and inverse direction
        let mut data = [cph];
        ctx.apply(op, Inv, &mut data)?;
        let inv = data[0];
        dbg!(inv);
        assert!((inv[3] - expected_length_of_correction) < 0.001);
        assert!((inv[3] - fwd[3]) < 0.001);

        // The Finnish town of Tornio is inside the "another_test" grid
        let tio = Coor4D::geo(65.85, 24.10, 0., 0.);
        let tio = ellps.cartesian(&tio);
        let mut data = [tio];
        ctx.apply(op, Fwd, &mut data)?;
        let fwd = data[0];
        dbg!(fwd);
        assert!(fwd[0].is_finite());

        // The Norwegian town of Longyearbyen is outside of both grids
        let lyb = Coor4D::geo(78.25, 15.5, 0., 0.);
        let lyb = ellps.cartesian(&lyb);
        let mut data = [lyb];
        ctx.apply(op, Fwd, &mut data)?;
        assert!(data[0][0].is_nan());

        Ok(())
    }
}
