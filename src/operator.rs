use crate::Operand;
use crate::OperatorArgs;

// Operator used to be a `pub type Operator = Box<dyn OperatorCore>`, but now it's
// a newtype around a Boxed OperatorCore, in order to be able to define methods on
// it. There's a good description of the crux here:
// https://stackoverflow.com/questions/35568871/is-it-possible-to-implement-methods-on-type-aliases
pub struct Operator(Box<dyn OperatorCore>);
impl Operator {
    /// The equivalent of the PROJ `proj_create()` function: Create an operator object
    /// from a text string.
    ///
    /// Example:
    /// ```rust
    /// // EPSG:1134 - 3 parameter, ED50/WGS84
    /// use geodesy::OperatorCore;
    /// let h = geodesy::Operator::new("helmert: {dx: -87, dy: -96, dz: -120}");
    /// assert!(h.is_ok());
    /// let h = h.unwrap();
    /// let mut o = geodesy::Operand::new();
    /// h.operate(&mut o, geodesy::fwd);
    /// ```
    pub fn new(definition: &str) -> Result<Operator, String> {
        let mut oa = OperatorArgs::global_defaults();
        oa.populate(definition, "");
        operator_factory(&mut oa)
    }

    pub fn forward(&self, ws: &mut Operand) -> bool {
        self.0.fwd(ws)
    }

    pub fn inverse(&self, ws: &mut Operand) -> bool {
        self.0.inv(ws)
    }
}

// Forwarding all OperatorCore methods to the boxed content
// Perhaps not necessary: We could deem Core low level and
// build a high level interface on top of Core (cf forward above).
impl OperatorCore for Operator {
    fn fwd(&self, ws: &mut Operand) -> bool {
        self.0.fwd(ws)
    }

    fn inv(&self, ws: &mut Operand) -> bool {
        self.0.inv(ws)
    }

    fn operate(&self, operand: &mut Operand, forward: bool) -> bool {
        self.0.operate(operand, forward)
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
/// which builds on this `trait` (which will probably soon lose its `pub`ness).
pub trait OperatorCore {
    fn fwd(&self, ws: &mut Operand) -> bool;

    // implementations must override at least one of {inv, invertible}
    fn inv(&self, operand: &mut Operand) -> bool {
        operand.last_failing_operation = self.name();
        operand.cause = "Operator not invertible";
        false
    }

    fn invertible(&self) -> bool {
        true
    }

    // operate fwd/inv, taking operator inversion into account.
    fn operate(&self, operand: &mut Operand, forward: bool) -> bool {
        // Short form of (inverted && !forward) || (forward && !inverted)
        if self.is_inverted() != forward {
            return self.fwd(operand);
        }
        // We do not need to check for self.invertible() here, since non-invertible
        // operators will return false as per the default-defined fn inv() above.
        self.inv(operand)
    }

    fn name(&self) -> &'static str {
        "UNKNOWN"
    }

    // number of steps. 0 unless the operator is a pipeline
    fn len(&self) -> usize {
        0_usize
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

pub fn operator_factory(args: &mut OperatorArgs) -> Result<Operator, String> {
    use crate::operator as co;

    // Pipelines do not need to be named "pipeline": They are characterized simply
    // by containing steps.
    if args.name == "pipeline" || args.numeric_value("operator_factory", "_nsteps", 0.0)? > 0.0 {
        let op = co::pipeline::Pipeline::new(args)?;
        return Ok(Operator(Box::new(op)));
    }
    if args.name == "cart" {
        let op = co::cart::Cart::new(args)?;
        return Ok(Operator(Box::new(op)));
    }
    if args.name == "helmert" {
        let op = co::helmert::Helmert::new(args)?;
        return Ok(Operator(Box::new(op)));
    }
    if args.name == "tmerc" {
        let op = co::tmerc::Tmerc::new(args)?;
        return Ok(Operator(Box::new(op)));
    }
    if args.name == "utm" {
        let op = co::tmerc::Tmerc::utm(args)?;
        return Ok(Operator(Box::new(op)));
    }
    if args.name == "noop" {
        let op = co::noop::Noop::new(args)?;
        return Ok(Operator(Box::new(op)));
    }

    // Herefter: Søg efter 'name' i filbøtten
    Err(format!("Unknown operator '{}'", args.name))
}

// --------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operator() {
        use crate::{fwd, inv, Operand, Operator, OperatorCore};
        let mut o = Operand::new();

        // A plain operator: Helmert, EPSG:1134 - 3 parameter, ED50/WGS84
        let h = Operator::new("helmert: {dx: -87, dy: -96, dz: -120}");
        assert!(h.is_ok());
        let h = h.unwrap();

        h.operate(&mut o, fwd);
        assert_eq!(o.coord.first(), -87.);
        assert_eq!(o.coord.second(), -96.);
        assert_eq!(o.coord.third(), -120.);

        h.operate(&mut o, inv);
        assert_eq!(o.coord.first(), 0.);
        assert_eq!(o.coord.second(), 0.);
        assert_eq!(o.coord.third(), 0.);

        h.forward(&mut o);
        assert_eq!(o.coord.first(), -87.);
        assert_eq!(o.coord.second(), -96.);
        assert_eq!(o.coord.third(), -120.);

        h.inverse(&mut o);
        assert_eq!(o.coord.first(), 0.);
        assert_eq!(o.coord.second(), 0.);
        assert_eq!(o.coord.third(), 0.);

        // A non-existing operator
        let h = Operator::new("unimplemented_operator: {dx: -87, dy: -96, dz: -120}");
        assert!(h.is_err());

        // A pipeline
        let pipeline = "ed50_etrs89: {
            steps: [
                cart: {ellps: intl},
                helmert: {dx: -87, dy: -96, dz: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";

        let h = Operator::new(pipeline);
        assert!(h.is_ok());
    }
}
