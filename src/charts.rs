use std::io::Cursor;

use image::{codecs::png::PngEncoder, ColorType, EncodableLayout, ImageEncoder, ImageError};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

#[derive(Debug, Deserialize, Validate, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chart {
    #[validate(min_items = 1)]
    entries: Vec<ChartEntry>,
    #[validate(minimum = 1)]
    #[validate(maximum = 5)]
    rows: u32,
    #[validate(minimum = 1)]
    #[validate(maximum = 5)]
    cols: u32,
    #[validate(minimum = 100)]
    #[validate(maximum = 500)]
    cover_size: u32,
}

#[derive(Debug, Deserialize, Validate, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartEntry {
    image_url: Option<String>,
    title: String,
    artist: String,
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    rating: Option<u32>,
}

pub fn create_chart(params: Chart) -> Result<Vec<u8>, ImageError> {
    let width = params.cols * params.cover_size;
    let height = params.rows * params.cover_size;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(width, height);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.3 * x as f32) as u8;
        let b = (0.3 * y as f32) as u8;
        *pixel = image::Rgb([r, 0, b]);
    }

    let mut output: Vec<u8> = Vec::new();
    let mut binding = Cursor::new(&mut output);
    let encoder = PngEncoder::new(&mut binding);

    encoder.write_image(
        imgbuf.as_bytes(),
        imgbuf.dimensions().0,
        imgbuf.dimensions().1,
        ColorType::Rgb8,
    )?;

    Ok(output)
}
