use super::*;

// Based on Karsten Engsager's implementation in set_dtm_1.c (trlib),
// but adds optional small angle approximation, and selection between
// the "position vector" and "coordinate frame" rotation conventions.
//
// TO' = scale * [ROTZ * ROTY * ROTX] * FROM' + [translation x, y, z]'
//
//        | cz sz 0 |           | cy 0 -sy |           | 1   0  0 |
// ROTZ = |-sz cz 0 |,   ROTY = | 0  1   0 |,   ROTX = | 0  cx sx |
//        |  0  0 1 |           | sy 0  cy |           | 0 -sx cx |
//
fn rotation_matrix(rx: f64, ry: f64, rz: f64, exact: bool, position_vector: bool) -> [[f64; 3]; 3] {
    // From seconds of arc to radians (TODO: should do this during construction)
    let rx = (rx / 3600.).to_radians();
    let ry = (ry / 3600.).to_radians();
    let rz = (rz / 3600.).to_radians();

    // Small angle approximations: sx = sin(rx) = rx,  cx = cos(rx) = 1,  etc.
    let (mut sx, mut sy, mut sz) = (rx, ry, rz);
    let (mut cx, mut cy, mut cz) = (1.0, 1.0, 1.0);
    if exact {
        (sx, cx) = rx.sin_cos();
        (sy, cy) = ry.sin_cos();
        (sz, cz) = rz.sin_cos();
    }

    // Leave out the second order infinitesimals in the rotation
    // matrix elements, when using small angle approximations
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


// ----- F O R W A R D --------------------------------------------------------------

fn addone_fwd(op: &Op, provider: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = 0;
    for o in operands {
        o[0] += 1.;
        n += 1;
    }
    n
}

// ----- I N V E R S E --------------------------------------------------------------

fn addone_inv(op: &Op, provider: &dyn Provider, operands: &mut [CoordinateTuple]) -> usize {
    let mut n = 0;
    for o in operands {
        o[0] -= 1.;
        n += 1;
    }
    n
}

// ----- C O N S T R U C T O R ------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 19] = [
    OpParameter::Flag { key: "inv" },

    // Translation
    OpParameter::Real { key: "x", default: 0f64 },
    OpParameter::Real { key: "y", default: 0f64 },
    OpParameter::Real { key: "z", default: 0f64 },

    // Time evolution of translation
    OpParameter::Real { key: "dx", default: 0f64 },
    OpParameter::Real { key: "dy", default: 0f64 },
    OpParameter::Real { key: "dz", default: 0f64 },

    // Rotation
    OpParameter::Real { key: "rx", default: 0f64 },
    OpParameter::Real { key: "ry", default: 0f64 },
    OpParameter::Real { key: "rz", default: 0f64 },

    // Time evolution of rotation
    OpParameter::Real { key: "drx", default: 0f64 },
    OpParameter::Real { key: "dry", default: 0f64 },
    OpParameter::Real { key: "drz", default: 0f64 },

    // Handling of rotation
    OpParameter::Text { key: "convention", default: "" },
    OpParameter::Flag { key: "exact" },

    // Scale and its time evoution
    OpParameter::Real { key: "s",  default: 0f64 },
    OpParameter::Real { key: "ds", default: 0f64 },  // TODO: scale by 1e-6

    // Epoch - "beginning of time for this transformation"
    OpParameter::Real { key: "epoch", default: std::f64::NAN },

    // Fixed observation time - ignore fourth coordinate.
    OpParameter::Real { key: "t_obs", default: std::f64::NAN },
];


pub fn new(parameters: &RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
    let def = &parameters.definition;
    let params = ParsedParameters::new(parameters, &GAMUT)?;

    // Translation
    let x = params.real.get("x")?;
    let y = params.real.get("y")?;
    let z = params.real.get("z")?;

    // Time evolution of translation
    let dx = params.real.get("dx")?;
    let dy = params.real.get("dy")?;
    let dz = params.real.get("dz")?;

    // Rotation
    let rx = params.real.get("rx")?;
    let ry = params.real.get("ry")?;
    let rz = params.real.get("rz")?;

    // Time evolution of rotation
    let drx = params.real.get("drx")?;
    let dry = params.real.get("dry")?;
    let drz = params.real.get("drz")?;

    // Handling of rotations
    let convention = params.text.get("convention")?;
    let rotated = !([rx, ry, rz] == [0., 0., 0.] && [drx, dry, drz] == [0., 0., 0.]);
    let mut position_vector = true;
    if rotated {
        if !["position_vector", "coordinate_frame"].contains(&convention) {
            return Err(Error::BadParam("convention".to_string(), convention));
        }
        if "coordinate_frame" == convention {
            position_vector = false;
        }
    }

    let mut args = res.to_args(0)?;

    // Epoch - "beginning of time for this transformation"
    let t_epoch = args.numeric("t_epoch", std::f64::NAN)?;

    // Fixed observation time - ignore fourth coordinate.
    let t_obs = args.numeric("t_obs", std::f64::NAN)?;

    // Scale and its time evoution
    let scale = args.numeric("s", 0.0)?;
    let dscale = args.numeric("ds", 0.0)? * 1e-6;

    // Handle rotations
    let convention = args.value("convention");
    let exact = args.flag("exact");
    let rotation = !((rx, ry, rz) == (0., 0., 0.) && (drx, dry, drz) == (0., 0., 0.));
    let mut position_vector = true;
    if rotation {
        match convention {
            Err(_) => {
                return Err(GeodesyError::General(
                    "Helmert: Need value for convention when rotating",
                ))
            }
            Ok(d) => match d {
                None => {
                    return Err(GeodesyError::General(
                        "Helmert: Need value for convention when rotating",
                    ))
                }
                Some(value) => {
                    if value == "position_vector" {
                        position_vector = true
                    } else if value == "coordinate_frame" {
                        position_vector = false
                    } else {
                        return Err(GeodesyError::General("Helmert: value for convention must be one of {position_vector, coordinate_frame}"));
                    }
                }
            },
        }
    }

    let inverted = args.flag("inv");
    let argsc = args.used;

    // Now make the args look like they do in the textbooks...
    let scale = 1.0 + scale * 1e-6;
    let T0 = [x, y, z];
    let dT = [dx, dy, dz];
    let R0 = [rx, ry, rz];
    let dR = [drx, dry, drz];

    let R = rotation_matrix(rx, ry, rz, exact, position_vector);


    let def = &parameters.definition;
    let params = ParsedParameters::new(parameters, &GAMUT)?;
    let fwd = InnerOp(addone_fwd);
    let inv = InnerOp(addone_inv);
    let base = Base::new(def, fwd, Some(inv));
    let steps = Vec::<Op>::new();
    Ok(Op {
        base,
        params,
        steps,
    })
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn addone() -> Result<(), Error> {
        let provider = Minimal::default();
        let op = Op::new("addone", &provider)?;
        let mut data = etc::some_basic_coordinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        op.apply(&provider, &mut data, Direction::Fwd);
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        op.apply(&provider, &mut data, Direction::Inv);
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);
        Ok(())
    }
}
