#![allow(non_snake_case)]
/// The Helmert transform performs reference frame shifts. It operates in the 3D cartesian
/// space, transforming 3D cartesian coordinates between static and/or dynamic reference
/// frames, e.g. from global reference frames to local static frames.
///
/// While generally also applicable to 2D coordinates, this functionality is not yet
/// implemented.
use crate::operator_authoring::*;

// ----- C O M M O N -------------------------------------------------------------------

// The forward and inverse implementations are virtually identical, so we combine them
// into one, with the functionality selected from the "direction" parameter.

fn helmert_common(
    op: &Op,
    _ctx: &dyn Context,
    operands: &mut dyn CoordinateSet,
    direction: Direction,
) -> usize {
    // Translation, Rotation, Scale
    let T = op.params.series("T").unwrap();
    let R = op.params.series("R").unwrap();
    let S = op.params.real("S").unwrap();

    // ... and their time evolution
    let DT = op.params.series("DT").unwrap();
    let DR = op.params.series("DR").unwrap();
    let DS = op.params.real("DS").unwrap();

    // The precomputed rotation matrix
    let M = op.params.series("ROTFLAT").unwrap();
    let mut ROT = [[M[0], M[1], M[2]], [M[3], M[4], M[5]], [M[6], M[7], M[8]]];

    // Predicates
    let rotated = op.params.boolean("rotated");
    let dynamic = op.params.boolean("dynamic");
    let fixed_t = op.params.boolean("fixed_time");
    let exact = op.params.boolean("exact");
    let position_vector = op.params.boolean("position_vector");

    let epoch = op.params.real("t_epoch").unwrap_or(0.);

    let mut TT = [T[0], T[1], T[2]];
    let mut SS = S;

    let mut prev_t = std::f64::NAN;
    let n = operands.len();
    for i in 0..n {
        let mut c = operands.get_coord(i);

        // Time varying case?
        if dynamic && !fixed_t {
            // Necessary to update parameters?
            #[allow(clippy::float_cmp)]
            if c[3] != prev_t {
                prev_t = c[3];
                let dt = c[3] - epoch;
                TT[0] += dt * DT[0];
                TT[1] += dt * DT[1];
                TT[2] += dt * DT[2];
                if rotated {
                    let RR = [R[0] + dt * DR[0], R[1] + dt * DR[1], R[2] + dt * DR[2]];
                    ROT = rotation_matrix(&RR, exact, position_vector);
                }
                SS = S + dt * DS;
            }
        }

        // ----- Forward direction -----

        if direction == Direction::Fwd {
            if rotated {
                // Rotate
                let x = c[0] * ROT[0][0] + c[1] * ROT[0][1] + c[2] * ROT[0][2];
                let y = c[0] * ROT[1][0] + c[1] * ROT[1][1] + c[2] * ROT[1][2];
                let z = c[0] * ROT[2][0] + c[1] * ROT[2][1] + c[2] * ROT[2][2];

                // scale and offset
                c[0] = SS * x + TT[0];
                c[1] = SS * y + TT[1];
                c[2] = SS * z + TT[2];
                operands.set_coord(i, &c);
                continue;
            }

            // scale and offset without rotation
            c[0] = SS * c[0] + TT[0];
            c[1] = SS * c[1] + TT[1];
            c[2] = SS * c[2] + TT[2];
            operands.set_coord(i, &c);
            continue;
        }

        // ----- Inverse direction -----

        // Deoffset and unscale
        let x = (c[0] - TT[0]) / SS;
        let y = (c[1] - TT[1]) / SS;
        let z = (c[2] - TT[2]) / SS;

        // Inverse rotation by transposed multiplication
        if rotated {
            c[0] = x * ROT[0][0] + y * ROT[1][0] + z * ROT[2][0];
            c[1] = x * ROT[0][1] + y * ROT[1][1] + z * ROT[2][1];
            c[2] = x * ROT[0][2] + y * ROT[1][2] + z * ROT[2][2];
        } else {
            c[0] = x;
            c[1] = y;
            c[2] = z;
        }
        operands.set_coord(i, &c);
    }
    n
}

// ----- F O R W A R D --------------------------------------------------------------

fn helmert_fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    helmert_common(op, _ctx, operands, Direction::Fwd)
}

// ----- I N V E R S E --------------------------------------------------------------

fn helmert_inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    helmert_common(op, _ctx, operands, Direction::Inv)
}

// ----- C O N S T R U C T O R ------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 19] = [
    OpParameter::Flag { key: "inv" },

    // Translation
    OpParameter::Real { key: "x", default: Some(0f64) },
    OpParameter::Real { key: "y", default: Some(0f64) },
    OpParameter::Real { key: "z", default: Some(0f64) },

    // Time evolution of translation
    OpParameter::Real { key: "dx", default: Some(0f64) },
    OpParameter::Real { key: "dy", default: Some(0f64) },
    OpParameter::Real { key: "dz", default: Some(0f64) },

    // Rotation
    OpParameter::Real { key: "rx", default: Some(0f64) },
    OpParameter::Real { key: "ry", default: Some(0f64) },
    OpParameter::Real { key: "rz", default: Some(0f64) },

    // Time evolution of rotation
    OpParameter::Real { key: "drx", default: Some(0f64) },
    OpParameter::Real { key: "dry", default: Some(0f64) },
    OpParameter::Real { key: "drz", default: Some(0f64) },

    // Handling of rotation
    OpParameter::Text { key: "convention", default: Some("") },
    OpParameter::Flag { key: "exact" },

    // Scale and its time evoution
    OpParameter::Real { key: "s",  default: Some(0f64) },
    OpParameter::Real { key: "ds", default: Some(0f64) },  // TODO: scale by 1e-6

    // Epoch - "beginning of time for this transformation"
    OpParameter::Real { key: "t_epoch", default: Some(std::f64::NAN) },

    // Fixed observation time - ignore the fourth coordinate.
    OpParameter::Real { key: "t_obs", default: Some(std::f64::NAN) },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;

    // Translation
    let x = params.real("x")?;
    let y = params.real("y")?;
    let z = params.real("z")?;
    let mut T = [x, y, z];

    // Time evolution of translation
    let dx = params.real("dx")?;
    let dy = params.real("dy")?;
    let dz = params.real("dz")?;
    let DT = [dx, dy, dz];

    // Rotation
    let rx = params.real("rx")?;
    let ry = params.real("ry")?;
    let rz = params.real("rz")?;
    let mut R = [
        (rx / 3600.).to_radians(),
        (ry / 3600.).to_radians(),
        (rz / 3600.).to_radians(),
    ];

    // Time evolution of rotation
    let drx = params.real("drx")?;
    let dry = params.real("dry")?;
    let drz = params.real("drz")?;
    let DR = [
        (drx / 3600.).to_radians(),
        (dry / 3600.).to_radians(),
        (drz / 3600.).to_radians(),
    ];

    // Handling of rotations: position vector vs. coordinate frame conventions.
    let convention = params.text("convention")?;
    let rotated = !(R == [0., 0., 0.] && DR == [0., 0., 0.]);
    let mut position_vector = true;
    if rotated {
        if !["position_vector", "coordinate_frame"].contains(&convention.as_str()) {
            return Err(Error::BadParam("convention".to_string(), convention));
        }
        if "coordinate_frame" == convention {
            position_vector = false;
        }
        params.boolean.insert("rotated");
    }
    if position_vector {
        params.boolean.insert("position_vector");
    }

    // Scale and its time evolution
    let mut S = 1.0 + params.real("s")? * 1e-6;
    let DS = params.real("ds")? * 1e-6;

    let dynamic = !(DT == [0., 0., 0.] && DR == [0., 0., 0.] && DS == 0.);
    if dynamic {
        params.boolean.insert("dynamic");

        // Check that epoch - "beginning of time for this transformation" is given
        let epoch = params.real("t_epoch")?;
        if epoch.is_nan() {
            return Err(Error::MissingParam("t_epoch".to_string()));
        }

        // Fixed observation time - ignore the fourth coordinate and just compute
        // the transformation matrix once
        if let Ok(t_obs) = params.real("t_obs") {
            if !t_obs.is_nan() {
                params.boolean.insert("fixed_time");
                for i in 0..3_usize {
                    T[i] += DT[i] * (t_obs - epoch);
                    R[i] += DR[i] * (t_obs - epoch);
                    S += DS * (t_obs - epoch);
                }
            }
        }
    }

    let exact = params.boolean("exact");
    params.series.insert("T", Vec::from(T));
    params.series.insert("DT", Vec::from(DT));
    params.series.insert("R", Vec::from(R));
    params.series.insert("DR", Vec::from(DR));
    params.real.insert("S", S);
    params.real.insert("DS", DS);

    // The rotation matrix is a 3x3 symmetric matrix
    let ROT = rotation_matrix(&R, exact, position_vector);

    // We need to turn the 3x3 into 1x9 to make it fit into the "series" store
    let mut ROTFLAT = Vec::from(ROT[0]);
    ROTFLAT.extend(ROT[1].iter());
    ROTFLAT.extend(ROT[2].iter());
    assert_eq!(ROTFLAT.len(), 9);
    params.series.insert("ROTFLAT", ROTFLAT);

    let fwd = InnerOp(helmert_fwd);
    let inv = InnerOp(helmert_inv);
    let descriptor = OpDescriptor::new(def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    let id = OpHandle::new();
    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// ----- R O T A T I O N   M A T R I X   B U I L D E R ---------------------------------

// Based on Karsten Engsager's implementation in set_dtm_1.c (trlib),
// but adds optional small-angle approximation, and selection between
// the "position vector" and "coordinate frame" rotation conventions.
//
// TO' = scale * [ROTZ * ROTY * ROTX] * FROM' + [translation x, y, z]'
//
//        | cz sz 0 |           | cy 0 -sy |           | 1   0  0 |
// ROTZ = |-sz cz 0 |,   ROTY = | 0  1   0 |,   ROTX = | 0  cx sx |
//        |  0  0 1 |           | sy 0  cy |           | 0 -sx cx |
//
fn rotation_matrix(r: &[f64], exact: bool, position_vector: bool) -> [[f64; 3]; 3] {
    let (rx, ry, rz) = (r[0], r[1], r[2]);

    // Small-angle approximations: sx = sin(rx) = rx,  cx = cos(rx) = 1,  etc.
    let (mut sx, mut sy, mut sz) = (rx, ry, rz);
    let (mut cx, mut cy, mut cz) = (1.0, 1.0, 1.0);

    // Leave out the second order infinitesimals in the rotation
    // matrix elements, when using small-angle approximations
    if exact {
        (sx, cx) = rx.sin_cos();
        (sy, cy) = ry.sin_cos();
        (sz, cz) = rz.sin_cos();
    }

    let r11 = cy * cz;
    let mut r12 = cx * sz;
    let mut r13 = -cx * sy * cz;

    let r21 = -cy * sz;
    let mut r22 = cx * cz;
    let mut r23 = sx * cz;

    let r31 = sy;
    let r32 = -sx * cy;
    let r33 = cx * cy;

    // But apply the second order terms in the exact case
    if exact {
        r12 += sx * sy * cz;
        r13 += sx * sz;

        r22 -= sx * sy * sz;
        r23 += cx * sy * sz;
    }

    if position_vector {
        return [[r11, r21, r31], [r12, r22, r32], [r13, r23, r33]];
    }
    [[r11, r12, r13], [r21, r22, r23], [r31, r32, r33]]
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    const GDA94: Coor4D = Coor4D([-4052051.7643, 4212836.2017, -2545106.0245, 0.0]);
    const GDA2020A: Coor4D = Coor4D([-4052052.7379, 4212835.9897, -2545104.5898, 0.0]);
    const GDA2020B: Coor4D = Coor4D([-4052052.7373, 4212835.9835, -2545104.5867, 2020.0]);
    const ITRF2014: Coor4D = Coor4D([-4052052.6588, 4212835.9938, -2545104.6946, 2018.0]);

    #[test]
    fn translation() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("helmert x=-87 y=-96 z=-120")?;

        // EPSG:1134 - 3 parameter, ED50/WGS84, s = sqrt(27) m
        let mut operands = [Coor4D::origin()];

        ctx.apply(op, Fwd, &mut operands)?;
        assert_eq!(operands[0][0], -87.);
        assert_eq!(operands[0][1], -96.);
        assert_eq!(operands[0][2], -120.);

        ctx.apply(op, Inv, &mut operands)?;
        assert_eq!(operands[0][0], 0.);
        assert_eq!(operands[0][1], 0.);
        assert_eq!(operands[0][2], 0.);
        Ok(())
    }

    // Test case from "Intergovernmental Committee on Surveying and Mapping (ICSM)
    // Permanent Committee on Geodesy (PCG)": Geocentric Datum of Australia 2020,
    // Technical Manual Version 1.0, 25 July 2017.
    // Transformation from GDA94 to GDA2020.
    #[test]
    fn translation_rotation_and_scale() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "
            helmert convention = coordinate_frame
            x =  0.06155  rx = -0.0394924
            y = -0.01087  ry = -0.0327221
            z = -0.04019  rz = -0.0328979
            s = -0.009994 exact
        ";
        let op = ctx.op(definition)?;

        // The forward transformation should hit closer than 75 um
        let mut operands = [GDA94];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!(GDA2020A.hypot3(&operands[0]) < 75e-6);

        // ... and an even better roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        assert!(GDA94.hypot3(&operands[0]) < 75e-7);

        Ok(())
    }

    // A time varying example from the same source: ITRF2014@2018 to GDA2020,
    // Test point ALIC (Alice Springs)
    #[test]
    fn dynamic() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "
            helmert  exact    convention = coordinate_frame
            drx = 0.00150379  dry = 0.00118346  drz = 0.00120716
            t_epoch = 2020.0
        ";
        let op = ctx.op(definition)?;

        // The forward transformation should hit closeer than 40 um
        let mut operands = [ITRF2014];
        ctx.apply(op, Fwd, &mut operands)?;
        assert!(GDA2020B.hypot3(&operands[0]) < 40e-6);

        // ... and even closer on the way back
        ctx.apply(op, Inv, &mut operands)?;
        assert!(ITRF2014.hypot3(&operands[0]) < 40e-8);

        Ok(())
    }

    // Same as above, but with fixed time `t_obs` option
    #[test]
    fn fixed_dynamic() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "
            helmert  exact    convention = coordinate_frame
            drx = 0.00150379  dry = 0.00118346  drz = 0.00120716
            t_epoch = 2020.0  t_obs = 2018
        ";
        let mut operands = [ITRF2014];
        operands[0][3] = 2030.;

        let op = ctx.op(definition)?;
        ctx.apply(op, Fwd, &mut operands)?;
        assert!(GDA2020B.hypot3(&operands[0]) < 40e-6);
        ctx.apply(op, Inv, &mut operands)?;
        assert!(ITRF2014.hypot3(&operands[0]) < 40e-8);

        Ok(())
    }
}
