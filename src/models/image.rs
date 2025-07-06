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
    pub fn read_image(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let image = ImageReader::open(&path)?.decode()?;
        let (width, height) = image.dimensions();
        let dimensions = ImageDimensions { height, width };
        Ok(Self { image, dimensions })
    }

    pub fn crop_to_square_from_center(image: Self) -> Self {
        let (width, height) = (image.dimensions.width, image.dimensions.height);

        if width == height {
            return image;
        }

        let side_length = width.min(height);
        let crop_x = if width > height {
            (width - height) / 2
        } else {
            0
        };
        let crop_y = if height > width {
            (height - width) / 2
        } else {
            0
        };

        let cropped_dyn_image = image
            .image
            .crop_imm(crop_x, crop_y, side_length, side_length);

        Self {
            image: cropped_dyn_image,
            dimensions: ImageDimensions {
                width: side_length,
                height: side_length,
            },
        }
    }

    pub fn save_image(self: Self, path: &PathBuf) {
        let _ = self.image.save(path);
    }

    pub fn resize_image(image: Self, new_size: ImageDimensions) -> Self {
        let new_image = image
            .image
            .resize(new_size.width, new_size.height, CatmullRom);
        let width = new_image.width();
        let height = new_image.height();
        Self {
            image: new_image,
            dimensions: ImageDimensions {
                width: width,
                height: height,
            },
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
