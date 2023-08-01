mod op_descriptor;
mod parameter;
mod parsed_parameters;
mod raw_parameters;

use crate::authoring::*;
use std::collections::BTreeMap;

pub use op_descriptor::OpDescriptor;
pub use parameter::OpParameter;
pub use parsed_parameters::ParsedParameters;
pub use raw_parameters::RawParameters;

/// The key, returned to the user, representing the actual operation handled by the `Context`
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
        ctx: &dyn Context,
        operands: &mut dyn CoordinateSet,
        direction: Direction,
    ) -> usize {
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
        inv: Option<InnerOp>,
        gamut: &[OpParameter],
        _ctx: &dyn Context,
    ) -> Result<Op, Error> {
        let def = parameters.definition.as_str();
        let mut params = ParsedParameters::new(parameters, gamut)?;

        // Convert lat_{0..4} and lon_{0..4} to radians
        for i in ["lat_0", "lat_1", "lat_2", "lat_3"] {
            let lat = *params.real.get(i).unwrap_or(&0.);
            params.real.insert(i, lat);
        }

        for i in ["lon_0", "lon_1", "lon_2", "lon_3"] {
            let lon = *params.real.get(i).unwrap_or(&0.);
            params.real.insert(i, lon);
        }

        let descriptor = OpDescriptor::new(def, fwd, inv);
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

// Helper function for 'split_into_steps' and 'split_into_parameters':
// Glue syntactic elements together: Elements separated by a single space,
// key-value pairs glued by '=':
//     key1= value1            key2    =value2  ->  key1=value1 key2=value2
// sigils '$' and ':' handled meaningfully:
//     foo: bar $ baz -> foo:bar $baz
// and sequence separators ',' trimmed:
//     foo = bar, baz  ->  foo=bar,baz
fn glue(definition: &str) -> String {
    let elements: Vec<_> = definition.split_whitespace().collect();
    elements
        .join(" ")
        .replace("= ", "=")
        .replace(": ", ":")
        .replace(", ", ",")
        .replace("| ", "|")
        .replace(" =", "=")
        .replace(" :", ":")
        .replace(" ,", ",")
        .replace(" |", "|")
        .replace("$ ", "$") // But keep " $" as is!
}

/// Split a pipeline definition into steps and a potentially empty docstring
pub fn split_into_steps(definition: &str) -> (Vec<String>, String) {
    // Impose some line ending sanity
    let all = definition
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_string();

    // Collect docstrings and remove plain comments
    let mut trimmed = String::new();
    let mut docstring = Vec::<String>::new();
    for line in all.lines() {
        let line = line.trim();

        // Collect docstrings
        if line.starts_with("##") {
            docstring.push((line.to_string() + "    ")[3..].trim_end().to_string());
            continue;
        }

        // Remove comments - both inline and separate lines
        let line: Vec<&str> = line.trim().split('#').collect();
        // Full line comment - just skip
        if line[0].starts_with('#') {
            continue;
        }

        // Inline comment, or no comment at all: Collect everything before `#`
        trimmed += " ";
        trimmed += line[0].trim();
    }

    // Finalize the docstring
    let docstring = docstring.join("\n").trim().to_string();

    // Remove empty steps and other non-significant whitespace
    let steps: Vec<String> = glue(&trimmed)
        // split into steps
        .split('|')
        // remove empty steps
        .filter(|x| !x.is_empty())
        // convert &str to String
        .map(|x| x.to_string())
        // and turn into Vec<String>
        .collect();

    (steps, docstring)
}

pub fn split_into_parameters(step: &str) -> BTreeMap<String, String> {
    // Remove non-significant whitespace
    let step = glue(step);
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

/// Translate a PROJ string into Rust Geodesy format. Since the PROJ syntax is
/// very unrestrictive, we do not try to detect any syntax errors: If the input
/// is so cursed as to be intranslatable, this will become clear when trying to
/// instantiate the result as a Geodesy operator.
pub fn parse_proj(definition: &str) -> String {
    // Impose some line ending sanity and remove the PROJ '+' prefix
    let all = definition
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace(" +", " ")
        .replace("\n+", " ")
        .trim()
        .trim_start_matches('+')
        .to_string();

    // Collect the PROJ string
    let mut trimmed = String::new();
    for line in all.lines() {
        let line = line.trim();

        // Remove block comments
        let line: Vec<&str> = line.trim().split('#').collect();
        // Full line (block) comment - just skip
        if line[0].starts_with('#') {
            continue;
        }

        // Inline comment, or no comment at all: Collect everything before `#`
        trimmed += " ";
        trimmed += line[0].trim();
    }

    // Now split the text into steps. First make sure we do not match
    //"step" as part of a word (stairSTEPping,  poSTEPileptic, STEPwise,
    // quickSTEP), by making it possible to only search for " step "
    trimmed = " ".to_string() + &glue(&trimmed) + " ";

    // Remove empty steps and other non-significant whitespace
    let steps: Vec<String> = trimmed
        // split into steps
        .split(" step ")
        // remove empty steps
        .filter(|x| !x.trim().trim_start_matches("step ").is_empty())
        // remove spurious 'step step' noise and convert &str to String
        .map(|x| x.trim().trim_start_matches("step ").to_string())
        // turn into Vec<String>
        .collect();

    // For accumulating the pipeline steps converted to geodesy syntax
    let mut geodesy_steps = Vec::new();

    // Geodesy does not suppport pipeline globals, so we must explicitly
    // insert them in the beginning of the argument list of each step
    let mut pipeline_globals = "".to_string();
    let mut pipeline_is_inverted = false;

    for step in steps {
        let mut elements: Vec<_> = step.split_whitespace().map(|x| x.to_string()).collect();

        // Move the "proj=..." element to the front of the collection, stripped for "proj="
        // and handle the pipeline globals, if any
        for (i, element) in elements.iter().enumerate() {
            // Mutating the Vec we are iterating over may seem dangerous but is
            // OK as we break out of the loop immediately after the mutation
            if element.starts_with("proj=") {
                elements.swap(i, 0);
                elements[0] = elements[0][5..].to_string();

                // In the proj=pipeline case, just collect the globals, without
                // introducing a new step into geodesy_steps
                if elements[0] == "pipeline" {
                    elements.remove(0);

                    // The case of 'inv' in globals must be handled separately, since it indicates
                    // the inversion of the entire pipeline, not just an inversion of each step
                    if elements.contains(&"inv".to_string()) {
                        pipeline_is_inverted = true;
                    }

                    // Remove all cases of 'inv' from the global arguments
                    let pipeline_globals_elements: Vec<String> = elements
                        .join(" ")
                        .trim()
                        .to_string()
                        .split_whitespace()
                        .filter(|x| x.trim() != "inv")
                        .map(|x| x.trim().to_string())
                        .collect();
                    pipeline_globals = pipeline_globals_elements.join(" ").trim().to_string();
                    elements.clear();
                }
                break;
            }
        }

        // Skip empty steps, insert pipeline globals, handle step and pipeline
        // inversions, and handle directional omissions (omit_fwd, omit_inv)
        let mut geodesy_step = elements.join(" ").trim().to_string();
        if !geodesy_step.is_empty() {
            if !pipeline_globals.is_empty() {
                elements.insert(1, pipeline_globals.clone());
            }

            let step_is_inverted = elements.contains(&"inv".to_string());
            elements = elements
                .iter()
                .filter(|x| x.as_str() != "inv")
                .map(|x| match x.as_str() {
                    "omit_fwd" => "omit_inv",
                    "omit_inv" => "omit_fwd",
                    _ => x,
                })
                .map(|x| x.to_string())
                .collect();

            if step_is_inverted != pipeline_is_inverted {
                elements.insert(1, "inv".to_string());
            }

            geodesy_step = elements.join(" ").trim().to_string();
            if pipeline_is_inverted {
                geodesy_steps.insert(0, geodesy_step);
            } else {
                geodesy_steps.push(geodesy_step);
            }
        }
    }
    geodesy_steps.join(" | ").trim().to_string()
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
        let mut data = some_basic_coor2dinates();
        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);
        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Also for an inverted operator: check forward and inverse operation
        let op = ctx.op("addone inv ")?;
        let mut data = some_basic_coor2dinates();
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
        let mut data = some_basic_coor2dinates();
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
        let mut data = some_basic_coor2dinates();
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
        let mut data = some_basic_coor2dinates();
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
        let mut data = some_basic_coor2dinates();
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
        let mut data = some_basic_coor2dinates();
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

    #[test]
    fn steps() -> Result<(), Error> {
        let (steps, _) = split_into_steps("  |\n#\n | |foo bar = baz |   bonk : bonk  $ bonk ||| ");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0], "foo bar=baz");
        assert_eq!(steps[1], "bonk:bonk $bonk");
        let (steps, _) = split_into_steps("\n\r\r\n    ||| | \n\n\r\n\r  |  \n\r\r \n  ");
        assert_eq!(steps.len(), 0);

        Ok(())
    }

    #[test]
    fn proj() -> Result<(), Error> {
        // Some trivial, but strangely formatted cases
        assert_eq!(parse_proj("a   =   1 +proj =foo    b= 2  "), "foo a=1 b=2");
        assert_eq!(
            parse_proj("+a   =   1 +proj =foo    +   b= 2  "),
            "foo a=1 b=2"
        );

        // An invalid PROJ string, that parses into an empty pipeline
        assert_eq!(parse_proj("      proj="), "");

        // A pipeline with a single step and a global argument
        assert_eq!(
            parse_proj("proj=pipeline +foo=bar +step proj=utm zone=32"),
            "utm foo=bar zone=32"
        );

        // A pipeline with 3 steps and 2 global arguments
        assert_eq!(
            parse_proj("proj=pipeline +foo = bar ellps=GRS80 step proj=cart step proj=helmert s=3 step proj=cart ellps=intl"),
            "cart foo=bar ellps=GRS80 | helmert foo=bar ellps=GRS80 s=3 | cart foo=bar ellps=GRS80 ellps=intl"
        );

        // Although PROJ would choke on this, we accept steps without an initial proj=pipeline
        assert_eq!(
            parse_proj("proj=utm zone=32 step proj=utm inv zone=32"),
            "utm zone=32 | utm inv zone=32"
        );

        // Check for accidental matching of 'step' - even for a hypothetical 'proj=step arg...'
        // and for args called 'step' (which, however, cannot be flags - must come with a value
        // to be recognized as a key=value pair)
        assert_eq!(
            parse_proj("  +step proj = step step=quickstep step step proj=utm inv zone=32 step proj=stepwise step proj=quickstep"),
            "step step=quickstep | utm inv zone=32 | stepwise | quickstep"
        );

        // Invert the entire pipeline, turning "zone 32-to-zone 33" into "zone 33-to-zone 32"
        // Also throw a few additional spanners in the works, in the form of some ugly, but
        // PROJ-accepted, syntactical abominations
        assert_eq!(
            parse_proj("inv ellps=intl proj=pipeline ugly=syntax +step inv proj=utm zone=32 step proj=utm zone=33"),
            "utm inv ellps=intl ugly=syntax zone=33 | utm ellps=intl ugly=syntax zone=32"
        );

        // Check for the proper inversion of directional omissions
        assert_eq!(
            parse_proj("proj=pipeline inv   +step   omit_fwd inv proj=utm zone=32   step   omit_inv proj=utm zone=33"),
            "utm inv omit_fwd zone=33 | utm omit_inv zone=32"
        );

        // Room here for testing of additional pathological cases...

        // Now check the sanity of the 'pipeline globals' handling
        let mut ctx = Minimal::default();

        // Check that we get the correct argument value when inserting pipeline globals
        // *at the top of the argument list*. Here: x=1 masquerades as the global value,
        // while x=2 is the step local one, which overwrites the global
        let op = ctx.op("helmert x=1 x=2")?;
        let mut operands = some_basic_coor2dinates();
        assert_eq!(2, ctx.apply(op, Fwd, &mut operands)?);
        assert_eq!(operands[0][0], 57.0);
        assert_eq!(operands[1][0], 61.0);

        Ok(())
    }
}
