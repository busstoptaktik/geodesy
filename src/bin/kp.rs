use geodesy::Gas;
fn main() {
    println!("Hello from kp!");
    let g = Gas::new("tests/geo.gas");
    println!("{:?}", g);
}
