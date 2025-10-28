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
    pub recursion_level: usize,
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

        // Not a pipeline? Then it is either a macro, a built-in or
        // a user defined op, and we can just carry on
        let definition = invocation.clone();
        RawParameters {
            invocation,
            definition,
            globals: globals.clone(),
            recursion_level: 0,
        }
    }
}

// ----- T E S T S ---------------------------------------------------------------------

// RawParameters gets its test coverage from the tests in `op/mod.rs`
