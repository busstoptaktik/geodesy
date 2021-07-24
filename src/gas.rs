use crate::CoordinateTuple;

#[derive(Debug, Default)]
pub struct Gas {
    header: GasHeader,
    v: Vec<f64>,
}

#[derive(Debug, Default)]
pub struct GasHeader {
    bbox: (CoordinateTuple, CoordinateTuple),
    delta: CoordinateTuple,
    dim: (usize, usize, usize, usize),
}

impl GasHeader {
    pub fn new() -> GasHeader {
        GasHeader{
            bbox: (
                CoordinateTuple(std::f64::NAN,std::f64::NAN,std::f64::NAN,std::f64::NAN),
                CoordinateTuple(std::f64::NAN,std::f64::NAN,std::f64::NAN,std::f64::NAN)
            ),
            delta: CoordinateTuple(std::f64::NAN,std::f64::NAN,std::f64::NAN,std::f64::NAN),
            dim: (1_usize, 1_usize, 1_usize, 1_usize)
        }
    }

    pub fn update(&mut self, text: &str) -> bool {
        let mut parts: Vec<&str> = text.split_whitespace().collect();
        let n = parts.len() - 1;
        if n < 1 || n > 8 {
            return false
        }
        let element = parts.remove(0);
        parts.extend(["NAN"; 8]);
        let mut b: Vec<f64> = vec![];
        for e in parts {
            b.push(e.parse().unwrap_or(std::f64::NAN))
        }

        if element=="bbox" {
            if  n==4 {  // 2D
                self.bbox = (
                    CoordinateTuple(b[1],b[0],b[7],b[7]),
                    CoordinateTuple(b[3],b[2],b[7],b[7])
                );
                return true;
            }
            if  n==6 {  // 3D
                self.bbox = (
                    CoordinateTuple(b[2],b[1],b[0],b[7]),
                    CoordinateTuple(b[5],b[4],b[3],b[7])
                );
                return true;
            }
            if  n==8 { // 4D
                self.bbox = (
                    CoordinateTuple(b[3],b[2],b[1],b[0]),
                    CoordinateTuple(b[3],b[2],b[1],b[0])
                );
                return true;
            }
            return false
        }

        if element=="delta" {
            return true;
        }



        false
    }
    pub fn is_complete(&self) -> bool {
        true
    }
}

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
#[derive(PartialEq, Debug)]
enum GasReaderState {Pre, Header, Grid, Post}


impl Gas {
    pub fn new(name: &str) -> Result<Gas, String> {
        if let Ok(lines) = Gas::read_lines(name) {
            let mut state = GasReaderState::Pre;
            let _values: Vec<f64> = vec![];
            let mut header = GasHeader::new();

            for line in lines {
                if let Ok(text) = line {
                    let parts: Vec<&str> = text.split('#').collect();
                    let text = parts[0].trim();
                    if text.is_empty() {
                        continue;
                    }
                    println!("{}", text);
                    match state {
                        GasReaderState::Pre =>
                            if text=="header" {state = GasReaderState::Header;},
                        GasReaderState::Header =>
                            if text=="grid" {
                                if header.is_complete() {
                                    state = GasReaderState::Grid;
                                    continue;
                                }
                                return Err("Incomplete grid header".to_string());
                            }
                            else {header.update(text); println!("{:?}", header)},
                        GasReaderState::Grid => break,
                        GasReaderState::Post => break,
                    }
                    continue;
                }
            }
            if state != GasReaderState::Post {
                return Err(String::from("Incomplete file"));
            }
        }
        Err(String::from("Not done"))
        //Ok(Gas {})
    }

    // Returns an Iterator to the Reader of the lines of the file.
    // The output is wrapped in a Result to allow matching on errors.
    // Following https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub fn value(_at: CoordinateTuple) -> CoordinateTuple {
        todo!()
    }
}

//----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn operand() {
        use crate::Context;
        let ond = Context::new();
        assert_eq!(ond.stack.len(), 0);
        assert_eq!(ond.coord.0, 0.);
        assert_eq!(ond.coord.1, 0.);
        assert_eq!(ond.coord.2, 0.);
        assert_eq!(ond.coord.3, 0.);
    }
}
