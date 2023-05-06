use std::io::Cursor;

use image::{
    codecs::png::PngEncoder,
    imageops::{self, FilterType},
    ColorType, DynamicImage, EncodableLayout, GenericImageView, ImageBuffer, ImageEncoder, Rgba,
};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

#[derive(Debug, Deserialize, Validate, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chart {
    #[validate(min_items = 1)]
    entries: Vec<ChartEntry>,
    #[validate(minimum = 1)]
    #[validate(maximum = 5)]
    rows: u8,
    #[validate(minimum = 1)]
    #[validate(maximum = 5)]
    cols: u8,
    #[validate(minimum = 100)]
    #[validate(maximum = 500)]
    cover_size: u16,
}

#[derive(Debug, Deserialize, Validate, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartEntry {
    image_url: Option<String>,
    title: String,
    artist: String,
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    rating: Option<u8>,
}

pub async fn create_chart(params: Chart) -> Result<Vec<u8>, anyhow::Error> {
    let width = (params.cols as u32) * (params.cover_size as u32);
    let height = (params.rows as u32) * (params.cover_size as u32);

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = ImageBuffer::new(width, height);

    for (
        i,
        ChartEntry {
            image_url,
            title,
            artist,
            rating,
        },
    ) in params.entries.into_iter().enumerate()
    {
        let x = (i as u32 % (params.cols as u32)) * (params.cover_size as u32);
        let y = (i as u32 / (params.cols as u32)) * (params.cover_size as u32);

        let mut avg_color: Option<Rgba<u8>> = None;
        if let Some(image_url) = image_url {
            let img_bytes = reqwest::get(image_url).await?.bytes().await?;
            let img = image::load_from_memory(&img_bytes)?;

            avg_color = Some(get_average_color(&img));

            let filter = if img.width() < (params.cover_size as u32)
                || img.height() < (params.cover_size as u32)
            {
                FilterType::CatmullRom
            } else {
                FilterType::Lanczos3
            };

            let scaled =
                img.resize_to_fill(params.cover_size as u32, params.cover_size as u32, filter);

            imageops::replace(&mut imgbuf, &scaled, x as i64, y as i64);
        }

        let (text_color, card_color) = match avg_color {
            Some(avg_color) if is_light(&avg_color) => (
                Rgba([0u8, 0u8, 0u8, 255u8]),       // black
                Rgba([255u8, 255u8, 255u8, 128u8]), // white
            ),
            _ => (
                Rgba([255u8, 255u8, 255u8, 255u8]), // black
                Rgba([0u8, 0u8, 0u8, 128u8]),       // white
            ),
        };

        let font = Vec::from(include_bytes!("../res/Inter-VariableFont_slnt,wght.ttf") as &[u8]);
        let font = Font::try_from_vec(font).unwrap();
        let size = get_font_size(16.0, &font, &title, params.cover_size as i32);
        let scale = Scale::uniform(size);
        draw_text_mut(
            &mut imgbuf,
            text_color,
            x as i32,
            y as i32,
            scale,
            &font,
            &title,
        );
    }

    let mut output: Vec<u8> = Vec::new();
    let mut binding = Cursor::new(&mut output);
    let encoder = PngEncoder::new(&mut binding);

    encoder.write_image(
        imgbuf.as_bytes(),
        imgbuf.dimensions().0,
        imgbuf.dimensions().1,
        ColorType::Rgba8,
    )?;

    Ok(output)
}

fn get_average_color(image: &DynamicImage) -> Rgba<u8> {
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;

    let pixels = image.pixels();

    let mut num_pixels = 0;
    for pixel in pixels {
        num_pixels += 1;
        let image::Rgba(data) = pixel.2;
        r += data[0] as u32;
        g += data[1] as u32;
        b += data[2] as u32;
    }

    r /= num_pixels;
    g /= num_pixels;
    b /= num_pixels;

    image::Rgba([r as u8, g as u8, b as u8, 255u8])
}

fn luminance(color: &Rgba<u8>) -> f32 {
    let image::Rgba([r, g, b, _]) = color;
    let r = *r as f32;
    let g = *g as f32;
    let b = *b as f32;

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn is_light(color: &Rgba<u8>) -> bool {
    luminance(color) >= 128.0
}

fn get_font_size(start_size: f32, font: &Font, text: &str, target_width: i32) -> f32 {
    let mut size = start_size;

    loop {
        if size == 0.0 {
            return 0.0;
        }

        let (width, _) = text_size(Scale::uniform(size), font, text);

        if width <= target_width {
            return size;
        }

        if size <= 1.0 {
            size -= 0.1;
        } else {
            size -= 1.0;
        }
    }
}
