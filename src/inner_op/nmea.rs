//! Read input data in "almost NMEA format", i.e. latitudes and longitudes in that
//! order, but encoded as  +/-DDDMM.mmm. In the actual NMEA-format, a postfix NSEW
//! is used to determine the sign of the angular coordinate.
//!
//! The obvious extension to NMEA, "NMEA with seconds", encoded as +/-DDDMMSS.sss
//! is supported using the dms flag.
//!
//! Output is a coordinate tuple in the internal format.
//!
//! EXAMPLE: convert NMEA to decimal degrees.
//! ```sh
//! $ echo 5530.15 -1245.15 | kp "nmea | geo inv"
//! > 55.5025  -12.7525 0 0
//! ```
//!
//! EXAMPLE: convert dms to decimal degrees.
//! ```sh
//! $ echo 553036. -124509 | kp "nmea dms | geo inv"
//! > 55.51  -12.7525 0 0
//! ```
use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _prv: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let dms = op.params.boolean("dms");
    let mut successes = 0_usize;
    for o in operands {
        if dms {
            *o = Coord::nmeass(o[0], o[1], o[2], o[3]);
        } else {
            *o = Coord::nmea(o[0], o[1], o[2], o[3]);
        }
        successes += 1;
    }

    Ok(successes)
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _prv: &dyn Context, operands: &mut [Coord]) -> Result<usize, Error> {
    let dms = op.params.boolean("dms");
    let mut successes = 0_usize;
    for o in operands {
        if dms {
            let longitude = Coord::dd_to_nmeass(o[0].to_degrees());
            let latitude = Coord::dd_to_nmeass(o[1].to_degrees());
            *o = Coord::raw(latitude, longitude, o[2], o[3]);
        } else {
            let longitude = Coord::dd_to_nmea(o[0].to_degrees());
            let latitude = Coord::dd_to_nmea(o[1].to_degrees());
            *o = Coord::raw(latitude, longitude, o[2], o[3]);
        }
        successes += 1;
    }

    Ok(successes)
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 2] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Flag { key: "dms" }
];

pub fn new(parameters: &RawParameters, provider: &dyn Context) -> Result<Op, Error> {
    Op::plain(parameters, InnerOp(fwd), InnerOp(inv), &GAMUT, provider)
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nmea() -> Result<(), Error> {
        let provider = Minimal::default();
        let op = Op::new("nmea", &provider)?;

        let mut operands = [Coord::raw(5530.15, -1245.15, 0., 0.)];

        // Forward: nmea to internal
        op.apply(&provider, &mut operands, Fwd)?;
        assert!((operands[0].first().to_degrees() - -12.7525).abs() < 1e-14);
        assert!((operands[0].second().to_degrees() - 55.5025).abs() < 1e-14);
        assert_eq!(operands[0].third(), 0.0);

        // Inverse + roundtrip: Internal to nmea
        op.apply(&provider, &mut operands, Inv)?;
        assert!((operands[0].first() - 5530.15).abs() < 1e-14);
        assert!((operands[0].second() - -1245.15).abs() < 1e-14);
        assert_eq!(operands[0].first(), 5530.15);
        assert_eq!(operands[0].second(), -1245.15);
        assert_eq!(operands[0].third(), 0.);
        Ok(())
    }

    #[test]
    fn nmea_dms() -> Result<(), Error> {
        let provider = Minimal::default();
        let op = Op::new("nmea dms", &provider)?;

        let mut operands = [Coord::raw(553036., -124509., 0., 0.)];
        let geo = Coord::geo(55.51, -12.7525, 0., 0.);

        op.apply(&provider, &mut operands, Fwd)?;
        assert!(operands[0].default_ellps_dist(&geo) < 1e-10);

        op.apply(&provider, &mut operands, Inv)?;
        assert!((operands[0][0] - 553036.).abs() < 1e-10);
        assert!((operands[0][1] + 124509.).abs() < 1e-10);

        Ok(())
    }
}
