#![allow(dead_code)]

use crate::CoordinateTuple;
use crate::Ellipsoid;
use crate::GeodesyError;
use crate::GysResource;
use crate::Operator;
use crate::OperatorConstructor;
use crate::OperatorCore;
use enum_iterator::IntoEnumIterator;
use uuid::Uuid;
pub mod gys;
pub mod minimal;
pub mod plain;

#[derive(Debug, IntoEnumIterator, Clone, Copy, Eq, PartialEq, PartialOrd)]
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

    fn globals(&self) -> &[(String, String)];

    fn gys_resource(
        &self,
        branch: &str,
        name: &str,
        globals: Vec<(String, String)>,
    ) -> Result<GysResource, GeodesyError> {
        let definition = self.gys_definition(branch, name)?;
        Ok(GysResource::new(&definition, &globals))
    }

    #[allow(unused_variables)]
    fn get_user_defined_macro(&self, name: &str) -> Option<&String> {
        None
    }

    #[allow(unused_variables)]
    fn get_user_defined_operator(&self, name: &str) -> Option<&OperatorConstructor> {
        None
    }

    fn gys_definition(&self, branch: &str, name: &str) -> Result<String, GeodesyError> {
        if branch == "macros" {
            if let Some(m) = self.get_user_defined_macro(name) {
                return Ok(String::from(m));
            }
        }

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

    fn operation(&mut self, definition: &str) -> Result<Uuid, GeodesyError>;

    #[allow(unused_variables)]
    fn operator(&mut self, id: Uuid) -> Result<&Operator, GeodesyError> {
        Err(GeodesyError::General("Operator extraction not supported"))
    }

    #[allow(unused_variables)]
    fn register_macro(&mut self, name: &str, definition: &str) -> Result<bool, GeodesyError> {
        Err(GeodesyError::General("Macro registration not supported"))
    }

    #[allow(unused_variables)]
    fn register_operator(
        &mut self,
        name: &str,
        constructor: OperatorConstructor,
    ) -> Result<bool, GeodesyError> {
        Err(GeodesyError::General("Operator registration not supported"))
    }

    /// Operate in forward direction.
    fn fwd(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, true)
    }

    /// Operate in inverse direction.
    fn inv(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.operate(operation, operands, false)
    }

    fn ellipsoid(&self, name: &str) -> Result<Ellipsoid, GeodesyError> {
        Ellipsoid::named(name)
    }

    // fn grid_descriptor(&self, name: &str) -> Result<GridDescriptor, GeodesyError> {
    //     Err(GeodesyError::NotFound(String::from(name)))
    // }
}

/// Roundtrip test that `operation` yields `results` when given `operands`.
#[allow(clippy::too_many_arguments)]
pub fn test(
    rp: &mut dyn Provider,
    operation: &str,
    fwd_metric: u8,
    fwd_delta: f64,
    inv_metric: u8,
    inv_delta: f64,
    operands: &mut [CoordinateTuple],
    results: &mut [CoordinateTuple],
) -> bool {
    let op = rp.operation(operation);
    if op.is_err() {
        println!("{:?}", op);
        return false;
    }
    let op = op.unwrap();

    // We need a copy of the operands as "expected results" in the roundtrip case
    // Note that the .to_vec() method actually copies, so .clone() is not needed.
    let roundtrip = operands.to_vec();

    // Forward test
    if !rp.fwd(op, operands) {
        println!("Fwd operation failed for {}", operation);
        return false;
    }
    for i in 0..operands.len() {
        let delta = match fwd_metric {
            0 => operands[i].hypot2(&results[i]),
            2 => operands[i].hypot2(&results[i]),
            _ => operands[i].hypot3(&results[i]),
        };
        if delta < fwd_delta {
            continue;
        }
        println!(
            "Failure in forward test[{}]: delta = {:.4e} (expected delta < {:e})",
            i, delta, fwd_delta
        );
        println!("    got       {:?}", operands[i]);
        println!("    expected  {:?}", results[i]);
        return false;
    }

    if !rp.operator(op).unwrap().invertible() {
        return true;
    }

    // Roundtrip
    if !rp.inv(op, results) {
        println!("Inv operation failed for {}", operation);
        return false;
    }
    for i in 0..operands.len() {
        let delta = match inv_metric {
            0 => roundtrip[i].default_ellps_dist(&results[i]),
            2 => roundtrip[i].hypot2(&results[i]),
            _ => roundtrip[i].hypot3(&results[i]),
        };
        if delta < inv_delta {
            continue;
        }
        println!(
            "Failure in inverse test[{}]: delta = {:.4e} (expected delta < {:e})",
            i, delta, inv_delta
        );
        println!("    got       {:?}", results[i]);
        println!("    expected  {:?}", roundtrip[i]);
        return false;
    }
    true
}
