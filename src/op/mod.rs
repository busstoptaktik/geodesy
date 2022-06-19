use super::internal::*;

mod op_descriptor;
mod parameter;
mod parsed_parameters;
mod raw_parameters;

pub use op_descriptor::OpDescriptor;
pub use parameter::OpParameter;
pub use parsed_parameters::ParsedParameters;
pub use raw_parameters::RawParameters;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct OpHandle(uuid::Uuid);
impl OpHandle {
    pub fn new() -> Self {
        OpHandle(uuid::Uuid::new_v4())
    }
}
impl Default for OpHandle {
    fn default() -> Self {
        OpHandle(uuid::Uuid::new_v4())
    }
}

/// The defining parameters and functions for an operator
#[derive(Debug)]
pub struct Op {
    pub descriptor: OpDescriptor,
    pub params: ParsedParameters,
    pub steps: Vec<Op>,
    pub id: OpHandle,
}

impl Op {
    // operate fwd/inv, taking operator inversion into account.
    pub fn apply(
        &self,
        ctx: &dyn Provider,
        operands: &mut [Coord],
        direction: Direction,
    ) -> Result<usize, Error> {
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
    // (and common) case, where the `InnerOp` constructor does not need to set any
    // other parameters than the ones defined by the instantiation parameter
    // arguments.
    pub fn plain(
        parameters: &RawParameters,
        fwd: InnerOp,
        inv: InnerOp,
        gamut: &[OpParameter],
        _provider: &dyn Provider,
    ) -> Result<Op, Error> {
        let def = parameters.definition.as_str();
        let params = ParsedParameters::new(parameters, gamut)?;
        let descriptor = OpDescriptor::new(def, fwd, Some(inv));
        let steps = Vec::<Op>::new();
        let id = OpHandle::new();

        Ok(Op {
            descriptor,
            params,
            steps,
            id,
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

        let name = operator_name(&parameters.definition, "");

        // A pipeline?
        if is_pipeline(&parameters.definition) {
            return super::inner_op::pipeline::new(&parameters, provider);
        }

        // A user defined operator?
        if !is_resource_name(&name) {
            if let Ok(constructor) = provider.get_op(&name) {
                return constructor.0(&parameters, provider)?.handle_op_inversion();
            }
        }
        // A user defined macro?
        else if let Ok(macro_definition) = provider.get_resource(&name) {
            // The " " sentinel simplifies search for "inv", by allowing us to search
            // for " inv " instead, avoiding matching words *containing* inv (such as
            // INVariant, subINVolution, and a few other pathological cases)
            let def = parameters.definition.clone() + " ";
            let inverted = def.contains(" inv ");
            let mut next_param = parameters.next(&parameters.definition);
            next_param.definition = macro_definition;
            return Op::op(next_param, provider)?.handle_inversion(inverted);
        }

        // A built in operator?
        if let Ok(constructor) = super::inner_op::builtin(&name) {
            return constructor.0(&parameters, provider)?.handle_op_inversion();
        }

        Err(super::Error::NotFound(
            name,
            ": ".to_string() + &parameters.definition,
        ))
    }

    fn handle_op_inversion(self) -> Result<Op, Error> {
        let inverted = self.params.boolean("inv");
        self.handle_inversion(inverted)
    }

    fn handle_inversion(mut self, inverted: bool) -> Result<Op, Error> {
        if self.descriptor.invertible {
            if inverted {
                self.descriptor.inverted = !self.descriptor.inverted;
            }
            return Ok(self);
        }
        if inverted {
            return Err(Error::NonInvertible(self.descriptor.definition));
        }

        Ok(self)
    }
}

// ----- A N C I L L A R Y   F U N C T I O N S -----------------------------------------

pub fn is_pipeline(definition: &str) -> bool {
    definition.contains('|')
}

pub fn is_resource_name(definition: &str) -> bool {
    operator_name(definition, "").contains(':')
}

pub fn operator_name(definition: &str, default: &str) -> String {
    if is_pipeline(definition) {
        return default.to_string();
    }
    split_into_parameters(definition)
        .get("name")
        .unwrap_or(&default.to_string())
        .to_string()
}

pub fn split_into_parameters(step: &str) -> BTreeMap<String, String> {
    // Conflate contiguous whitespace, then remove whitespace after {"=",  ":",  ","}
    let step = step.trim().to_string();
    let elements: Vec<_> = step.split_whitespace().collect();
    let step = elements
        .join(" ")
        .replace("= ", "=")
        .replace(": ", ":")
        .replace(", ", ",")
        .replace(" =", "=")
        .replace(" :", ":")
        .replace(" ,", ",");

    let mut params = BTreeMap::new();
    let elements: Vec<_> = step.split_whitespace().collect();
    for element in elements {
        // Split a key=value-pair into key and value parts
        let mut parts: Vec<&str> = element.trim().split('=').collect();
        // Add a boolean true part, to make sure we have a value, even for flags
        // (flags are booleans that are true when specified, false when not)
        parts.push("true");
        assert!(parts.len() > 1);

        // If the first arg is a key-without-value, it is the name of the operator
        if params.is_empty() && parts.len() == 2 {
            params.insert(String::from("name"), String::from(parts[0]));
            continue;
        }

        params.insert(String::from(parts[0]), String::from(parts[1]));
    }

    params
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
        let mut data = some_basic_coordinates();
        provider.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        provider.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Also for an inverted operator: check forward and inverse operation
        let op = provider.op("addone inv")?;
        // let op = Op::new("addone inv", &provider)?;
        let mut data = some_basic_coordinates();
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
        let mut data = some_basic_coordinates();
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

    #[test]
    fn macro_expansion() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut prv = Minimal::default();
        prv.register_resource("sub:one", "addone inv");
        let op = prv.op("addone|sub:one|addone")?;

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    #[test]
    fn macro_expansion_inverted() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut prv = Minimal::default();
        prv.register_resource("sub:one", "addone inv");
        let op = prv.op("addone|sub:one inv|addone")?;

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    #[test]
    fn macro_expansion_with_embedded_pipeline() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut prv = Minimal::default();
        prv.register_resource("sub:three", "addone inv|addone inv|addone inv");
        let op = prv.op("addone|sub:three")?;

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 53.);
        assert_eq!(data[1][0], 57.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        let op = prv.op("addone|sub:three inv")?;
        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 59.);
        assert_eq!(data[1][0], 63.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    #[test]
    fn macro_expansion_with_defaults_provided() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut prv = Minimal::default();

        // A macro providing a default value of 1 for the x parameter
        prv.register_resource("helmert:one", "helmert x=*1");

        // Instantiating the macro without parameters - getting the default
        let op = prv.op("helmert:one")?;

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Instantiating the macro with parameters - overwriting the default
        // For good measure, check that it also works inside a pipeline
        let op = prv.op("addone|helmert:one x=2")?;

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Overwrite the default, and provide additional args
        let op = prv.op("helmert:one x=2 inv")?;

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 53.);
        assert_eq!(data[1][0], 57.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Overwrite the default, and provide additional args
        let op = prv.op("addone|helmert:one inv x=2")?;

        prv.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 54.);
        assert_eq!(data[1][0], 58.);

        prv.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }
}
