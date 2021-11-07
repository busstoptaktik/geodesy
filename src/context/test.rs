use crate::operator::OperatorCore;
use crate::Context;
use crate::CoordinateTuple;

impl Context {
    /// Roundtrip test that `operation` yields `results` when given `operands`.
    #[allow(clippy::too_many_arguments)]
    pub fn test(
        operation: &str,
        fwd_metric: u8,
        fwd_delta: f64,
        inv_metric: u8,
        inv_delta: f64,
        operands: &mut [CoordinateTuple],
        results: &mut [CoordinateTuple],
    ) -> bool {
        let mut ctx = Context::new();
        let op = ctx.operation(operation);
        if op.is_err() {
            println!("{:?}", op);
            return false;
        }
        let op = op.unwrap();

        // We need a copy of the operands as "expected results" in the roundtrip case
        // Note that the .to_vec() method actually copies, so .clone() is not needed.
        let roundtrip = operands.to_vec();

        // Forward test
        if !ctx.fwd(op, operands) {
            println!("Fwd operation failed for {}", ctx.operations[op].name());
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

        if !ctx.operations[op].invertible() {
            return true;
        }

        // Roundtrip
        if !ctx.inv(op, results) {
            println!("Inv operation failed for {}", ctx.operations[op].name());
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
