use macroquad::prelude::*;
use macroquad::{color::BLACK, shapes::draw_rectangle};
use palette::{rgb::Rgba, FromColor, Hsla, Srgb};

use libdoctave::color_generator::radix::{get_palette, ColorFamily, ColorScale};
use libdoctave::color_generator::stylesheets::get_radix_families;

#[macroquad::main("Color Palette")]
async fn main() {
    request_new_screen_size(1900., 800.);

    let mut family = get_palette("#ffc53d", get_radix_families());

    let mut hex = String::new();

    loop {
        clear_background(BLACK);

        if let Some(char) = get_char_pressed() {
            if char == '\r' {
                if hex.len() == 7 {
                    family = get_palette(&hex, get_radix_families());
                }

                hex.clear();
            } else if char == '\u{7f}' {
                hex.pop();
            } else {
                hex.push(char);
            }
        }

        draw_text("Type a hex color code and press ENTER", 10., 30., 20., WHITE);
        draw_text(&hex, 10., 60., 20., WHITE);

        draw_palette(&family, 100., 0., "Reference color scale:");
        if let Some(gray_family) = family.gray_family.as_deref() {
            draw_palette(gray_family, 100., 200., "Grayscale:");
        }

        next_frame().await
    }
}

fn draw_palette(family: &ColorFamily, x_offset: f32, y_offset: f32, label_prefix: &str) {
    draw_text(&format!("{label_prefix} {:?}", family.ref_family), 50. + x_offset, 80. + y_offset, 20., WHITE);
    draw_relative_to_palette(&family.light, x_offset, 100. + y_offset, false, family.original_color.as_ref());
    draw_relative_to_palette(&family.light_alpha, x_offset, 133. + y_offset, false, family.original_color.as_ref());
    draw_relative_to_palette(&family.dark, x_offset, 166. + y_offset, true, family.original_color.as_ref());
    draw_relative_to_palette(&family.dark_alpha, x_offset, 199. + y_offset, true, family.original_color.as_ref());
}

fn draw(x: f32, y: f32, color: Hsla, invert: bool, original_color: Option<&Hsla>) {
    let new_color: Rgba = Srgb::from_color(color).into();
    let macroquad_color = Color::new(new_color.red, new_color.green, new_color.blue, color.alpha);

    let bg_color = if invert { BLACK } else { WHITE };

    draw_rectangle(x, y, 48.0, 32.0, bg_color);
    draw_rectangle(x, y, 48.0, 32.0, macroquad_color);

    if let Some(original_color) = original_color {
        if original_color == &color {
            draw_rectangle(x, y, 10.0, 10.0, RED);
        }
    }
}

fn draw_relative_to_palette(palette: &ColorScale, x: f32, y: f32, invert: bool, original_color: Option<&Hsla>) {
    draw(49. * 1. + x, y, palette.c_1, invert, original_color);
    draw(49. * 2. + x, y, palette.c_2, invert, original_color);
    draw(49. * 3. + x, y, palette.c_3, invert, original_color);
    draw(49. * 4. + x, y, palette.c_4, invert, original_color);
    draw(49. * 5. + x, y, palette.c_5, invert, original_color);
    draw(49. * 6. + x, y, palette.c_6, invert, original_color);
    draw(49. * 7. + x, y, palette.c_7, invert, original_color);
    draw(49. * 8. + x, y, palette.c_8, invert, original_color);
    draw(49. * 9. + x, y, palette.c_9, invert, original_color);
    draw(49. * 10. + x, y, palette.c_10, invert, original_color);
    draw(49. * 11. + x, y, palette.c_11, invert, original_color);
    draw(49. * 12. + x, y, palette.c_12, invert, original_color);
}
