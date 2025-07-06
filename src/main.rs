mod models;
mod utils;

use std::collections::HashSet;

use clap::Parser;
use image::DynamicImage;

use crate::models::circle::Circle;
use crate::models::circle::{IntegerPoint, Point};
use crate::models::cli::Cli;
use crate::models::image::{Image, ImageDimensions};
use crate::utils::generate_circle::get_circle_points;
use crate::utils::plotter::save_path_as_svg;
use crate::utils::rasterizer::bresenham;

fn point_to_index(n: usize, i: usize, j: usize) -> usize {
    if i < j {
        i * (n - 1) - i * (i + 1) / 2 + j - 1
    } else {
        j * (n - 1) - j * (j + 1) / 2 + i - 1
    }
}

fn get_line_darkness(line: &Vec<IntegerPoint>, image: &Image) -> u32 {
    let mut darkness: u32 = 0;
    if let DynamicImage::ImageLuma8(luma_image) = &image.image {
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
            let pixel_y = (image.dimensions.height as i32 - y - 1) as u32;
            let pixel_value = luma_image.get_pixel(x as u32, pixel_y)[0];
            darkness += 255 - pixel_value as u32;
        }
    }
    darkness
}

fn lighten_image_with_line(image: &mut Image, line: &Vec<IntegerPoint>, lighten_amount: u8) {
    if let DynamicImage::ImageLuma8(luma_image) = &mut image.image {
        for point in line {
            let x = point.x;
            let y = point.y;
            if x >= 0
                && y >= 0
                && x < image.dimensions.width as i32
                && y < image.dimensions.height as i32
            {
                let pixel_y = (image.dimensions.height as i32 - y - 1) as u32;
                let current_pixel = luma_image.get_pixel_mut(x as u32, pixel_y);
                current_pixel[0] = current_pixel[0].saturating_add(lighten_amount);
            }
        }
    }
}

fn solve(
    points: &Vec<Point>,
    image: &mut Image,
    n: usize,
    max_lines: usize,
    lighten_amount: u8,
) -> Vec<usize> {
    let mut current_point_index: usize = 0;
    let mut path: Vec<usize> = Vec::new();
    let mut visited_lines: HashSet<usize> = HashSet::new();
    path.push(current_point_index);

    for _ in 0..max_lines {
        let mut best_line_darkness: u32 = 0;
        let mut best_next_point: usize = current_point_index;

        for j in 0..n {
            if current_point_index == j {
                continue;
            }

            let line_id = point_to_index(n, current_point_index, j);
            if visited_lines.contains(&line_id) {
                continue;
            }

            let line = bresenham(points[current_point_index], points[j]);
            let darkness = get_line_darkness(&line, image);

            if darkness > best_line_darkness {
                best_line_darkness = darkness;
                best_next_point = j;
            }
        }

        if current_point_index == best_next_point {
            break;
        }

        let final_line = bresenham(points[current_point_index], points[best_next_point]);
        lighten_image_with_line(image, &final_line, lighten_amount);

        let line_id = point_to_index(n, current_point_index, best_next_point);
        visited_lines.insert(line_id);

        current_point_index = best_next_point;
        path.push(best_next_point);
    }
    path
}

fn main() {
    let args = Cli::parse();
    let image: Image = Image::read_image(args.image_path);
    let max_height: u32 = 1500;
    let resized_image: Image = Image::resize_image(
        image,
        ImageDimensions {
            width: max_height,
            height: max_height,
        },
    );
    let mut image_bw = Image::to_black_and_white(resized_image);
    let n: usize = 1024;
    let max_lines = 10 * n;
    let lighten_amount: u8 = 16;

    let circle = Circle {
        center: IntegerPoint {
            x: max_height as i32 / 2,
            y: max_height as i32 / 2,
        },
        radius: max_height / 2,
    };
    let points = get_circle_points(circle, n);

    let path = solve(&points, &mut image_bw, n, max_lines, lighten_amount);
    println!("Path with {} lines found.", path.len());

    if let Some(output_path) = args.output {
        println!("Saving SVG to {}...", output_path.display());
        let image_dims = ImageDimensions {
            width: max_height,
            height: max_height,
        };
        match save_path_as_svg(&output_path, &image_dims, &points, &path) {
            Ok(_) => println!("Successfully saved SVG."),
            Err(e) => eprintln!("Error saving SVG: {}", e),
        }
    } else {
        println!("\nNo output path provided. Skipping save.");
        println!("To save the result, run with --output <filename.svg>");
    }
}
