use crate::CoordinateTuple;

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
}

impl GasHeader {
    pub fn new() -> GasHeader {
        let blank = CoordinateTuple(std::f64::NAN, std::f64::NAN, std::f64::NAN, std::f64::NAN);
        GasHeader {
            bbox: [blank, blank],
            delta: blank,
            dim: [1, 1, 1, 1, 1],
            skip: 0,
            size: 0,
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
                    CoordinateTuple(b[1], b[0], b[7], b[7]), // LL
                    CoordinateTuple(b[3], b[2], b[7], b[7]), // UR
                ];
                return true;
            }
            if n == 6 {
                // 3D
                self.bbox = [
                    CoordinateTuple(b[2], b[1], b[0], b[7]),
                    CoordinateTuple(b[5], b[4], b[3], b[7]),
                ];
                return true;
            }
            if n == 8 {
                // 4D
                self.bbox = [
                    CoordinateTuple(b[3], b[2], b[1], b[0]),
                    CoordinateTuple(b[3], b[2], b[1], b[0]),
                ];
                return true;
            }
            return false;
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
        self.delta.0 = (self.bbox[1].0 - self.bbox[0].0) / self.dim[0] as f64;
        self.delta.1 = (self.bbox[1].1 - self.bbox[0].1) / self.dim[1] as f64;
        self.delta.2 = (self.bbox[1].2 - self.bbox[0].2) / self.dim[2] as f64;
        self.delta.3 = (self.bbox[1].3 - self.bbox[0].3) / self.dim[3] as f64;
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

    // Returns an Iterator to the Reader of the lines of the file.
    // The output is wrapped in a Result to allow matching on errors.
    // Following https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub fn value(_at: CoordinateTuple) -> CoordinateTuple {
        // Grid interpolation!
        todo!()
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn splatsplatsplat() {
        use crate::Context;
        let ond = Context::new();
        assert_eq!(ond.stack.len(), 0);
        assert_eq!(ond.coord.0, 0.);
        assert_eq!(ond.coord.1, 0.);
        assert_eq!(ond.coord.2, 0.);
        assert_eq!(ond.coord.3, 0.);
    }
}
