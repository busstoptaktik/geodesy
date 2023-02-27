// Operator utilizing the "proj" command line program to provide access to the
// enormous number of projections available in the PROJ library.
//
// Extremely experimental, undocumented, and with too few checks on return
// values - but (under Windows at least) amazingly, it seems to work...
use crate::operator_authoring::*;

use std::io::Write;
use std::mem::size_of;
use std::process::{Command, Stdio};

// ----- W O R K H O R S E ----------------------------------------------------------

fn proj(args: &str, forward: bool, operands: &mut dyn CoordinateSet) -> usize {
    // Build the command line arguments needed to spawn proj, including '-b' for
    // binary i/o, and '-I' to indicate the inverse operation (if that is the case)
    let mut the_args = "-b ".to_string();
    if !forward {
        the_args += "-I ";
    }
    the_args += args;
    let proj_args: Vec<&str> = the_args.split_whitespace().collect();

    // Spawn the process
    let Ok(mut child) = Command::new("proj")
        .args(&proj_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() else {return 0};

    // Extract the 2D coordinates from the operands, and convert them into bytes
    // for interprocess communication
    let length = operands.len();
    let buffer_size = 2 * length * size_of::<f64>();
    let mut coo = Vec::with_capacity(buffer_size);
    for i in 0..length {
        let op = operands.get_coord(i);
        coo.extend_from_slice(&op[0].to_ne_bytes());
        coo.extend_from_slice(&op[1].to_ne_bytes());
    }

    // Write the input coordinates to proj

    // If the child process fills its stdout buffer, it may end up
    // waiting until the parent reads the stdout, and not be able to
    // read stdin in the meantime, causing a deadlock.
    // Writing from another thread ensures that stdout is being read
    // at the same time, avoiding the problem.
    let mut stdin = child.stdin.take().expect("failed to get stdin");
    std::thread::spawn(move || {
        stdin.write_all(&coo).expect("failed to write to stdin");
    });

    // Read the output bytes
    let output = child.wait_with_output().expect("failed to wait on child");
    if output.stdout.len() != buffer_size {
        warn!("proj: Unexpected return size");
        return 0;
    }

    // Turn the output bytes into doubles and put them properly back into the operands
    let mut errors = 0_usize;
    for i in 0..length {
        let start = 16 * i;
        let ebytes: [u8; 8] = output.stdout[start..start + 8].try_into().unwrap_or([0; 8]);
        let nbytes: [u8; 8] = output.stdout[start + 8..start + 16]
            .try_into()
            .unwrap_or([0; 8]);
        let mut e = f64::from_ne_bytes(ebytes);
        let mut n = f64::from_ne_bytes(nbytes);

        // PROJ uses the C constant HUGE_VAL (i.e. the IEEE infinity value)
        // to indicate errors, while RG uses NAN
        if e == f64::INFINITY || n == f64::INFINITY {
            e = f64::NAN;
            n = f64::NAN;
            errors += 1;
        }
        let mut coord = operands.get_coord(i);
        coord[0] = e;
        coord[1] = n;
        operands.set_coord(i, &coord);
    }

    operands.len() - errors
}

// ----- F O R W A R D --------------------------------------------------------------

fn proj_fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    proj(&op.params.text["proj_args"], true, operands)
}

// ----- I N V E R S E --------------------------------------------------------------

fn proj_inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    proj(&op.params.text["proj_args"], false, operands)
}

// ----- C O N S T R U C T O R ------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let given_args = ParsedParameters::new(parameters, &GAMUT)?.given;
    if Command::new("proj").stderr(Stdio::piped()).spawn().is_err() {
        return Err(Error::NotFound(
            "proj".to_string(),
            "Cannot locate the 'proj' executable".to_string(),
        ));
    }

    // Construct the proj command line args (the '+'-prefixed stuff in e.g. 'proj +proj=utm +zone=32')
    let mut proj_args = String::new();
    for (k, v) in given_args {
        // Remove "proj" or "proj inv" prefixes
        if k == "inv" || k == "name" {
            continue;
        }
        proj_args += " +";
        proj_args += &k;
        if v == "true" {
            continue;
        }
        proj_args += "=";
        proj_args += &v;
    }
    proj_args = proj_args.trim().to_string();

    // Make the proj argument string available to the operator implementation
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;
    params.text.insert("proj_args", proj_args);
    let fwd = InnerOp(proj_fwd);
    let inv = InnerOp(proj_inv);
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

// ----- T E S T S ------------------------------------------------------------------

// echo 12 55 | proj -f %.5f +proj=utm +zone=32
// 691875.63214    6098907.82501

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proj() -> Result<(), Error> {
        if Command::new("proj").stderr(Stdio::piped()).spawn().is_err() {
            return Ok(());
        }

        // Test projection: utm zone 32
        let mut ctx = Minimal::default();
        let op = ctx.op("proj proj=utm zone=32")?;

        // Test values: geo, utm and roundtrip (another copy of geo)
        let mut geo = [Coord::geo(55., 12., 0., 0.)];
        let utm = [Coord::raw(691875.63214, 6098907.82501, 0., 0.)];
        let rtp = [Coord::geo(55., 12., 0., 0.)];

        ctx.apply(op, Fwd, &mut geo)?;
        assert!(geo[0].hypot2(&utm[0]) < 1e-5);

        ctx.apply(op, Inv, &mut geo)?;
        assert!(rtp[0].default_ellps_dist(&geo[0]) < 1e-5);

        // Inverted invocation - note "proj inv ..."
        let op = ctx.op("proj inv proj=utm zone=32")?;

        // Test values: utm and geo swaps roles here
        let geo = [Coord::geo(55., 12., 0., 0.)];
        let mut utm = [Coord::raw(691875.63214, 6098907.82501, 0., 0.)];
        let rtp = [Coord::raw(691875.63214, 6098907.82501, 0., 0.)];

        // Now, we get the inverse utm projection when calling the operator in the Fwd direction
        ctx.apply(op, Fwd, &mut utm)?;
        assert!(geo[0].default_ellps_dist(&utm[0]) < 1e-5);
        // ...and roundtrip back to utm
        ctx.apply(op, Inv, &mut utm)?;
        assert!(rtp[0].hypot2(&utm[0]) < 1e-5);

        Ok(())
    }
}
