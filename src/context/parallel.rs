use crate::context_authoring::*;
pub use std::path::PathBuf;

// ----- T H E   P A R A L L E L   C O N T E X T   P R O V I D E R ---------------------

/// Stub for a planned parallel-execution context

pub struct CoordinateSubSet<'a> {
    operands: &'a mut dyn CoordinateSet,
    begin: usize,
    end: usize
    // .. and a mock Coordinate metadata struct, to be mangled by the pipeline operator
}
// let end = operands.len();
// let mut cs = CoordinateSubSet { operands, begin: 0, end };

impl<'a> CoordinateSet for CoordinateSubSet<'a> {
    fn len(&self) -> usize {
        self.operands.len()
    }

    fn get(&self, index: usize) -> Coord {
        self.operands.get(index + self.begin)
    }

    fn set(&mut self, index: usize, value: &Coord) {
        self.operands.set(index + self.begin, value);
    }
}
