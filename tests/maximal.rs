use geodesy::authoring::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

// ----- U S E R   P R O V I D E D   C O N T E X T ----------------------------------

/// A direct copy of the code of the Minimal Context provider, renamed as Maximal.
/// Here used as a test and demo of how to write/use a user-provided context
/// provider.
///
/// Since the integration tests in the "tests" directory of a crate are handled as
/// independent crates, this provider could just as well have been built entirely
/// outside of the Rust Geodesy source tree.
///
/// The test in the "tests" section is identical to the one from inner-ops/gridshift.rs
/// and serves only to show that a user provided context provider is used in exactly
///  the same way as a system provided.

#[derive(Debug, Default)]
pub struct Maximal {
    /// Constructors for user defined operators
    constructors: BTreeMap<String, OpConstructor>,
    /// User defined resources (macros)
    resources: BTreeMap<String, String>,
    /// Instantiations of operators
    operators: BTreeMap<OpHandle, Op>,
}

const BAD_ID_MESSAGE: Error = Error::General("Maximal: Unknown operator id");

impl Context for Maximal {
    fn new() -> Maximal {
        Maximal::default()
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
                return Err(Error::General("Maximal: Bad step index"));
            }
            return Ok(op.params.clone());
        }

        // Not leaf level
        if index >= op.steps.len() {
            return Err(Error::General("Maximal: Bad step index"));
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
        if let Some(result) = self.resources.get(name) {
            return Ok(result.to_string());
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
        let path: PathBuf = [".", "geodesy", ext, name].iter().collect();
        Ok(std::fs::read(path)?)
    }
    /// Access grid resources by identifier
    fn get_grid(&self, name: &str) -> Result<Arc<dyn Grid>, Error> {
        let buf = self.get_blob(name)?;
        let grid = BaseGrid::gravsoft(&buf)?;

        Ok(Arc::new(grid))
    }
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::Maximal;
    use geodesy::authoring::*;

    // Test that the fundamental tokenization functionality also works
    // outside of the library
    #[test]
    fn token() -> Result<(), Error> {
        assert_eq!("foo bar $ baz = bonk".normalize(), "foo bar $baz=bonk");
        assert_eq!(
            "foo |  bar baz  =  bonk, bonk , bonk".normalize(),
            "foo|bar baz=bonk,bonk,bonk"
        );
        assert_eq!(
            "foo |  bar baz  =  bonk, bonk , bonk".split_into_steps()[0],
            "foo"
        );
        assert_eq!("foo bar baz=bonk".split_into_parameters()["_name"], "foo");
        assert_eq!("foo bar baz=bonk".split_into_parameters()["bar"], "true");
        assert_eq!("foo bar baz=bonk".split_into_parameters()["baz"], "bonk");
        assert!("foo | bar".is_pipeline());
        assert!("foo:bar".is_resource_name());
        assert_eq!("foo bar baz=bonk".operator_name(), "foo");
        assert_eq!("foo bar baz=  $bonk".operator_name(), "foo");
        Ok(())
    }

    #[test]
    fn maximal() -> Result<(), Error> {
        let mut ctx = Maximal::default();
        let op = ctx.op("gridshift grids=test.datum")?;
        let cph = Coor4D::geo(55., 12., 0., 0.);
        let mut data = [cph];

        ctx.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!((res[0] - 55.015278).abs() < 1e-6);
        assert!((res[1] - 12.003333).abs() < 1e-6);

        ctx.apply(op, Inv, &mut data)?;
        assert!((data[0][0] - cph[0]).abs() < 1e-10);
        assert!((data[0][1] - cph[1]).abs() < 1e-10);

        let cph = Coor4D::geo(55., 12., 0., 0.);

        // a somewhat unrelated test comparing RG's TM implementation with PROJ's
        // Guarded by if let Ok, since we do not want to fail this test in case of
        // no access to the proj executable
        if let Ok(pj) = ctx.op("proj proj=utm zone=32") {
            let rg = ctx.op("utm zone=32")?;
            let mut data_rg = [cph];
            let mut data_pj = [cph];
            ctx.apply(rg, Fwd, &mut data_rg)?;
            ctx.apply(pj, Fwd, &mut data_pj)?;
            assert!((data_rg[0].hypot2(&data_pj[0]) < 1e-4));
        }

        Ok(())
    }
}
