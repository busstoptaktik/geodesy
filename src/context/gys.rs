use crate::Context;

impl Context {
    /// Convert "Geodetic YAML Shorthand" to YAML
    pub fn gys_to_yaml(gys: &str) -> String {
        let lines = gys.lines();
        let mut s = Vec::new();
        for line in lines {
            if line.trim().starts_with('#') {
                continue;
            }
            s.push(line);
        }
        let gys = s.join("\n").trim().to_string();

        // Appears to be YAML already - do nothing!
        if !Context::is_gys(&gys) {
            return gys;
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
            // Strip inline comments
            let strip = step
                .find('#')
                .map(|index| &step[..index])
                .unwrap_or(step)
                .trim()
                .to_string();
            let mut elements: Vec<&str> = strip.split_whitespace().collect();
            let n = elements.len();
            if n == 0 {
                return String::from("Error: Empty step!");
            }

            // changing indent after use to get linebreaks after the first step
            yaml += indent;
            indent = ",\n    ";

            yaml += elements[0];
            yaml += ":";

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
    pub fn is_gys(gys: &str) -> bool {
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
        if !gys
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .ends_with(':')
        {
            return true;
        }

        // Otherwise not a GYS - hopefully it's YAML then!
        false
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn gys() {
        use crate::Context;
        use crate::CoordinateTuple as C;

        let mut ctx = Context::new();

        // Test the corner case of giving just "inv" as operation name
        let inv = ctx.operation("[inv]");
        assert!(inv.is_err());

        // Test that an inv-operator actually instantiates
        let invcart = ctx.operation("[cart inv]");
        assert!(invcart.is_ok());

        // Check that the GYS syntactical indicators trigger
        assert!(Context::is_gys("[cart]"));
        assert!(Context::is_gys("|cart|"));
        assert!(Context::is_gys("|cart"));
        assert!(Context::is_gys("cart|"));
        assert!(!Context::is_gys("[cart"));
        assert!(!Context::is_gys("cart]"));

        // Now a more complete test of YAML vs. GYS

        // A pipeline in YAML
        let pipeline = "ed50_etrs89: {
            # with cucumbers
            steps: [
                cart: {ellps: intl},
                helmert: {x: -87, y: -96, z: -120},
                cart: {inv: true, ellps: GRS80}
            ]
        }";

        // Same pipeline in Geodetic YAML Shorthand (GYS), with some nasty
        // inline comments to stress test gys_to_yaml().
        let gys = "# bla bla\n\n   cart ellps: intl # another comment ending at newline\n | helmert x:-87 y:-96 z:-120 # inline comment ending at step, not at newline | cart inv ellps:GRS80";

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
