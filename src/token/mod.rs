use crate::Error;
use std::collections::BTreeMap;

/// Convenience methods for lexical analysis of operator definitions.
/// - For splitting a pipeline into steps
/// - For splitting a step into parameters (i.e. key=value-pairs)
/// - For syntactical normalization by eliminating non-significant whitespace
/// - For checking whether a given operator is singular or a pipeline
/// - For checking whether a key is a macro name ("resource name"), and
/// - For accessing the name of a given operator.
pub trait Tokenize {
    /// Split a pipeline definition into steps and a potentially empty docstring
    fn split_into_steps(&self) -> (Vec<String>, String);

    /// Split a step/an operation into parameters. Give special treatment
    /// to names and flags:
    /// ```txt
    /// 'foo bar=baz bonk=blue flag' -> ('name=foo', 'bar=baz', 'bonk=blue', 'flag=true')
    /// ```
    fn split_into_parameters(&self) -> BTreeMap<String, String>;

    /// Helper function for 'split_into_steps' and 'split_into_parameters':
    /// Glue syntactical elements together, and separate from each other
    /// by a single space:
    ///
    /// 1. Glue key-value pairs together by omitting whitespace around '=':
    ///    ```txt
    ///    key1= value1            key2    =value2  ->  key1=value1 key2=value2
    ///    ```
    /// 2. Trim whitespace on both sides of the macro sigil ':' and leave only
    ///    one space to the left of the dereference sigil '$':
    ///    ```txt
    ///    foo: bar $ baz -> foo:bar $baz
    ///    ```
    /// 3. Trim whitespace around sequence separators ',' and '|':
    ///    ```txt
    ///     foo | bar baz=bonk   ,    bonk  ->  foo|bar baz=bonk,bonk
    ///    ```
    fn normalize(&self) -> String;

    fn is_pipeline(&self) -> bool;
    fn is_resource_name(&self) -> bool;
    fn operator_name(&self, default: &str) -> String;
}

/// Tokenize implementation for string-like objects
impl<T> Tokenize for T
where
    T: AsRef<str>,
{
    fn split_into_steps(&self) -> (Vec<String>, String) {
        // Impose some line ending sanity
        let all = self
            .as_ref()
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
        let steps: Vec<String> = trimmed
            .normalize()
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

    fn split_into_parameters(&self) -> BTreeMap<String, String> {
        // Remove non-significant whitespace
        let step = self.normalize();
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

    fn normalize(&self) -> String {
        let elements: Vec<_> = self.as_ref().split_whitespace().collect();
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

    fn is_pipeline(&self) -> bool {
        self.as_ref().contains('|')
    }

    fn is_resource_name(&self) -> bool {
        self.operator_name("").contains(':')
    }

    fn operator_name(&self, default: &str) -> String {
        if self.is_pipeline() {
            return default.to_string();
        }
        self.split_into_parameters()
            .get("name")
            .unwrap_or(&default.to_string())
            .to_string()
    }
}

/// Translate a PROJ string into Rust Geodesy format. Since PROJ is syntactically
/// unrestrictive, we do not try to detect any syntax errors: If the input
/// is so cursed as to be intranslatable, this will become clear when trying to
/// instantiate the result as a Geodesy operator. We do, however, check for and
/// report on two *semantically* refusable cases: First, that PROJ does not support
/// nested pipelines (the nesting must be done indirectly through an init-file),
/// second that Rust Geodesy does not support init-files. Hence no support for
/// any kind of nesting here.
pub fn parse_proj(definition: &str) -> Result<String, Error> {
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
    trimmed = " ".to_string() + &trimmed.normalize() + " ";

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

    for (step_index, step) in steps.iter().enumerate() {
        let mut elements: Vec<_> = step.split_whitespace().map(|x| x.to_string()).collect();

        // Move the "proj=..." element to the front of the collection, stripped for "proj="
        // and handle the pipeline globals, if any
        for (i, element) in elements.iter().enumerate() {
            // Mutating the Vec we are iterating over may seem dangerous but is
            // OK as we break out of the loop immediately after the mutation
            if element.starts_with("init=") {
                return Err(Error::Unsupported(
                    "parse_proj does not support PROJ init clauses: ".to_string() + step,
                ));
            }

            if element.starts_with("proj=") {
                elements.swap(i, 0);
                elements[0] = elements[0][5..].to_string();

                // In the proj=pipeline case, just collect the globals, without
                // introducing a new step into geodesy_steps
                if elements[0] == "pipeline" {
                    if step_index != 0 {
                        return Err(Error::Unsupported(
                            "PROJ does not support nested pipelines: ".to_string() + &trimmed,
                        ));
                    }
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
    Ok(geodesy_steps.join(" | ").trim().to_string())
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    // Test the fundamental tokenization functionality
    #[test]
    fn token() -> Result<(), Error> {
        assert_eq!("foo bar $ baz = bonk".normalize(), "foo bar $baz=bonk");
        assert_eq!(
            "foo |  bar baz  =  bonk, bonk , bonk".normalize(),
            "foo|bar baz=bonk,bonk,bonk"
        );
        assert_eq!(
            "foo |  bar baz  =  bonk, bonk , bonk".split_into_steps().0[0],
            "foo"
        );
        assert_eq!("foo bar baz=bonk".split_into_parameters()["name"], "foo");
        assert_eq!("foo bar baz=bonk".split_into_parameters()["bar"], "true");
        assert_eq!("foo bar baz=bonk".split_into_parameters()["baz"], "bonk");
        assert!("foo | bar".is_pipeline());
        assert!("foo:bar".is_resource_name());
        assert_eq!("foo bar baz=bonk".operator_name(""), "foo");
        assert_eq!("foo bar baz=  $bonk".operator_name(""), "foo");
        Ok(())
    }

    // The PROJ language provides ample opportunity to explore pathological cases
    #[test]
    fn proj() -> Result<(), Error> {
        // Some trivial, but strangely formatted cases
        assert_eq!(
            parse_proj("+a   =   1 +proj =foo    b= 2  ")?,
            "foo a=1 b=2"
        );
        assert_eq!(
            parse_proj("+a   =   1 +proj =foo    +   b= 2  ")?,
            "foo a=1 b=2"
        );

        // An invalid PROJ string, that parses into an empty pipeline
        assert_eq!(parse_proj("      proj=")?, "");

        // A pipeline with a single step and a global argument
        assert_eq!(
            parse_proj("proj=pipeline +foo=bar +step proj=utm zone=32")?,
            "utm foo=bar zone=32"
        );

        // A pipeline with 3 steps and 2 global arguments
        assert_eq!(
            parse_proj("proj=pipeline +foo = bar ellps=GRS80 step proj=cart step proj=helmert s=3 step proj=cart ellps=intl")?,
            "cart foo=bar ellps=GRS80 | helmert foo=bar ellps=GRS80 s=3 | cart foo=bar ellps=GRS80 ellps=intl"
        );

        // Although PROJ would choke on this, we accept steps without an initial proj=pipeline
        assert_eq!(
            parse_proj("proj=utm zone=32 step proj=utm inv zone=32")?,
            "utm zone=32 | utm inv zone=32"
        );

        // Check for accidental matching of 'step' - even for a hypothetical 'proj=step arg...'
        // and for args called 'step' (which, however, cannot be flags - must come with a value
        // to be recognized as a key=value pair)
        assert_eq!(
            parse_proj("  +step proj = step step=quickstep step step proj=utm inv zone=32 step proj=stepwise step proj=quickstep")?,
            "step step=quickstep | utm inv zone=32 | stepwise | quickstep"
        );

        // Invert the entire pipeline, turning "zone 32-to-zone 33" into "zone 33-to-zone 32"
        // Also throw a few additional spanners in the works, in the form of some ugly, but
        // PROJ-accepted, syntactical abominations
        assert_eq!(
            parse_proj("inv ellps=intl proj=pipeline ugly=syntax +step inv proj=utm zone=32 step proj=utm zone=33")?,
            "utm inv ellps=intl ugly=syntax zone=33 | utm ellps=intl ugly=syntax zone=32"
        );

        // Check for the proper inversion of directional omissions
        assert_eq!(
            parse_proj("proj=pipeline inv   +step   omit_fwd inv proj=utm zone=32   step   omit_inv proj=utm zone=33")?,
            "utm inv omit_fwd zone=33 | utm omit_inv zone=32"
        );

        // Nested pipelines are not supported...

        // Nested pipelines in PROJ requires an `init=` indirection
        assert!(matches!(
            parse_proj("proj=pipeline step proj=pipeline"),
            Err(Error::Unsupported(_))
        ));
        // ...but `init` is not supported by Rust Geodesy, since that
        // would require a full implementation of PROJ's resolution
        // system - which would be counter to RG's raison d'etre
        assert!(matches!(
            parse_proj("pipeline step init=another_pipeline step proj=noop"),
            Err(Error::Unsupported(_))
        ));

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
