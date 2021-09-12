use crate::operator::OperatorCore;
use crate::Context;
use crate::CoordinateTuple;

impl Context {
    /// Roundtrip test that `operation` yields `results` when given `operands`.
    #[allow(clippy::too_many_arguments)]
    pub fn test(
        &mut self,
        operation: usize,
        fwd_metric: u8,
        fwd_delta: f64,
        inv_metric: u8,
        inv_delta: f64,
        operands: &mut [CoordinateTuple],
        results: &mut [CoordinateTuple],
    ) -> bool {
        if operation >= self.operations.len() {
            self.last_failing_operation = String::from("Invalid");
            self.cause = String::from("Attempt to access an invalid operator from test");
            println!("{}", self.report());
            return false;
        }

        // We need a copy of the operands as "expected results" in the roundtrip case
        // Note that the .to_vec() method actually copies, so .clone() is not needed.
        let roundtrip = operands.to_vec();

        // Forward test
        if !self.fwd(operation, operands) {
            println!("{}", self.report());
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

        if !self.operations[operation].invertible() {
            return true;
        }

        // Roundtrip
        if !self.inv(operation, results) {
            println!("{}", self.report());
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
}
