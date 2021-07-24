use crate::CoordinateTuple;

#[derive(Debug, Default)]
pub struct Gas {
    bbox: (CoordinateTuple, CoordinateTuple),
    delta: CoordinateTuple,
    dim: (usize, usize, usize, usize),
    v: Vec<f64>,
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
            let values: Vec<f64> = vec![];

            for line in lines {
                if let Ok(txt) = line {
                    let parts: Vec<&str> = txt.split('#').collect();
                    let text = parts[0].trim();
                    if text.is_empty() {
                        continue;
                    }
                    println!("{}", text);
                    match state {
                        GasReaderState::Pre => if txt=="header" {state = GasReaderState::Header;},
                        GasReaderState::Header => break,
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
