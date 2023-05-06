use std::{io::Cursor, vec};

use image::{
    codecs::png::PngEncoder,
    imageops::{self, FilterType},
    ColorType, DynamicImage, EncodableLayout, GenericImageView, ImageBuffer, ImageEncoder, Rgba,
};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut, text_size, Blend, Canvas},
    rect::Rect,
};
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

type Image = Blend<ImageBuffer<Pixel, Vec<u8>>>;
type Pixel = Rgba<u8>;

pub async fn create_chart(params: Chart) -> Result<Vec<u8>, anyhow::Error> {
    let width = (params.cols as u32) * (params.cover_size as u32);
    let height = (params.rows as u32) * (params.cover_size as u32);

    let outer_margin = (params.cover_size as f32 * 0.025) as u32;
    let inner_margin = (params.cover_size as f32 * 0.025) as u32;

    let max_card_width = params.cover_size as u32 - 2 * outer_margin;
    let max_card_width_inner = max_card_width - 2 * inner_margin;
    let card_corner_radius = (params.cover_size as f32 * 0.02) as u32;

    let line_spacing = (params.cover_size as f32 * 0.01) as u32;

    let num_displayed_covers = params.rows * params.cols;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = Blend(ImageBuffer::new(width, height));

    for (
        i,
        ChartEntry {
            image_url,
            title,
            artist,
            rating,
        },
    ) in params
        .entries
        .into_iter()
        .take(num_displayed_covers as usize)
        .enumerate()
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

            imageops::replace(&mut imgbuf.0, &scaled, x as i64, y as i64);
        }

        let (text_color, card_color) = match avg_color {
            Some(avg_color) if is_light(&avg_color) => (
                Rgba([0u8, 0u8, 0u8, 255u8]),       // black
                Rgba([255u8, 255u8, 255u8, 127u8]), // white
            ),
            _ => (
                Rgba([255u8, 255u8, 255u8, 255u8]), // black
                Rgba([0u8, 0u8, 0u8, 127u8]),       // white
            ),
        };

        let font_reg = Vec::from(include_bytes!("../res/Inter-Regular.ttf") as &[u8]);
        let font_reg =
            Font::try_from_vec(font_reg).ok_or(anyhow::anyhow!("Failed to load regular font"))?;

        let font_bold = Vec::from(include_bytes!("../res/Inter-Bold.ttf") as &[u8]);
        let font_bold =
            Font::try_from_vec(font_bold).ok_or(anyhow::anyhow!("Failed to load bold font"))?;

        let lines = {
            let mut lines = vec![artist, title];
            if let Some(rating) = rating.and_then(rating_to_string) {
                lines.push(rating.to_owned());
            }
            lines
        };

        let max_font_size = (params.cover_size as f32) * 0.053333;
        let (calculated_lines, max_text_width) = {
            let mut max_width = 0;
            let calculated_lines = lines
                .into_iter()
                .enumerate()
                .map(|(i, line)| {
                    let is_artist = i == 0;
                    let font = if is_artist { &font_bold } else { &font_reg };
                    let (font_size, (width, height)) =
                        get_font_size(max_font_size, &font, &line, max_card_width_inner);
                    max_width = max_width.max(width);
                    (line, font, font_size, (width, height))
                })
                .collect::<Vec<_>>();

            (calculated_lines, max_width)
        };

        let total_text_height = calculated_lines
            .iter()
            .map(|(_, _, _, (_, h))| h)
            .sum::<u32>()
            + (calculated_lines.len() as u32 - 1) * line_spacing;
        let card_height = total_text_height + 2 * inner_margin;
        let card_width = max_text_width + 2 * inner_margin;

        let card_x = x + (params.cover_size as u32 - card_width) / 2;

        draw_rounded_rect_mut(
            &mut imgbuf,
            card_x,
            y + params.cover_size as u32 - outer_margin - card_height,
            card_width,
            card_height,
            card_corner_radius,
            card_color,
        );

        let mut drawn_height = 0;
        for (line, font, font_size, (width, height)) in calculated_lines.into_iter().rev() {
            let x = x + (params.cover_size as u32 - width) / 2;
            let y = y + params.cover_size as u32
                - outer_margin
                - inner_margin
                - height
                - drawn_height
                - 1;

            drawn_height += height + line_spacing;

            draw_text_mut(
                &mut imgbuf,
                text_color,
                x as i32,
                y as i32,
                Scale::uniform(font_size),
                font,
                &line,
            );
        }
    }

    let mut output: Vec<u8> = Vec::new();
    let mut binding = Cursor::new(&mut output);
    let encoder = PngEncoder::new(&mut binding);

    encoder.write_image(
        imgbuf.0.as_bytes(),
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

fn get_font_size(start_size: f32, font: &Font, text: &str, target_width: u32) -> (f32, (u32, u32)) {
    let mut size = start_size;

    loop {
        if size == 0.0 {
            return (0.0, (0, 0));
        }

        let (width, height) = text_size(Scale::uniform(size), font, text);

        if width as u32 <= target_width {
            return (size, (width as u32, height as u32));
        }

        if size <= 1.0 {
            size -= 0.1;
        } else {
            size -= 1.0;
        }
    }
}

fn draw_rounded_rect_mut(
    canvas: &mut Image,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    radius: u32,
    color: Pixel,
) {
    // full middle rect
    draw_filled_rect_mut(
        canvas,
        Rect::at(x as i32, (y + radius + 1) as i32).of_size(width, height - (2 * radius) - 1),
        color,
    );
    // top rect
    draw_filled_rect_mut(
        canvas,
        Rect::at((x + radius + 1) as i32, y as i32).of_size(width - (2 * radius) - 2, radius + 1),
        color,
    );
    // bottom rect
    draw_filled_rect_mut(
        canvas,
        Rect::at((x + radius + 1) as i32, (y + height - radius) as i32)
            .of_size(width - (2 * radius) - 2, radius + 1),
        color,
    );

    // corners
    draw_filled_circle_part_mut(
        canvas,
        (x + radius, y + radius),
        radius,
        color,
        CirclePart::TopLeft,
    );
    draw_filled_circle_part_mut(
        canvas,
        (x + width - radius - 1, y + radius),
        radius,
        color,
        CirclePart::TopRight,
    );
    draw_filled_circle_part_mut(
        canvas,
        (x + radius, y + height - radius),
        radius,
        color,
        CirclePart::BottomLeft,
    );
    draw_filled_circle_part_mut(
        canvas,
        (x + width - radius - 1, y + height - radius),
        radius,
        color,
        CirclePart::BottomRight,
    );
}

enum CirclePart {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
fn draw_filled_circle_part_mut(
    canvas: &mut Image,
    center: (u32, u32),
    radius_: u32,
    color: Pixel,
    part: CirclePart,
) {
    let radius = radius_ + 1;
    let radius_fl = radius as f32;

    for y in 0..radius + 1 {
        for x in 0..radius + 1 {
            let dx = (radius - x) as f32;
            let dy = (radius - y) as f32;
            let dist = (dx * dx + dy * dy).sqrt();

            // point lies outside circle
            if dist - radius_fl > 1.0 {
                continue;
            }

            // edge threshold
            if radius_fl / dist < 0.9 {
                continue;
            }

            let (x, y) = match part {
                CirclePart::TopLeft => (center.0 - radius + x, center.1 - radius + y),
                CirclePart::TopRight => (center.0 + radius - x, center.1 - radius + y),
                CirclePart::BottomLeft => (center.0 - radius + x, center.1 + radius - y),
                CirclePart::BottomRight => (center.0 + radius - x, center.1 + radius - y),
            };

            let antialiased_alpha = (radius as f32 - dist).clamp(0.0, 1.0);
            let original_alpha = color[3] as f32 / 255.0;
            let alpha = antialiased_alpha * original_alpha;
            let alpha_color = Rgba([color[0], color[1], color[2], (alpha * 255.0) as u8]);

            canvas.draw_pixel(x, y, alpha_color);
        }
    }
}

fn rating_to_string(rating: u8) -> Option<&'static str> {
    match rating {
        1 => Some("½"),
        2 => Some("★"),
        3 => Some("★½"),
        4 => Some("★★"),
        5 => Some("★★½"),
        6 => Some("★★★"),
        7 => Some("★★★½"),
        8 => Some("★★★★"),
        9 => Some("★★★★½"),
        10 => Some("★★★★★"),
        _ => None,
    }
}
