#![cfg(with_plain)]

use crate::context_authoring::*;
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
        const BAD_ID_MESSAGE: Error = Error::General("Local: Unknown operator id");
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        Ok(op.apply(self, operands, direction))
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
        if let Some(result) = self.resources.get(name) {
            return Ok(result.to_string());
        }

        // TODO: Check for "known prefixes": 'ellps:', 'datum:', etc.

        // We cannot have ':' in filenames on Windows, so we swap them for '_',
        // and add the ".macro" extension
        #[allow(clippy::single_char_pattern)]
        let name = name.replace(":", "_") + ".macro";

        let section = "macro";
        for path in &self.paths {
            let mut path = path.clone();
            path.push(section);
            path.push(&name);
            if let Ok(result) = std::fs::read_to_string(path) {
                return Ok(result);
            }
        }

        Err(Error::NotFound(name, ": User defined resource".to_string()))
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
    fn get_grid(&self, _name: &str) -> Result<Grid, Error> {
        Err(Error::General(
            "Grid access by identifier not supported by the Plain context provider",
        ))
    }
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() -> Result<(), Error> {
        let mut ctx = Plain::new();

        // The "stupid way of adding 1" macro from geodesy/macro/stupid_way.macro
        let op = ctx.op("stupid:way")?;

        let mut data = some_basic_coordinates();
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        ctx.apply(op, Fwd, &mut data)?;
        assert_eq!(data[0][0], 56.);
        assert_eq!(data[1][0], 60.);

        ctx.apply(op, Inv, &mut data)?;
        assert_eq!(data[0][0], 55.);
        assert_eq!(data[1][0], 59.);

        Ok(())
    }
}
