//! Lambert Conformal Conic
use std::f64::consts::FRAC_PI_2;

use super::OperatorArgs;
use super::OperatorCore;
use crate::operator_construction::*;
use crate::Context;
use crate::CoordinateTuple;
use crate::Ellipsoid;
use crate::GeodesyError;

const EPS10: f64 = 1e-10;

#[derive(Debug)]
pub struct Lcc {
    ellps: Ellipsoid,
    inverted: bool,

    k_0: f64,
    lon_0: f64,
    lat_0: f64,
    x_0: f64,
    y_0: f64,

    phi1: f64,
    phi2: f64,
    n: f64,
    rho0: f64,
    c: f64,

    args: OperatorArgs,
}

impl Lcc {
    pub fn new(args: &mut OperatorArgs) -> Result<Lcc, GeodesyError> {
        let ellps = Ellipsoid::named(&args.value("ellps", "GRS80"));
        let inverted = args.flag("inv");

        let mut phi1 = args.numeric_value("lat_1", f64::NAN)?;
        let mut phi2 = args.numeric_value("lat_2", phi1)?;
        let mut lat_0 = if (phi1 - phi2).abs() < EPS10 {
            args.numeric_value("lat_0", phi1)?
        } else {
            args.numeric_value("lat_0", 0.)?
        };

        lat_0 = lat_0.to_radians();
        phi1 = phi1.to_radians();
        phi2 = phi2.to_radians();

        let sc = phi1.sin_cos();
        let mut n = sc.0;
        let e = ellps.eccentricity();
        let es = ellps.eccentricity_squared();

        if (phi1 + phi2).abs() < EPS10 {
            return Err(GeodesyError::General(
                "Lcc: Invalid value for lat_1 and lat_2: |lat_1 + lat_2| should be > 0",
            ));
        }
        if sc.1.abs() < EPS10 || phi1.abs() >= FRAC_PI_2 {
            return Err(GeodesyError::General(
                "Lcc: Invalid value for lat_1: |lat_1| should be < 90°",
            ));
        }
        if phi2.cos().abs() < EPS10 || phi2.abs() >= FRAC_PI_2 {
            return Err(GeodesyError::General(
                "Lcc: Invalid value for lat_2: |lat_2| should be < 90°",
            ));
        }

        // Snyder (1982) eq. 12-15
        let m1 = crate::internals::pj_msfn(sc, es);

        // Snyder (1982) eq. 7-10: exp(-𝜓)
        let ml1 = crate::internals::pj_tsfn(sc, e);

        // Secant case?
        if (phi1 - phi2).abs() >= EPS10 {
            let sc = phi2.sin_cos();
            n = (m1 / crate::internals::pj_msfn(sc, es)).ln();
            if n == 0. {
                return Err(GeodesyError::General("Lcc: Invalid value for eccentricity"));
            }
            let ml2 = crate::internals::pj_tsfn(sc, e);
            let denom = (ml1 / ml2).ln();
            if denom == 0. {
                return Err(GeodesyError::General("Lcc: Invalid value for eccentricity"));
            }
            n /= denom;
        }

        let c = m1 * ml1.powf(-n) / n;
        let mut rho0 = 0.;
        if (lat_0.abs() - FRAC_PI_2).abs() > EPS10 {
            rho0 = c * crate::internals::pj_tsfn(lat_0.sin_cos(), e).powf(n);
        }

        let lon_0 = args.numeric_value("lon_0", 0.)?.to_radians();
        let k_0 = args.numeric_value("k_0", 1.)?;
        let x_0 = args.numeric_value("x_0", 0.)?;
        let y_0 = args.numeric_value("y_0", 0.)?;
        let args = args.clone();

        Ok(Lcc {
            ellps,
            inverted,
            k_0,
            lon_0,
            lat_0,
            x_0,
            y_0,

            phi1,
            phi2,
            n,
            rho0,
            c,

            args,
        })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, GeodesyError> {
        let op = crate::operator::lcc::Lcc::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

// #[allow(non_snake_case)]
impl OperatorCore for Lcc {
    // Forward Lambert conformal conic, following the PROJ implementation,
    // cf.  https://proj.org/operations/projections/lcc.html
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        let a = self.ellps.semimajor_axis();
        let e = self.ellps.eccentricity();
        for coord in operands {
            let lam = coord[0] - self.lon_0;
            let phi = coord[1];
            let mut rho = 0.;

            // Close to one of the poles?
            if (phi.abs() - FRAC_PI_2).abs() < EPS10 {
                if phi * self.n <= 0. {
                    *coord = CoordinateTuple::nan();
                    continue;
                }
            } else {
                rho = self.c * crate::internals::pj_tsfn(phi.sin_cos(), e).powf(self.n);
            }
            let sc = (lam * self.n).sin_cos();
            coord[0] = a * self.k_0 * rho * sc.0 - self.x_0;
            coord[1] = a * self.k_0 * (self.rho0 - rho * sc.1) - self.y_0;
        }
        true
    }

    fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        let a = self.ellps.semimajor_axis();
        let e = self.ellps.eccentricity();
        for coord in operands {
            let mut x = coord[0] / (a * self.k_0);
            let mut y = self.rho0 - coord[1] / (a * self.k_0);

            let mut rho = x.hypot(y);

            // On one of the poles
            if rho == 0. {
                coord[0] = 0.;
                coord[1] = FRAC_PI_2.copysign(self.n);
                continue;
            }

            // Standard parallel on the southern hemisphere
            if self.n < 0. {
                rho = -rho;
                x = -x;
                y = -y;
            }

            let ts0 = (rho / self.c).powf(1. / self.n);
            let phi = crate::internals::pj_phi2(ts0, e);
            if phi.is_infinite() || phi.is_nan() {
                *coord = CoordinateTuple::nan();
                continue;
            }
            coord[0] = x.atan2(y) / self.n + self.lon_0;
            coord[1] = phi;
        }
        true
    }

    fn name(&self) -> &'static str {
        "lcc"
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
    use crate::CoordinateTuple as C;

    #[test]
    fn one_standard_parallel() {
        let op = "lcc lat_1:57 lon_0:12";

        // Validation values from PROJ:
        //     echo 12 55 0 0 | cct -d18 proj=lcc lat_1=57 lon_0=12  -- | clip
        //     echo 10 55 0 0 | cct -d18 proj=lcc lat_1=57 lon_0=12  -- | clip

        let mut operands = [
            C::geo(55., 12., 0., 0.),
            C::geo(55., 10., 0., 0.),
            C::geo(59., 14., 0., 0.),
        ];

        let mut results = [
            C::raw(-0.000000000101829246, -222728.122307816054672003, 0., 0.),
            C::raw(-128046.4724386522429995, -220853.7001605064142495, 0., 0.),
            C::raw(115005.41456620067765471, 224484.5143763388914522, 0., 0.),
        ];

        assert!(Context::test(
            op,
            0,
            2e-9,
            0,
            1e-9,
            &mut operands,
            &mut results
        ));
    }

    #[test]
    fn two_standard_parallels() {
        let op = "lcc lat_1:33 lat_2:45 lon_0:10";

        // Validation value from PROJ:
        // echo 12 40 0 0 | cct -d12 proj=lcc lat_1=33 lat_2=45 lon_0=10 -- | clip
        let mut operands = [C::geo(40., 12., 0., 0.)];
        let mut results = [C::raw(169863.026093938359, 4735925.219292452559, 0., 0.)];
        assert!(Context::test(
            op,
            0,
            20e-9,
            0,
            20e-9,
            &mut operands,
            &mut results
        ));
    }

    #[test]
    fn one_standard_parallel_and_latitudinal_offset() {
        let op = "lcc lat_1:39 lat_0:35 lon_0:10";

        // Validation value from PROJ:
        // echo 12 40 0 0 | cct -d12 proj=lcc lat_1=39 lat_0=35 lon_0=10 -- | clip
        let mut operands = [C::geo(40., 12., 0., 0.)];
        let mut results = [C::raw(170800.011728740647, 557172.361112929415, 0., 0.)];
        assert!(Context::test(
            op,
            0,
            2e-9,
            0,
            1e-8,
            &mut operands,
            &mut results
        ));
    }

    #[test]
    fn two_standard_parallels_and_latitudinal_offset() {
        let op = "lcc lat_1:33 lat_2:45 lat_0:35 lon_0:10";

        // Validation value from PROJ:
        // echo 12 40 0 0 | cct -d12 proj=lcc lat_1=33 lat_2=45 lat_0=35 lon_0=10 -- | clip
        let mut operands = [C::geo(40., 12., 0., 0.)];
        let mut results = [C::raw(169863.026093938359, 554155.440793916583, 0., 0.)];
        assert!(Context::test(
            op,
            0,
            2e-9,
            0,
            1e-9,
            &mut operands,
            &mut results
        ));
    }
}
