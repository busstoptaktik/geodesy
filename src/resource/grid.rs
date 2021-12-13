// use log::info;
/// Plain resource provider. Support for user defined operators
/// and macros using a text file library
use uuid::Uuid;

use super::GysResource;
// use crate::CoordinateTuple;
use crate::GeodesyError;
use crate::Provider;
// use crate::{Operator, OperatorConstructor, OperatorCore};

#[derive(Default, Debug)]
struct GridDescriptor {
    /// Grid dimensions: Bands, Columns, Rows, Levels, Steps
    dim: [usize; 5],

    /// Distance from the start of each dimensional entity to the
    /// start of its successor, i.e.
    /// `[1, Bands, Bands*Columns, B*C*Rows, B*C*R*Levels]`
    stride: [usize; 5],

    /// The [First, Last] pair comprises the generalized bounding box:

    /// Generalized coordinates for the first element of the grid:
    /// First band, leftmost plane coordinate, topmost plane coordinate,
    /// lower height coordinate, first time step
    first: [f64; 5],

    /// Generalized coordinates for the last element of the grid:
    /// Last band, rightmost plane coordinate, bottommost plane coordinate,
    /// upper height coordinate, last time step
    last: [f64; 5],

    delta: [f64; 5],
    scale: [f64; 8],
    offset: [f64; 8],

    /// `None` if using grid access via ResourceProvider,
    /// `Some(Vec<f32>)` if the grid is internalized
    grid: Option<Vec<f32>>
}

impl GridDescriptor {
    pub fn new(description: &str) -> Result<GridDescriptor, GeodesyError> {
        let gys = GysResource::new(&description, &[]);
        println!("GYS: {:#?}", gys);
        let mut args = gys.to_args(0)?;
        println!("ARGS: {:?}", args);

        let left = args.numeric("Left", f64::NAN)?;
        let right = args.numeric("Right", f64::NAN)?;

        let top = args.numeric("Top", f64::NAN)?;
        let bottom = args.numeric("Bottom", f64::NAN)?;

        let bands = args.numeric("Bands", 1.)?;
        let cols = args.numeric("Columns", f64::NAN)?;
        let rows = args.numeric("Rows", f64::NAN)?;
        let levels = args.numeric("Levels", 1.)?;
        let steps = args.numeric("Steps", 1.)?;



        assert!(cols > 1.);
        assert!(rows > 1.);
        println!("size: [{} x {}]", cols, rows);

        // from first to last
        println!("e interval: [{}; {}]", left, right);
        println!("n interval: [{}; {}]", top, bottom);

        // last minus first
        let de = (right - left) / (cols - 1.);
        let dn = (bottom - top) / (rows - 1.);
        println!("step: [{} x {}]", de, dn);

        Ok(GridDescriptor::default())
    }
}


#[cfg(test)]
mod grid_descriptor_tests {
    use super::*;
    use crate::GysResource;
    use crate::Plain;
    use crate::SearchLevel;
    #[test]
    fn plain() -> Result<(), GeodesyError> {
        let rp_patch = Plain::new(SearchLevel::LocalPatches, false);
        let foo = rp_patch.get_resource_definition("macros", "foo")?;
        assert_eq!(foo, "bar");
        let rp_local = Plain::new(SearchLevel::Locals, false);
        let foo = rp_local.get_resource_definition("macros", "foo")?;
        assert_eq!(foo, "baz");

        let pop = rp_local.get_resource_definition("pile", "geoid")?;
        println!("POP: {}", pop);
        let gys = GysResource::new(&pop, &[]);
        println!("GYS: {:#?}", gys);
        let mut args = gys.to_args(0)?;
        println!("ARGS: {:?}", args);

        let left = args.numeric("Left", f64::NAN)?;
        let right = args.numeric("Right", f64::NAN)?;

        let top = args.numeric("Top", f64::NAN)?;
        let bottom = args.numeric("Bottom", f64::NAN)?;

        let cols = args.numeric("Columns", f64::NAN)?;
        let rows = args.numeric("Rows", f64::NAN)?;

        assert!(cols > 1.);
        assert!(rows > 1.);
        println!("size: [{} x {}]", cols, rows);

        // from first to last
        println!("e interval: [{}; {}]", left, right);
        println!("n interval: [{}; {}]", top, bottom);

        // last minus first
        let de = (right - left) / (cols - 1.);
        let dn = (bottom - top) / (rows - 1.);
        println!("step: [{} x {}]", de, dn);

        assert!(1 == 0);
        Ok(())
    }
}
