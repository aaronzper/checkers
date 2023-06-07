#[derive(Debug, PartialEq)]
pub struct Point {
    pub x: u8,
    pub y: u8
}

impl From<(u8, u8)> for Point {
    fn from(point: (u8, u8)) -> Self {
        Point {
            x: point.0,
            y: point.1
        }
    }
}
