use image::{
    imageops::{dither, BiLevel, FilterType::CatmullRom},
    io::Reader as ImageReader,
    DynamicImage, GenericImageView,
};
use std::path::PathBuf;

pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

pub struct Image {
    pub image: image::DynamicImage,
    pub dimensions: ImageDimensions,
}

impl Image {
    pub fn read_image(path: PathBuf) -> Self {
        let image = ImageReader::open(&path)
            .expect("Failed to open image file")
            .decode()
            .expect("Failed to decode image");
        let (width, height) = image.dimensions();
        let dimensions = ImageDimensions { height, width };
        Self { image, dimensions }
    }

    pub fn resize_image(image: Self, new_size: ImageDimensions) -> Self {
        let new_image = image
            .image
            .resize(new_size.width, new_size.height, CatmullRom);
        Self {
            image: new_image,
            dimensions: new_size,
        }
    }

    pub fn to_black_and_white(image: Self) -> Self {
        let mut grayscale_image = image.image.to_luma8();
        dither(&mut grayscale_image, &BiLevel);
        Self {
            image: DynamicImage::ImageLuma8(grayscale_image),
            dimensions: image.dimensions,
        }
    }
}
