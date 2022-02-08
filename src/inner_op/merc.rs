//! Mercator
use super::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let ellps = op.params.ellps[0];
    let a = ellps.semimajor_axis();
    let k_0 = op.params.k[0];
    let x_0 = op.params.x[0];
    let y_0 = op.params.y[0];
    let lat_0 = op.params.lat[0];
    let lon_0 = op.params.lon[0];

    let mut successes = 0_usize;
    for coord in operands {
        // Longitude
        coord[0] = (coord[0] - lon_0) * k_0 * a - x_0;

        // Latitude
        let lat = coord[1] + lat_0;
        coord[1] = a * k_0 * ellps.isometric_latitude(lat, Fwd) - y_0;

        successes += 1;
    }

    Ok(successes)
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _prv: &dyn Provider, operands: &mut [Coord]) -> Result<usize, Error> {
    let ellps = op.params.ellps[0];
    let a = ellps.semimajor_axis();
    let k_0 = op.params.k[0];
    let x_0 = op.params.x[0];
    let y_0 = op.params.y[0];
    let lat_0 = op.params.lat[0];
    let lon_0 = op.params.lon[0];

    let mut successes = 0_usize;
    for coord in operands {
        // Easting -> Longitude
        let x = coord[0] + x_0;
        coord[0] = x / (a * k_0) - lon_0;

        // Northing -> Latitude
        let y = coord[1] + y_0;
        let psi = y / (a * k_0);
        coord[1] = ellps.isometric_latitude(psi, Inv) - lat_0;
        successes += 1;
    }

    Ok(successes)
}

// ----- C O N S T R U C T O R ---------------------------------------------------------

#[rustfmt::skip]
pub const GAMUT: [OpParameter; 8] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "ellps",  default: Some("GRS80") },

    OpParameter::Real { key: "lat_0",  default: Some(0_f64) },
    OpParameter::Real { key: "lon_0",  default: Some(0_f64) },
    OpParameter::Real { key: "x_0",    default: Some(0_f64) },
    OpParameter::Real { key: "y_0",    default: Some(0_f64) },

    OpParameter::Real { key: "k_0",    default: Some(1_f64) },
    OpParameter::Real { key: "lat_ts", default: Some(0_f64) },
];

pub fn new(parameters: &RawParameters, _provider: &dyn Provider) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;
    let ellps = params.ellps[0];

    let lat_ts = params.real("lat_ts")?;
    if lat_ts.abs() > 90. {
        return Err(Error::General(
            "Merc: Invalid value for lat_ts: |lat_ts| should be <= 90°",
        ));
    }

    // lat_ts trumps k_0
    if lat_ts != 0.0 {
        let sc = lat_ts.to_radians().sin_cos();
        params.k[0] = sc.1 / (1. - ellps.eccentricity_squared() * sc.0 * sc.0).sqrt()
    }

    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));
    let steps = Vec::<Op>::new();

    Ok(Op {
        descriptor,
        params,
        steps,
    })
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merc() -> Result<(), Error> {
        let prv = Minimal::default();
        let definition = "merc";
        let op = Op::new(definition, &prv)?;

        // Validation value from PROJ: echo 12 55 0 0 | cct -d18 +proj=merc
        // followed by quadrant tests from PROJ builtins.gie
        let geo = [
            Coord::geo(55., 12., 0., 0.),
            Coord::geo(1., 2., 0., 0.),
            Coord::geo(-1., 2., 0., 0.),
            Coord::geo(1., -2., 0., 0.),
            Coord::geo(-1., -2., 0., 0.),
        ];

        let projected = [
            Coord::raw(1335833.889519282850, 7326837.714873877354, 0., 0.),
            Coord::raw(222638.981586547, 110579.965218249, 0., 0.),
            Coord::raw(222638.981586547, -110579.965218249, 0., 0.),
            Coord::raw(-222638.981586547, 110579.965218249, 0., 0.),
            Coord::raw(-222638.981586547, -110579.965218249, 0., 0.),
        ];

        // Forward
        let mut operands = geo.clone();
        op.apply(&prv, &mut operands, Fwd)?;
        for i in 0..operands.len() {
            assert!(dbg!(operands[i].hypot2(&projected[i])) < 20e-9);
        }

        // Roundtrip
        op.apply(&prv, &mut operands, Inv)?;
        for i in 0..operands.len() {
            dbg!(operands[i].to_degrees());
            assert!(dbg!(operands[i].hypot2(&geo[i])) < 20e-9);
        }

        Ok(())
    }

    #[test]
    fn merc_lat_ts() -> Result<(), Error> {
        let prv = Minimal::default();
        let definition = "merc lat_ts=56";
        let op = Op::new(definition, &prv)?;

        // Validation value from PROJ: echo 12 55 0 0 | cct -d18 +proj=merc +lat_ts=56
        let geo = [Coord::geo(55., 12., 0., 0.)];

        let projected = [Coord::raw(
            748_713.257_925_886_777,
            4_106_573.862_841_270_398,
            0.,
            0.,
        )];

        // Forward
        let mut operands = geo.clone();
        op.apply(&prv, &mut operands, Fwd)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 20e-9);
        }

        // Roundtrip
        op.apply(&prv, &mut operands, Inv)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 20e-9);
        }

        Ok(())
    }
}
