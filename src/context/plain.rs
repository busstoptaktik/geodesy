#[cfg(feature = "with_plain")]
use crate::authoring::*;
use std::path::PathBuf;

// ----- T H E   P L A I N   C O N T E X T ---------------------------------------------

/// A context provider, supporting built in and run-time defined operators,
/// external grids, and macros.
/// Sufficient for most uses, especially geodetic grid development.
/// May get somewhat clunky when working with large numbers of grids or macros,
/// as each reside in its own file.
#[derive(Debug)]
pub struct Plain {
    constructors: BTreeMap<String, OpConstructor>,
    resources: BTreeMap<String, String>,
    operators: BTreeMap<OpHandle, Op>,
    paths: Vec<std::path::PathBuf>,
}

const BAD_ID_MESSAGE: Error = Error::General("Plain: Unknown operator id");

impl Default for Plain {
    fn default() -> Plain {
        let constructors = BTreeMap::new();
        let resources = BTreeMap::new();
        let operators = BTreeMap::new();
        let mut paths = Vec::new();

        let localpath: PathBuf = [".", "geodesy"].iter().collect();
        paths.push(localpath);

        if let Some(mut userpath) = dirs::data_local_dir() {
            userpath.push("geodesy");
            paths.push(userpath);
        }

        Plain {
            constructors,
            resources,
            operators,
            paths,
        }
    }
}

impl Context for Plain {
    fn new() -> Plain {
        let mut ctx = Plain::default();
        for item in BUILTIN_ADAPTORS {
            ctx.register_resource(item.0, item.1);
        }
        ctx
    }

    fn op(&mut self, definition: &str) -> Result<OpHandle, Error> {
        let op = Op::new(definition, self)?;
        let id = op.id;
        self.operators.insert(id, op);
        assert!(self.operators.contains_key(&id));
        Ok(id)
    }

    fn apply(
        &self,
        op: OpHandle,
        direction: Direction,
        operands: &mut dyn CoordinateSet,
    ) -> Result<usize, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        Ok(op.apply(self, operands, direction))
    }

    fn steps(&self, op: OpHandle) -> Result<&Vec<String>, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        Ok(&op.descriptor.steps)
    }

    fn params(&self, op: OpHandle, index: usize) -> Result<ParsedParameters, Error> {
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        // Leaf level?
        if op.steps.is_empty() {
            if index > 0 {
                return Err(Error::General("Plain: Bad step index"));
            }
            return Ok(op.params.clone());
        }

        // Not leaf level
        if index >= op.steps.len() {
            return Err(Error::General("Plain: Bad step index"));
        }
        Ok(op.steps[index].params.clone())
    }

    fn globals(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("ellps".to_string(), "GRS80".to_string())])
    }

    fn register_op(&mut self, name: &str, constructor: OpConstructor) {
        self.constructors.insert(String::from(name), constructor);
    }

    fn get_op(&self, name: &str) -> Result<OpConstructor, Error> {
        if let Some(result) = self.constructors.get(name) {
            return Ok(OpConstructor(result.0));
        }

        Err(Error::NotFound(
            name.to_string(),
            ": User defined constructor".to_string(),
        ))
    }

    fn register_resource(&mut self, name: &str, definition: &str) {
        self.resources
            .insert(String::from(name), String::from(definition));
    }

    fn get_resource(&self, name: &str) -> Result<String, Error> {
        // There may be an unidentified use case for user registered
        // resources lacking the ':'-sigil. So we postpone the check
        // for sigil until we know it is not a run-time user defined
        // resource we're looking for
        if let Some(result) = self.resources.get(name) {
            return Ok(result.to_string());
        }

        // TODO: Check for "known prefixes": 'ellps:', 'datum:', etc.
        let parts = name.split(':').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(Error::BadParam(
                "needing prefix:suffix format".to_string(),
                name.to_string(),
            ));
        }
        let prefix = parts[0];
        let suffix = parts[1];
        let section = "resources";

        // We do not know yet whether the resource is in a separate resource
        // file or in a resource register, so we generate file names for
        // both cases.
        let resource = prefix.to_string() + "_" + suffix + ".resource";
        let register = prefix.to_string() + ".register";
        let tag = "<".to_string() + suffix + ">";

        for path in &self.paths {
            // Is it in a separate file?
            let mut full_path = path.clone();
            full_path.push(section);
            full_path.push(&resource);
            if let Ok(result) = std::fs::read_to_string(full_path) {
                return Ok(result.trim().to_string());
            }

            // If not, search in a resource register
            let mut full_path = path.clone();
            full_path.push(section);
            full_path.push(&register);
            if let Ok(result) = std::fs::read_to_string(full_path) {
                let Some(mut start) = result.find(&tag) else {
                    continue;
                };
                start += tag.len();
                let Some(length) = result[start..].find('<') else {
                    // Search for end-of-item reached end-of-file
                    let result = result[start..].trim().to_string();
                    return Ok(result);
                };
                let result = result[start..start + length].trim().to_string();
                return Ok(result);
            }
        }

        Err(Error::NotFound(
            name.to_string(),
            ": User defined resource".to_string(),
        ))
    }

    fn get_blob(&self, name: &str) -> Result<Vec<u8>, Error> {
        let n = PathBuf::from(name);
        let ext = n
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        for path in &self.paths {
            let mut path = path.clone();
            path.push(ext);
            path.push(name);
            if let Ok(result) = std::fs::read(path) {
                return Ok(result);
            }
        }
        Err(Error::NotFound(name.to_string(), ": Blob".to_string()))
    }

    /// Access grid resources by identifier
    fn get_grid(&self, name: &str) -> Result<Grid, Error> {
        let n = PathBuf::from(name);
        let ext = n
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        for path in &self.paths {
            let mut path = path.clone();
            path.push(ext);
            path.push(name);
            let Ok(grid) = std::fs::read(path) else {
                continue;
            };
            return Grid::gravsoft(&grid);
        }
        Err(Error::NotFound(name.to_string(), ": Blob".to_string()))
    }
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use float_eq::assert_float_eq;

    #[test]
    fn basic() -> Result<(), Error> {
        let mut ctx = Plain::new();

        // Test the check for syntactic correctness (i.e. prefix:suffix-form)
        assert!(matches!(
            ctx.get_resource("foo"),
            Err(Error::BadParam(_, _))
        ));
        // Do we get the proper error code for non-existing resources?
        assert!(matches!(
            ctx.get_resource("foo:bar"),
            Err(Error::NotFound(_, _))
        ));
        // ...and the proper error code for non-existing grids?
        assert!(matches!(ctx.get_grid("foo"), Err(Error::NotFound(_, _))));

        // Try to instantiate the "stupid way of adding 1" macro
        // from geodesy/resources/stupid_way.resource
        let op = ctx.op("stupid:way")?;

        // ...and it works as expected?
        let mut data = some_basic_coor2dinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        // Now test that the look-up functionality works in general

        // Do we get the end address right in registers?
        assert!(ctx.get_resource("stupid:way_too")?.ends_with("addone"));
        // ...also at the end of the file?
        assert!(ctx.get_resource("stupid:way_too")?.ends_with("addone"));
        // And do we also get the start address right?
        assert!(ctx.get_resource("stupid:way_three")?.starts_with("addone"));

        // And just to be sure: once again for the plain resource file
        assert!(ctx.get_resource("stupid:way")?.starts_with("#"));
        assert!(ctx.get_resource("stupid:way")?.ends_with("addone"));

        // Now make sure, we can actually also *instantiate* a recipe
        // from a register
        let op = ctx.op("stupid:way_too")?;

        // ...and it works as expected?
        let mut data = some_basic_coor2dinates();

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 57.);
        assert_eq!(data[1][0], 61.);

        // Make sure we can access "sigil-less runtime defined resources"
        ctx.register_resource("foo", "bar");
        assert!(ctx.get_resource("foo")? == "bar");

        // We are *not* supposed to be able to instantiate a sigil-less resource
        ctx.register_resource("baz", "utm zone=32");
        assert!(ctx.op("baz").is_err());

        // But this classic should work...
        let op = ctx.op("geo:in | utm zone=32")?;
        let mut data = some_basic_coor2dinates();
        ctx.apply(op, Fwd, &mut data)?;
        let expected = [691875.6321396609, 6098907.825005002];
        assert_float_eq!(data[0].0, expected, abs_all <= 1e-9);

        Ok(())
    }
}
