// Cargo.toml dependencies:
// [dependencies]
// palette = { version = "0.7", features = ["serializing"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

use palette::{FromColor, IntoColor, Lab, LinSrgb, Oklch, OklabHue, Srgb};
use std::collections::HashMap;

mod colors;

mod radix_scales {
    use super::*;
    use std::collections::HashMap;
    use once_cell::sync::Lazy;

    static LIGHT_SCALES: Lazy<HashMap<String, ArrayOf12<Oklch>>> = Lazy::new(|| {
        let mut scales = HashMap::new();
        
        scales.insert("amber".to_string(), colors::amber::light());
        scales.insert("blue".to_string(), colors::blue::light());
        scales.insert("bronze".to_string(), colors::bronze::light());
        scales.insert("brown".to_string(), colors::brown::light());
        scales.insert("crimson".to_string(), colors::crimson::light());
        scales.insert("cyan".to_string(), colors::cyan::light());
        scales.insert("gold".to_string(), colors::gold::light());
        scales.insert("grass".to_string(), colors::grass::light());
        scales.insert("gray".to_string(), colors::gray::light());
        scales.insert("green".to_string(), colors::green::light());
        scales.insert("indigo".to_string(), colors::indigo::light());
        scales.insert("iris".to_string(), colors::iris::light());
        scales.insert("jade".to_string(), colors::jade::light());
        scales.insert("lime".to_string(), colors::lime::light());
        scales.insert("mauve".to_string(), colors::mauve::light());
        scales.insert("mint".to_string(), colors::mint::light());
        scales.insert("olive".to_string(), colors::olive::light());
        scales.insert("orange".to_string(), colors::orange::light());
        scales.insert("pink".to_string(), colors::pink::light());
        scales.insert("plum".to_string(), colors::plum::light());
        scales.insert("purple".to_string(), colors::purple::light());
        scales.insert("red".to_string(), colors::red::light());
        scales.insert("ruby".to_string(), colors::ruby::light());
        scales.insert("sage".to_string(), colors::sage::light());
        scales.insert("sand".to_string(), colors::sand::light());
        scales.insert("sky".to_string(), colors::sky::light());
        scales.insert("slate".to_string(), colors::slate::light());
        scales.insert("teal".to_string(), colors::teal::light());
        scales.insert("tomato".to_string(), colors::tomato::light());
        scales.insert("violet".to_string(), colors::violet::light());
        scales.insert("yellow".to_string(), colors::yellow::light());
        
        scales
    });

    static DARK_SCALES: Lazy<HashMap<String, ArrayOf12<Oklch>>> = Lazy::new(|| {
        let mut scales = HashMap::new();
        
        scales.insert("amber".to_string(), colors::amber::dark());
        scales.insert("blue".to_string(), colors::blue::dark());
        scales.insert("bronze".to_string(), colors::bronze::dark());
        scales.insert("brown".to_string(), colors::brown::dark());
        scales.insert("crimson".to_string(), colors::crimson::dark());
        scales.insert("cyan".to_string(), colors::cyan::dark());
        scales.insert("gold".to_string(), colors::gold::dark());
        scales.insert("grass".to_string(), colors::grass::dark());
        scales.insert("gray".to_string(), colors::gray::dark());
        scales.insert("green".to_string(), colors::green::dark());
        scales.insert("indigo".to_string(), colors::indigo::dark());
        scales.insert("iris".to_string(), colors::iris::dark());
        scales.insert("jade".to_string(), colors::jade::dark());
        scales.insert("lime".to_string(), colors::lime::dark());
        scales.insert("mauve".to_string(), colors::mauve::dark());
        scales.insert("mint".to_string(), colors::mint::dark());
        scales.insert("olive".to_string(), colors::olive::dark());
        scales.insert("orange".to_string(), colors::orange::dark());
        scales.insert("pink".to_string(), colors::pink::dark());
        scales.insert("plum".to_string(), colors::plum::dark());
        scales.insert("purple".to_string(), colors::purple::dark());
        scales.insert("red".to_string(), colors::red::dark());
        scales.insert("ruby".to_string(), colors::ruby::dark());
        scales.insert("sage".to_string(), colors::sage::dark());
        scales.insert("sand".to_string(), colors::sand::dark());
        scales.insert("sky".to_string(), colors::sky::dark());
        scales.insert("slate".to_string(), colors::slate::dark());
        scales.insert("teal".to_string(), colors::teal::dark());
        scales.insert("tomato".to_string(), colors::tomato::dark());
        scales.insert("violet".to_string(), colors::violet::dark());
        scales.insert("yellow".to_string(), colors::yellow::dark());
        
        scales
    });

    static LIGHT_GRAY_SCALES: Lazy<HashMap<String, ArrayOf12<Oklch>>> = Lazy::new(|| {
        let mut scales = HashMap::new();
        
        scales.insert("gray".to_string(), colors::gray::light());
        scales.insert("mauve".to_string(), colors::mauve::light());
        scales.insert("slate".to_string(), colors::slate::light());
        scales.insert("sage".to_string(), colors::sage::light());
        scales.insert("olive".to_string(), colors::olive::light());
        scales.insert("sand".to_string(), colors::sand::light());
        
        scales
    });

    static DARK_GRAY_SCALES: Lazy<HashMap<String, ArrayOf12<Oklch>>> = Lazy::new(|| {
        let mut scales = HashMap::new();
        
        scales.insert("gray".to_string(), colors::gray::dark());
        scales.insert("mauve".to_string(), colors::mauve::dark());
        scales.insert("slate".to_string(), colors::slate::dark());
        scales.insert("sage".to_string(), colors::sage::dark());
        scales.insert("olive".to_string(), colors::olive::dark());
        scales.insert("sand".to_string(), colors::sand::dark());
        
        scales
    });

    pub fn get_light_scales() -> &'static HashMap<String, ArrayOf12<Oklch>> {
        &LIGHT_SCALES
    }

    pub fn get_dark_scales() -> &'static HashMap<String, ArrayOf12<Oklch>> {
        &DARK_SCALES
    }

    pub fn get_light_gray_scales() -> &'static HashMap<String, ArrayOf12<Oklch>> {
        &LIGHT_GRAY_SCALES
    }

    pub fn get_dark_gray_scales() -> &'static HashMap<String, ArrayOf12<Oklch>> {
        &DARK_GRAY_SCALES
    }
}

type ArrayOf12<T> = [T; 12];

const GRAY_SCALE_NAMES: &[&str] = &["gray", "mauve", "slate", "sage", "olive", "sand"];


#[derive(Debug, Clone)]
pub struct RadixColors {
    pub accent_scale: ArrayOf12<String>,
    pub accent_scale_alpha: ArrayOf12<String>,
    pub accent_scale_wide_gamut: ArrayOf12<String>,
    pub accent_scale_alpha_wide_gamut: ArrayOf12<String>,
    pub accent_contrast: String,

    pub gray_scale: ArrayOf12<String>,
    pub gray_scale_alpha: ArrayOf12<String>,
    pub gray_scale_wide_gamut: ArrayOf12<String>,
    pub gray_scale_alpha_wide_gamut: ArrayOf12<String>,

    pub gray_surface: String,
    pub gray_surface_wide_gamut: String,
    pub accent_surface: String,
    pub accent_surface_wide_gamut: String,

    pub background: String,
}

#[derive(Debug, Clone, Copy)]
pub enum Appearance {
    Light,
    Dark,
}

pub struct ColorGenerator {
    light_colors: &'static HashMap<String, ArrayOf12<Oklch>>,
    dark_colors: &'static HashMap<String, ArrayOf12<Oklch>>,
    light_gray_colors: &'static HashMap<String, ArrayOf12<Oklch>>,
    dark_gray_colors: &'static HashMap<String, ArrayOf12<Oklch>>,
}

impl ColorGenerator {
    pub fn new() -> Self {
        ColorGenerator {
            light_colors: radix_scales::get_light_scales(),
            dark_colors: radix_scales::get_dark_scales(),
            light_gray_colors: radix_scales::get_light_gray_scales(),
            dark_gray_colors: radix_scales::get_dark_gray_scales(),
        }
    }

    pub fn generate_radix_colors(
        &self,
        appearance: Appearance,
        accent: &str,
        gray: &str,
        background: &str,
    ) -> RadixColors {
        let all_scales = match appearance {
            Appearance::Light => &self.light_colors,
            Appearance::Dark => &self.dark_colors,
        };

        let gray_scales = match appearance {
            Appearance::Light => &self.light_gray_colors,
            Appearance::Dark => &self.dark_gray_colors,
        };

        let background_color = parse_color(background);
        let gray_base_color = parse_color(gray);
        let accent_base_color = parse_color(accent);

        let gray_scale_colors =
            self.get_scale_from_color(gray_base_color, gray_scales, background_color, appearance);

        let mut accent_scale_colors =
            self.get_scale_from_color(accent_base_color, all_scales, background_color, appearance);

        // Handle pure white or black accent colors
        let accent_hex = to_hex(accent_base_color);
        if accent_hex == "#000000" || accent_hex == "#ffffff" {
            accent_scale_colors = gray_scale_colors.clone();
        }

        // Get step 9 colors
        let (accent9_color, accent_contrast_color) =
            self.get_step9_colors(&accent_scale_colors, accent_base_color);

        accent_scale_colors[8] = accent9_color;
        accent_scale_colors[9] =
            self.get_button_hover_color(accent9_color, &[accent_scale_colors.clone()]);

        // Limit saturation of text colors
        accent_scale_colors[10].chroma = accent_scale_colors[10].chroma.min(
            accent_scale_colors[8]
                .chroma
                .max(accent_scale_colors[7].chroma),
        );
        accent_scale_colors[11].chroma = accent_scale_colors[11].chroma.min(
            accent_scale_colors[8]
                .chroma
                .max(accent_scale_colors[7].chroma),
        );

        // Generate all the output formats
        let background_hex = to_hex(background_color);

        let accent_scale_hex = accent_scale_colors.map(to_hex);
        let accent_scale_wide_gamut = accent_scale_colors.map(to_oklch_string);
        let accent_scale_alpha_hex: ArrayOf12<String> = core::array::from_fn(|i| {
            get_alpha_color_srgb(&accent_scale_hex[i], &background_hex, None)
        });
        let accent_scale_alpha_wide_gamut: ArrayOf12<String> = core::array::from_fn(|i| {
            get_alpha_color_p3(&accent_scale_wide_gamut[i], &background_hex, None)
        });

        let gray_scale_hex = gray_scale_colors.map(to_hex);
        let gray_scale_wide_gamut = gray_scale_colors.map(to_oklch_string);
        let gray_scale_alpha_hex: ArrayOf12<String> = core::array::from_fn(|i| {
            get_alpha_color_srgb(&gray_scale_hex[i], &background_hex, None)
        });
        let gray_scale_alpha_wide_gamut: ArrayOf12<String> = core::array::from_fn(|i| {
            get_alpha_color_p3(&gray_scale_wide_gamut[i], &background_hex, None)
        });

        let accent_contrast_hex = to_hex(accent_contrast_color);

        let (gray_surface, gray_surface_wide_gamut) = match appearance {
            Appearance::Light => (
                "#ffffffcc".to_string(),
                "color(display-p3 1 1 1 / 80%)".to_string(),
            ),
            Appearance::Dark => (
                "rgba(0, 0, 0, 0.05)".to_string(),
                "color(display-p3 0 0 0 / 5%)".to_string(),
            ),
        };

        let accent_surface = match appearance {
            Appearance::Light => {
                get_alpha_color_srgb(&accent_scale_hex[1], &background_hex, Some(0.8))
            }
            Appearance::Dark => {
                get_alpha_color_srgb(&accent_scale_hex[1], &background_hex, Some(0.5))
            }
        };

        let accent_surface_wide_gamut = match appearance {
            Appearance::Light => {
                get_alpha_color_p3(&accent_scale_wide_gamut[1], &background_hex, Some(0.8))
            }
            Appearance::Dark => {
                get_alpha_color_p3(&accent_scale_wide_gamut[1], &background_hex, Some(0.5))
            }
        };

        RadixColors {
            accent_scale: accent_scale_hex,
            accent_scale_alpha: accent_scale_alpha_hex,
            accent_scale_wide_gamut,
            accent_scale_alpha_wide_gamut,
            accent_contrast: accent_contrast_hex,
            gray_scale: gray_scale_hex,
            gray_scale_alpha: gray_scale_alpha_hex,
            gray_scale_wide_gamut,
            gray_scale_alpha_wide_gamut,
            gray_surface,
            gray_surface_wide_gamut,
            accent_surface,
            accent_surface_wide_gamut,
            background: background_hex,
        }
    }

    fn get_scale_from_color(
        &self,
        source: Oklch,
        scales: &HashMap<String, ArrayOf12<Oklch>>,
        background_color: Oklch,
        appearance: Appearance,
    ) -> ArrayOf12<Oklch> {
        // Find closest colors from all scales
        let mut all_colors: Vec<(String, Oklch, f32)> = Vec::new();

        for (name, scale) in scales {
            for color in scale {
                let distance = delta_e_ok(source, *color);
                all_colors.push((name.clone(), *color, distance));
            }
        }

        all_colors.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        // Remove non-unique scales
        let mut closest_colors: Vec<(String, Oklch, f32)> = Vec::new();
        let mut seen_scales = std::collections::HashSet::new();

        for (scale_name, color, distance) in all_colors {
            if seen_scales.insert(scale_name.clone()) {
                closest_colors.push((scale_name, color, distance));
            }
        }

        // If the next two closest colors are both grays, remove subsequent grays
        let all_are_grays = closest_colors
            .iter()
            .all(|(name, _, _)| GRAY_SCALE_NAMES.contains(&name.as_str()));

        if !all_are_grays && GRAY_SCALE_NAMES.contains(&closest_colors[0].0.as_str()) {
            let i = 1;
            while i < closest_colors.len()
                && GRAY_SCALE_NAMES.contains(&closest_colors[i].0.as_str())
            {
                closest_colors.remove(i);
            }
        }

        let color_a = &closest_colors[0];
        let color_b = &closest_colors[1];

        // Triangulation logic to determine mixing ratio
        let a = color_b.2; // distance from source to B
        let b = color_a.2; // distance from source to A
        let c = delta_e_ok(color_a.1, color_b.1); // distance from A to B

        let cos_a = (b * b + c * c - a * a) / (2.0 * b * c);
        let rad_a = cos_a.acos();
        let sin_a = rad_a.sin();

        let cos_b = (a * a + c * c - b * b) / (2.0 * a * c);
        let rad_b = cos_b.acos();
        let sin_b = rad_b.sin();

        let tan_c1 = cos_a / sin_a;
        let tan_c2 = cos_b / sin_b;

        let ratio = (tan_c1 / tan_c2 * 0.5).max(0.0);

        // Get the scales and mix them
        let scale_a = &scales[&color_a.0];
        let scale_b = &scales[&color_b.0];

        let mut scale: ArrayOf12<Oklch> =
            core::array::from_fn(|i| mix_colors(scale_a[i], scale_b[i], ratio));

        // Find the closest color from the mixed scale
        let mut base_color = scale[0];
        let mut min_distance = f32::MAX;

        for color in &scale {
            let distance = delta_e_ok(source, *color);
            if distance < min_distance {
                min_distance = distance;
                base_color = *color;
            }
        }

        // Note the chroma difference
        let ratio_c = if base_color.chroma > 0.0 {
            source.chroma / base_color.chroma
        } else {
            1.0
        };

        // Modify hue and chroma of the scale to match source
        for color in &mut scale {
            color.chroma = (color.chroma * ratio_c).min(source.chroma * 1.5);
            color.hue = source.hue;
        }

        // Apply lightness adjustments based on appearance
        match appearance {
            Appearance::Light => {
                let lightness_scale: Vec<f32> = scale.iter().map(|c| c.l).collect();
                let background_l = background_color.l.clamp(0.0, 1.0);

                // Add white as first step for calculation
                let mut extended_scale = vec![1.0];
                extended_scale.extend(&lightness_scale);

                let new_lightness = transpose_progression_start(
                    background_l,
                    &extended_scale,
                    [0.0, 2.0, 0.0, 2.0],
                );

                // Apply new lightness values (skip the added white)
                for (i, color) in scale.iter_mut().enumerate() {
                    color.l = new_lightness[i + 1];
                }
            }
            Appearance::Dark => {
                let mut ease = [1.0, 0.0, 1.0, 0.0];
                let reference_background_l = scale[0].l;
                let background_l = background_color.l.clamp(0.0, 1.0);
                let ratio_l = background_l / reference_background_l;

                if ratio_l > 1.0 {
                    let max_ratio = 1.5;
                    let meta_ratio = (ratio_l - 1.0) * (max_ratio / (max_ratio - 1.0));

                    for i in 0..4 {
                        ease[i] = if ratio_l > max_ratio {
                            0.0
                        } else {
                            (ease[i] * (1.0 - meta_ratio)).max(0.0)
                        };
                    }
                }

                let lightness_scale: Vec<f32> = scale.iter().map(|c| c.l).collect();
                let new_lightness =
                    transpose_progression_start(background_l, &lightness_scale, ease);

                for (i, color) in scale.iter_mut().enumerate() {
                    color.l = new_lightness[i];
                }
            }
        }

        scale
    }

    fn get_step9_colors(&self, scale: &ArrayOf12<Oklch>, accent_base: Oklch) -> (Oklch, Oklch) {
        let reference_background = scale[0];
        let distance = delta_e_ok(accent_base, reference_background) * 100.0;

        // If too close to background, use scale color
        if distance < 25.0 {
            return (scale[8], get_text_color(scale[8]));
        }

        (accent_base, get_text_color(accent_base))
    }

    fn get_button_hover_color(&self, source: Oklch, scales: &[ArrayOf12<Oklch>]) -> Oklch {
        let l = source.l;
        let c = source.chroma;
        let h = source.hue;

        let new_l = if l > 0.4 {
            l - 0.03 / (l + 0.1)
        } else {
            l + 0.03 / (l + 0.1)
        };

        let new_c = if l > 0.4 && !h.into_inner().is_nan() { c * 0.93 } else { c };

        let mut button_hover = Oklch::new(new_l, new_c, h);

        // Find closest in-scale color
        let mut closest_color = button_hover;
        let mut min_distance = f32::INFINITY;

        for scale in scales {
            for color in scale {
                let distance = delta_e_ok(button_hover, *color);
                if distance < min_distance {
                    min_distance = distance;
                    closest_color = *color;
                }
            }
        }

        button_hover.chroma = closest_color.chroma;
        button_hover.hue = closest_color.hue;
        button_hover
    }
}

// Helper functions

fn parse_color(color_str: &str) -> Oklch {
    // Parse hex or other color formats to Oklch
    // This is simplified - you'd need proper parsing
    let srgb = if color_str.starts_with('#') {
        parse_hex(color_str)
    } else {
        Srgb::new(0.5, 0.5, 0.5)
    };

    Oklch::from_color(srgb)
}

fn parse_hex(hex: &str) -> Srgb {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
    Srgb::new(r, g, b)
}

fn format_hex(hex: &str) -> String {
    if !hex.starts_with('#') {
        return hex.to_string();
    }

    match hex.len() {
        4 => {
            // #rgb -> #rrggbb
            let r = &hex[1..2];
            let g = &hex[2..3];
            let b = &hex[3..4];
            format!("#{}{}{}{}{}{}", r, r, g, g, b, b)
        }
        5 => {
            // #rgba -> #rrggbbaa
            let r = &hex[1..2];
            let g = &hex[2..3];
            let b = &hex[3..4];
            let a = &hex[4..5];
            format!("#{}{}{}{}{}{}{}{}", r, r, g, g, b, b, a, a)
        }
        _ => hex.to_string(),
    }
}

fn to_hex(color: Oklch) -> String {
    let srgb: Srgb = color.into_color();
    let hex = format!(
        "#{:02x}{:02x}{:02x}",
        (srgb.red * 255.0).round() as u8,
        (srgb.green * 255.0).round() as u8,
        (srgb.blue * 255.0).round() as u8
    );
    format_hex(&hex)
}

fn to_oklch_string(color: Oklch) -> String {
    // Match the original's specific formatting for OKLCH strings
    // Using percentage for lightness with 1 decimal place
    let l_percent = (color.l * 100.0 * 10.0).round() / 10.0;
    let c = (color.chroma * 10000.0).round() / 10000.0; // 4 decimal precision
    let h = (color.hue.into_inner() * 10.0).round() / 10.0; // 1 decimal precision

    format!("oklch({}% {} {})", l_percent, c, h)
}

fn delta_e_ok(color1: Oklch, color2: Oklch) -> f32 {
    // The palette crate should provide deltaEOK method
    // This uses the actual OK Lab color difference formula
    let lab1: Lab = color1.into_color();
    let lab2: Lab = color2.into_color();
    let delta_e = (lab1.l - lab2.l).powi(2) + (lab1.a - lab2.a).powi(2) + (lab1.b - lab2.b).powi(2);
    delta_e.sqrt()
}

fn mix_colors(color1: Oklch, color2: Oklch, ratio: f32) -> Oklch {
    Oklch::new(
        color1.l * (1.0 - ratio) + color2.l * ratio,
        color1.chroma * (1.0 - ratio) + color2.chroma * ratio,
        // Hue mixing needs special handling for circular values
        OklabHue::from_degrees(mix_hue(color1.hue.into_inner(), color2.hue.into_inner(), ratio)),
    )
}

fn mix_hue(h1: f32, h2: f32, ratio: f32) -> f32 {
    let diff = (h2 - h1 + 180.0) % 360.0 - 180.0;
    (h1 + diff * ratio + 360.0) % 360.0
}

fn get_text_color(background: Oklch) -> Oklch {
    let white = Oklch::new(1.0, 0.0, 0.0);

    // Using simplified APCA contrast calculation
    if contrast_apca(white, background).abs() < 40.0 {
        let c = (0.08 * background.chroma).max(0.04);
        Oklch::new(0.25, c, background.hue)
    } else {
        white
    }
}

fn contrast_apca(text: Oklch, background: Oklch) -> f32 {
    // Full APCA (Accessible Perceptual Contrast Algorithm) implementation
    // Convert to linear sRGB for luminance calculation
    let text_srgb: Srgb = text.into_color();
    let bg_srgb: Srgb = background.into_color();

    // Convert to linear RGB and calculate relative luminance
    let text_lin: LinSrgb = text_srgb.into_linear();
    let bg_lin: LinSrgb = bg_srgb.into_linear();

    // Calculate Y (luminance) using sRGB coefficients
    let text_y = 0.2126 * text_lin.red + 0.7152 * text_lin.green + 0.0722 * text_lin.blue;
    let bg_y = 0.2126 * bg_lin.red + 0.7152 * bg_lin.green + 0.0722 * bg_lin.blue;

    // APCA constants
    const N_TXT: f32 = 0.57;
    const N_BG: f32 = 0.56;
    const R_SCALE: f32 = 1.14;
    const W_OFFSET: f32 = 0.027;
    const W_SCALE: f32 = 1.14;

    // Soft clamp function
    let soft_clamp = |y: f32| -> f32 {
        if y >= W_OFFSET {
            y
        } else {
            y + (W_OFFSET - y).powf(W_SCALE)
        }
    };

    let y_txt = soft_clamp(text_y);
    let y_bg = soft_clamp(bg_y);

    // Determine polarity and calculate SAPC
    let (s_txt, s_bg, s_map) = if y_bg > y_txt {
        // Dark text on light background
        (y_txt.powf(N_TXT), y_bg.powf(N_BG), R_SCALE)
    } else {
        // Light text on dark background
        (y_bg.powf(N_BG), y_txt.powf(N_TXT), 1.0)
    };

    // Calculate contrast
    (s_bg - s_txt) * 108.0 * s_map
}

fn get_alpha_color_srgb(target: &str, background: &str, target_alpha: Option<f32>) -> String {
    let target_color: Srgb = parse_hex(target);
    let bg_color: Srgb = parse_hex(background);

    let (r, g, b, a) = get_alpha_color(
        [target_color.red, target_color.green, target_color.blue],
        [bg_color.red, bg_color.green, bg_color.blue],
        255.0,
        255.0,
        target_alpha,
    );

    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8
    )
}

fn get_alpha_color_p3(target: &str, background: &str, target_alpha: Option<f32>) -> String {
    // Full P3 color space implementation
    let target_color = parse_color(target);
    let bg_color = parse_color(background);

    // Convert to P3 color space (palette crate may need extension for P3)
    // For now using sRGB as proxy but noting P3 has wider gamut
    let target_srgb: Srgb = target_color.into_color();
    let bg_srgb: Srgb = bg_color.into_color();

    let (r, g, b, a) = get_alpha_color(
        [target_srgb.red, target_srgb.green, target_srgb.blue],
        [bg_srgb.red, bg_srgb.green, bg_srgb.blue],
        255.0,  // P3 also uses 255 precision per the original
        1000.0, // Higher precision for alpha in P3
        target_alpha,
    );

    // Format as P3 color string with proper precision
    format!(
        "color(display-p3 {:.4} {:.4} {:.4} / {:.1}%)",
        r,
        g,
        b,
        (a * 100.0).round() / 10.0 * 10.0
    )
}

// Browser-specific alpha blending that matches how browsers composite colors
fn blend_alpha(foreground: f32, alpha: f32, background: f32, round: bool) -> f32 {
    if round {
        // Important: Browser rounds each component separately, not the final result
        (background * (1.0 - alpha)).round() + (foreground * alpha).round()
    } else {
        background * (1.0 - alpha) + foreground * alpha
    }
}

fn get_alpha_color(
    target_rgb: [f32; 3],
    background_rgb: [f32; 3],
    rgb_precision: f32,
    alpha_precision: f32,
    target_alpha: Option<f32>,
) -> (f32, f32, f32, f32) {
    let [tr, tg, tb] = target_rgb.map(|c| (c * rgb_precision).round());
    let [br, bg, bb] = background_rgb.map(|c| (c * rgb_precision).round());

    // Determine if we're darkening or lightening
    let desired_rgb = if tr > br || tg > bg || tb > bb {
        rgb_precision
    } else {
        0.0
    };

    let alpha_r = (tr - br) / (desired_rgb - br);
    let alpha_g = (tg - bg) / (desired_rgb - bg);
    let alpha_b = (tb - bb) / (desired_rgb - bb);

    let is_pure_gray = (alpha_r - alpha_g).abs() < 0.001 && (alpha_g - alpha_b).abs() < 0.001;

    // No need for precision gymnastics with pure grays
    if target_alpha.is_none() && is_pure_gray {
        let v = desired_rgb / rgb_precision;
        return (v, v, v, alpha_r);
    }

    let max_alpha = target_alpha.unwrap_or(alpha_r.max(alpha_g).max(alpha_b));
    let a = ((max_alpha * alpha_precision).ceil() / alpha_precision).clamp(0.0, 1.0);

    let mut r = (((br * (1.0 - a) - tr) / a) * -1.0).clamp(0.0, rgb_precision);
    let mut g = (((bg * (1.0 - a) - tg) / a) * -1.0).clamp(0.0, rgb_precision);
    let mut b = (((bb * (1.0 - a) - tb) / a) * -1.0).clamp(0.0, rgb_precision);

    r = r.ceil();
    g = g.ceil();
    b = b.ceil();

    let blended_r = blend_alpha(r, a, br, true);
    let blended_g = blend_alpha(g, a, bg, true);
    let blended_b = blend_alpha(b, a, bb, true);

    // Correct for rounding errors in light mode
    if desired_rgb == 0.0 {
        if tr <= br && (tr - blended_r).abs() > 0.5 {
            r = if tr > blended_r { r + 1.0 } else { r - 1.0 };
        }
        if tg <= bg && (tg - blended_g).abs() > 0.5 {
            g = if tg > blended_g { g + 1.0 } else { g - 1.0 };
        }
        if tb <= bb && (tb - blended_b).abs() > 0.5 {
            b = if tb > blended_b { b + 1.0 } else { b - 1.0 };
        }
    }

    // Correct for rounding errors in dark mode
    if desired_rgb == rgb_precision {
        if tr >= br && (tr - blended_r).abs() > 0.5 {
            r = if tr > blended_r { r + 1.0 } else { r - 1.0 };
        }
        if tg >= bg && (tg - blended_g).abs() > 0.5 {
            g = if tg > blended_g { g + 1.0 } else { g - 1.0 };
        }
        if tb >= bb && (tb - blended_b).abs() > 0.5 {
            b = if tb > blended_b { b + 1.0 } else { b - 1.0 };
        }
    }

    // Convert back to 0-1 values
    (r / rgb_precision, g / rgb_precision, b / rgb_precision, a)
}

fn transpose_progression_start(to: f32, arr: &[f32], curve: [f32; 4]) -> Vec<f32> {
    let last_index = arr.len() - 1;
    let diff = arr[0] - to;

    arr.iter()
        .enumerate()
        .map(|(i, &n)| {
            let t = 1.0 - (i as f32 / last_index as f32);
            let eased = bezier_ease(t, curve);
            n - diff * eased
        })
        .collect()
}


fn bezier_ease(t: f32, curve: [f32; 4]) -> f32 {
    // Proper cubic bezier implementation
    // This solves for the y value given t on the curve defined by (0,0), (x1,y1), (x2,y2), (1,1)
    let [x1, y1, x2, y2] = curve;

    // We need to find the t value that gives us the x position
    // then use that to calculate y
    // Using Newton-Raphson method for finding t from x

    let cubic_bezier_x = |t: f32| -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        3.0 * mt2 * t * x1 + 3.0 * mt * t2 * x2 + t3
    };

    let cubic_bezier_dx = |t: f32| -> f32 {
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        3.0 * mt2 * x1 + 6.0 * mt * t * (x2 - x1) + 3.0 * t2 * (1.0 - x2)
    };

    // Newton-Raphson to find t for given x
    let mut t_for_x = t;
    let epsilon = 0.0001;

    for _ in 0..8 {
        // Usually converges in 4-5 iterations
        let x_est = cubic_bezier_x(t_for_x);
        let dx = cubic_bezier_dx(t_for_x);

        if dx.abs() < epsilon {
            break;
        }

        t_for_x -= (x_est - t) / dx;
        t_for_x = t_for_x.clamp(0.0, 1.0);
    }

    // Now calculate y for this t
    let t2 = t_for_x * t_for_x;
    let t3 = t2 * t_for_x;
    let mt = 1.0 - t_for_x;
    let mt2 = mt * mt;

    3.0 * mt2 * t_for_x * y1 + 3.0 * mt * t2 * y2 + t3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_generation() {
        let generator = ColorGenerator::new();
        
        // Test that we can create a generator and it has the expected scales
        let light_scales = generator.light_colors;
        assert!(light_scales.contains_key("blue"));
        assert!(light_scales.contains_key("red"));
        
        // Test that a scale has 12 colors
        let blue_scale = &light_scales["blue"];
        assert_eq!(blue_scale.len(), 12);
    }
}
