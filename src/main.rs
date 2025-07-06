mod models;
mod utils;

use clap::Parser;
use image::{DynamicImage, ImageBuffer, Rgb};
use quantette::{ColorSpace, ImagePipeline, PaletteSize, QuantizeMethod};
use rayon::prelude::*;
use std::error::Error;

use crate::models::circle::Circle;
use crate::models::circle::{IntegerPoint, Point};
use crate::models::cli::Cli;
use crate::models::image::{Image, ImageDimensions};
use crate::utils::generate_circle::get_circle_points;
use crate::utils::plotter::save_paths_as_svg;
use crate::utils::rasterizer::bresenham;

fn color_distance_sq(c1: Rgb<u8>, c2: Rgb<u8>) -> f64 {
    let r_diff = (c1[0] as f64 - c2[0] as f64).powi(2);
    let g_diff = (c1[1] as f64 - c2[1] as f64).powi(2);
    let b_diff = (c1[2] as f64 - c2[2] as f64).powi(2);
    r_diff + g_diff + b_diff
}

fn get_palette(
    source_image: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    num_colors: usize,
) -> Result<Vec<Rgb<u8>>, Box<dyn Error>> {
    let quantette_image = quantette::image::ImageBuffer::<quantette::image::Rgb<u8>, _>::from_raw(
        source_image.width(),
        source_image.height(),
        source_image.clone().into_raw(),
    )
    .ok_or("Failed to create quantette image")?;

    let mut pipeline = ImagePipeline::try_from(&quantette_image)?;
    let (quantized_palette, _) = pipeline
        .palette_size(PaletteSize::from(num_colors as u8))
        .dither(true)
        .colorspace(ColorSpace::Srgb)
        .quantize_method(QuantizeMethod::kmeans())
        .indexed_palette_par();

    let palette = quantized_palette
        .iter()
        .map(|c| Rgb([c.red, c.green, c.blue]))
        .collect();

    Ok(palette)
}

fn draw_line_on_canvas(
    canvas: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    line: &[IntegerPoint],
    color: Rgb<u8>,
    opacity: f64,
    height: u32,
) {
    for point in line {
        if point.x >= 0
            && point.y >= 0
            && point.x < canvas.width() as i32
            && point.y < height as i32
        {
            let pixel_y = (height as i32 - point.y - 1) as u32;
            let background_pixel = *canvas.get_pixel(point.x as u32, pixel_y);

            let r = (background_pixel[0] as f64 * (1.0 - opacity) + color[0] as f64 * opacity)
                .round() as u8;
            let g = (background_pixel[1] as f64 * (1.0 - opacity) + color[1] as f64 * opacity)
                .round() as u8;
            let b = (background_pixel[2] as f64 * (1.0 - opacity) + color[2] as f64 * opacity)
                .round() as u8;

            canvas.put_pixel(point.x as u32, pixel_y, Rgb([r, g, b]));
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let max_height: u32 = 720;

    let source_image_struct = Image::read_image(args.image_path)?;
    let cropped_image_struct = Image::crop_to_square_from_center(source_image_struct);
    let resized_image_struct = Image::resize_image(
        cropped_image_struct,
        ImageDimensions {
            width: max_height,
            height: max_height,
        },
    );

    let circle = Circle {
        center: IntegerPoint {
            x: max_height as i32 / 2,
            y: max_height as i32 / 2,
        },
        radius: max_height / 2,
    };
    let points = get_circle_points(circle, args.nodes);

    let (source_rgb, palette) = if args.colors <= 1 {
        println!("Processing in black and white mode.");
        let bw_image = Image::to_black_and_white(resized_image_struct);
        let source_rgb = bw_image.image.to_rgb8();
        let palette = vec![Rgb([0, 0, 0])];
        (source_rgb, palette)
    } else {
        println!(
            "Processing in multi-color mode with {} colors.",
            args.colors
        );
        let source_rgb = resized_image_struct.image.to_rgb8();
        let palette = get_palette(&source_rgb, args.colors)?;
        println!("Using colors: {:?}", palette);
        (source_rgb, palette)
    };

    let mut canvas =
        ImageBuffer::<Rgb<u8>, _>::from_pixel(max_height, max_height, Rgb([255, 255, 255]));

    let mut paths: Vec<(Vec<usize>, Rgb<u8>)> =
        palette.iter().map(|&color| (vec![0], color)).collect();

    let line_opacity = 0.16;

    for i in 0..args.max_lines {
        if i % 100 == 0 {
            println!("Finding line {}/{}...", i + 1, args.max_lines);
        }

        let best_move = paths
            .par_iter()
            .enumerate()
            .map(|(color_idx, (path, line_color))| {
                let current_pin_idx = *path.last().unwrap();
                let mut best_score_for_color = -1.0;
                let mut best_next_pin_for_color = current_pin_idx;

                for next_pin_idx in 0..args.nodes {
                    if next_pin_idx == current_pin_idx {
                        continue;
                    }

                    let line = bresenham(points[current_pin_idx], points[next_pin_idx]);
                    let mut current_score = 0.0;

                    for p in &line {
                        if p.x >= 0
                            && p.y >= 0
                            && p.x < max_height as i32
                            && p.y < max_height as i32
                        {
                            let pixel_y = (max_height as i32 - p.y - 1) as u32;
                            let source_pixel = *source_rgb.get_pixel(p.x as u32, pixel_y);
                            let canvas_pixel = *canvas.get_pixel(p.x as u32, pixel_y);

                            let error_before = color_distance_sq(source_pixel, canvas_pixel);

                            let blended_r = (canvas_pixel[0] as f64 * (1.0 - line_opacity)
                                + line_color[0] as f64 * line_opacity)
                                .round() as u8;
                            let blended_g = (canvas_pixel[1] as f64 * (1.0 - line_opacity)
                                + line_color[1] as f64 * line_opacity)
                                .round() as u8;
                            let blended_b = (canvas_pixel[2] as f64 * (1.0 - line_opacity)
                                + line_color[2] as f64 * line_opacity)
                                .round() as u8;
                            let blended_color = Rgb([blended_r, blended_g, blended_b]);

                            let error_after = color_distance_sq(source_pixel, blended_color);

                            current_score += error_before - error_after;
                        }
                    }

                    if current_score > best_score_for_color {
                        best_score_for_color = current_score;
                        best_next_pin_for_color = next_pin_idx;
                    }
                }
                (best_score_for_color, color_idx, best_next_pin_for_color)
            })
            .reduce(|| (-1.0, 0, 0), |a, b| if a.0 > b.0 { a } else { b });

        let (score, best_color_idx, best_next_pin) = best_move;

        if score <= 0.0 {
            println!("No more improvements found. Stopping at line {}.", i);
            break;
        }

        let (path, color) = &mut paths[best_color_idx];
        let start_pin = *path.last().unwrap();

        let line_to_draw = bresenham(points[start_pin], points[best_next_pin]);
        draw_line_on_canvas(&mut canvas, &line_to_draw, *color, line_opacity, max_height);

        path.push(best_next_pin);
    }

    if let Some(output_path) = args.output {
        println!("Saving SVG to {}...", output_path.display());
        let image_dims = ImageDimensions {
            width: max_height,
            height: max_height,
        };
        save_paths_as_svg(&output_path, &image_dims, &points, &paths)?;
        println!("Successfully saved SVG.");
    } else {
        println!("\nNo output path provided. Skipping save.");
        println!("To save the result, run with --output <filename.svg>");
    }

    Ok(())
}
