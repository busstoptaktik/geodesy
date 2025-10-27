use super::*;

/// Interface between the high level [Op::op()](crate::op::Op) and the low level
/// functionality in the [InnerOp](crate::inner_op::InnerOp)s
///
/// `RawParameters` is the vehicle used by the `Op`erator factory in `Op::op(...)`,
/// to ferry args around from the invocator into the constructor of the individual
/// `InnerOp`s.
///
/// The `InnerOp`constructor typically interprets the contents of
/// `RawParameters`, and converts it into a more runtime friendly instance of
/// `ParsedParameters`.
#[derive(Debug, Default, Clone)]
pub struct RawParameters {
    pub invocation: String,
    pub definition: String,
    pub globals: BTreeMap<String, String>,
    recursion_level: usize,
}

impl RawParameters {
    pub fn new(invocation: &str, globals: &BTreeMap<String, String>) -> RawParameters {
        // This, and RawParameters::expand() should be the only places, where the syntax
        // cleanup functions from the Tokenize trait, is needed
        let invocation = invocation.remove_comments().normalize();

        let globals = globals.clone();
        let recursion_level = 0;

        // The intricacies of pipeline instantiation is handled directly by Op::op(),
        // which calls pipeline's constructor to do the hard work. So from here, we just
        // hand over the invocation and globals for them to use
        if invocation.is_pipeline() {
            let definition = invocation.clone();
            return RawParameters {
                invocation,
                definition,
                globals,
                recursion_level,
            };
        }

        // The FIRST step of a pipeline may start with one of the modifiers
        // (inv, omit_inv, omit_fwd), but we cannot rotate that to the end
        // of the invocation, because that would be moving it to the end of
        // the LAST step of the pipeline

        // But if we're here, we're not handling a pipeline, so we can safely rotate
        // modifiers to the end.
        let invocation = invocation.handle_prefix_modifiers();

        // If it is a macro invocation, the `recurse()` method is called
        // to do the parameter handling
        if invocation.is_resource_name() {
            let resource_definition = invocation.clone();
            return RawParameters {
                invocation,
                definition: "".to_string(),
                globals: globals.clone(),
                recursion_level: 0,
            }
            .recurse(&resource_definition);
        }

        // Not a macro, and not a pipeline? Then it is either a built-in or
        // a user defined op, and we can just carry on
        let definition = invocation.clone();
        RawParameters {
            invocation,
            definition,
            globals: globals.clone(),
            recursion_level: 0,
        }
    }

    // If the step is a macro (i.e. potentially an embedded pipeline),
    // we take a copy of the arguments from the macro invocation and enter
    // them into the globals.
    // Otherwise, we just copy the globals from the previous step, and
    // update the recursion counter.
    pub fn recurse(&self, definition: &str) -> RawParameters {
        let mut globals = self.globals.clone();

        globals.remove("_name");
        globals.extend(definition.split_into_parameters());

        // In the macro case, inversion is handled one level up by the `Op::op`
        // constructor through the `handle_inversion()` method: We do not yet
        // know whether the macro is really a pipeline, so at this level, we remove
        // the `inv` from the globals, to avoid poisoning the pipeline at the single
        // step level
        globals.remove("inv");

        let invocation = self.invocation.clone();
        let definition = definition.trim().to_string();
        RawParameters {
            invocation,
            definition,
            globals,
            recursion_level: self.recursion_level + 1,
        }
    }

    pub fn nesting_too_deep(&self) -> bool {
        self.recursion_level > 100
    }
}

// ----- T E S T S ---------------------------------------------------------------------

// RawParameters gets its test coverage from the tests in `op/mod.rs`
