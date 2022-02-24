use super::*;
use std::collections::BTreeMap;
use crate::Provider;
use std::io::BufRead;

#[allow(dead_code)]
#[rustfmt::skip]
pub const GAMUT: [OpParameter; 15] = [
    OpParameter::Real { key: "Left", default: None },
    OpParameter::Real { key: "Right", default: None },

    OpParameter::Real { key: "Top", default: Some(0.) },
    OpParameter::Real { key: "Bottom", default: Some(0.) },

    OpParameter::Real { key: "Lower", default: Some(0.) },
    OpParameter::Real { key: "Upper", default: Some(0.) },

    OpParameter::Real { key: "Start", default: Some(0.) },
    OpParameter::Real { key: "End", default: Some(0.) },

    OpParameter::Natural { key: "Bands", default: Some(1) },
    OpParameter::Natural { key: "Columns", default: None },
    OpParameter::Natural { key: "Rows", default: Some(1) },
    OpParameter::Natural { key: "Levels", default: Some(1) },
    OpParameter::Natural { key: "Steps", default: Some(1) },

    OpParameter::Natural { key: "Whence", default: Some(0) },
    OpParameter::Text { key: "name", default: Some("") },
];

#[derive(Default, Debug)]
pub struct Grid {
    pub id: Uuid,

    /// Offset from start of storage to start of grid
    pub whence: usize,

    /// Grid dimensions: Bands, Columns, Rows, Levels, Steps
    pub dim: [usize; 5],

    /// Maximum numerical index (dim[i] - 1)
    pub max: [usize; 5],

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

fn gravsoft_grid_reader(name: &str, provider: &dyn Provider) -> Result<Grid, Error> {
    let buf = provider.access(name)?;
    let all = std::io::BufReader::new(buf.as_slice());

    // Split the contents into a vector of tokens
    let mut tokens = String::default();
    for line in all.lines() {
        let mut line = line?.clone();
        // Throw away comments
        let line = line.split('#').collect::<Vec<_>>()[0];
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        tokens += &line;
        tokens += " ";
    }
    let tokens = tokens.trim().split_whitespace().collect::<Vec<_>>();

    Grid::new("", None)
}


impl Grid {
    pub fn new(description: &str, grid: Option<Vec<f32>>) -> Result<Grid, Error> {
        let globals = BTreeMap::new();
        let raw = RawParameters::new(description, &globals);
        let params = ParsedParameters::new(&raw, &GAMUT)?;
        if description.starts_with("grid") {
            let p = Minimal::default();
            return gravsoft_grid_reader(description, &p);
        }

        let whence = params.natural("Whence")? as usize;

        let left = params.real("Left")?;
        let right = params.real("Right")?;

        let top = params.real("Top")?;
        let bottom = params.real("Bottom")?;

        let lower = params.real("Lower")?;
        let upper = params.real("Upper")?;

        let start = params.real("Start")?;
        let end = params.real("End")?;

        let bands = params.natural("Bands")? as usize;
        let columns = params.natural("Columns")? as usize;
        let rows = params.natural("Rows")? as usize;
        let levels = params.natural("Levels")? as usize;
        let steps = params.natural("Steps")? as usize;

        let first = [0., left, top, lower, start];
        let last = [bands as f64 - 1., right, bottom, upper, end];

        let dim = [bands, columns, rows, levels, steps] as [usize; 5];
        let mut max = [0_usize; 5];
        for i in 0..4 {
            max[i] = if dim[i + 1] >= 2 { dim[i + 1] - 2 } else { 0 };
        }
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
        Ok(Grid {
            id,
            whence,
            dim,
            max,
            stride,
            first,
            last,
            delta,
            scale,
            offset,
            grid,
        })
    }

    // The coordinate of a point given in units of the grid sample distance
    fn fractional_index(&self, at: Coord) -> Coord {
        let mut index = Coord::default();
        for i in 0_usize..4 {
            index[i] = (at[i] - self.first[i + 1]) / self.delta[i + 1];
        }
        index
    }

    // The index-coordinate of the grid point north-west of the point `at`
    fn cell_index(&self, at: Coord) -> [usize; 4] {
        let mut index = [0_usize; 4];
        for i in 0_usize..4 {
            index[i] = at[i].floor().max(0.).min(self.max[i] as f64) as usize;
        }
        index
    }

    // The grid value at a given integer index
    pub fn value(&self, at: &[usize; 4]) -> Coord {
        let mut lindex = 0;
        for i in 0_usize..4 {
            let c = at[i].min(self.max[i]).max(0);
            lindex += c * self.stride[i + 1];
        }
        if let Some(data) = &self.grid {
            let mut result = Coord::nan();
            for i in 0_usize..self.dim[0] {
                result[i] = data[lindex + i] as f64;
            }
            return result;
        }
        Coord::nan()
    }

    // 2D interpolation
    pub fn bilinear_value(&self, at: Coord) -> Coord {
        let index = self.fractional_index(at);

        let mut cell_ij = self.cell_index(index);
        let cell = Coord([
            cell_ij[0] as f64,
            cell_ij[1] as f64,
            cell_ij[2] as f64,
            cell_ij[3] as f64,
        ]);
        let delta = index - cell;

        let ul = self.value(&cell_ij);

        cell_ij[0] += 1;
        let ur = self.value(&cell_ij);
        dbg!(ur[0]);

        cell_ij[1] += 1;
        let lr = self.value(&cell_ij);
        dbg!(lr[0]);

        if cell_ij[0] > 0 {
            cell_ij[0] -= 1;
        }
        let ll = self.value(&cell_ij);
        dbg!(ll[0]);

        let top = ul.scale(1. - delta[0]) + ur.scale(delta[0]);
        let bot = ll.scale(1. - delta[0]) + lr.scale(delta[0]);
        bot.scale(delta[1]) + top.scale(1. - delta[1])
    }
}

// ----- T E S T S ---------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[allow(dead_code)]
    const HEADER: [f64; 6] = [54., 58., 8., 16., 1., 1.];

    #[rustfmt::skip]
    const GEOID: [f32; 5*9] = [
        58.08, 58.09, 58.10, 58.11, 58.12, 58.13, 58.14, 58.15, 58.16,
        57.08, 57.09, 57.10, 57.11, 57.12, 57.13, 57.14, 57.15, 57.16,
        56.08, 56.09, 56.10, 56.11, 56.12, 56.13, 56.14, 56.15, 56.16,
        55.08, 55.09, 55.10, 55.11, 55.12, 55.13, 55.14, 55.15, 55.16,
        54.08, 54.09, 54.10, 54.11, 54.12, 54.13, 54.14, 54.15, 54.16,
    ];

    #[allow(dead_code)]
    #[rustfmt::skip]
    const DATUM: [f32; 5*2*9] = [
        58., 08., 58., 09., 58., 10., 58., 11., 58., 12., 58., 13., 58., 14., 58., 15., 58., 16.,
        57., 08., 57., 09., 57., 10., 57., 11., 57., 12., 57., 13., 57., 14., 57., 15., 57., 16.,
        56., 08., 56., 09., 56., 10., 56., 11., 56., 12., 56., 13., 56., 14., 56., 15., 56., 16.,
        55., 08., 55., 09., 55., 10., 55., 11., 55., 12., 55., 13., 55., 14., 55., 15., 55., 16.,
        54., 08., 54., 09., 54., 10., 54., 11., 54., 12., 54., 13., 54., 14., 54., 15., 54., 16.,
    ];

    #[test]
    fn geoid_grid() -> Result<(), Error> {
        let description = "test_geoid Left=8 Right=16 Top=58 Bottom=54 Columns=9 Rows=5";
        let grid = Grid::new(description, Some(Vec::from(GEOID)))?;

        assert_eq!(grid.dim, [1, 9, 5, 1, 1]);
        assert_eq!(grid.first, [0., 8., 58., 0., 0.]);
        assert_eq!(grid.delta, [0., 1., -1., 0., 0.]);

        let index = [0_usize, 0, 0, 0];
        assert!((dbg!(grid.value(&index)[0]) - 58.08).abs() < 1e-5);
        let index = [2_usize, 1, 0, 0];
        assert!((dbg!(grid.value(&index)[0]) - 57.10).abs() < 1e-5);
        let at = Coord::raw(9.5, 57.5, 0., 0.);
        dbg!(at);
        let frac = dbg!(grid.fractional_index(at));
        dbg!(frac);
        let index = grid.cell_index(frac);
        assert!((dbg!(grid.value(&index)[0]) - 58.09).abs() < 1e-5);
        assert!((grid.bilinear_value(at)[0] - 57.595).abs() < 1e-5);

        Ok(())
    }

    #[test]
    fn datum_grid() -> Result<(), Error> {
        let description = "test_datum Left=8 Right=16 Top=58 Bottom=54  Columns=9 Rows=5 Bands=2";
        let grid = Grid::new(description, Some(Vec::from(DATUM)))?;

        assert_eq!(grid.dim, [2, 9, 5, 1, 1]);
        assert_eq!(grid.first, [0., 8., 58., 0., 0.]);
        assert_eq!(grid.delta, [1., 1., -1., 0., 0.]);

        let index = [0_usize, 0, 0, 0];
        assert!((dbg!(grid.value(&index)[0]) - 58.0).abs() < 1e-5);
        assert!((dbg!(grid.value(&index)[1]) - 08.0).abs() < 1e-5);
        let index = [2_usize, 1, 0, 0];
        assert!((dbg!(grid.value(&index)[0]) - 57.0).abs() < 1e-5);
        assert!((dbg!(grid.value(&index)[1]) - 10.0).abs() < 1e-5);
        let at = Coord::raw(9.4, 57.6, 0., 0.);
        dbg!(at);
        let frac = dbg!(grid.fractional_index(at));
        dbg!(frac);
        let index = grid.cell_index(frac);
        assert!((dbg!(grid.value(&index)[0]) - 58.).abs() < 1e-5);
        assert!((dbg!(grid.bilinear_value(at)[0]) - 57.6).abs() < 1e-5);
        assert!((dbg!(grid.bilinear_value(at)[1]) - 09.4).abs() < 1e-5);

        Ok(())
    }
}
