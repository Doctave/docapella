// Cargo.toml dependencies:
// [dependencies]
// palette = { version = "0.7", features = ["serializing"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

use palette::{FromColor, IntoColor, Lab, LinSrgb, OklabHue, Oklch, Srgb};
use std::collections::HashMap;

pub mod colors;

mod radix_scales {
    use super::*;
    use once_cell::sync::Lazy;
    use std::collections::HashMap;

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
pub struct Scale {
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

impl Scale {
    pub fn generate_css(&self, color_name: &str, theme_selector: &str) -> String {
        let mut css = String::new();

        // Regular hex values
        css.push_str(&format!("{} {{\n", theme_selector));

        // Accent scale (1-12)
        for (i, color) in self.accent_scale.iter().enumerate() {
            css.push_str(&format!("  --{}-{}: {};\n", color_name, i + 1, color));
        }
        css.push_str("\n");

        // Accent alpha scale (a1-a12)
        for (i, color) in self.accent_scale_alpha.iter().enumerate() {
            css.push_str(&format!("  --{}-a{}: {};\n", color_name, i + 1, color));
        }
        css.push_str("\n");

        // Gray scale (1-12)
        for (i, color) in self.gray_scale.iter().enumerate() {
            css.push_str(&format!("  --gray-{}: {};\n", i + 1, color));
        }
        css.push_str("\n");

        // Gray alpha scale (a1-a12)
        for (i, color) in self.gray_scale_alpha.iter().enumerate() {
            css.push_str(&format!("  --gray-a{}: {};\n", i + 1, color));
        }
        css.push_str("\n");

        // Accent special colors
        css.push_str(&format!(
            "  --{}-contrast: {};\n",
            color_name, self.accent_contrast
        ));
        css.push_str(&format!(
            "  --{}-surface: {};\n",
            color_name, self.accent_surface
        ));
        css.push_str(&format!(
            "  --{}-indicator: {};\n",
            color_name, self.accent_scale[8]
        )); // step 9
        css.push_str(&format!(
            "  --{}-track: {};\n",
            color_name, self.accent_scale[8]
        )); // step 9

        // Gray special colors
        css.push_str(&format!("  --gray-surface: {};\n", self.gray_surface));

        // Background
        css.push_str(&format!("  --background: {};\n", self.background));

        css.push_str("}\n\n");

        // P3 wide-gamut support
        css.push_str("@supports (color: color(display-p3 1 1 1)) {\n");
        css.push_str("  @media (color-gamut: p3) {\n");
        css.push_str(&format!("    {} {{\n", theme_selector));

        // P3 accent scale (1-12)
        for (i, color) in self.accent_scale_wide_gamut.iter().enumerate() {
            css.push_str(&format!("      --{}-{}: {};\n", color_name, i + 1, color));
        }
        css.push_str("\n");

        // P3 accent alpha scale (a1-a12)
        for (i, color) in self.accent_scale_alpha_wide_gamut.iter().enumerate() {
            css.push_str(&format!("      --{}-a{}: {};\n", color_name, i + 1, color));
        }
        css.push_str("\n");

        // P3 gray scale (1-12)
        for (i, color) in self.gray_scale_wide_gamut.iter().enumerate() {
            css.push_str(&format!("      --gray-{}: {};\n", i + 1, color));
        }
        css.push_str("\n");

        // P3 gray alpha scale (a1-a12)
        for (i, color) in self.gray_scale_alpha_wide_gamut.iter().enumerate() {
            css.push_str(&format!("      --gray-a{}: {};\n", i + 1, color));
        }
        css.push_str("\n");

        // P3 accent special colors
        css.push_str(&format!(
            "      --{}-contrast: {};\n",
            color_name, self.accent_contrast
        ));
        css.push_str(&format!(
            "      --{}-surface: {};\n",
            color_name, self.accent_surface_wide_gamut
        ));
        css.push_str(&format!(
            "      --{}-indicator: {};\n",
            color_name, self.accent_scale_wide_gamut[8]
        )); // step 9
        css.push_str(&format!(
            "      --{}-track: {};\n",
            color_name, self.accent_scale_wide_gamut[8]
        )); // step 9

        // P3 gray special colors
        css.push_str(&format!(
            "      --gray-surface: {};\n",
            self.gray_surface_wide_gamut
        ));

        css.push_str("    }\n");
        css.push_str("  }\n");
        css.push_str("}\n");

        css
    }
}

impl Default for ColorGenerator {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn generate_scale(
        &self,
        appearance: Appearance,
        accent: &str,
        gray: &str,
        background: &str,
    ) -> Scale {
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
            accent_scale_colors = gray_scale_colors;
        }

        // Get step 9 colors
        let (accent9_color, accent_contrast_color) =
            self.get_step9_colors(&accent_scale_colors, accent_base_color);

        accent_scale_colors[8] = accent9_color;
        accent_scale_colors[9] = self.get_button_hover_color(accent9_color, &[accent_scale_colors]);

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

        Scale {
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

                    #[allow(clippy::needless_range_loop)]
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

        let new_c = if l > 0.4 && !h.into_inner().is_nan() {
            c * 0.93
        } else {
            c
        };

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

fn parse_oklch_string(oklch_str: &str) -> Oklch {
    // Parse OKLCH string like "oklch(62% 0.0915 230.7)"
    let inner = oklch_str.trim_start_matches("oklch(").trim_end_matches(')');
    let parts: Vec<&str> = inner.split_whitespace().collect();

    if parts.len() != 3 {
        return Oklch::new(0.5, 0.0, 0.0);
    }

    let l = parts[0]
        .trim_end_matches('%')
        .parse::<f32>()
        .unwrap_or(50.0)
        / 100.0;
    let c = parts[1].parse::<f32>().unwrap_or(0.0);
    let h = parts[2].parse::<f32>().unwrap_or(0.0);

    Oklch::new(l, c, OklabHue::from_degrees(h))
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
    // Match the original's rounding precision exactly
    let r = (srgb.red * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (srgb.green * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (srgb.blue * 255.0).round().clamp(0.0, 255.0) as u8;

    let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
    format_hex(&hex)
}

fn to_oklch_string(color: Oklch) -> String {
    // Match the original's specific formatting for OKLCH strings
    // Using percentage for lightness with 1 decimal place
    let l_percent = (color.l * 100.0 * 10.0).round() / 10.0;
    let c = (color.chroma * 10000.0).round() / 10000.0; // 4 decimal precision
    let h = (color.hue.into_inner() * 10.0).round() / 10.0; // 1 decimal precision

    // Match exact format from original (no unnecessary decimals)
    if c == 0.0 {
        format!("oklch({}% 0 {})", l_percent, h)
    } else if c.fract() == 0.0 {
        format!("oklch({}% {} {})", l_percent, c as i32, h)
    } else {
        format!("oklch({}% {} {})", l_percent, c, h)
    }
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
        OklabHue::from_degrees(mix_hue(
            color1.hue.into_inner(),
            color2.hue.into_inner(),
            ratio,
        )),
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
    // Parse colors properly - handle both hex and OKLCH input
    let target_srgb = if target.starts_with("oklch(") {
        parse_oklch_string(target).into_color()
    } else {
        parse_hex(target)
    };
    let bg_srgb = parse_hex(background);

    let (r, g, b, a) = get_alpha_color(
        [target_srgb.red, target_srgb.green, target_srgb.blue],
        [bg_srgb.red, bg_srgb.green, bg_srgb.blue],
        255.0,  // P3 also uses 255 precision per the original
        1000.0, // Higher precision for alpha in P3
        target_alpha,
    );

    // Format as P3 color string with proper precision and percentage format
    format!("color(display-p3 {:.4} {:.4} {:.4} / {:.3})", r, g, b, a)
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

    #[test]
    fn test_css_generation() {
        // Create a sample Scale struct for testing
        let colors = Scale {
            accent_scale: [
                "#f9fcfd".to_string(),
                "#f3f8fb".to_string(),
                "#e2f3fc".to_string(),
                "#d2ecfa".to_string(),
                "#c1e3f5".to_string(),
                "#aed8ee".to_string(),
                "#95c9e3".to_string(),
                "#6bb3d6".to_string(),
                "#4490b3".to_string(),
                "#3683a6".to_string(),
                "#257597".to_string(),
                "#193644".to_string(),
            ],
            accent_scale_alpha: [
                "#2aa9d406".to_string(),
                "#157fbf0c".to_string(),
                "#089eed1d".to_string(),
                "#0599e82d".to_string(),
                "#048fd93e".to_string(),
                "#0387cc51".to_string(),
                "#027fbe6a".to_string(),
                "#007cb994".to_string(),
                "#006898bb".to_string(),
                "#00628ec9".to_string(),
                "#005e85da".to_string(),
                "#002030e6".to_string(),
            ],
            accent_scale_wide_gamut: [
                "oklch(98.9% 0.0031 230.7)".to_string(),
                "oklch(97.7% 0.0072 230.7)".to_string(),
                "oklch(95.4% 0.0213 230.7)".to_string(),
                "oklch(92.8% 0.0331 230.7)".to_string(),
                "oklch(89.7% 0.0431 230.7)".to_string(),
                "oklch(86% 0.0529 230.7)".to_string(),
                "oklch(80.8% 0.0655 230.7)".to_string(),
                "oklch(73.3% 0.0881 230.7)".to_string(),
                "oklch(62% 0.0915 230.7)".to_string(),
                "oklch(57.8% 0.0915 230.7)".to_string(),
                "oklch(53% 0.0915 230.7)".to_string(),
                "oklch(31.7% 0.0432 230.7)".to_string(),
            ],
            accent_scale_alpha_wide_gamut: [
                "color(display-p3 0.0157 0.5059 0.7529 / 0.016)".to_string(),
                "color(display-p3 0.0157 0.4078 0.702 / 0.04)".to_string(),
                "color(display-p3 0.0078 0.5216 0.8784 / 0.099)".to_string(),
                "color(display-p3 0.0039 0.5137 0.8471 / 0.154)".to_string(),
                "color(display-p3 0.0039 0.4824 0.7961 / 0.213)".to_string(),
                "color(display-p3 0.0039 0.4588 0.749 / 0.284)".to_string(),
                "color(display-p3 0.0039 0.4275 0.6784 / 0.371)".to_string(),
                "color(display-p3 0 0.4078 0.6588 / 0.512)".to_string(),
                "color(display-p3 0 0.3255 0.5294 / 0.654)".to_string(),
                "color(display-p3 0 0.302 0.4902 / 0.705)".to_string(),
                "color(display-p3 0 0.2784 0.451 / 0.76)".to_string(),
                "color(display-p3 0 0.0941 0.1569 / 0.875)".to_string(),
            ],
            accent_contrast: "#fff".to_string(),
            gray_scale: core::array::from_fn(|_| "#000000".to_string()),
            gray_scale_alpha: core::array::from_fn(|_| "#00000000".to_string()),
            gray_scale_wide_gamut: core::array::from_fn(|_| "oklch(0% 0 0)".to_string()),
            gray_scale_alpha_wide_gamut: core::array::from_fn(|_| {
                "color(display-p3 0 0 0 / 0)".to_string()
            }),
            gray_surface: "#ffffffcc".to_string(),
            gray_surface_wide_gamut: "color(display-p3 1 1 1 / 80%)".to_string(),
            accent_surface: "#f0f6facc".to_string(),
            accent_surface_wide_gamut: "color(display-p3 0.9451 0.9647 0.9804 / 0.8)".to_string(),
            background: "#ffffff".to_string(),
        };

        let css = colors.generate_css("blue", ":root, .light, .light-theme");

        // Test that the CSS contains expected elements
        assert!(css.contains("--blue-1: #f9fcfd;"));
        assert!(css.contains("--blue-a1: #2aa9d406;"));
        assert!(css.contains("--blue-contrast: #fff;"));
        assert!(css.contains("--blue-surface: #f0f6facc;"));

        // Test grayscale variables are generated
        assert!(css.contains("--gray-1: #000000;"));
        assert!(css.contains("--gray-a1: #00000000;"));
        assert!(css.contains("--gray-surface: #ffffffcc;"));
        assert!(css.contains("--background: #ffffff;"));

        assert!(css.contains("@supports (color: color(display-p3 1 1 1))"));
        assert!(css.contains("oklch(98.9% 0.0031 230.7)"));
        assert!(css.contains("color(display-p3 0.0157 0.5059 0.7529 / 0.016)"));
    }
}
