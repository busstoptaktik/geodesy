mod op_descriptor;
mod parameter;
mod parsed_parameters;
mod raw_parameters;

use crate::operator_authoring::*;
use std::collections::BTreeMap;

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
    pub fn apply(&self, ctx: &dyn Context, operands: &mut dyn CoordinateSet, direction: Direction) -> usize {
        let forward = direction == Direction::Fwd;
        // Short form of (inverted && !forward) || (forward && !inverted)
        if self.descriptor.inverted != forward {
            return self.descriptor.fwd.0(self, ctx, operands);
        }
        self.descriptor.inv.0(self, ctx, operands)
    }

    pub fn new(definition: &str, ctx: &dyn Context) -> Result<Op, Error> {
        let globals = ctx.globals();
        let parameters = RawParameters::new(definition, &globals);
        Self::op(parameters, ctx)
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
        _ctx: &dyn Context,
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
    pub fn op(parameters: RawParameters, ctx: &dyn Context) -> Result<Op, Error> {
        if parameters.nesting_too_deep() {
            return Err(Error::Recursion(
                parameters.invocation,
                parameters.definition,
            ));
        }

        let name = operator_name(&parameters.definition, "");

        // A pipeline?
        if is_pipeline(&parameters.definition) {
            return super::inner_op::pipeline::new(&parameters, ctx);
        }

        // A user defined operator?
        if !is_resource_name(&name) {
            if let Ok(constructor) = ctx.get_op(&name) {
                return constructor.0(&parameters, ctx)?.handle_op_inversion();
            }
        }
        // A user defined macro?
        else if let Ok(macro_definition) = ctx.get_resource(&name) {
            // search for whitespace-delimited "inv" in order to avoid matching
            // tokens *containing* inv (INVariant, subINVolution, and a few other
            // pathological cases)
            let def = &parameters.definition;
            let inverted = def.contains(" inv ") || def.ends_with(" inv");
            let mut next_param = parameters.next(def);
            next_param.definition = macro_definition;
            return Op::op(next_param, ctx)?.handle_inversion(inverted);
        }

        // A built in operator?
        if let Ok(constructor) = super::inner_op::builtin(&name) {
            return constructor.0(&parameters, ctx)?.handle_op_inversion();
        }

        Err(Error::NotFound(
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
        let mut ctx = Minimal::default();

        // Try to invoke garbage as a user defined Op
        assert!(matches!(Op::new("_foo", &ctx), Err(Error::NotFound(_, _))));

        // Check forward and inverse operation
        let op = ctx.op("addone")?;
        let mut data = some_basic_coordinates();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Also for an inverted operator: check forward and inverse operation
        let op = ctx.op("addone inv ")?;
        let mut data = some_basic_coordinates();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 54.);
        assert_eq!(data[1][0], 58.);
        // Corner case: " inv " vs. "inv"
        let op = ctx.op("addone inv")?;
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    // Test that the recursion-breaker works properly, by defining two mutually
    // dependent macros: `foo:bar=foo:baz` and `foo:baz=foo:bar`, and checking
    // that the instantiation fails with an `Error::Recursion(...)`
    #[test]
    fn nesting() -> Result<(), Error> {
        let mut ctx = Minimal::default();
        ctx.register_resource("foo:baz", "foo:bar");
        ctx.register_resource("foo:bar", "foo:baz");

        assert_eq!("foo:baz", ctx.get_resource("foo:bar")?);
        assert_eq!("foo:bar", ctx.get_resource("foo:baz")?);

        assert!(matches!(ctx.op("foo:baz"), Err(Error::Recursion(_, _))));
        Ok(())
    }

    #[test]
    fn pipeline() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut ctx = Minimal::default();
        let op = ctx.op("addone|addone|addone")?;

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    #[test]
    fn macro_expansion() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut ctx = Minimal::default();
        ctx.register_resource("sub:one", "addone inv");
        let op = ctx.op("addone|sub:one|addone")?;

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    #[test]
    fn macro_expansion_inverted() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut ctx = Minimal::default();
        ctx.register_resource("sub:one", "addone inv");
        let op = ctx.op("addone|sub:one inv|addone")?;

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    #[test]
    fn macro_expansion_with_embedded_pipeline() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut ctx = Minimal::default();
        ctx.register_resource("sub:three", "addone inv|addone inv|addone inv");
        let op = ctx.op("addone|sub:three")?;

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 53.);
        assert_eq!(data[1][0], 57.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        let op = ctx.op("addone|sub:three inv")?;
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 59.);
        assert_eq!(data[1][0], 63.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }

    #[test]
    fn macro_expansion_with_defaults_provided() -> Result<(), Error> {
        let mut data = some_basic_coordinates();
        let mut ctx = Minimal::default();

        // A macro providing a default value of 1 for the x parameter
        ctx.register_resource("helmert:one", "helmert x=*1");

        // Instantiating the macro without parameters - getting the default
        let op = ctx.op("helmert:one")?;

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Instantiating the macro with parameters - overwriting the default
        // For good measure, check that it also works inside a pipeline
        let op = ctx.op("addone|helmert:one x=2")?;

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 58.);
        assert_eq!(data[1][0], 62.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Overwrite the default, and provide additional args
        let op = ctx.op("helmert:one x=2 inv")?;

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 53.);
        assert_eq!(data[1][0], 57.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Overwrite the default, and provide additional args
        let op = ctx.op("addone|helmert:one inv x=2")?;

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 54.);
        assert_eq!(data[1][0], 58.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }
}
