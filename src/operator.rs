use crate::operator_construction::OperatorArgs;
use crate::Context;
use crate::CoordinateTuple;

// Operator is a newtype around a Boxed OperatorCore,
// in order to be able to define methods on it.
// There's a good description of the crux here:
// https://stackoverflow.com/questions/35568871/is-it-possible-to-implement-methods-on-type-aliases
pub struct Operator(pub Box<dyn OperatorCore>);

impl Operator {
    /// The equivalent of the PROJ `proj_create()` function: Create an operator object
    /// from a text string.
    ///
    /// Example:
    /// ```rust
    /// // EPSG:1134 - 3 parameter Helmert, ED50/WGS84
    /// let mut ctx = geodesy::Context::new();
    /// let op = ctx.operation("helmert: {x: -87, y: -96, z: -120}");
    /// assert!(op.is_some());
    /// let op = op.unwrap();
    /// let mut operands = [geodesy::CoordinateTuple::geo(55., 12.,0.,0.)];
    /// ctx.fwd(op, &mut operands);
    /// ctx.inv(op, &mut operands);
    /// assert!((operands[0][0].to_degrees() - 12.).abs() < 1.0e-10);
    /// ```
    pub fn new(definition: &str, ctx: &mut Context) -> Option<Operator> {
        // If it is a macro without arguments, we append an empty argument list
        let mut definition = definition.trim().to_string();
        let first_word: String = definition.split(':').take(1).collect();
        if definition.len() == first_word.len() {
            definition += ": {}";
        }

        let mut oa = OperatorArgs::new();
        oa.populate(&definition, "");
        operator_factory(&mut oa, ctx, 0)
    }

    pub fn forward(&self, ws: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        self.0.fwd(ws, operands)
    }

    pub fn inverse(&self, ws: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        self.0.inv(ws, operands)
    }
}

use core::fmt::Debug;
impl Debug for Operator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Operator {{{}}}", self.name())
    }
}

// Forwarding all OperatorCore methods to the boxed content
// Perhaps not necessary: We could deem Core low level and
// build a high level interface on top of Core (cf forward above).
impl OperatorCore for Operator {
    fn fwd(&self, ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        self.0.fwd(ctx, operands)
    }

    fn inv(&self, ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        self.0.inv(ctx, operands)
    }

    fn operate(
        &self,
        operand: &mut Context,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        self.0.operate(operand, operands, forward)
    }

    fn invertible(&self) -> bool {
        self.0.invertible()
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn args(&self, step: usize) -> &OperatorArgs {
        self.0.args(step)
    }

    fn is_inverted(&self) -> bool {
        self.0.is_inverted()
    }
}

/// The core functionality exposed by the individual operator implementations.
/// This is not immediately intended for application program consumption: The
/// actual API is in the `impl`ementation for the [`Operator`](Operator) newtype struct,
/// which builds on this `trait` (which only holds `pub`ness in order to support
/// construction of user-defined operators).
pub trait OperatorCore {
    fn fwd(&self, ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool;

    // implementations must override at least one of {inv, invertible}
    #[allow(unused_variables)]
    fn inv(&self, ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
        ctx.error(self.name(), "Operator not invertible");
        false
    }

    fn invertible(&self) -> bool {
        true
    }

    // operate fwd/inv, taking operator inversion into account.
    fn operate(&self, ctx: &mut Context, operands: &mut [CoordinateTuple], forward: bool) -> bool {
        // Short form of (inverted && !forward) || (forward && !inverted)
        if self.is_inverted() != forward {
            return self.fwd(ctx, operands);
        }
        // We do not need to check for self.invertible() here, since non-invertible
        // operators will return false as per the default-defined fn inv() above.
        self.inv(ctx, operands)
    }

    fn name(&self) -> &'static str {
        "UNKNOWN"
    }

    // number of steps. 0 unless the operator is a pipeline
    fn len(&self) -> usize {
        0_usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn args(&self, step: usize) -> &OperatorArgs;

    fn is_inverted(&self) -> bool;

    //fn left(&self) -> CoordType;
    //fn right(&self) -> CoordType;
}

mod cart;
mod helmert;
mod noop;
mod pipeline;
mod tmerc;

pub(crate) fn operator_factory(
    args: &mut OperatorArgs,
    ctx: &mut Context,
    recursions: usize,
) -> Option<Operator> {
    use crate::operator as co;

    if recursions > 100 {
        ctx.error("Unknown", "Operator definition too deeply nested");
        return None;
    }

    // Look for runtime defined macros
    if let Some(definition) = ctx.locate_macro(&args.name) {
        let mut moreargs = args.spawn(definition);
        return operator_factory(&mut moreargs, ctx, recursions + 1);
    }

    // Look for macros defined by files in the current working directory
    if let Ok(definition) = std::fs::read_to_string(args.name.clone() + ".yml") {
        let mut moreargs = args.spawn(&definition);
        return operator_factory(&mut moreargs, ctx, recursions + 1);
    }

    // Look for runtime defined operators
    if let Some(op) = ctx.locate_operator(&args.name) {
        let op = op(args, ctx);
        match op {
            Err(e) => {
                ctx.error(e, "Runtime defined operator lookup");
                return None;
            }
            Ok(op) => {
                return Some(op);
            }
        }
    }

    // Builtins

    // Pipelines are not characterized by the name "pipeline", but simply by containing steps.
    if let Ok(steps) = args.numeric_value("_nsteps", 0.0) {
        if steps > 0.0 {
            match co::pipeline::Pipeline::new(args, ctx) {
                Err(err) => {
                    ctx.error(err, "pipeline");
                    return None;
                }
                Ok(ok) => {
                    return Some(Operator(Box::new(ok)));
                }
            }
        }
    }

    let mut op: Result<Operator, &'static str> = Err("Operator name not found");
    let mut opname = "operator_factory";
    if args.name == "helmert" {
        op = crate::operator::helmert::Helmert::operator(args);
        opname = "helmert";
    } else if args.name == "cart" {
        op = crate::operator::cart::Cart::operator(args);
        opname = "cart";
    } else if args.name == "tmerc" {
        op = co::tmerc::Tmerc::operator(args);
        opname = "tmerc";
    } else if args.name == "utm" {
        op = co::tmerc::Tmerc::utmoperator(args);
        opname = "utm";
    } else if args.name == "noop" {
        op = co::noop::Noop::operator(args);
        opname = "noop";
    }

    // Look in the shared assets directory ($HOME/share/geodesy or somesuch)
    if let Some(mut dir) = dirs::data_local_dir() {
        dir.push("geodesy");
        let filename = args.name.clone() + ".yml";
        let mut fullpath = dir.clone();
        fullpath.push("cookbook");
        fullpath.push("transformations");
        fullpath.push(filename.clone());
        if let Ok(definition) = std::fs::read_to_string(fullpath) {
            let mut moreargs = args.spawn(&definition);
            return operator_factory(&mut moreargs, ctx, recursions + 1);
        }

        // If not found as freestanding file, try cookbook.zip
        use std::io::prelude::*;
        dir.push("cookbook.zip");
        // Open the physical zip file
        if let Ok(zipfile) = std::fs::File::open(dir) {
            // Hand it over to the zip archive reader
            if let Ok(mut archive) = zip::ZipArchive::new(zipfile) {
                // Is there a file with the name we're looking for in the zip archive?
                let full_filename = String::from("cookbook/transformations/") + &filename;
                if let Ok(mut file) = archive.by_name(&full_filename) {
                    let mut definition = String::new();
                    if file.read_to_string(&mut definition).is_ok() {
                        // Fine! Treat it just like any other macro!
                        let mut moreargs = args.spawn(&definition);
                        return operator_factory(&mut moreargs, ctx, recursions + 1);
                    }
                }
            }
        }
    }

    // Done - translate Result<Operator, str> to Option<Operator>...
    match op {
        Err(err) => {
            ctx.error(opname, err);
            None
        }
        Ok(ok) => Some(ok),
    }
}

// --------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::CoordinateTuple;

    #[test]
    fn operator() {
        use crate::operator_construction::*;
        use crate::{fwd, inv, Context};
        let mut o = Context::new();

        // A non-existing operator
        let h = Operator::new("unimplemented_operator: {x: -87, y: -96, z: -120}", &mut o);
        assert!(h.is_none());

        // Define "hilmert" and "halmert" to circularly define each other, in order
        // to test the operator_factory recursion breaker
        assert!(o.register_macro("halmert", "hilmert: {}"));
        assert!(o.register_macro("hilmert", "halmert: {}"));
        if let None = Operator::new("halmert: {x: -87, y: -96, z: -120}", &mut o) {
            assert!(o.report().contains("too deeply nested"));
        } else {
            panic!();
        }

        // Define "hulmert" as a macro forwarding its args to the "helmert" builtin
        assert!(o.register_macro("hulmert", "helmert: {x: ^x, y: ^y, z: ^z}"));

        // A plain operator: Helmert, EPSG:1134 - 3 parameter, ED50/WGS84
        let hh = Operator::new("helmert: {x: -87, y: -96, z: -120}", &mut o);
        assert!(hh.is_some());
        let hh = hh.unwrap();

        // Same operator, defined through the "hulmert" macro
        let h = Operator::new("hulmert: {x: -87, y: -96, z: -120}", &mut o);
        assert!(h.is_some());
        let h = h.unwrap();

        assert_eq!(hh.args(0).name, h.args(0).name);
        assert_eq!(hh.args(0).used, h.args(0).used);

        let mut operands = [CoordinateTuple::raw(0., 0., 0., 0.)];

        h.operate(&mut o, operands.as_mut(), fwd);
        assert_eq!(operands[0].first(), -87.);
        assert_eq!(operands[0].second(), -96.);
        assert_eq!(operands[0].third(), -120.);

        h.operate(&mut o, operands.as_mut(), inv);
        assert_eq!(operands[0].first(), 0.);
        assert_eq!(operands[0].second(), 0.);
        assert_eq!(operands[0].third(), 0.);

        h.forward(&mut o, operands.as_mut());
        assert_eq!(operands[0].first(), -87.);
        assert_eq!(operands[0].second(), -96.);
        assert_eq!(operands[0].third(), -120.);

        h.inverse(&mut o, operands.as_mut());
        assert_eq!(operands[0].first(), 0.);
        assert_eq!(operands[0].second(), 0.);
        assert_eq!(operands[0].third(), 0.);

        // A pipeline
        let pipeline = "ed50_etrs89: {
            steps: [
                cart: {ellps: intl},
                helmert: {x: -87, y: -96, z: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";
        let h = Operator::new(pipeline, &mut o);
        assert!(h.is_some());
        let h = h.unwrap();

        let mut operands = [CoordinateTuple::gis(12., 55., 100., 0.)];
        h.forward(&mut o, operands.as_mut());
        let d = operands[0].to_degrees();
        let r = CoordinateTuple::raw(
            11.998815342385209,
            54.99938264895106,
            131.20240108577374,
            0.0,
        );

        assert!((d.first() - r.first()).abs() < 1.0e-10);
        assert!((d.second() - r.second()).abs() < 1.0e-10);
        assert!((d.third() - r.third()).abs() < 1.0e-8);

        // An externally defined version
        let h = Operator::new("tests/ed50_etrs89", &mut o);
        assert!(h.is_some());

        // Try to access it from data_local_dir (i.e. $HOME/share or somesuch)
        let h = Operator::new("ed50_etrs89", &mut o);
        // If we have access to "cookbook.zip" we expect to succeed
        if let Some(mut cookbook) = dirs::data_local_dir() {
            cookbook.push("geodesy");
            cookbook.push("cookbook.zip");
            if cookbook.exists() {
                assert!(h.is_some());
                let mut operands = [CoordinateTuple::gis(12., 55., 100., 0.)];
                h.unwrap().forward(&mut o, operands.as_mut());
                let d = operands[0].to_degrees();

                assert!((d.first() - r.first()).abs() < 1.0e-10);
                assert!((d.second() - r.second()).abs() < 1.0e-10);
                assert!((d.third() - r.third()).abs() < 1.0e-8);
            } else {
                assert!(h.is_none());
            }
        }

        // A parameterized macro pipeline version
        let pipeline_as_macro = "pipeline: {
            globals: {
                leftleft: ^left
            },
            steps: [
                cart: {ellps: ^leftleft},
                helmert: {x: ^x, y: ^y, z: ^z},
                cart: {inv: true, ellps: ^right}
            ]
        }";

        assert!(o.register_macro("geohelmert", pipeline_as_macro));
        let ed50_etrs89 = Operator::new(
            "geohelmert: {left: intl, right: GRS80, x: -87, y: -96, z: -120}",
            &mut o,
        );
        assert!(ed50_etrs89.is_some());
        let ed50_etrs89 = ed50_etrs89.unwrap();
        let mut operands = [CoordinateTuple::gis(12., 55., 100., 0.)];

        ed50_etrs89.forward(&mut o, operands.as_mut());
        let d = operands[0].to_degrees();

        assert!((d.first() - r.first()).abs() < 1.0e-10);
        assert!((d.second() - r.second()).abs() < 1.0e-10);
        assert!((d.third() - r.third()).abs() < 1.0e-8);

        ed50_etrs89.inverse(&mut o, operands.as_mut());
        let d = operands[0].to_degrees();

        assert!((d.first() - 12.).abs() < 1.0e-10);
        assert!((d.second() - 55.).abs() < 1.0e-10);
        assert!((d.third() - 100.).abs() < 1.0e-8);
    }

    use super::Context;
    use super::Operator;
    use super::OperatorArgs;
    use super::OperatorCore;

    pub struct Nnoopp {
        args: OperatorArgs,
    }

    impl Nnoopp {
        fn new(args: &mut OperatorArgs) -> Result<Nnoopp, &'static str> {
            Ok(Nnoopp { args: args.clone() })
        }

        pub(crate) fn operator(
            args: &mut OperatorArgs,
            _ctx: &mut Context,
        ) -> Result<Operator, &'static str> {
            let op = Nnoopp::new(args)?;
            Ok(Operator { 0: Box::new(op) })
        }
    }

    impl OperatorCore for Nnoopp {
        fn fwd(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
            for coord in operands {
                coord[0] = 42.;
            }
            true
        }

        fn inv(&self, _ctx: &mut Context, operands: &mut [CoordinateTuple]) -> bool {
            for coord in operands {
                coord[0] = 24.;
            }
            true
        }

        fn name(&self) -> &'static str {
            "nnoopp"
        }

        fn is_inverted(&self) -> bool {
            false
        }

        fn args(&self, _step: usize) -> &OperatorArgs {
            &self.args
        }
    }

    #[test]
    fn user_defined_operator() {
        let mut ctx = Context::new();
        ctx.register_operator("nnoopp", Nnoopp::operator);

        let op = ctx.operation("nnoopp: {}").unwrap();
        let mut operands = [CoordinateTuple::raw(12., 55., 100., 0.)];
        let _aha = ctx.fwd(op, operands.as_mut());
        assert_eq!(operands[0][0], 42.);
    }
}
