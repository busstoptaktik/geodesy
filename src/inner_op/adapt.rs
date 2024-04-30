/*! Declarative approach to adapting input data in one format to output data in another.

Example:

```sh
adapt from=neuf_deg  to=enuf_rad
```

We introduce the coordinate type designations *eastish, northish, upish, futurish*,
and their geometrical inverses *westish, southish, downish, pastish*,
with mostly evident meaning: A coordinate is *eastish* if you would typically draw
it along an abscissa, *northish* if you would typically draw it along an ordinate,
*upish* if you would need to draw it out of the paper, and "futurish"
if it represents ordinary, forward evolving time. *Westish, southish, downish*, and
*reversed-timeish* are the axis-reverted versions of the former four.

These 8 spatio-temporal directional designations have convenient short forms,
`e, n, u, t` and `w, s, d, r`, respectively.

Also, we introduce the 3 common angular representations "degrees, gradians, radians",
conveniently abbrevieated as "deg", "gon" and "rad".

The Rust Geodesy internal format of a four dimensional coordinate tuple is e, n, u, f,
and the internal unit of measure for angular coordinates is radians. In `adapt`, terms,
this is described as `enuf_rad`.

`adapt` covers the same ground as the `PROJ` operator `axisswap`, but using a somewhat
different approach: You never tell `adapt` what you want it to do - you only tell it
what you want to go `from`, and what you want to come `to` (and in most cases actually
only one of those). Then `adapt` figures out how to fulfill the order.

The example above specifies that an input coordinate tuple with coordinate order
**latitude, longitude, height, time**, with latitude and longitude in degrees, should be
converted to an output coordinate in radians and with latitude and longitude swapped.
That output format is identical to the default internal format, so it can actually
be left out, and the order be written as:

```sh
adapt from=neuf_deg
```

Typically, `adapt` is used in both ends of a pipeline, to match data between the
RG internal representation and the requirements of the embedding system:

```sh
adapt from=neuf_deg | cart ... | helmert ... | cart inv ... | adapt to=neuf_deg
```

Note that `adapt to=...` and `adapt inv from=...` are equivalent.

Some RG context providers supply predefined symbolic coordinate handling macros,
as in:

```sh
geo:in | cart ... | helmert ... | cart inv ... | geo:out
```

!*/

use crate::authoring::*;

const POST_DEFAULT: [f64; 4] = [0., 1., 2., 3.];
const MULT_DEFAULT: [f64; 4] = [1., 1., 1., 1.];

// ----- F O R W A R D --------------------------------------------------------------

fn fwd(op: &Op, _ctx: &dyn Context, data: &mut dyn CoordinateSet) -> usize {
    let n = data.len();
    if op.params.boolean("noop") {
        return n;
    }

    let post = op.params.series("post").unwrap_or(&POST_DEFAULT);
    let post = [
        post[0] as usize,
        post[1] as usize,
        post[2] as usize,
        post[3] as usize,
    ];
    let mult = op.params.series("mult").unwrap_or(&MULT_DEFAULT);
    for i in 0..n {
        let mut coord = data.get_coord(i);
        coord = Coor4D([
            coord[post[0]] * mult[0],
            coord[post[1]] * mult[1],
            coord[post[2]] * mult[2],
            coord[post[3]] * mult[3],
        ]);
        data.set_coord(i, &coord);
    }
    n
}

// ----- I N V E R S E --------------------------------------------------------------

fn inv(op: &Op, _ctx: &dyn Context, data: &mut dyn CoordinateSet) -> usize {
    let n = data.len();
    if op.params.boolean("noop") {
        return n;
    }

    let post = op.params.series("post").unwrap_or(&POST_DEFAULT);
    let post = [
        post[0] as usize,
        post[1] as usize,
        post[2] as usize,
        post[3] as usize,
    ];

    let mult = op.params.series("mult").unwrap_or(&MULT_DEFAULT);
    let mult = [1. / mult[0], 1. / mult[1], 1. / mult[2], 1. / mult[3]];

    for i in 0..n {
        let coord = data.get_coord(i);
        let mut c = Coor4D::default();
        for j in 0..4_usize {
            c[post[j]] = coord[j] * mult[post[j]];
        }
        data.set_coord(i, &c);
    }

    n
}

// ----- C O N S T R U C T O R ------------------------------------------------------

// Example...
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 3] = [
    OpParameter::Flag { key: "inv" },
    OpParameter::Text { key: "from", default: Some("enuf") },
    OpParameter::Text { key: "to", default: Some("enuf") },
];

pub fn new(parameters: &RawParameters, _ctx: &dyn Context) -> Result<Op, Error> {
    let mut params = ParsedParameters::new(parameters, &GAMUT)?;
    let descriptor = OpDescriptor::new(&parameters.definition, InnerOp(fwd), Some(InnerOp(inv)));
    let steps = Vec::<Op>::new();

    // What we go `from` and what we go `to` both defaults to the internal
    // representation - i.e. "do nothing", neither on in- nor output.
    let from = params.text("from")?;
    let to = params.text("to")?;

    let desc = coordinate_order_descriptor(&from);
    if desc.is_none() {
        return Err(Error::Operator("Adapt", "Bad value for 'from'"));
    }
    let from = desc.unwrap();

    let desc = coordinate_order_descriptor(&to);
    if desc.is_none() {
        return Err(Error::Operator("Adapt", "Bad value for 'to'"));
    }
    let to = desc.unwrap();

    // Eliminate redundancy for over-specified cases.
    let give = combine_descriptors(&from, &to);
    if give.noop {
        params.boolean.insert("noop");
    }
    let post = [
        give.post[0] as f64,
        give.post[1] as f64,
        give.post[2] as f64,
        give.post[3] as f64,
    ];
    params.series.insert("post", Vec::from(post));
    params.series.insert("mult", Vec::from(give.mult));
    let id = OpHandle::new();

    Ok(Op {
        descriptor,
        params,
        steps,
        id,
    })
}

// ----- A N C I L L A R Y   F U N C T I O N S   G O   H E R E -------------------------

#[derive(Debug, Default, Clone)]
struct CoordinateOrderDescriptor {
    post: [usize; 4],
    mult: [f64; 4],
    noop: bool,
}

#[allow(clippy::float_cmp)]
fn coordinate_order_descriptor(desc: &str) -> Option<CoordinateOrderDescriptor> {
    let mut post = [0_usize, 1, 2, 3];
    let mut mult = [1_f64, 1., 1., 1.];
    if desc == "pass" {
        return Some(CoordinateOrderDescriptor {
            post,
            mult,
            noop: true,
        });
    }

    if desc.len() != 4 && desc.len() != 8 {
        return None;
    }

    let mut torad = 1_f64;
    if desc.len() == 8 {
        let good_angular = desc.ends_with("_deg")
            || desc.ends_with("_gon")
            || desc.ends_with("_rad")
            || desc.ends_with("_any");
        if !good_angular {
            return None;
        }
        if desc.ends_with("_deg") {
            torad = std::f64::consts::PI / 180.;
        } else if desc.ends_with("_gon") {
            torad = std::f64::consts::PI / 200.;
        }
    }

    // Now figure out what goes (resp. comes from) where
    let desc: Vec<char> = desc[0..4].chars().collect();
    let mut indices = [1i32, 2, 3, 4];
    for i in 0..4 {
        let d = desc[i];

        // Unknown designator
        if !"neufswdp".contains(d) {
            return None;
        }
        // Sign and position in the internal representation
        let dd: i32 = match d {
            'w' => -1,
            's' => -2,
            'd' => -3,
            'p' => -4,
            'e' => 1,
            'n' => 2,
            'u' => 3,
            'f' => 4,
            _ => 0, // cannot happen: We already err'ed on unknowns
        };
        indices[i] = dd;
    }

    // Check that the descriptor describes a true permutation:
    // all inputs go to a unique output
    let mut count = [0_usize, 0, 0, 0];
    for i in 0..4 {
        count[(indices[i].abs() - 1) as usize] += 1;
    }
    if count != [1, 1, 1, 1] {
        warn!("adapt: {:?} is not a proper permutation", desc);
        return None;
    }

    // Now untangle the sign and position parts of 'indices'
    for i in 0..4 {
        let d = indices[i];
        post[i] = (d.abs() - 1) as usize;
        mult[i] = d.signum() as f64 * if i > 1 { 1.0 } else { torad };
    }
    let noop = mult == [1.0; 4] && post == [0_usize, 1, 2, 3];

    Some(CoordinateOrderDescriptor { post, mult, noop })
}

#[allow(clippy::float_cmp)]
fn combine_descriptors(
    from: &CoordinateOrderDescriptor,
    to: &CoordinateOrderDescriptor,
) -> CoordinateOrderDescriptor {
    let mut give = CoordinateOrderDescriptor::default();
    for i in 0..4 {
        give.mult[i] = from.mult[i] / to.mult[i];
        give.post[i] = from.post.iter().position(|&p| p == to.post[i]).unwrap();
    }
    give.noop = give.mult == [1.0; 4] && give.post == [0_usize, 1, 2, 3];
    give
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    // Test that the underlying descriptor-functionality works
    #[test]
    fn descriptor() {
        use coordinate_order_descriptor as descriptor;

        // Axis swap n<->e
        assert_eq!([1usize, 0, 2, 3], descriptor("neuf").unwrap().post);

        // Axis inversion for n+u. Check for all valid angular units
        assert_eq!([1usize, 0, 2, 3], descriptor("sedf_rad").unwrap().post);
        assert_eq!([1usize, 0, 2, 3], descriptor("sedf_gon").unwrap().post);
        assert_eq!([1usize, 0, 2, 3], descriptor("sedf_deg").unwrap().post);
        assert_eq!([-1., 1., -1., 1.], descriptor("sedf_any").unwrap().mult);

        // noop
        assert!(!descriptor("sedf_any").unwrap().noop);
        assert!(descriptor("enuf_any").unwrap().noop);
        assert!(descriptor("enuf_rad").unwrap().noop);
        assert!(descriptor("enuf").unwrap().noop);
        assert!(descriptor("pass").unwrap().noop);

        // Invalid angular unit "pap"
        assert!(descriptor("sedf_pap").is_none());

        // Invalid: Overlapping axes, "ns"
        assert!(descriptor("nsuf").is_none());

        // Now a combination, where we swap both axis order and orientation
        let from = descriptor("neuf_deg").unwrap();
        let to = descriptor("wndf_gon").unwrap();
        let give = combine_descriptors(&from, &to);
        assert_eq!([1_usize, 0, 2, 3], give.post);
        assert!(give.mult[0] + 400. / 360. < 1e-10); // mult[0] is negative for westish
        assert!(give.mult[1] - 400. / 360. < 1e-10); // mult[1] is positive for northish
        assert!(give.mult[2] + 1.0 < 1e-10); // mult[2] is negative for downish
        assert!(give.mult[3] - 1.0 < 1e-10); // mult[3] is positive for timeish
        assert!(!give.noop);
    }

    // Test the basic adapt functionality
    #[test]
    fn adapt() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let gonify = ctx.op("adapt from = neuf_deg   to = enuf_gon")?;

        let mut data = [
            Coor4D::raw(90., 180., 0., 0.),
            Coor4D::raw(45., 90., 0., 0.),
        ];

        assert_eq!(ctx.apply(gonify, Fwd, &mut data)?, 2);
        assert!((data[0][0] - 200.0).abs() < 1e-10);
        assert!((data[0][1] - 100.0).abs() < 1e-10);

        assert!((data[1][0] - 100.0).abs() < 1e-10);
        assert!((data[1][1] - 50.0).abs() < 1e-10);

        assert_eq!(data[1][2], 0.);
        assert_eq!(data[1][3], 0.);

        assert_eq!(ctx.apply(gonify, Inv, &mut data)?, 2);
        assert!((data[0][0] - 90.0).abs() < 1e-10);
        assert!((data[0][1] - 180.0).abs() < 1e-10);
        assert!((data[1][0] - 45.0).abs() < 1e-10);
        assert!((data[1][1] - 90.0).abs() < 1e-10);

        Ok(())
    }

    // Test that 'inv' behaves as if 'from' and 'to' were swapped
    #[test]
    fn adapt_inv() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let degify = ctx.op("adapt inv from = neuf_deg   to = enuf_gon")?;

        let mut data = [
            Coor4D::raw(200., 100., 0., 0.),
            Coor4D::raw(100., 50., 0., 0.),
        ];

        assert_eq!(ctx.apply(degify, Fwd, &mut data)?, 2);
        assert_float_eq!(data[0].0, [90., 180., 0., 0.], abs_all <= 1e-10);
        assert_float_eq!(data[1].0, [45., 90., 0., 0.], abs_all <= 1e-10);

        assert_eq!(ctx.apply(degify, Inv, &mut data)?, 2);
        assert_float_eq!(data[0].0, [200., 100., 0., 0.], abs_all <= 1e-10);
        assert_float_eq!(data[1].0, [100., 50., 0., 0.], abs_all <= 1e-10);

        Ok(())
    }

    // Test that operation without unit conversion works as expected
    #[test]
    fn no_unit_conversion() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        let mut data = crate::test_data::coor2d();
        let swap = ctx.op("adapt from=neuf")?;
        assert_eq!(ctx.apply(swap, Fwd, &mut data)?, 2);
        assert_eq!(data[0][0], 12.0);
        assert_eq!(data[0][1], 55.0);
        Ok(())
    }

    // Test invocation through the geo:* and gis:* macros
    #[test]
    fn geo_gis_and_all_that() -> Result<(), Error> {
        let mut ctx = Minimal::default();

        // Separate :in- and :out-versions, for better readability
        ctx.register_resource("geo:in", "adapt from = neuf_deg");
        ctx.register_resource("geo:out", "geo:in inv");
        ctx.register_resource("gis:in", "adapt from = enuf_deg");
        ctx.register_resource("gis:out", "gis:in inv");

        let utm = ctx.op("geo:in | utm zone=32")?;
        let geo = ctx.op("utm zone=32 inv | geo:out")?;
        let expected = [691875.6321396609, 6098907.825005002];

        // Roundtrip geo->utm->geo, using separate ops for fwd and inv
        let mut data = crate::test_data::coor2d();

        assert_eq!(ctx.apply(utm, Fwd, &mut data)?, 2);
        assert_float_eq!(data[0].0, expected, abs_all <= 1e-9);

        assert_eq!(ctx.apply(geo, Fwd, &mut data)?, 2);
        assert_float_eq!(data[0].0, [55., 12.], abs_all <= 1e-9);

        // Same, but using a plain Inv invocation for the return trip
        let mut data = crate::test_data::coor2d();

        assert_eq!(ctx.apply(utm, Fwd, &mut data)?, 2);
        assert_float_eq!(data[0].0, expected, abs_all <= 1e-9);

        assert_eq!(ctx.apply(utm, Inv, &mut data)?, 2);
        assert_float_eq!(data[0].0, [55., 12.], abs_all <= 1e-9);

        // Swap data by reading them as geo, writing them as gis
        let mut data = crate::test_data::coor2d();
        let swap = ctx.op("geo:in | gis:out")?;
        assert_eq!(ctx.apply(swap, Fwd, &mut data)?, 2);
        assert_float_eq!(data[0].0, [12., 55.], abs_all <= 1e-9);

        Ok(())
    }
}
