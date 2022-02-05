use super::internal::*;

mod op_descriptor;
mod parameter;
mod parsed_parameters;
mod raw_parameters;

pub use op_descriptor::OpDescriptor;
pub use parameter::OpParameter;
pub use parsed_parameters::ParsedParameters;
pub use raw_parameters::RawParameters;

#[derive(Debug)]
pub struct Op {
    pub descriptor: OpDescriptor,
    pub params: ParsedParameters,
    pub steps: Vec<Op>,
}

impl Op {
    // operate fwd/inv, taking operator inversion into account.
    pub fn apply(
        &self,
        ctx: &dyn Provider,
        operands: &mut [CoordinateTuple],
        direction: Direction,
    ) -> usize {
        let forward = direction == Direction::Fwd;
        // Short form of (inverted && !forward) || (forward && !inverted)
        if self.descriptor.inverted != forward {
            return self.descriptor.fwd.0(self, ctx, operands);
        }
        self.descriptor.inv.0(self, ctx, operands)
    }

    pub fn new(definition: &str, provider: &dyn Provider) -> Result<Op, Error> {
        let globals = provider.globals();
        let parameters = RawParameters::new(definition, &globals);
        Self::op(parameters, provider)
    }

    // Helper for implementation of `InnerOp`s: Instantiate an `Op` for the simple
    // (and common) case, where the `InnerOp` constructor does mot need to set any
    // additional parameters than the ones defined by the instantiation parameter
    // arguments.
    pub fn plain(
        parameters: &RawParameters,
        fwd: InnerOp,
        inv: InnerOp,
        gamut: &[OpParameter],
        _provider: &dyn Provider,
    ) -> Result<Op, Error> {
        let def = &parameters.definition;
        let params = ParsedParameters::new(parameters, &gamut)?;
        let descriptor = OpDescriptor::new(def, fwd, Some(inv));
        let steps = Vec::<Op>::new();
        Ok(Op {
            descriptor,
            params,
            steps,
        })
    }

    // Instantiate the actual operator, taking into account the relative order
    // of precendence between pipelines, user defined operators, macros, and
    // built-in operators
    #[allow(clippy::self_named_constructors)]
    pub fn op(parameters: RawParameters, provider: &dyn Provider) -> Result<Op, Error> {
        if parameters.nesting_too_deep() {
            return Err(Error::Recursion(
                parameters.invocation,
                parameters.definition,
            ));
        }
        let definition = parameters.definition.clone();
        let name = etc::operator_name(&definition, "");

        // A pipeline?
        if etc::is_pipeline(&definition) {
            return super::inner_op::pipeline::new(&parameters, provider)?.handle_inversion();
        }

        // A user defined operator?
        if !etc::is_resource_name(&name) {
            if let Ok(constructor) = provider.get_op(&name) {
                return constructor.0(&parameters, provider)?.handle_inversion();
            }
        }
        // A user defined macro?
        else if let Ok(macro_definition) = provider.get_resource(&name) {
            let mut next_param = parameters.next(&definition);
            next_param.definition = macro_definition;
            return Op::op(next_param, provider)?.handle_inversion();
        }

        // A built in operator?
        if let Ok(constructor) = super::inner_op::builtin(&name) {
            return constructor.0(&parameters, provider)?.handle_inversion();
        }

        Err(super::Error::NotFound(name, ": ".to_string() + &definition))
    }

    fn handle_inversion(mut self) -> Result<Op, Error> {
        if self.params.boolean("inv") {
            self.descriptor.inverted = true;
        }
        Ok(self)
    }
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Test the fundamental Op-functionality: That we can actually instantiate
    // an Op, and invoke its forward and backward operational modes
    #[test]
    fn basic() -> Result<(), Error> {
        let mut provider = Minimal::default();

        // Try to invoke garbage as a user defined Op
        assert!(matches!(
            Op::new("_foo", &provider),
            Err(Error::NotFound(_, _))
        ));

        // Check forward and inverse operation
        // let op = Op::new("addone", &provider)?;
        let op = provider.op("addone")?;
        let mut data = etc::some_basic_coordinates();
        provider.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        provider.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Also for an inverted operator: check forward and inverse operation
        let op = provider.op("addone inv")?;
        // let op = Op::new("addone inv", &provider)?;
        let mut data = etc::some_basic_coordinates();
        provider.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 54.);
        assert_eq!(data[1][0], 58.);
        provider.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    // Test that the recursion-breaker works properly, by defining two mutually
    // dependent macros: `foo:bar=foo:baz` and `foo:baz=foo:bar`, and checking
    // that the instantiation fails with an `Error::Recursion(...)`
    #[test]
    fn nesting() -> Result<(), Error> {
        let mut prv = Minimal::default();
        prv.register_resource("foo:baz", "foo:bar");
        prv.register_resource("foo:bar", "foo:baz");

        assert_eq!("foo:baz", prv.get_resource("foo:bar")?);
        assert_eq!("foo:bar", prv.get_resource("foo:baz")?);

        assert!(matches!(prv.op("foo:baz"), Err(Error::Recursion(_, _))));
        Ok(())
    }

    #[test]
    fn pipeline() -> Result<(), Error> {
        let mut data = etc::some_basic_coordinates();
        let mut prv = Minimal::default();
        let op = prv.op("addone|addone|addone")?;

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    // TODO/todo!(): Test macro expansion (likely wrong - check raw_parameters.rs)
}
