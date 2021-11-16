#![allow(non_snake_case)]

use crate::operator_construction::*;
use crate::Context;
use crate::CoordinateTuple;
use crate::GeodesyError;

#[derive(Debug)]
pub struct Helmert {
    R: [[f64; 3]; 3],
    T0: [f64; 3],
    R0: [f64; 3],
    dR: [f64; 3],
    dT: [f64; 3],
    t_epoch: f64,
    t_obs: f64,
    scale: f64,
    dscale: f64,
    exact: bool,
    position_vector: bool,
    rotation: bool,
    inverted: bool,
    args: OperatorArgs,
}

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
    // From seconds of arc to radians
    let rx = (rx / 3600.).to_radians();
    let ry = (ry / 3600.).to_radians();
    let rz = (rz / 3600.).to_radians();

    // Small angle approximations: sx = sin(rx) = rx,  cx = cos(rx) = 1,  etc.
    let (mut sx, mut sy, mut sz) = (rx, ry, rz);
    let (mut cx, mut cy, mut cz) = (1.0, 1.0, 1.0);
    if exact {
        let scx = rx.sin_cos();
        let scy = ry.sin_cos();
        let scz = rz.sin_cos();

        // Destructuring assignments are unstable, so we desctructure manually
        sx = scx.0;
        cx = scx.1;
        sy = scy.0;
        cy = scy.1;
        sz = scz.0;
        cz = scz.1;
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

impl Helmert {
    fn new(args: &mut OperatorArgs) -> Result<Helmert, GeodesyError> {
        // Translation
        let x = args.numeric_value("x", 0.0)?;
        let y = args.numeric_value("y", 0.0)?;
        let z = args.numeric_value("z", 0.0)?;

        // Rotation
        let rx = args.numeric_value("rx", 0.0)?;
        let ry = args.numeric_value("ry", 0.0)?;
        let rz = args.numeric_value("rz", 0.0)?;

        // Time evolution of translation
        let dx = args.numeric_value("dx", 0.0)?;
        let dy = args.numeric_value("dy", 0.0)?;
        let dz = args.numeric_value("dz", 0.0)?;

        // Time evolution of rotation
        let drx = args.numeric_value("drx", 0.0)?;
        let dry = args.numeric_value("dry", 0.0)?;
        let drz = args.numeric_value("drz", 0.0)?;

        // Epoch - "beginning of time for this transformation"
        let t_epoch = args.numeric_value("t_epoch", std::f64::NAN)?;

        // Fixed observation time - ignore fourth coordinate.
        let t_obs = args.numeric_value("t_obs", std::f64::NAN)?;

        // Scale and its time evoution
        let scale = args.numeric_value("s", 0.0)?;
        let dscale = args.numeric_value("ds", 0.0)? * 1e-6;

        // Handle rotations
        let convention = args.value("convention", "");
        let exact = args.flag("exact");
        let rotation = !((rx, ry, rz) == (0., 0., 0.) && (drx, dry, drz) == (0., 0., 0.));
        if rotation {
            if convention.is_empty() {
                return Err(GeodesyError::General(
                    "Helmert: Need value for convention when rotating",
                ));
            }
            if convention != "position_vector" && convention != "coordinate_frame" {
                return Err(GeodesyError::General(
                    "Helmert: value for convention must be one of {position_vector, coordinate_frame}",
                ));
            }
        }

        // We cannot call args.clone until we're done accessing the args.
        let inverted = args.flag("inv");
        let argsc = args.clone();

        // Now make the args look like they do in the textbooks...
        let scale = 1.0 + scale * 1e-6;
        let T0 = [x, y, z];
        let dT = [dx, dy, dz];
        let R0 = [rx, ry, rz];
        let dR = [drx, dry, drz];
        let position_vector = convention == "position_vector";

        let R = rotation_matrix(rx, ry, rz, exact, position_vector);

        Ok(Helmert {
            R,
            R0,
            dR,
            T0,
            dT,
            scale,
            dscale,
            t_epoch,
            t_obs,
            exact,
            position_vector,
            rotation,
            inverted,
            args: argsc,
        })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, GeodesyError> {
        let op = crate::operator::helmert::Helmert::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for Helmert {
    fn fwd(&self, _ctx: &Context, operands: &mut [CoordinateTuple]) -> bool {
        let mut scale = self.scale;
        let mut R = self.R;
        let mut T = self.T0;
        let mut prev_t = std::f64::NAN;
        for c in operands {
            // Time varying case?
            if !self.t_epoch.is_nan() {
                // Necessary to update parameters?
                let t = if self.t_obs.is_nan() {
                    c[3]
                } else {
                    self.t_obs
                };
                #[allow(clippy::float_cmp)]
                if t != prev_t {
                    prev_t = t;
                    let dt = t - self.t_epoch;
                    T[0] += dt * self.dT[0];
                    T[1] += dt * self.dT[1];
                    T[2] += dt * self.dT[2];
                    let rx = self.R0[0] + dt * self.dR[0];
                    let ry = self.R0[1] + dt * self.dR[1];
                    let rz = self.R0[2] + dt * self.dR[2];
                    if self.rotation {
                        R = rotation_matrix(rx, ry, rz, self.exact, self.position_vector);
                    }
                    scale += dt * self.dscale;
                }
            }

            if self.rotation {
                // Rotate
                let x = c[0] * R[0][0] + c[1] * R[0][1] + c[2] * R[0][2];
                let y = c[0] * R[1][0] + c[1] * R[1][1] + c[2] * R[1][2];
                let z = c[0] * R[2][0] + c[1] * R[2][1] + c[2] * R[2][2];

                // scale and offset
                c[0] = scale * x + T[0];
                c[1] = scale * y + T[1];
                c[2] = scale * z + T[2];
                continue;
            }

            // scale and offset without rotation
            c[0] = scale * c[0] + T[0];
            c[1] = scale * c[1] + T[1];
            c[2] = scale * c[2] + T[2];
        }
        true
    }

    fn inv(&self, _ctx: &Context, operands: &mut [CoordinateTuple]) -> bool {
        let mut scale = self.scale;
        let mut R = self.R;
        let mut T = self.T0;
        let mut prev_t = std::f64::NAN;

        for c in operands {
            // Time varying case?
            #[allow(clippy::float_cmp)]
            if !self.t_epoch.is_nan() {
                // Necessary to update parameters?
                let t = if self.t_obs.is_nan() {
                    c[3]
                } else {
                    self.t_obs
                };
                if t != prev_t {
                    prev_t = t;
                    let dt = t - self.t_epoch;
                    T[0] += dt * self.dT[0];
                    T[1] += dt * self.dT[1];
                    T[2] += dt * self.dT[2];
                    let rx = self.R0[0] + dt * self.dR[0];
                    let ry = self.R0[1] + dt * self.dR[1];
                    let rz = self.R0[2] + dt * self.dR[2];
                    if self.rotation {
                        R = rotation_matrix(rx, ry, rz, self.exact, self.position_vector);
                    }
                    scale += dt * self.dscale;
                }
            }

            // Deoffset and unscale
            let x = (c[0] - T[0]) / scale;
            let y = (c[1] - T[1]) / scale;
            let z = (c[2] - T[2]) / scale;

            // Inverse rotation by transposed multiplication
            if self.rotation {
                c[0] = x * R[0][0] + y * R[1][0] + z * R[2][0];
                c[1] = x * R[0][1] + y * R[1][1] + z * R[2][1];
                c[2] = x * R[0][2] + y * R[1][2] + z * R[2][2];
            } else {
                c[0] = x;
                c[1] = y;
                c[2] = z;
            }
        }
        true
    }

    fn name(&self) -> &'static str {
        "helmert"
    }

    fn debug(&self) -> String {
        format!("{:#?}", self)
    }

    fn is_inverted(&self) -> bool {
        self.inverted
    }

    fn args(&self, _step: usize) -> &OperatorArgs {
        &self.args
    }
}

#[cfg(test)]
mod tests {
    use crate::operator::operator_factory;

    #[test]
    fn helmert() {
        use super::*;
        let mut ctx = Context::new();
        let mut args = OperatorArgs::new();

        // Check that non-numeric value, for key expecting numeric, errs properly.
        args.name("helmert");
        args.insert("x", "foo"); // Bad value here.
        args.insert("y", "-96");
        args.insert("z", "-120");

        let h = operator_factory(&mut args, &mut ctx, 0);
        assert!(h.is_err());

        // EPSG:1134 - 3 parameter, ED50/WGS84, s = sqrt(27) m
        args.insert("x", "-87");
        assert_eq!(args.value("x", ""), "-87");
        assert_eq!(args.value("y", ""), "-96");
        assert_eq!(args.value("z", ""), "-120");

        let h = operator_factory(&mut args, &mut ctx, 0).unwrap();

        let mut operands = [CoordinateTuple::origin()];
        h.fwd(&mut ctx, operands.as_mut());
        assert_eq!(operands[0].first(), -87.);
        assert_eq!(operands[0].second(), -96.);
        assert_eq!(operands[0].third(), -120.);

        h.inv(&mut ctx, operands.as_mut());
        assert_eq!(operands[0].first(), 0.);
        assert_eq!(operands[0].second(), 0.);
        assert_eq!(operands[0].third(), 0.);

        // ---------------------------------------------------------------------------
        // Test case from "Intergovernmental Committee on Surveying and Mapping (ICSM)
        // Permanent Committee on Geodesy (PCG)": Geocentric Datum of Australia 2020,
        // Technical Manual Version 1.0, 25 July 2017.
        // Transformation from GDA94 to GDA2020.
        // ---------------------------------------------------------------------------
        let definition = "helmert: {
            convention: coordinate_frame,
            x:  0.06155,  rx: -0.0394924,
            y: -0.01087,  ry: -0.0327221,
            z: -0.04019,  rz: -0.0328979,  s: -0.009994
        }";

        let op = ctx.operation(definition).unwrap();
        let GDA94 = CoordinateTuple([-4052051.7643, 4212836.2017, -2545106.0245, 0.0]);
        let GDA2020 = CoordinateTuple([-4052052.7379, 4212835.9897, -2545104.5898, 0.0]);

        // The forward transformation should hit closeer than 75 um
        let mut operands = [GDA94];
        ctx.fwd(op, &mut operands);
        assert!(GDA2020.hypot3(&operands[0]) < 75e-6);

        // ... and even closer on the way back
        ctx.inv(op, &mut operands);
        assert!(GDA94.hypot3(&operands[0]) < 75e-7);

        // ---------------------------------------------------------------------------
        // And a time varying example from the same source: ITRF2014@2018 to GDA2020,
        // Test point ALIC (Alice Springs)
        // ---------------------------------------------------------------------------
        let definition = "helmert: {
            exact: true, convention: coordinate_frame,
            x: 0,  rx: 0,   dx: 0,   drx: 0.00150379,
            y: 0,  ry: 0,   dy: 0,   dry: 0.00118346,
            z: 0,  rz: 0,   dz: 0,   drz: 0.00120716,
            s: 0,  ds: 0,   t_epoch: 2020.0
        }";
        let op = ctx.operation(definition).unwrap();

        let ITRF2014 = CoordinateTuple([-4052052.6588, 4212835.9938, -2545104.6946, 2018.0]);
        let GDA2020 = CoordinateTuple([-4052052.7373, 4212835.9835, -2545104.5867, 2020.0]);

        // The forward transformation should hit closeer than 40 um
        let mut operands = [ITRF2014];
        ctx.fwd(op, &mut operands);
        assert!(GDA2020.hypot3(&operands[0]) < 40e-6);

        // ... and even closer on the way back
        ctx.inv(op, &mut operands);
        assert!(ITRF2014.hypot3(&operands[0]) < 40e-8);
    }
}
