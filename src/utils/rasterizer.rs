use crate::models::circle::{IntegerPoint, Point};

pub fn bresenham(start: Point, end: Point) -> Vec<IntegerPoint> {
    let (x1, y1) = (start.x.round() as u32, start.y.round() as u32);
    let (x2, y2) = (end.x.round() as u32, end.y.round() as u32);
    let sign_x: i32 = if x1 < x2 { 1 } else { -1 };
    let delta_x: i32 = if x1 < x2 {
        x2 as i32 - x1 as i32
    } else {
        x1 as i32 - x2 as i32
    };
    let sign_y: i32 = if y1 < y2 { 1 } else { -1 };
    let delta_y: i32 = if y1 < y2 {
        y2 as i32 - y1 as i32
    } else {
        y1 as i32 - y2 as i32
    };
    let mut error: i32 = delta_x - delta_y;
    let mut x: i32 = x1 as i32;
    let mut y: i32 = y1 as i32;

    let mut points: Vec<IntegerPoint> = Vec::new();
    for _ in 0..delta_x + delta_y + 1 {
        points.push(IntegerPoint { x, y });
        if x == x2 as i32 && y == y2 as i32 {
            break;
        }
        let e = 2 * error;
        if e > -delta_y {
            error -= delta_y;
            x += sign_x;
        }
        if e < delta_x {
            error += delta_x;
            y += sign_y;
        }
    }
    points
}
