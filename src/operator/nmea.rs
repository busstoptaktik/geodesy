//! Read input data in "almost NMEA format", i.e. latitudes and longitudes in that
//! order, but encoded as  +/-DDDMM.mmm. In the actual NMEA-format, a postfix NSEW
//! is used to determine the sign of the angular coordinate.
//!
//! The obvious extension to NMEA, "NMEA with seconds", encoded as +/-DDDMMSS.sss
//! is supported using the dms-entry point.
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
//! $ echo 553036. -124509 | kp "dms | geo inv"
//! > 55.51  -12.7525 0 0
//! ```

use super::Context;
use super::OperatorArgs;
use super::OperatorCore;
use crate::operator_construction::*;
use crate::CoordinateTuple as Coord;
use crate::GeodesyError;

pub struct Nmea {
    args: OperatorArgs,
    inverted: bool,
    dms: bool,
}

impl Nmea {
    /// nmea (DDDMM.mmm)
    pub fn new(args: &mut OperatorArgs) -> Result<Nmea, GeodesyError> {
        let inverted = args.flag("inv");
        Ok(Nmea {
            args: args.clone(),
            inverted,
            dms: false,
        })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, GeodesyError> {
        let op = crate::operator::nmea::Nmea::new(args)?;
        Ok(Operator(Box::new(op)))
    }

    /// dms (DDDMMSS.sss)
    pub fn dms(args: &mut OperatorArgs) -> Result<Nmea, GeodesyError> {
        let inverted = args.flag("inv");
        Ok(Nmea {
            args: args.clone(),
            inverted,
            dms: true,
        })
    }

    pub(crate) fn dmsoperator(args: &mut OperatorArgs) -> Result<Operator, GeodesyError> {
        let op = crate::operator::nmea::Nmea::dms(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for Nmea {
    fn fwd(&self, _ctx: &mut Context, operands: &mut [Coord]) -> bool {
        for o in operands {
            if self.dms {
                *o = Coord::nmeass(o[0], o[1], o[2], o[3]);
                continue;
            }
            *o = Coord::nmea(o[0], o[1], o[2], o[3]);
        }
        true
    }

    fn inv(&self, _ctx: &mut Context, operands: &mut [Coord]) -> bool {
        for o in operands {
            if self.dms {
                let longitude = Coord::dd_to_nmeass(o[0].to_degrees());
                let latitude = Coord::dd_to_nmeass(o[1].to_degrees());
                *o = Coord::raw(latitude, longitude, o[2], o[3]);
                continue;
            }
            let longitude = Coord::dd_to_nmea(o[0].to_degrees());
            let latitude = Coord::dd_to_nmea(o[1].to_degrees());
            *o = Coord::raw(latitude, longitude, o[2], o[3]);
        }
        true
    }

    fn name(&self) -> &'static str {
        "nmea"
    }

    fn is_noop(&self) -> bool {
        false
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
    use crate::Context;
    use crate::CoordinateTuple as Coord;
    #[test]
    fn nmea() {
        let mut ctx = Context::new();
        let nmea = ctx.operation("nmea").unwrap();
        let aemn = ctx.operation("nmea inv").unwrap();

        let coord_nmea = Coord::raw(5530.15, -1245.15, 0., 0.);
        let coord_internal = Coord::geo(55.5025, -12.7525, 0., 0.);

        let mut operands = [coord_nmea];
        ctx.fwd(nmea, &mut operands);
        assert!(operands[0].default_ellps_dist(&coord_internal) < 1e-10);
        assert!((operands[0][0].to_degrees() + 12.7525).abs() < 1e-10);
        assert!((operands[0][1].to_degrees() - 55.5025).abs() < 1e-10);

        ctx.inv(nmea, &mut operands);
        assert!((operands[0][0] - 5530.15).abs() < 1e-10);
        assert!((operands[0][1] + 1245.15).abs() < 1e-10);

        let mut operands = [coord_internal];
        ctx.fwd(aemn, &mut operands);
        assert!((operands[0][0] - 5530.15).abs() < 1e-10);
        assert!((operands[0][1] + 1245.15).abs() < 1e-10);
        ctx.inv(aemn, &mut operands);
        assert!(operands[0].default_ellps_dist(&coord_internal) < 1e-10);
    }

    #[test]
    fn dms() {
        let mut ctx = Context::new();
        let dms = ctx.operation("dms").unwrap();

        let coord_dms = Coord::raw(553036., -124509., 0., 0.);
        let coord_internal = Coord::geo(55.51, -12.7525, 0., 0.);

        let mut operands = [coord_dms];
        ctx.fwd(dms, &mut operands);
        assert!(operands[0].default_ellps_dist(&coord_internal) < 1e-10);

        ctx.inv(dms, &mut operands);
        assert!((operands[0][0] - 553036.).abs() < 1e-10);
        assert!((operands[0][1] + 124509.).abs() < 1e-10);
    }
}
