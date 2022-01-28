use crate::internal::*;

#[cfg_attr(test, automock)]
pub trait Provider {
    fn globals(&self) -> BTreeMap<String, String>;
    fn register_op(&mut self, name: &str, constructor: OpConstructor);
    fn get_op(&self, name: &str) -> Result<OpConstructor, Error>;
    fn register_resource(&mut self, name: &str, definition: &str);
    fn get_resource(&self, name: &str) -> Result<String, Error>;
    fn apply(&self, op: Uuid, direction: Direction, operands: &mut [CoordinateTuple]) -> usize;
}

#[derive(Debug, Default)]
pub struct Minimal {
    constructors: BTreeMap<String, OpConstructor>,
    resources: BTreeMap<String, String>,
}

impl Provider for Minimal {
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

    fn apply(&self, op: Uuid, direction: Direction, operands: &mut [CoordinateTuple]) -> usize {
        todo!()
    }
}
