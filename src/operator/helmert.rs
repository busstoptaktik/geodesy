#![allow(non_snake_case)]

use crate::operator_construction::*;
use crate::Context;
use crate::CoordinateTuple;

pub struct Helmert {
    R: [[f64; 3]; 3],
    T: [f64; 3],
    scale: f64,
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

    // drop second order infinitesimals when using small angle approximations
    let drop = if exact { 1.0 } else { 0.0 };
    let keep = 1.0;

    let r11 = keep * (cy * cz);
    let r12 = keep * (cx * sz) + drop * (sx * sy * cz);
    let r13 = drop * (sx * sz) - keep * (cx * sy * cz);

    let r21 = -keep * (cy * sz);
    let r22 = keep * (cx * cz) - drop * (sx * sy * sz);
    let r23 = keep * (sx * cz) + drop * (cx * sy * sz);

    let r31 = keep * (sy);
    let r32 = -keep * (sx * cy);
    let r33 = keep * (cx * cy);

    if position_vector {
        return [[r11, r21, r31], [r12, r22, r32], [r13, r23, r33]];
    }
    [[r11, r12, r13], [r21, r22, r23], [r31, r32, r33]]
}

impl Helmert {
    fn new(args: &mut OperatorArgs) -> Result<Helmert, &'static str> {
        let x = args.numeric_value("x", 0.0)?;
        let y = args.numeric_value("y", 0.0)?;
        let z = args.numeric_value("z", 0.0)?;

        let rx = args.numeric_value("rx", 0.0)?;
        let ry = args.numeric_value("ry", 0.0)?;
        let rz = args.numeric_value("rz", 0.0)?;

        let scale = args.numeric_value("s", 0.0)?;

        // Handle rotations
        let convention = args.value("convention", "");
        let exact = args.flag("exact");
        let mut rotation = false;
        if (rx, ry, rz) != (0., 0., 0.) {
            rotation = true;
            if convention.is_empty() {
                return Err("Need value for convention when rotating");
            }
            if convention != "position_vector" && convention != "coordinate_frame" {
                return Err(
                    "value for convention must be one of {position_vector, coordinate_frame}",
                );
            }
        }

        // We cannot call args.clone until we're done accessing the args.
        let inverted = args.flag("inv");
        let argsc = args.clone();

        // Now make the args look like in the textbooks...
        let scale = 1.0 + scale / 1_000_000.0;
        let T = [x, y, z];
        let R = rotation_matrix(rx, ry, rz, exact, convention == "position_vector");

        Ok(Helmert {
            R,
            T,
            scale,
            rotation,
            inverted,
            args: argsc,
        })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, &'static str> {
        let op = crate::operator::helmert::Helmert::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for Helmert {
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        let scale = self.scale;
        let R = self.R;
        let T = self.T;
        for c in operands {
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

    fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        let scale = self.scale;
        let R = self.R;
        let T = self.T;
        for c in operands {
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
        assert!(h.is_none());

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

        let definition = "helmert: {
            convention: coordinate_frame,
            x:  0.06155,  rx: -0.0394924,
            y: -0.01087,  ry: -0.0327221,
            z: -0.04019,  rz: -0.0328979,  s: -0.009994
        }";

        let op = ctx.operator(definition).unwrap();
        let mut operands = [CoordinateTuple([
            -4052051.7643,
            4212836.2017,
            -2545106.0245,
            0.0,
        ])];
        let expect = CoordinateTuple([-4052052.7379, 4212835.9897, -2545104.5898, 0.0]);
        ctx.fwd(op, &mut operands);
        // Expected to be better than 75 um
        assert!(expect.hypot3(&operands[0]) < 75e-6);
    }
}
