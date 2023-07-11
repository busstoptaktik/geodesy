//! Read input data in "almost ISO-6709 format", i.e. latitudes and longitudes in that
//! order, but encoded as  +/-DDDMM.mmm. In the actual ISO-6709 format, a postfix NSEW
//! is used to determine the sign of the angular coordinate. Here, that part is handled
//! by the i/o routine, using the parse_sexagesimal function.
//!
//! ISO-6709 supports both the nautical +/-DDDMM.mmm "degrees and decimal minutes"
//! format, and its obvious extension with seconds, encoded as +/-DDDMMSS.sss.
//! This is supported using the dms flag.
//!
//! Output is a coordinate tuple in the internal format.
//!
//! EXAMPLE: convert DDDMM.mmm to decimal degrees.
//! ```sh
//! $ echo 5530.15 -1245.15 | kp "dm | geo:out"
//! > 55.5025  -12.7525 0 0
//! ```
//!
//! EXAMPLE: convert dms to decimal degrees.
//! ```sh
//! $ echo 553036. -124509 | kp "dms | geo inv"
//! > 55.51  -12.7525 0 0
//! ```
use crate::operator_authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn dm_fwd(_op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;
    let length = operands.len();
    for i in 0..length {
        let mut o = operands.get_coord(i);
        o = Coor4D::iso_dm(o[0], o[1], o[2], o[3]);
        operands.set_coord(i, &o);
        successes += 1;
    }

    successes
}

fn dms_fwd(_op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;
    let length = operands.len();
    for i in 0..length {
        let mut o = operands.get_coord(i);
        o = Coor4D::iso_dms(o[0], o[1], o[2], o[3]);
        operands.set_coord(i, &o);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn dm_inv(_op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;
    let length = operands.len();
    for i in 0..length {
        let mut o = operands.get_coord(i);
        let longitude = angular::dd_to_iso_dm(o[0].to_degrees());
        let latitude = angular::dd_to_iso_dm(o[1].to_degrees());
        o = Coor4D::raw(latitude, longitude, o[2], o[3]);
        operands.set_coord(i, &o);
        successes += 1;
    }

    successes
}

fn dms_inv(_op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let mut successes = 0_usize;
    let length = operands.len();
    for i in 0..length {
        let mut o = operands.get_coord(i);
        let longitude = angular::dd_to_iso_dms(o[0].to_degrees());
        let latitude = angular::dd_to_iso_dms(o[1].to_degrees());
        o = Coor4D::raw(latitude, longitude, o[2], o[3]);
        operands.set_coord(i, &o);
        successes += 1;
    }

    successes
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 1] = [
    OpParameter::Flag { key: "inv" },
];

pub fn dm(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    Op::plain(
        parameters,
        InnerOp(dm_fwd),
        Some(InnerOp(dm_inv)),
        &GAMUT,
        ctx,
    )
}

pub fn dms(parameters: &RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
    Op::plain(
        parameters,
        InnerOp(dms_fwd),
        Some(InnerOp(dms_inv)),
        &GAMUT,
        ctx,
    )
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dm() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("dm")?;

        let mut operands = [Coor4D::raw(5530.15, -1245.15, 0., 0.)];

        // Forward: iso_dm to internal
        ctx.apply(op, Fwd, &mut operands)?;
        assert!((operands[0][0].to_degrees() - -12.7525).abs() < 1e-14);
        assert!((operands[0][1].to_degrees() - 55.5025).abs() < 1e-14);
        assert_eq!(operands[0][2], 0.0);

        // Inverse + roundtrip: Internal to iso_dm
        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][0] - 5530.15).abs() < 1e-14);
        assert!((operands[0][1] - -1245.15).abs() < 1e-14);
        assert_eq!(operands[0][0], 5530.15);
        assert_eq!(operands[0][1], -1245.15);
        assert_eq!(operands[0][2], 0.);
        Ok(())
    }

    #[test]
    fn dms() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let op = ctx.op("dms")?;

        let mut operands = [Coor4D::raw(553036., -124509., 0., 0.)];
        let geo = Coor4D::geo(55.51, -12.7525, 0., 0.);

        ctx.apply(op, Fwd, &mut operands)?;
        assert!(operands[0].default_ellps_dist(&geo) < 1e-10);

        ctx.apply(op, Inv, &mut operands)?;
        assert!((operands[0][0] - 553036.).abs() < 1e-10);
        assert!((operands[0][1] + 124509.).abs() < 1e-10);

        Ok(())
    }
}
