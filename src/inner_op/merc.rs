//! Mercator
use crate::authoring::*;

// ----- F O R W A R D -----------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);
    let a = ellps.semimajor_axis();
    let k_0 = op.params.k(0);
    let x_0 = op.params.x(0);
    let y_0 = op.params.y(0);
    let lat_0 = op.params.lat(0);
    let lon_0 = op.params.lon(0);

    let mut successes = 0_usize;
    for i in 0..operands.len() {
        let (lon, lat) = operands.xy(i);

        let easting = (lon - lon_0) * k_0 * a - x_0;
        let isometric = ellps.latitude_geographic_to_isometric(lat + lat_0);
        let northing = a * k_0 * isometric - y_0;

        operands.set_xy(i, easting, northing);
        successes += 1;
    }

    successes
}

// ----- I N V E R S E -----------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, operands: &mut dyn CoordinateSet) -> usize {
    let ellps = op.params.ellps(0);
    let a = ellps.semimajor_axis();
    let k_0 = op.params.k(0);
    let x_0 = op.params.x(0);
    let y_0 = op.params.y(0);
    let lat_0 = op.params.lat(0);
    let lon_0 = op.params.lon(0);

    let mut successes = 0_usize;
    for i in 0..operands.len() {
        let (mut x, mut y) = operands.xy(i);

        // Easting -> Longitude
        x += x_0;
        let lon = x / (a * k_0) - lon_0;

        // Northing -> Latitude
        y += y_0;
        let psi = y / (a * k_0);
        let lat = ellps.latitude_isometric_to_geographic(psi) - lat_0;
        operands.set_xy(i, lon, lat);
        successes += 1;
    }

    successes
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

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let def = &parameters.definition;
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;
    let ellps = params.ellps(0);

    let lat_ts = params.real("lat_ts")?;
    if lat_ts.abs() > 90. {
        return Err(Error::General(
            "Merc: Invalid value for lat_ts: |lat_ts| should be <= 90°",
        ));
    }

    // lat_ts trumps k_0
    if lat_ts != 0.0 {
        let sc = lat_ts.to_radians().sin_cos();
        let k_0 = sc.1 / (1. - ellps.eccentricity_squared() * sc.0 * sc.0).sqrt();
        params.real.insert("k_0", k_0);
    }

    let descriptor = OpDescriptor::new(def, InnerOp(fwd), Some(InnerOp(inv)));

    Ok(Op {
        descriptor,
        params,
        steps: None,
        id: OpHandle::new(),
    })
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merc() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "merc";
        let op = ctx.op(definition)?;

        // Validation value from PROJ: echo 12 55 0 0 | cct -d18 +proj=merc
        // followed by quadrant tests from PROJ builtins.gie
        let geo = [
            Coor4D::geo(55., 12., 0., 0.),
            Coor4D::geo(1., 2., 0., 0.),
            Coor4D::geo(-1., 2., 0., 0.),
            Coor4D::geo(1., -2., 0., 0.),
            Coor4D::geo(-1., -2., 0., 0.),
        ];

        let projected = [
            Coor4D::raw(1_335_833.889_519_282_8, 7_326_837.714_873_877, 0., 0.),
            Coor4D::raw(222_638.981_586_547, 110_579.965_218_249, 0., 0.),
            Coor4D::raw(222_638.981_586_547, -110_579.965_218_249, 0., 0.),
            Coor4D::raw(-222_638.981_586_547, 110_579.965_218_249, 0., 0.),
            Coor4D::raw(-222_638.981_586_547, -110_579.965_218_249, 0., 0.),
        ];

        // Forward
        let mut operands = geo;
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 20e-9);
        }

        // Roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 20e-9);
        }

        Ok(())
    }

    #[test]
    fn merc_lat_ts() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let definition = "merc lat_ts=56";
        let op = ctx.op(definition)?;

        let geo = [Coor4D::geo(55., 12., 0., 0.)];

        // Validation value from PROJ: echo 12 55 0 0 | cct -d18 +proj=merc +lat_ts=56
        let projected = [Coor4D::raw(
            748_713.257_925_886_8,
            4_106_573.862_841_270_4,
            0.,
            0.,
        )];

        // Forward
        let mut operands = geo;
        ctx.apply(op, Fwd, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&projected[i]) < 20e-9);
        }

        // Roundtrip
        ctx.apply(op, Inv, &mut operands)?;
        for i in 0..operands.len() {
            assert!(operands[i].hypot2(&geo[i]) < 20e-9);
        }

        Ok(())
    }
}
