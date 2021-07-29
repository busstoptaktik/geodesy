use crate::CoordinateTuple;

/// Handler for the **G**eodetic grid **A**uthoring **S**ystem
#[derive(Debug, Default)]
pub struct Gas {
    header: GasHeader,
    grid: Vec<f64>,
}

#[derive(Debug, Default)]
pub struct GasHeader {
    bbox: [CoordinateTuple; 2],
    delta: CoordinateTuple,
    dim: [usize; 5],
    skip: usize,
    size: usize,
    scale: f64,
    offset: f64,
}

impl GasHeader {
    pub fn new() -> GasHeader {
        GasHeader {
            bbox: [CoordinateTuple::nan(), CoordinateTuple::nan()],
            delta: CoordinateTuple::nan(),
            dim: [1, 1, 1, 1, 1],
            skip: 0,
            size: 0,
            scale: 1.,
            offset: 0.,
        }
    }

    pub fn update(&mut self, text: &str) -> bool {
        // Parse the input line as a record type id followed by 1-8 numbers
        let mut parts: Vec<&str> = text.split_whitespace().collect();
        let n = parts.len() - 1;

        if !(1..=8).contains(&n) {
            return false;
        }

        let element = parts.remove(0);
        parts.extend(["NAN"; 8]);
        let mut b: Vec<f64> = vec![];
        for e in parts {
            b.push(e.parse().unwrap_or(std::f64::NAN))
        }

        // The Bounding Box consists of 2 CoordinateTuples, lower left and upper right
        if element == "bbox" {
            if n == 4 {
                // 2D
                self.bbox = [
                    CoordinateTuple::raw(b[1], b[0], b[7], b[7]), // LL
                    CoordinateTuple::raw(b[3], b[2], b[7], b[7]), // UR
                ];
                return true;
            }
            if n == 6 {
                // 3D
                self.bbox = [
                    CoordinateTuple::raw(b[2], b[1], b[0], b[7]),
                    CoordinateTuple::raw(b[5], b[4], b[3], b[7]),
                ];
                return true;
            }
            if n == 8 {
                // 4D
                self.bbox = [
                    CoordinateTuple::raw(b[3], b[2], b[1], b[0]),
                    CoordinateTuple::raw(b[3], b[2], b[1], b[0]),
                ];
                return true;
            }
            return false;
        }

        if element == "scale" {
            self.scale = b[0];
            return true;
        }

        if element == "offset" {
            self.offset = b[0];
            return true;
        }

        if element == "cols" || element == "columns" {
            self.dim[0] = b[0] as usize;
            return true;
        }

        if element == "rows" {
            self.dim[1] = b[0] as usize;
            return true;
        }

        if element == "levels" {
            self.dim[2] = b[0] as usize;
            return true;
        }

        if element == "steps" {
            self.dim[3] = b[0] as usize;
            return true;
        }

        if element == "skip" {
            self.skip = b[0] as usize;
            return true;
        }

        if element == "bands" || element == "channels" {
            self.dim[4] = b[0] as usize;
            return true;
        }

        false
    }

    pub fn finalize(&mut self) -> bool {
        self.delta[0] = (self.bbox[1][0] - self.bbox[0][0]) / self.dim[0] as f64;
        self.delta[1] = (self.bbox[1][1] - self.bbox[0][1]) / self.dim[1] as f64;
        self.delta[2] = (self.bbox[1][2] - self.bbox[0][2]) / self.dim[2] as f64;
        self.delta[3] = (self.bbox[1][3] - self.bbox[0][3]) / self.dim[3] as f64;
        self.size = self.dim.iter().product();
        true
    }
}

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
#[derive(PartialEq, Debug)]
enum GasReaderState {
    Pre,
    Header,
    Grid,
    Post,
}

impl Gas {
    pub fn new(name: &str) -> Result<Gas, String> {
        let mut grid: Vec<f64> = vec![];
        let mut header = GasHeader::new();

        // Loop over all lines in the file. Select actions through state changes
        if let Ok(lines) = Gas::read_lines(name) {
            let mut state = GasReaderState::Pre;

            for line in lines.flatten() {
                let parts: Vec<&str> = line.split('#').collect();
                let line = parts[0].trim();
                if line.is_empty() {
                    continue;
                }
                match state {
                    // Skip the prose in the file preamble
                    GasReaderState::Pre => {
                        if line == "header" {
                            state = GasReaderState::Header
                        }
                    }

                    // Read and finalize the header
                    GasReaderState::Header => {
                        if line == "grid" {
                            if !header.finalize() {
                                return Err("Bad header".to_string());
                            }
                            state = GasReaderState::Grid;
                        } else if !header.update(line) {
                            return Err("Bad header: ".to_string() + line);
                        }
                    }

                    // Read the grid
                    GasReaderState::Grid => {
                        let mut parts: Vec<&str> = line.split_whitespace().collect();
                        let n = parts.len();
                        if n < header.skip + header.dim[4] {
                            return Err("Bad format: ".to_string() + line);
                        }
                        parts.drain(0..header.skip);
                        parts.truncate(header.dim[4]);
                        for v in parts {
                            grid.push(v.parse().unwrap_or(std::f64::NAN));
                        }
                        if grid.len() >= header.size {
                            state = GasReaderState::Post;
                        }
                    }

                    // Tell the world we're done, so we can skip any remaining parts
                    GasReaderState::Post => break,
                }
                continue;
            }

            if state != GasReaderState::Post {
                return Err(String::from("Incomplete file"));
            }
        }
        Ok(Gas { header, grid })
    }

    // Iterator to the Reader of the lines of the file. Following the example
    // https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
    fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    #[allow(clippy::many_single_char_names)]
    pub fn value(&self, at: CoordinateTuple) -> CoordinateTuple {
        // TODO: Generalize to 1D, 3D and 4D - currently interpolates
        // any number of channels, but in the first plane only.

        // For brevity: References to bbox, delta and dim.
        let b = &self.header.bbox;
        let d = &self.header.delta;
        let dim = &self.header.dim;

        // Fractional row/column numbers, i.e. the distance from the lower
        // left grid corner, measured in units of the grid sample distance
        // (note: grid corner - not coverage corner)
        let cc: f64 = (at[0] - (b[0][0] + d[0] / 2.0)) / d[0];
        let rr: f64 = (at[1] - (b[0][1] + d[1] / 2.0)) / d[1];

        // As long as we're inside of the grid coverage, (ii,jj) becomes
        // the lower left corner of the cell containing (rr,cc). But we
        // prepare for extrapolation by clamping the cell extent to the
        // grid extent:
        //
        //     The last row of the grid is numbered nrows-1,
        //     so the bottom of the interpolation cell cannot
        //     go further up than nrows-2
        //
        // and:
        //
        //     The last column of the grid is numbered ncols-1,
        //     so the left side of the interpolation cell
        //     cannot go further right than ncols-2
        let ii = std::cmp::min(rr.floor() as usize, dim[1].saturating_sub(2));
        let jj = std::cmp::min(cc.floor() as usize, dim[0].saturating_sub(2));

        // And how far into the cell are we?
        // Note: dr and dc are NOT constrained to [0:1], so they
        // invoke extrapolation when outside of the grid coverage.
        let dr = rr - ii as f64;
        let dc = cc - jj as f64;

        // We need the starting addresses of each of the interpolation
        // cell corners.
        let record_size = dim[4];
        let ll = (ii * dim[0] + jj) * record_size;
        let ul = ((ii + 1) * dim[0] + jj) * record_size;
        let lr = ll + record_size;
        let ur = ul + record_size;

        // Do the interpolation...
        let mut v = [std::f64::NAN; 4];
        #[allow(clippy::needless_range_loop)]
        for i in 0..record_size {
            let left = self.grid[ll + i] * (1. - dr) + self.grid[ul + i] * dr;
            let right = self.grid[lr + i] * (1. - dr) + self.grid[ur + i] * dr;
            v[i] = left * (1. - dc) + right * dc;
        }
        let s = self.header.scale;
        let o = self.header.offset;
        CoordinateTuple::raw(o + s * v[0], o + s * v[1], v[2], v[3])
    }

    pub fn fwd(&self, at: CoordinateTuple) -> CoordinateTuple {
        let dv = self.value(at);
        CoordinateTuple::raw(at[0] + dv[0], at[1] + dv[1], at[2], at[3])
    }

    pub fn inv(&self, at: CoordinateTuple) -> CoordinateTuple {
        // The naive first guess at where we came from
        let mut dv = self.value(at);
        let mut v = CoordinateTuple::raw(at[0] - dv[0], at[1] - dv[1], at[2], at[3]);

        for _i in 1..30 {
            // If that guess was correct, `vv` would match `at` exactly
            dv = self.value(v);
            let vv = CoordinateTuple::raw(v[0] + dv[0], v[1] + dv[1], at[2], at[3]);

            // This is the correction, giving us an exact match for `at`: vv + dv = at
            dv = CoordinateTuple::raw(at[0] - vv[0], at[1] - vv[1], at[2], at[3]);
            // We suppose it is "almost as good" over at the other end
            v = CoordinateTuple::raw(v[0] + dv[0], v[1] + dv[1], at[2], at[3]);
            if dv[0].hypot(dv[1]) < 1.0e-10 {
                break;
            }
        }
        v
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::CoordinateTuple;
    use crate::Gas;
    #[test]
    fn interpolation() {
        use std::f64::NAN;
        let g = Gas::new("tests/geo.gas").unwrap();
        let b = g.header.bbox[0];
        let c = g.header.bbox[1];
        let d = g.header.delta;
        let dim = g.header.dim;

        // Was the header read correctly?
        assert_eq!(b[0], 7.0);
        assert_eq!(b[1], 54.0);
        assert_eq!(c[0], 13.0);
        assert_eq!(c[1], 58.0);
        assert_eq!(d[0], 1.0);
        assert_eq!(d[1], 0.5);
        assert_eq!(dim, [6, 8, 1, 1, 2]);
        assert_eq!(g.header.scale, 1.);
        assert_eq!(g.header.offset, 0.);

        // Does the grid have the right size?
        assert_eq!(g.grid.len(), 96);

        // Does the interpolation work correctly?

        // Lower left corner
        let at = CoordinateTuple::raw(b[0] + d[0] / 2., b[1] + d[1] / 2., NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);

        // Lower right corner
        let at = CoordinateTuple::raw(c[0] - d[0] / 2., b[1] + d[1] / 2., NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);

        // Upper left corner
        let at = CoordinateTuple::raw(b[0] + d[0] / 2., c[1] - d[1] / 2., NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);

        // Upper right corner
        let at = CoordinateTuple::raw(c[0] - d[0] / 2., c[1] - d[1] / 2., NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);

        // Left of lower left corner
        let at = CoordinateTuple::raw(b[0], b[1] + d[1] / 2., NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);

        // Below lower left corner
        let at = CoordinateTuple::raw(b[0] + d[0] / 2., b[1], NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);

        // MUCH below and to the left of lower left corner
        let at = CoordinateTuple::raw(b[0] - 4., b[1] - 3., NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);

        // MUCH above and to the right of upper right corner
        let at = CoordinateTuple::raw(c[0] + 4., c[1] + 3., NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);

        // A non-rational place inside the grid
        let at = CoordinateTuple::raw(8.77, 55.11, NAN, NAN);
        let v = g.value(at);
        assert!(v.hypot2(&at) < 1.0e-10);
    }

    #[test]
    fn transformation() {
        use std::f64::NAN;
        let g = Gas::new("tests/ungeo.gas").unwrap();
        let b = g.header.bbox[0];
        let c = g.header.bbox[1];
        let d = g.header.delta;
        let dim = g.header.dim;

        // Was the header read correctly?
        assert_eq!(b[0], 7.0);
        assert_eq!(b[1], 54.0);
        assert_eq!(c[0], 13.0);
        assert_eq!(c[1], 58.0);
        assert_eq!(d[0], 1.0);
        assert_eq!(d[1], 0.5);
        assert_eq!(dim, [6, 8, 1, 1, 2]);
        // assert_eq!(g.header.scale, 0.001);
        assert_eq!(g.header.offset, 0.);

        // Does the grid have the right size?
        assert_eq!(g.grid.len(), 96);

        // Does the inverse interpolation work correctly?

        // A non-rational place inside the grid
        let at = CoordinateTuple::raw(8.77, 55.11, NAN, NAN);
        let transformed_at = g.fwd(at);
        let backtransformed_at = g.inv(transformed_at);
        assert!(at.hypot2(&backtransformed_at) < 1.0e-10);
    }
}
