pub mod app_message;
pub mod theme;

use image::GenericImageView;
use rand::Rng;
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

pub fn render_image(area: Rect, buf: &mut Buffer, path: &str, transparent_color: Color) {
    let img = image::open(path).expect("Failed to open image");

    let (w, h) = img.dimensions();

    // 32 x 24 canvas needed
    let width = w as u16;
    let height = h as u16;

    let origin_x = area.left() + (area.width / 2) - (width / 2);
    let origin_y = area.top() + (area.height / 2) - (height / 4);

    let mut cell = &mut buf[(0 + origin_x, 0 + origin_y)];

    for x in 0..width {
        for y in 0..height {
            let rgba = img.get_pixel(x as u32, y as u32).0;

            let c;
            if rgba[3] == 0 {
                c = transparent_color;
            } else {
                c = Color::Rgb(rgba[0], rgba[1], rgba[2]);
            }

            if y % 2 == 0 {
                cell = &mut buf[(x + origin_x, y / 2 + origin_y)];
                cell.set_char('▀');

                cell.fg = c;
            } else {
                cell.bg = c;
            }
        }
        // text.push('\n');
    }
}

pub fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0..=255);
    let g = rng.gen_range(0..=255);
    let b = rng.gen_range(0..=255);

    // Combine into a u32 in the format 0x00RRGGBB
    let color_value = (r << 16) | (g << 8) | b; // 0x00RRGGBB
    Color::from_u32(color_value)
}
