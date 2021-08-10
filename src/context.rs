use std::collections::HashMap;
use std::usize;

use crate::operator_construction::*;
use crate::CoordinateTuple;

/// The central administration of the transformation functionality
#[derive(Default)]
pub struct Context {
    pub stack: Vec<Vec<CoordinateTuple>>,
    minions: Vec<Context>,
    user_defined_operators: HashMap<String, OperatorConstructor>,
    user_defined_macros: HashMap<String, String>,
    operations: Vec<Operator>,
    last_failing_operation_definition: String,
    last_failing_operation: String,
    cause: String,
}

impl Context {
    /// Number of chunks to process in (principle in) parallel.
    const CHUNKS: usize = 3;

    /// Maximum size of each chunk.
    const CHUNK_SIZE: usize = 1000;

    pub fn new() -> Context {
        let mut ctx = Context::_new();
        for _ in 0..Self::CHUNKS {
            ctx.minions.push(Context::_new());
        }
        ctx
    }

    fn _new() -> Context {
        Context {
            stack: Vec::new(),
            minions: Vec::new(),
            last_failing_operation_definition: String::new(),
            last_failing_operation: String::new(),
            cause: String::new(),
            user_defined_operators: HashMap::new(),
            user_defined_macros: HashMap::new(),
            operations: Vec::new(),
        }
    }

    // Parallel execution helper for `operate`, below
    fn _operate(
        &mut self,
        operator: &Operator,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        operator.operate(self, operands, forward)
    }

    pub fn operate(
        &mut self,
        operation: usize,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool {
        if operation >= self.operations.len() {
            self.last_failing_operation = String::from("Invalid");
            self.cause = String::from("Attempt to access an invalid operator from context");
            return false;
        }
        let mut i = 0_usize;
        let mut result = true;
        for chunk in operands.chunks_mut(Self::CHUNK_SIZE) {
            // Need a bit more std::thread-Rust-fu to do actual mutithreading.
            // For now, we just split the input data in chunks, process them
            // and verify that the parallel stack-functionality works.
            result &= self.minions[i]._operate(&self.operations[operation], chunk, forward);
            self.minions[i].stack.clear();
            i = (i + 1) % Self::CHUNKS;
        }
        result
    }

    pub fn fwd(&mut self, operation: usize, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, true)
    }

    pub fn inv(&mut self, operation: usize, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }

    pub fn register_operator(&mut self, name: &str, constructor: OperatorConstructor) {
        self.user_defined_operators
            .insert(name.to_string(), constructor);
    }

    pub(crate) fn locate_operator(&mut self, name: &str) -> Option<&OperatorConstructor> {
        self.user_defined_operators.get(name)
    }

    #[must_use]
    pub fn register_macro(&mut self, name: &str, definition: &str) -> bool {
        // Registering a macro under the same name as its definition name
        // leads to infinite nesting - so we prohibit that
        let illegal_start = name.to_string() + ":";
        if definition.trim_start().starts_with(&illegal_start) {
            return false;
        }

        if self
            .user_defined_macros
            .insert(name.to_string(), definition.to_string())
            .is_some()
        {
            return false;
        }
        true
    }

    pub(crate) fn locate_macro(&mut self, name: &str) -> Option<&String> {
        self.user_defined_macros.get(name)
    }

    pub fn operation(&mut self, definition: &str) -> Option<usize> {
        self.last_failing_operation_definition = definition.to_string();
        self.last_failing_operation.clear();
        self.cause.clear();
        let definition = gys_to_yaml(definition);
        let op = Operator::new(&definition, self)?;
        let index = self.operations.len();
        self.operations.push(op);
        Some(index)
    }

    pub fn error(&mut self, which: &str, why: &str) {
        self.last_failing_operation = String::from(which);
        self.cause = String::from(why);
    }

    pub fn report(&mut self) -> String {
        format!(
            "Last failure in {}: {}\n{}",
            self.last_failing_operation, self.cause, self.last_failing_operation_definition
        )
    }
}

/// Convert "Ghastly YAML Shorthand" to YAML
fn gys_to_yaml(gys: &str) -> String {
    // Appears to be YAML already - do nothing!
    if !is_gys(gys) {
        return String::from(gys);
    }

    // Strip off superfluous GYS indicators
    let gys = gys.trim_matches('|');
    let gys = gys.trim_matches('[');
    let gys = gys.trim_matches(']');

    let mut yaml = String::new();
    let mut indent = "";
    let steps: Vec<&str> = gys.split('|').collect();
    let nsteps = steps.len();
    if nsteps > 1 {
        yaml += "pipeline_from_gys: {\n  steps: [\n";
        indent = "    ";
    }
    for step in steps {
        let mut elements: Vec<&str> = step.split_whitespace().collect();
        let n = elements.len();
        if n == 0 {
            return String::from("Error: Empty step!");
        }

        // First the operator name
        yaml += indent;
        yaml += elements[0];
        yaml += ":";

        // linebreaks after the first step
        indent = ",\n    ";
        // No args? Then insert an empty argument list
        if n == 1 {
            yaml += " {}";
            continue;
        }

        // Handle args
        yaml += " {";

        for i in 1..n {
            // We constructed a key-value par in last iteration?
            if elements[i].is_empty() {
                continue;
            }
            let e = elements[i].to_string();
            if e.ends_with(':') {
                if i == n - 1 {
                    return String::from("Missing value for key '") + &e + "'";
                }
                yaml += &e;
                yaml += " ";
                yaml += elements[i + 1];
                if i + 2 < n {
                    yaml += ", ";
                }
                elements[i + 1] = "";
                continue;
            };

            // Ultra compact notation: key:value, no whitespace
            if e.contains(':') {
                yaml += &e.replace(":", ": ");
                if i + 1 < n {
                    yaml += ", ";
                }
                continue;
            }

            // Key with no value? provide "true"
            yaml += &e;
            yaml += ": true";
            if i + 1 < n {
                yaml += ", ";
            }
        }
        yaml += "}";
    }

    if nsteps > 1 {
        yaml += "\n  ]\n}";
    }

    yaml
}

// True if a str appears to be in GYS format
fn is_gys(gys: &str) -> bool {
    // GYS if contains a whitespace-wrapped pipe
    if gys.contains(" | ") {
        return true;
    }

    // GYS if starting or ending with an empty step
    if gys.starts_with('|') {
        return true;
    }
    if gys.ends_with('|') {
        return true;
    }

    // GYS if wrapped in square brackets: [gys]. Note that
    // we cannot merge these two ifs without damaging the
    // following test for "no trailing colon"
    if gys.starts_with('[') {
        return gys.ends_with(']');
    }
    if gys.ends_with(']') {
        return gys.starts_with('[');
    }

    // GYS if no trailing colon on first token
    if !gys.split_whitespace().next().unwrap_or_default().ends_with(':') {
        return true
    }

    // Otherwise not a GYS - hopefully it's YAML then!
    false
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operand() {
        use crate::Context;
        let ctx = Context::new();
        assert_eq!(ctx.stack.len(), 0);
    }

    #[test]
    fn operate() {
        use crate::Context;
        use crate::CoordinateTuple;

        let pipeline = "ed50_etrs89: {
            steps: [
                cart: {ellps: intl},
                helmert: {x: -87, y: -96, z: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";

        let mut ctx = Context::new();
        let op = ctx.operation(pipeline);
        assert!(op.is_some());
        let op = op.unwrap();
        let geo = CoordinateTuple::gis(12., 55., 100., 0.);
        let mut operands = [geo];

        ctx.fwd(op, &mut operands);
        let result = operands[0].to_degrees();
        assert!((result[0] - 11.998815342385206861).abs() < 1e-10);
        assert!((result[1] - 54.999382648950991381).abs() < 1e-10);

        ctx.inv(op, &mut operands);
        let result = operands[0].to_degrees();
        assert!((result[0] - 12.).abs() < 1e-12);
        assert!((result[1] - 55.).abs() < 1e-12);
    }

    #[test]
    fn gys() {
        use crate::Context;
        use super::is_gys;
        use crate::CoordinateTuple as C;

        // A pipeline in YAML
        let pipeline = "ed50_etrs89: {
            steps: [
                cart: {ellps: intl},
                helmert: {x: -87, y: -96, z: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";

        // Same pipeline in Ghastly YAML Shorthand (GYS)
        let gys = "cart ellps: intl | helmert x:-87 y:-96 z:-120 | cart inv ellps:GRS80";

        let mut ctx = Context::new();

        // Check that the GYS syntactical indicators trigger
        assert!(is_gys("[cart]"));
        assert!(is_gys("|cart|"));
        assert!(is_gys("|cart"));
        assert!(is_gys("cart|"));
        assert!(!is_gys("[cart"));
        assert!(!is_gys("cart]"));

        // Check that GYS instantiates exactly as the corresponding YAML
        let op_yaml = ctx.operation(pipeline).unwrap();
        let op_gys = ctx.operation(gys).unwrap();

        let copenhagen = C::geo(55., 12., 0., 0.);
        let stockholm = C::geo(59., 18., 0., 0.);
        let mut yaml_data = [copenhagen, stockholm];
        let mut gys_data = [copenhagen, stockholm];

        ctx.fwd(op_yaml, &mut yaml_data);
        ctx.fwd(op_gys, &mut gys_data);

        C::geo_all(&mut yaml_data);
        C::geo_all(&mut gys_data);

        // We assert that the difference is exactly zero, since the operations
        // should be identical. But float equality comparisons are frowned at...
        assert!(yaml_data[0].hypot3(&gys_data[0]) < 1e-30);
        assert!(yaml_data[1].hypot3(&gys_data[1]) < 1e-30);
    }
}
