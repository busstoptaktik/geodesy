use crate::operator::pipeline::Pipeline;
use crate::CoordinateTuple;
use crate::GeodesyError;
use crate::GysResource;
use crate::Provider;
use log::warn;

pub mod builtins {
    use crate::GeodesyError;
    use crate::OperatorConstructor;

    // A BTreeMap would have been a better choice,for the OPERATOR_LIST, except
    // for the annoying fact that it cannot be compile-time const-constructed
    #[rustfmt::skip]
    const OPERATOR_LIST: [(&str, OperatorConstructor); 11] = [
        ("adapt",      crate::operator::adapt::Adapt::operator),
        ("cart",       crate::operator::cart::Cart::operator),
        ("noop",       crate::operator::noop::Noop::operator),
        ("helmert",    crate::operator::helmert::Helmert::operator),
        ("lcc",        crate::operator::lcc::Lcc::operator),

        ("merc",       crate::operator::merc::Merc::operator),
        ("tmerc",      crate::operator::tmerc::Tmerc::operator),
        ("utm",        crate::operator::tmerc::Tmerc::utmoperator),
        ("molodensky", crate::operator::molodensky::Molodensky::operator),
        ("nmea",       crate::operator::nmea::Nmea::operator),
        ("nmeass",     crate::operator::nmea::Nmea::dmsoperator),
/*
        ("dm",         crate::operator::nmea::Nmea::operator),
        ("dms",        crate::operator::nmea::Nmea::dmsoperator),

 */   ];

    /// Handle instantiation of built-in operators.
    pub fn builtin(name: &str) -> Result<OperatorConstructor, GeodesyError> {
        // The operator name may be prefixed with "builtin_", so operator-named
        // macros can delegate the hard work to the operators they shadow.
        let mut opname = String::from(name).to_lowercase();
        if let Some(stripped) = opname.strip_prefix("builtin_") {
            opname = String::from(stripped)
        }

        if let Some(index) = OPERATOR_LIST.iter().position(|&op| op.0 == opname) {
            return Ok(OPERATOR_LIST[index].1);
        }

        // Not a built in operator
        Err(GeodesyError::NotFound(opname))
    }
}

mod adapt;
mod cart;
mod helmert;
mod lcc;
mod merc;
mod molodensky;
mod nmea;
mod noop;
mod pipeline;
mod tmerc;

// Operator is a newtype around a Boxed trait OperatorCore,
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
    /// # use std::error::Error; fn foo() -> Result<(), Box<dyn Error>> {
    /// let mut ctx = geodesy::Context::new();
    /// let op = ctx.operation("helmert: {x: -87, y: -96, z: -120}")?;
    /// let mut operands = [geodesy::CoordinateTuple::geo(55., 12.,0.,0.)];
    /// ctx.fwd(op, &mut operands);
    /// ctx.inv(op, &mut operands);
    /// assert!((operands[0][0].to_degrees() - 12.).abs() < 1.0e-10);
    /// # Ok(())}
    /// ```
    ///
    pub fn new(definition: &str, ctx: &dyn Provider) -> Result<Operator, GeodesyError> {
        let res = GysResource::new(definition, ctx.globals());
        let op = Pipeline::new(&res, ctx, 0)?;
        Ok(op)
    }
}

use core::fmt::Debug;
impl Debug for Operator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Operator {{{}}}", self.debug())
    }
}

// Forwarding all OperatorCore methods to the boxed content
// Perhaps not necessary: We could deem Core low level and
// build a high level interface on top of Core.
impl OperatorCore for Operator {
    fn fwd(&self, ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        self.0.fwd(ctx, operands)
    }

    fn inv(&self, ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        self.0.inv(ctx, operands)
    }

    fn operate(
        &self,
        operand: &dyn Provider,
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

    fn debug(&self) -> String {
        self.0.debug()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn args(&self, step: usize) -> &[(String, String)] {
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
    fn fwd(&self, ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool;

    // implementations must override at least one of {inv, invertible}
    #[allow(unused_variables)]
    fn inv(&self, ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
        warn!("Operator {} not invertible", self.name());
        false
    }

    fn invertible(&self) -> bool {
        true
    }

    fn is_noop(&self) -> bool {
        false
    }

    // operate fwd/inv, taking operator inversion into account.
    fn operate(&self, ctx: &dyn Provider, operands: &mut [CoordinateTuple], forward: bool) -> bool {
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

    fn debug(&self) -> String {
        String::from(self.name())
    }

    // number of steps. 0 unless the operator is a pipeline
    fn len(&self) -> usize {
        0_usize
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn args(&self, step: usize) -> &[(String, String)];

    fn is_inverted(&self) -> bool;
}

// --------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::CoordinateTuple;
    use crate::GeodesyError;

    #[test]
    fn operator() -> Result<(), GeodesyError> {
        use crate::resource::SearchLevel;
        use crate::{Operator, Plain};
        use crate::{FWD, INV};
        let mut o = Plain::new(SearchLevel::LocalPatches, false);

        // A non-existing operator
        let h = Operator::new("unimplemented_operator: x: -87 y: -96 z: -120", &mut o);
        assert!(h.is_err());

        // Define "hilmert" and "halmert" to circularly define each other, in order
        // to test the operator_factory recursion breaker
        o.register_macro("halmert", "hilmert")?;
        o.register_macro("hilmert", "halmert")?;
        if let Err(err) = Operator::new("halmert x: -87 y: -96 z: -120", &mut o) {
            assert!(err.to_string().contains("too deep recursion"));
        } else {
            panic!();
        }

        // Define "hulmert" as a macro forwarding its args to the "helmert" builtin
        o.register_macro("hulmert", "helmert")?;

        // A plain operator: Helmert, EPSG:1134 - 3 parameter, ED50/WGS84
        let hh = Operator::new("helmert x: -87 y: -96 z: -120", &mut o)?;

        // Same operator, defined through the "hulmert" macro
        let h = Operator::new("hulmert x: -87 y: -96 z: -120", &mut o)?;

        assert_eq!(hh.args(0), h.args(0));

        // Check that the "builtin_" prefix works properly: Shadow "helmert" with a
        // forwarding macro of the same name - without making trouble for later use
        o.register_macro("helmert", "builtin_helmert")?;

        let mut operands = [CoordinateTuple::raw(0., 0., 0., 0.)];

        h.operate(&mut o, operands.as_mut(), FWD);
        assert_eq!(operands[0].first(), -87.);
        assert_eq!(operands[0].second(), -96.);
        assert_eq!(operands[0].third(), -120.);

        h.operate(&mut o, operands.as_mut(), INV);
        assert_eq!(operands[0].first(), 0.);
        assert_eq!(operands[0].second(), 0.);
        assert_eq!(operands[0].third(), 0.);

        // A pipeline
        let pipeline = "cart ellps:intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80";
        let h = Operator::new(pipeline, &mut o)?;

        let mut operands = [CoordinateTuple::gis(12., 55., 100., 0.)];
        h.operate(&mut o, operands.as_mut(), FWD);
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

        /*
                // An externally defined version
                let _h = Operator::new("ed50_etrs89", &mut o)?;

                // Try to access it from data_local_dir (i.e. $HOME/share or somesuch)
                if let Some(mut assets) = dirs::data_local_dir() {
                    assets.push("geodesy");
                    assets.push("assets.zip");
                    if assets.exists() {
                        // If we have access to "assets.zip" we expect to succeed
                        let h = Operator::new("ed50_etrs89", &mut o)?;
                        let mut operands = [CoordinateTuple::gis(12., 55., 100., 0.)];
                        h.operate(&mut o, &mut operands, FWD);
                        let d = operands[0].to_degrees();

                        assert!((d.first() - r.first()).abs() < 1.0e-10);
                        assert!((d.second() - r.second()).abs() < 1.0e-10);
                        assert!((d.third() - r.third()).abs() < 1.0e-8);
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

                o.register_macro("geohelmert", pipeline_as_macro)?;
                let ed50_etrs89 = Operator::new(
                    "geohelmert: {left: intl, right: GRS80, x: -87, y: -96, z: -120}",
                    &mut o,
                )?;
                let mut operands = [CoordinateTuple::gis(12., 55., 100., 0.)];

                ed50_etrs89.operate(&mut o, &mut operands, FWD);
                let d = operands[0].to_degrees();

                assert!((d.first() - r.first()).abs() < 1.0e-10);
                assert!((d.second() - r.second()).abs() < 1.0e-10);
                assert!((d.third() - r.third()).abs() < 1.0e-8);

                ed50_etrs89.operate(&mut o, &mut operands, INV);
                let d = operands[0].to_degrees();

                assert!((d.first() - 12.).abs() < 1.0e-10);
                assert!((d.second() - 55.).abs() < 1.0e-10);
                assert!((d.third() - 100.).abs() < 1.0e-8);
        */
        Ok(())
    }

    use super::OperatorCore;
    use crate::GysResource;
    use crate::Operator;
    use crate::Provider;

    pub struct Nnoopp {
        args: Vec<(String, String)>,
    }

    impl Nnoopp {
        fn new(res: &GysResource) -> Result<Nnoopp, GeodesyError> {
            let args = res.to_args(0)?;
            Ok(Nnoopp { args: args.used })
        }

        pub(crate) fn operator(
            args: &GysResource,
            _rp: &dyn Provider,
        ) -> Result<Operator, GeodesyError> {
            let op = Nnoopp::new(args)?;
            Ok(Operator(Box::new(op)))
        }
    }

    impl OperatorCore for Nnoopp {
        fn fwd(&self, _ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
            for coord in operands {
                coord[0] = 42.;
            }
            true
        }

        fn inv(&self, _ctx: &dyn Provider, operands: &mut [CoordinateTuple]) -> bool {
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

        fn args(&self, _step: usize) -> &[(String, String)] {
            &self.args
        }
    }

    #[test]
    fn user_defined_operator() -> Result<(), GeodesyError> {
        let mut ctx = crate::Plain::default();
        ctx.register_operator("nnoopp", Nnoopp::operator)?;

        let op = ctx.operation("nnoopp");
        dbg!(&op);
        let op = op.unwrap();
        let mut operands = [CoordinateTuple::raw(12., 55., 100., 0.)];
        let _aha = ctx.fwd(op, operands.as_mut());
        assert_eq!(operands[0][0], 42.);
        Ok(())
    }
}
