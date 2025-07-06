use crate::models::circle::Point;
use crate::models::image::ImageDimensions;
use std::path::PathBuf;
use svg::node::element::{Line, Rectangle};
use svg::Document;

pub fn save_path_as_svg(
    file_path: &PathBuf,
    dimensions: &ImageDimensions,
    points: &Vec<Point>,
    path: &Vec<usize>,
) -> Result<(), std::io::Error> {
    let mut document = Document::new().set("viewBox", (0, 0, dimensions.width, dimensions.height));

    let background = Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "white");
    document = document.add(background);

    if path.len() < 2 {
        return svg::save(file_path, &document);
    }

    for i in 0..(path.len() - 1) {
        let start_point_index = path[i];
        let end_point_index = path[i + 1];

        let p1 = points[start_point_index];
        let p2 = points[end_point_index];

        let line = Line::new()
            .set("x1", p1.x)
            .set("y1", dimensions.height as f64 - p1.y)
            .set("x2", p2.x)
            .set("y2", dimensions.height as f64 - p2.y)
            .set("stroke", "black")
            .set("stroke-width", 0.4)
            .set("stroke-opacity", 0.55);

        document = document.add(line);
    }

    svg::save(file_path, &document)
}
