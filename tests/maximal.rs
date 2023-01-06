use geodesy::internal::*;

// ----- U S E R   P R O V I D E D   P R O V I D E R ----------------------------------

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
        operands: &mut [Coord],
    ) -> Result<usize, Error> {
        const BAD_ID_MESSAGE: Error = Error::General("Minimal: Unknown operator id");
        let op = self.operators.get(&op).ok_or(BAD_ID_MESSAGE)?;
        op.apply(self, operands, direction)
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
    fn get_grid(&self, _name: &str) -> Result<Grid, Error> {
        Err(Error::General(
            "Grid access by identifier not supported by the Maximal context provider",
        ))
    }
}

// ----- T E S T S ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::Maximal;
    use geodesy::preamble::*;

    #[test]
    fn maximal() -> Result<(), Error> {
        let mut prv = Maximal::default();
        let op = prv.op("gridshift grids=test.datum")?;
        let cph = Coord::geo(55., 12., 0., 0.);
        let mut data = [cph];

        prv.apply(op, Fwd, &mut data)?;
        let res = data[0].to_geo();
        assert!((res[0] - 55.015278).abs() < 1e-6);
        assert!((res[1] - 12.003333).abs() < 1e-6);

        prv.apply(op, Inv, &mut data)?;
        assert!((data[0][0] - cph[0]).abs() < 1e-10);
        assert!((data[0][1] - cph[1]).abs() < 1e-10);

        let cph = Coord::geo(55., 12., 0., 0.);

        // a somewhat unrelated test comparing RG's TM implementation with PROJ's
        // Guarded by if let Ok, since we do not want to fail this test in case of
        // no access to the proj executable
        if let Ok(pj) = prv.op("proj proj=utm zone=32") {
            let rg = prv.op("utm zone=32")?;
            let mut data_rg = [cph];
            let mut data_pj = [cph];
            prv.apply(rg, Fwd, &mut data_rg)?;
            prv.apply(pj, Fwd, &mut data_pj)?;
            assert!((data_rg[0].hypot2(&data_pj[0]) < 1e-4));
        }

        Ok(())
    }
}
