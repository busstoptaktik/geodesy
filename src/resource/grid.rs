#![allow(dead_code)]
#![allow(unused_variables)]

// use log::info;
/// Plain resource provider. Support for user defined operators
/// and macros using a text file library
use uuid::Uuid;

use super::GysResource;
use crate::CoordinateTuple;
use crate::GeodesyError;
// use crate::Provider;

#[derive(Default, Debug)]
pub struct GridDescriptor {
    pub id: Uuid,

    /// Offset from start of storage to start of grid
    pub whence: usize,

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
    pub grid: Option<Vec<f32>>,
}

impl GridDescriptor {
    pub fn new(description: &str) -> Result<GridDescriptor, GeodesyError> {
        let gys = GysResource::new(&description, &[]);
        println!("GYS: {:#?}", gys);
        let mut args = gys.to_args(0)?;
        println!("ARGS: {:?}", args);

        let whence = args.numeric("Whence", 0.)? as usize;

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
        let stride = [
            1usize,
            bands,
            bands * columns,
            bands * columns * rows,
            bands * columns * rows * levels,
        ];
        let mut delta = [0_f64; 5];
        for i in 0..5 {
            delta[i] = if dim[i] < 2 {
                0.
            } else {
                (last[i] - first[i]) / (dim[i] - 1) as f64
            }
        }

        let scale = [1f64; 8];
        let offset = [0f64; 8];
        let id = Uuid::new_v4();

        assert!(columns > 1);
        assert!(rows > 1);
        Ok(GridDescriptor {
            id,
            whence,
            dim,
            stride,
            first,
            last,
            delta,
            scale,
            offset,
            grid: None,
        })
    }

    pub fn fractional_index(&self, at: CoordinateTuple) -> CoordinateTuple {
        let mut index = CoordinateTuple::default();
        for i in 0_usize..4 {
            index[i] = (at[i] - self.first[i + 1]) / self.delta[i + 1];
        }
        index
    }

    pub fn clamped_fractional_index(&self, at: CoordinateTuple) -> CoordinateTuple {
        let mut index = self.fractional_index(at);
        for i in 0_usize..4 {
            index[i] = index[i].clamp(0., (self.dim[i] - 1).max(0) as f64)
        }
        index
    }

    pub fn floor_frac_ceil(
        &self,
        at: CoordinateTuple,
    ) -> ([usize; 4], CoordinateTuple, [usize; 4]) {
        let mut floor = [0_usize; 4];
        let mut ceil = [0_usize; 4];
        let mut frac = CoordinateTuple::origin();
        let index = self.clamped_fractional_index(at);
        for i in 0_usize..4 {
            let f = index[i].floor();
            floor[i] = f as usize;
            frac[i] = index[i] - f;
            ceil[i] = index[i].ceil() as usize;
        }
        (floor, frac, ceil)
    }

    pub fn bilinear_value(&self, at: CoordinateTuple, storage: &[f32]) -> CoordinateTuple {
        let correction = CoordinateTuple::origin();
        for i in 3usize..=0 {
            todo!()
        }
        correction
    }
}

#[cfg(test)]
mod grid_descriptor_tests {
    use super::*;
    use crate::GysResource;
    use crate::Plain;
    use crate::Provider;
    use crate::SearchLevel;
    #[test]
    fn plain() -> Result<(), GeodesyError> {
        let ctx = Plain::new(SearchLevel::Locals, false);
        let txt = ctx.get_resource_definition("pile", "geoid")?;
        println!("TXT: {}", txt);
        let gys = GysResource::new(&txt, &[]);
        println!("GYS: {:#?}", gys);
        let mut args = gys.to_args(0)?;
        println!("ARGS: {:?}", args);

        let left = args.numeric("Left", f64::NAN)?;
        let right = args.numeric("Right", f64::NAN)?;

        let top = args.numeric("Top", f64::NAN)?;
        let bottom = args.numeric("Bottom", f64::NAN)?;

        let columns = args.numeric("Columns", f64::NAN)?;
        let rows = args.numeric("Rows", f64::NAN)?;

        let geoid = ctx.get_resource_definition("pile", "geoid")?;
        let g = GridDescriptor::new(&geoid)?;
        dbg!(&g);
        assert_eq!(g.first[0], 0.);
        assert_eq!(g.first[1], left);
        assert_eq!(g.first[2], top);

        let datum = ctx.get_resource_definition("pile", "datum")?;
        let g = GridDescriptor::new(&datum)?;
        dbg!(&g);

        assert_eq!(g.last[0], 1.);
        assert_eq!(g.last[1], right);
        assert_eq!(g.last[2], bottom);
        let at = CoordinateTuple([2., 59., 0., 0.]);
        let fi = g.fractional_index(at);
        assert_eq!(fi[0], 2.);
        assert_eq!(fi[1], 1.);

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
        Ok(())
    }
}
