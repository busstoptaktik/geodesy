use crate::CoordinateTuple;
use crate::Ellipsoid;
use crate::GeodesyError;
use crate::GysResource;
use crate::Operator;
use crate::OperatorConstructor;
use crate::OperatorCore;
use uuid::Uuid;
pub mod gys;
pub mod minimal;
pub mod plain;
pub mod grid;

pub trait Provider {
    fn globals(&self) -> &[(String, String)];

    fn gys_resource(
        &self,
        branch: &str,
        name: &str,
        globals: Vec<(String, String)>,
    ) -> Result<GysResource, GeodesyError> {
        let definition = self.get_resource_definition(branch, name)?;
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

    #[allow(unused_variables)]
    fn get_resource_definition(&self, branch: &str, name: &str) -> Result<String, GeodesyError> {
        Err(GeodesyError::General(
            "Definition lookup not supported by this provider",
        ))
    }

    fn apply_operation(
        &self,
        operation: Uuid,
        operands: &mut [CoordinateTuple],
        forward: bool,
    ) -> bool;

    fn define_operation(&mut self, definition: &str) -> Result<Uuid, GeodesyError>;

    #[allow(unused_variables)]
    fn get_operation(&mut self, id: Uuid) -> Result<&Operator, GeodesyError> {
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
        self.apply_operation(operation, operands, true)
    }

    /// Operate in inverse direction.
    fn inv(&self, operation: Uuid, operands: &mut [CoordinateTuple]) -> bool {
        self.apply_operation(operation, operands, false)
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
pub fn roundtrip(
    rp: &mut dyn Provider,
    operation: &str,
    fwd_metric: u8,
    fwd_delta: f64,
    inv_metric: u8,
    inv_delta: f64,
    operands: &mut [CoordinateTuple],
    results: &mut [CoordinateTuple],
) -> bool {
    let op = rp.define_operation(operation);
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

    if !rp.get_operation(op).unwrap().invertible() {
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
