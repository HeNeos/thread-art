#[derive(Debug)]
pub struct IntegerPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct Circle {
    pub center: IntegerPoint,
    pub radius: u32,
}
