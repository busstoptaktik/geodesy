use geodesy::CoordinateTuple;
use geodesy::Gas;

fn main() {
    println!("Hello from kp!");
    let g = Gas::new("tests/geo.gas").unwrap();
    println!("{:?}", g);
    println!("{:?}", g.value(CoordinateTuple(8.5, 55.00, 0., 0.)));
}
