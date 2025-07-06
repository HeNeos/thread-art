use std::f64::consts;

use crate::models::circle::{Circle, Point};

pub fn get_circle_points(circle: Circle, n: usize) -> Vec<Point> {
    let mut angle_partitions = vec![0_f64; n];
    for (i, angle) in angle_partitions.iter_mut().enumerate() {
        *angle = 2_f64 * consts::PI * i as f64 / (n as f64);
    }
    angle_partitions
        .into_iter()
        .map(|theta| Point {
            x: circle.center.x as f64 + circle.radius as f64 * theta.cos(),
            y: circle.center.y as f64 + circle.radius as f64 * theta.sin(),
        })
        .collect()
}
