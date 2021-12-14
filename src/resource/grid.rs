// use log::info;
/// Plain resource provider. Support for user defined operators
/// and macros using a text file library
use uuid::Uuid;

use super::GysResource;
// use crate::CoordinateTuple;
use crate::{GeodesyError, CoordinateTuple};
// use crate::Provider;

#[derive(Default, Debug)]
pub struct GridDescriptor {
    pub id: Uuid,

    /// Grid dimensions: Bands, Columns, Rows, Levels, Steps
    pub dim: [usize; 5],

    /// Distance from the start of each dimensional entity to the
    /// start of its successor, i.e.
    /// `[1, Bands, Bands*Columns, B*C*Rows, B*C*R*Levels]`
    pub stride: [usize; 5],

    /// The [First, Last] pair comprises the generalized bounding box:

    /// Generalized coordinates for the first element of the grid:
    /// First band, leftmost plane coordinate, topmost plane coordinate,
    /// lower height coordinate, first time step.
    /// *The origin of the grid*
    pub first: [f64; 5],

    /// Generalized coordinates for the last element of the grid:
    /// Last band, rightmost plane coordinate, bottommost plane coordinate,
    /// upper height coordinate, last time step.
    /// *The outer boundary of the grid*
    pub last: [f64; 5],

    pub delta: [f64; 5],
    pub scale: [f64; 8],
    pub offset: [f64; 8],

    /// `None` if using grid access via ResourceProvider,
    /// `Some(Vec<f32>)` if the grid is internalized
    pub grid: Option<Vec<f32>>
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

        let lower = args.numeric("Lower", f64::NAN)?;
        let upper = args.numeric("Upper", f64::NAN)?;

        let start = args.numeric("Start", f64::NAN)?;
        let end = args.numeric("End", f64::NAN)?;

        let bands = args.numeric("Bands", 1.)? as usize;
        let columns = args.required_numeric("Columns")? as usize;
        let rows = args.required_numeric("Rows")? as usize;
        let levels = args.numeric("Levels", 1.)? as usize;
        let steps = args.numeric("Steps", 1.)? as usize;

        let first = [0., left, top, lower, start];
        let last = [bands as f64 - 1., right, bottom, upper, end];

        let dim = [bands, columns, rows, levels, steps] as [usize; 5];
        let stride = [1usize, bands, bands*columns, bands*columns*rows, bands*columns*rows*levels];
        let mut delta = [0f64; 5];
        for i in 0..5 {
            delta[i] = if dim[i] < 2 {0.} else {(last[i] - first[i]) / (dim[i] - 1) as f64}
        }

        let scale = [1f64; 8];
        let offset = [0f64; 8];
        let id = Uuid::new_v4();

        assert!(columns > 1);
        assert!(rows > 1);
        Ok(GridDescriptor{id, dim, stride, first, last, delta, scale, offset, grid: None})
    }

    pub fn fractional_index(&self, at: CoordinateTuple) -> CoordinateTuple {
        todo!()
    }
}


#[cfg(test)]
mod grid_descriptor_tests {
    use crate::Provider;
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

        let columns = args.numeric("Columns", f64::NAN)?;
        let rows = args.numeric("Rows", f64::NAN)?;


        let geoid = rp_local.get_resource_definition("pile", "geoid")?;
        let g = GridDescriptor::new(&geoid)?;
        dbg!(&g);
        assert_eq!(g.first[0], 0.);
        assert_eq!(g.first[1], left);
        assert_eq!(g.first[2], top);

        let datum = rp_local.get_resource_definition("pile", "datum")?;
        let g = GridDescriptor::new(&datum)?;
        dbg!(&g);
        assert_eq!(g.last[0], 1.);
        assert_eq!(g.last[1], right);
        assert_eq!(g.last[2], bottom);


        assert!(columns > 1.);
        assert!(rows > 1.);
        println!("size: [{} x {}]", columns, rows);

        // from first to last
        println!("e interval: [{}; {}]", left, right);
        println!("n interval: [{}; {}]", top, bottom);

        // last minus first
        let de = (right - left) / (columns - 1.);
        let dn = (bottom - top) / (rows - 1.);
        println!("step: [{} x {}]", de, dn);


        // Fractional index numbers, i.e. the distance from the lower
        // left grid corner, measured in units of the grid sample distance
        // (note: grid corner - not coverage corner)
        // let cc: f64 = (at[0] - (b[0][0] + d[0] / 2.0)) / d[0];
        // let rr: f64 = (at[1] - (b[0][1] + d[1] / 2.0)) / d[1];

        assert!(1 == 0);
        Ok(())
    }
}
