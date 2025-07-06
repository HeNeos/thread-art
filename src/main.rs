mod models;
mod utils;

use std::collections::HashSet;

use clap::Parser;
use image::{GenericImageView, Rgba};

use crate::models::circle::Point;
use crate::models::circle::{Circle, IntegerPoint};
use crate::models::cli::Cli;
use crate::models::image::{Image, ImageDimensions};
use crate::utils::generate_circle::get_circle_points;
use crate::utils::rasterizer::bresenham;

fn point_to_index(n: usize, i: usize, j: usize) -> usize {
    if i < j {
        i * (n - 1) - i * (i + 1) / 2 + j - 1
    } else {
        j * (n - 1) - j * (j + 1) / 2 + i - 1
    }
}

fn get_metrics(line: &Vec<IntegerPoint>, image: &Image) -> (u32, u32) {
    let mut accuracy: u32 = 0;
    let mut error: u32 = 0;
    for point in line {
        let x = point.x;
        let y = point.y;
        if x < 0
            || y < 0
            || x >= image.dimensions.width as i32
            || y >= image.dimensions.height as i32
        {
            continue;
        }
        let pixel = image
            .image
            .get_pixel(x as u32, (image.dimensions.height as i32 - y - 1) as u32);
        accuracy += match pixel {
            Rgba([255, 255, 255, 255]) => 0,
            _ => 1,
        };
        error += match pixel {
            Rgba([255, 255, 255, 255]) => 1,
            _ => 0,
        };
    }
    (accuracy, error)
}

fn solve(bresenham_lines: Vec<Vec<IntegerPoint>>, image: &Image, n: usize) -> Vec<usize> {
    let mut current_point_index: usize = 0;
    let mut path: Vec<usize> = Vec::new();
    let mut visited: HashSet<usize> = HashSet::new();
    path.push(current_point_index);
    loop {
        let mut current_accuracy: u32 = 0;
        let mut current_error: u32 = 2 * n as u32;
        let mut best_point_index = current_point_index;
        for j in 0..n {
            if current_point_index == j {
                continue;
            }
            let current_index = point_to_index(n, current_point_index, j);
            if visited.contains(&current_index) {
                continue;
            }
            let bresenham_line = &bresenham_lines[current_index];
            let (accuracy, error) = get_metrics(bresenham_line, image);
            if accuracy > current_accuracy
                || (accuracy == current_accuracy && error < current_error)
            {
                current_accuracy = accuracy;
                current_error = error;
                best_point_index = j;
            }
        }
        if current_point_index == best_point_index {
            break;
        }
        if current_accuracy as f32 / ((current_accuracy + current_error) as f32) < 0.45_f32 {
            break;
        }
        let line_index = point_to_index(n, current_point_index, best_point_index);
        visited.insert(line_index);
        current_point_index = best_point_index;
        path.push(best_point_index);
    }
    path
}

fn main() {
    let args = Cli::parse();
    let image: Image = Image::read_image(args.image_path);
    let max_height: u32 = 512;
    let resized_image: Image = Image::resize_image(
        image,
        ImageDimensions {
            width: max_height,
            height: max_height,
        },
    );
    let image_bw = Image::to_black_and_white(resized_image);
    let n: usize = 1024;
    let circle = Circle {
        center: IntegerPoint {
            x: max_height as i32 / 2,
            y: max_height as i32 / 2,
        },
        radius: max_height / 2,
    };
    let points = get_circle_points(circle, n);
    let mut all_index: Vec<(usize, usize)> = vec![(0, 0); n * (n - 1) / 2];
    let mut all_lines: Vec<(Point, Point)> =
        vec![(Point { x: 0_f64, y: 0_f64 }, Point { x: 0_f64, y: 0_f64 }); n * (n - 1) / 2];
    for i in 0..n {
        for j in i + 1..n {
            let index: usize = point_to_index(n, i, j);
            all_index[index] = (i, j);
            all_lines[index] = (
                Point {
                    x: points[i].x,
                    y: points[i].y,
                },
                Point {
                    x: points[j].x,
                    y: points[j].y,
                },
            );
        }
    }
    let bresenham_lines: Vec<Vec<IntegerPoint>> = all_lines
        .into_iter()
        .map(|line| bresenham(line.0, line.1))
        .collect();

    let path = solve(bresenham_lines, &image_bw, n);
    println!("{}", path.len());
    for point_index in path {
        let point = points[point_index];
        println!("{{ {},{} }},", point.x, point.y);
    }
}
