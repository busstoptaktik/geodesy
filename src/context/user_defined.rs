use std::path::PathBuf;

use crate::operator_construction::Operator;
use crate::operator_construction::OperatorConstructor;
use crate::Context;
use crate::GeodesyError;

impl Context {
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

    pub fn operation(&mut self, definition: &str) -> Result<usize, GeodesyError> {
        self.last_failing_operation_definition = definition.to_string();
        self.last_failing_operation.clear();
        self.cause.clear();
        let op = Operator::new(definition, self)?;
        let index = self.operations.len();
        self.operations.push(op);
        Ok(index)
    }

    /// Get definition string from the assets in the shared assets directory
    /// ($HOME/share or whatever passes for data_local_dir on the platform)
    pub fn get_shared_asset(branch: &str, name: &str, ext: &str) -> Option<String> {
        if let Some(mut dir) = dirs::data_local_dir() {
            dir.push("geodesy");
            return Context::get_asset(&mut dir, branch, name, ext);
        }
        None
    }

    /// Get definition string from the assets in the current directory
    pub fn get_private_asset(branch: &str, name: &str, ext: &str) -> Option<String> {
        let mut dir = PathBuf::from(".");
        Context::get_asset(&mut dir, branch, name, ext)
    }

    /// Workhorse for `get_shared_asset` and `get_private_asset`
    fn get_asset(dir: &mut PathBuf, branch: &str, name: &str, ext: &str) -> Option<String> {
        // This is the base directory we look in
        //dir.push("geodesy");

        // This is the filename we're looking for
        let mut filename = name.to_string();
        filename += ext;

        // We first look for standalone files that match
        let mut fullpath = dir.clone();
        fullpath.push("assets");
        fullpath.push(branch);
        fullpath.push(filename.clone());
        if let Ok(definition) = std::fs::read_to_string(fullpath) {
            return Some(definition);
        }

        // If not found as a freestanding file, try assets.zip
        use std::io::prelude::*;
        dir.push("assets.zip");
        // Open the physical zip file
        if let Ok(zipfile) = std::fs::File::open(dir) {
            // Hand it over to the zip archive reader
            if let Ok(mut archive) = zip::ZipArchive::new(zipfile) {
                // Is there a file with the name we're looking for in the zip archive?
                let mut full_filename = String::from("assets/");
                full_filename += branch;
                full_filename += "/";
                full_filename += &filename;
                if let Ok(mut file) = archive.by_name(&full_filename) {
                    let mut definition = String::new();
                    if file.read_to_string(&mut definition).is_ok() {
                        return Some(definition);
                    }
                }
            }
        }
        None
    }
}
