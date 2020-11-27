pub mod helmert;
pub mod hulmert;
pub type Operation = Box<dyn Fn(&mut Coord, bool) -> bool>;
#[derive(Debug)]
pub struct Coord {
    pub first: f64,
    pub second: f64,
    pub third: f64,
    pub fourth: f64,
}
