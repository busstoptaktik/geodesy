/*! Declarative approach to fitting input data in one format to output data in another.

Example:

```gys
fit have: neut_deg  want: enut_rad
```

We introduce the coordinate type designations "eastish, northish, upish, timish",
and their geometrical inverses "westish, southish, downish, reversed-timeish",
with mostly evident meaning, and convenient short forms `e, n, u, t` and `w, s, d, r`,
respectively.

Also, we introduce the 3 external angular representations "degrees, gradians, radians",
conveniently abbrevieated as "deg", "gon" and "rad".

The Rust Geodesy internal format of a four dimensional coordinate tuple is e, n, u, t,
and the internal unit of measure for anglular coordinates is radians. In `fit`, terms,
this is described as `enut_rad`.

`fit` covers the same ground as the `PROJ` operator `axisswap`, but using a somewhat
different approach: You never tell `fit` what you want it to do - you only tell it
what you `have`, and what you `want` (and in most cases actually only one of those), then
`fit` figures out how to fulfill the order.

The example above specifies that an input coordinate tuple with coordinate order
**latitude, longitude, height, time**, with latitude and longitude in degrees, should be
converted to an output coordinate in radians and with latitude and longitude swapped.
That output format is identical to the default internal format, so it can actually
be left out, and the order can simply be written as

```gys
fit have: neut_deg
```

Typically, `fit` is used in both ends of a pipeline, to match input and output data:

```gys
fit have: neut_deg | cart ... | helmert ... | cart inv ... | fit want: neut_deg
```

Note that `fit want: ...` and `fit inv have: ...`. The latter format is useful when
using RG's predefined symbolic definitions, as in:

```gys
geo | cart ... | helmert ... | cart inv ... | geo inv
```

As `fit` really provides impedance matching between input and output conventions,
it can also be called as `match`:

```gys
match have: neut_deg  want: enut_rad
```

which can be straightforwardly read as "match this to that". Since `match` is a
reserved word in the Rust programming language, using it as an identifier of
code structure is inconvenient, hence the primary name of `fit`.

!*/

use super::OperatorArgs;
use super::OperatorCore;
use crate::operator_construction::*;
use crate::Context;
use crate::CoordinateTuple;

pub struct Fit {
    args: OperatorArgs,
    inverted: bool,
    post: [usize; 4],
    mult: [f64; 4],
    noop: bool,
}

#[derive(Debug, Default, Clone)]
struct CoordinateOrderDescriptor {
    post: [usize; 4],
    mult: [f64; 4],
    noop: bool,
}

#[allow(clippy::float_cmp)]
fn descriptor(desc: &str) -> Option<CoordinateOrderDescriptor> {
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
        if !"neutswdr".contains(d) {
            return None;
        }
        // Sign and position in the internal representation
        let dd: i32 = match d {
            'w' => -1,
            's' => -2,
            'd' => -3,
            'r' => -4,
            'e' => 1,
            'n' => 2,
            'u' => 3,
            't' => 4,
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
        println!("Overlaps: {:?}", indices);
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
    have: &CoordinateOrderDescriptor,
    want: &CoordinateOrderDescriptor,
) -> CoordinateOrderDescriptor {
    let mut give = CoordinateOrderDescriptor::default();
    for i in 0..4 {
        give.mult[i] = have.mult[i] / want.mult[i];
        give.post[i] = have.post.iter().position(|&p| p == want.post[i]).unwrap();
    }
    give.noop = give.mult == [1.0; 4] && give.post == [0_usize, 1, 2, 3];
    give
}

impl Fit {
    pub fn new(args: &mut OperatorArgs) -> Result<Fit, &'static str> {
        let inverted = args.flag("inv");

        // What we `have` and what we `want` both defaults to the internal
        // representation - i.e. "do nothing", neither on in- or output.
        let have = args.value("have", "enut");
        let want = args.value("want", "enut");

        let desc = descriptor(&have);
        if desc.is_none() {
            return Err("Bad value for 'have'");
        }
        let have = desc.unwrap();

        let desc = descriptor(&want);
        if desc.is_none() {
            return Err("Bad value for 'want'");
        }
        let want = desc.unwrap();

        // Eliminate redundancy for over-specified cases.
        let give = combine_descriptors(&have, &want);

        Ok(Fit {
            args: args.clone(),
            inverted,
            post: give.post,
            mult: give.mult,
            noop: give.noop,
        })
    }

    pub(crate) fn operator(args: &mut OperatorArgs) -> Result<Operator, &'static str> {
        let op = crate::operator::fit::Fit::new(args)?;
        Ok(Operator(Box::new(op)))
    }
}

impl OperatorCore for Fit {
    fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        if self.noop {
            return true;
        }
        for o in operands {
            *o = CoordinateTuple([
                o[self.post[0]] * self.mult[0],
                o[self.post[1]] * self.mult[1],
                o[self.post[2]] * self.mult[2],
                o[self.post[3]] * self.mult[3],
            ]);
        }
        true
    }

    fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        if self.noop {
            return true;
        }
        for o in operands {
            let mut c = CoordinateTuple::default();
            for i in 0..4_usize {
                c[self.post[i]] = o[i] / self.mult[i];
            }
            *o = c;
        }
        true
    }

    fn name(&self) -> &'static str {
        "fit"
    }

    fn is_noop(&self) -> bool {
        self.noop
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
    #[test]
    fn descriptor() {
        use super::combine_descriptors;
        use super::descriptor;

        // Axis swap n<->e
        assert_eq!([1usize, 0, 2, 3], descriptor("neut").unwrap().post);

        // Axis inversion for n+u. Check for all valid angular units
        assert_eq!([1usize, 0, 2, 3], descriptor("sedt_rad").unwrap().post);
        assert_eq!([1usize, 0, 2, 3], descriptor("sedt_gon").unwrap().post);
        assert_eq!([1usize, 0, 2, 3], descriptor("sedt_deg").unwrap().post);
        assert_eq!([-1., 1., -1., 1.], descriptor("sedt_any").unwrap().mult);

        // noop
        assert_eq!(false, descriptor("sedt_any").unwrap().noop);
        assert_eq!(true, descriptor("enut_any").unwrap().noop);
        assert_eq!(true, descriptor("enut_rad").unwrap().noop);
        assert_eq!(true, descriptor("enut").unwrap().noop);
        assert_eq!(true, descriptor("pass").unwrap().noop);

        // Invalid angular unit "pap"
        assert!(descriptor("sedt_pap").is_none());

        // Invalid: Overlapping axes, "ns"
        assert!(descriptor("nsut").is_none());

        // Now a combination, where we swap both axis order and orientation
        let have = descriptor("neut_deg").unwrap();
        let want = descriptor("wndt_gon").unwrap();
        let give = combine_descriptors(&have, &want);
        assert_eq!([1_usize, 0, 2, 3], give.post);
        assert!(give.mult[0] + 400. / 360. < 1e-10);
        assert!(give.mult[1] - 400. / 360. < 1e-10);
        assert!(give.mult[2] + 1.0 < 1e-10);
        assert!(give.mult[3] - 1.0 < 1e-10);
        assert!(give.noop == false);
    }

    #[test]
    fn fit() {
        use crate::Context;
        use crate::CoordinateTuple;
        let mut ctx = Context::new();

        let gonify = ctx
            .operation("match: {have: neut_deg, want: enut_gon}")
            .unwrap();
        let mut operands = [
            CoordinateTuple::raw(90., 180., 0., 0.),
            CoordinateTuple::raw(45., 90., 0., 0.),
        ];

        ctx.fwd(gonify, &mut operands);
        assert!((operands[0][0] - 200.0).abs() < 1e-10);
        assert!((operands[0][1] - 100.0).abs() < 1e-10);
        assert!((operands[1][0] - 100.0).abs() < 1e-10);
        assert!((operands[1][1] - 50.0).abs() < 1e-10);

        ctx.inv(gonify, &mut operands);
        assert!((operands[0][0] - 90.0).abs() < 1e-10);
        assert!((operands[0][1] - 180.0).abs() < 1e-10);
        assert!((operands[1][0] - 45.0).abs() < 1e-10);
        assert!((operands[1][1] - 90.0).abs() < 1e-10);
    }
}
