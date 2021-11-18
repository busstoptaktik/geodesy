#![allow(dead_code)]

use crate::context::gys::GysResource;
use crate::CoordinateTuple;
use crate::Ellipsoid;
use crate::GeodesyError;
use enum_iterator::IntoEnumIterator;
use uuid::Uuid;
pub mod plain;

#[derive(Debug, IntoEnumIterator, Clone, Copy, PartialEq, PartialOrd)]
pub enum SearchLevel {
    LocalPatches,
    Locals,
    GlobalPatches,
    Globals,
    Builtins,
}

pub trait Provider {
    fn searchlevel(&self) -> SearchLevel {
        SearchLevel::Builtins
    }
    fn persistent_builtins(&self) -> bool {
        true
    }

    fn gys_resource(
        &self,
        branch: &str,
        name: &str,
        globals: Vec<(String, String)>,
    ) -> Result<GysResource, GeodesyError> {
        let definition = self.gys_definition(branch, name)?;
        Ok(GysResource::new(&definition, &globals))
    }

    fn gys_definition(&self, branch: &str, name: &str) -> Result<String, GeodesyError> {
        for i in SearchLevel::into_enum_iter() {
            if i < self.searchlevel() && i != SearchLevel::Builtins {
                continue;
            }
            if let Some(definition) = Self::get_gys_definition_from_level(self, i, branch, name) {
                return Ok(definition.trim().to_string());
            }
        }
        Err(GeodesyError::NotFound(format!("{}({})", branch, name)))
    }

    /// workhorse for gys_definition (perhaps implement as generic instead, since we don't actually want this to be user visible)
    fn get_gys_definition_from_level(
        &self,
        level: SearchLevel,
        branch: &str,
        name: &str,
    ) -> Option<String> {
        use std::io::BufRead;
        let filename: std::path::PathBuf = match level {
            SearchLevel::GlobalPatches => {
                // $HOME/share/geodesy/branch/name.gys
                let mut d = dirs::data_local_dir().unwrap_or_default();
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", name));
                d
            }
            SearchLevel::Globals => {
                // $HOME/share/geodesy/branch/branch.gys
                let mut d = dirs::data_local_dir().unwrap_or_default();
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", branch));
                d
            }
            SearchLevel::LocalPatches => {
                // ./geodesy/branch/name.gys
                let mut d = std::path::PathBuf::from(".");
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", name));
                d
            }
            SearchLevel::Locals => {
                // ./geodesy/branch/branch.gys
                let mut d = std::path::PathBuf::from(".");
                d.push("geodesy");
                d.push(branch);
                d.push(format!("{}.gys", branch));
                d
            }
            _ => return None,
        };

        let file = std::fs::File::open(filename);
        if file.is_err() {
            return None;
        }
        let mut file = file.unwrap();

        // Patches
        if level == SearchLevel::LocalPatches || level == SearchLevel::GlobalPatches {
            use std::io::Read;
            let mut definition = String::new();
            if file.read_to_string(&mut definition).is_ok() {
                return Some(definition);
            }
            return None;
        }

        // For non-patches, we search for the *last* occurence of a section with
        // the name we want, since updates are *appended* to the existing file
        let mut definition = String::new();
        let target = format!("<{}>", name);
        let mut skipping = true;

        let lines = std::io::BufReader::new(file).lines();
        for line in lines {
            if line.is_err() {
                continue;
            }
            let line = line.ok()?;
            if skipping && line.trim() == target {
                skipping = false;
                definition.clear();
                continue;
            }
            if skipping {
                continue;
            }
            if line.trim().starts_with('<') {
                if line.trim() != target {
                    skipping = true;
                    continue;
                }
                // Another instance of the same target
                definition.clear();
                continue;
            }
            definition += &line;
            definition += "\n";
        }

        if definition.is_empty() {
            return None;
        }
        Some(definition)
    }

    fn operate(&self, operation: Uuid, operands: &mut [CoordinateTuple], forward: bool) -> bool;

    /// Forward operation.
    fn fwd(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, true)
    }

    /// Inverse operation.
    fn inv(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }

    fn ellipsoid(&self, name: &str) -> Result<Ellipsoid, GeodesyError> {
        if name == "GRS80" {
            return Ok(Ellipsoid::default());
        }
        Err(GeodesyError::NotFound(String::from(name)))
    }

    // fn grid_descriptor(&self, name: &str) -> Result<GridDescriptor, GeodesyError> {
    //     Err(GeodesyError::NotFound(String::from(name)))
    // }
}
