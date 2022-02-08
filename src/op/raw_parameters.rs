use super::*;

#[derive(Debug, Default)]
pub struct RawParameters {
    pub invocation: String,
    pub definition: String,
    pub globals: BTreeMap<String, String>,
    recursion_level: usize,
}

impl RawParameters {
    pub fn new(invocation: &str, globals: &BTreeMap<String, String>) -> RawParameters {
        let recursion_level = 0;
        let globals = globals.clone();
        let invocation = invocation.to_string();
        let definition = invocation.clone();

        // Direct invocation of a pipeline: "foo | bar baz | bonk"
        if super::is_pipeline(&invocation) {
            return RawParameters {
                invocation,
                definition,
                globals,
                recursion_level,
            };
        }

        // Direct invocation of a primitive operation: "foo bar=baz bonk"
        if !super::is_resource_name(&invocation) {
            return RawParameters {
                invocation,
                definition,
                globals,
                recursion_level,
            };
        }

        // Macro expansion initialization
        // The tough parameter handling is carried out by
        // the `next()` method in the upcomming step.
        let definition = "".to_string();
        let previous = RawParameters {
            invocation,
            definition,
            globals,
            recursion_level,
        };
        previous.next(&previous.invocation)
    }

    // If the next step is a macro (i.e. potentially an embedded pipeline), we
    // get the arguments from the invocation and bring them into the globals.
    // Otherwise, we just copy the globals from the previous step, and
    // update the recursion counter.
    pub fn next(&self, definition: &str) -> RawParameters {
        let mut recursion_level = self.recursion_level + 1;
        let mut globals = self.globals.clone();
        if super::is_resource_name(definition) {
            globals.remove("name");
            globals.remove("inv");
            globals.extend(super::split_into_parameters(definition).into_iter());
            recursion_level += 1;
        }
        let invocation = self.invocation.clone();
        let definition = definition.to_string();
        RawParameters {
            invocation,
            definition,
            globals,
            recursion_level,
        }
    }

    pub fn nesting_too_deep(&self) -> bool {
        self.recursion_level > 100
    }
}

// ----- T E S T S ---------------------------------------------------------------------

// RawParameters gets its test coverage from the tests in `op.rs`
