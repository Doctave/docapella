use palette::{color_difference::Ciede2000, convert::FromColorUnclamped, rgb::Rgba, Hsla, Lab};

pub fn hex_to_rgba(hex: String) -> Rgba {
    let c = csscolorparser::parse(&hex).unwrap();

    Rgba::new(
        c.r as f32 * 255.,
        c.g as f32 * 255.,
        c.b as f32 * 255.,
        c.a as f32,
    )
}

pub fn rgba_to_hex(color: Rgba) -> String {
    let new_color = csscolorparser::Color::new(
        color.red as f64 / 255.,
        color.green as f64 / 255.,
        color.blue as f64 / 255.,
        color.alpha as f64,
    );

    new_color.to_hex_string()
}

pub fn hex_to_hsla(hex: String) -> Result<Hsla, String> {
    let c = csscolorparser::parse(&hex)
        .map(|c| c.to_hsla())
        .map_err(|_| format!("Invalid color `{}` found", hex))?;

    Ok(Hsla::new(c.0 as f32, c.1 as f32, c.2 as f32, c.3 as f32))
}

pub fn hsla_to_hex(color: Hsla) -> String {
    let new_color = csscolorparser::Color::from_hsla(
        color.hue.into_inner().into(),
        color.saturation.into(),
        color.lightness.into(),
        color.alpha.into(),
    );

    new_color.to_hex_string()
}

/// Converts an RGB color to RGBA with the lowest alpha value against a white/black background.
///
/// Alpha conversion is slightly inaccurate, but the human eye can't really tell the difference.
/// It probably suffers from f32 precision issues. Could spend more time on it if we need it.
///
/// TODO: clean this up more...
pub fn alpha_convert(hex: &str, background: Rgba) -> Result<Hsla, String> {
    fn blend_alpha(foreground: f32, alpha: f32, background: f32, round: bool) -> f32 {
        if round {
            (background * (1. - alpha)).round() + (foreground * alpha).round()
        } else {
            background * (1. - alpha) + foreground * alpha
        }
    }

    let alpha_precision = 255.;
    let rgb_precision = 255.;

    let target = hex_to_rgba(hex.to_string());

    let tr = target.red;
    let tg = target.green;
    let tb = target.blue;

    let br = background.red;
    let bg = background.green;
    let bb = background.blue;

    // Is the background color lighter, RGB-wise, than target color?
    // Decide whether we want to add as little color or as much color as possible,
    // darkening or lightening the background respectively.
    // If at least one of the bits of the target RGB value
    // is lighter than the background, we want to lighten it.
    let desired_rgb: f32 = if tr > br || tg > bg || tb > bb {
        rgb_precision
    } else {
        0.
    };

    let alpha_r = (tr - br) / (desired_rgb - br);
    let alpha_g = (tg - bg) / (desired_rgb - bg);
    let alpha_b = (tb - bb) / (desired_rgb - bb);

    let clamp_rgb = |n: f32| -> f32 {
        if n.is_nan() {
            0.
        } else {
            n.min(rgb_precision).max(0.)
        }
    };

    let clamp_a = |n: f32| -> f32 {
        if n.is_nan() {
            0.
        } else {
            n.min(alpha_precision).max(0.)
        }
    };

    let max_alpha = alpha_r.max(alpha_g.max(alpha_b));

    let a = clamp_a((max_alpha * alpha_precision).ceil()) / alpha_precision;
    let mut r = clamp_rgb(((br * (1. - a) - tr) / a) * -1.);
    let mut g = clamp_rgb(((bg * (1. - a) - tg) / a) * -1.);
    let mut b = clamp_rgb(((bb * (1. - a) - tb) / a) * -1.);

    r = r.ceil();
    g = g.ceil();
    b = b.ceil();

    let blended_r = blend_alpha(r, a, br, false);
    let blended_g = blend_alpha(g, a, bg, false);
    let blended_b = blend_alpha(b, a, bb, false);

    if desired_rgb == 0. {
        if tr <= br && tr != blended_r {
            r = if tr > blended_r { r + 1. } else { r - 1. };
        }
        if tg <= bg && tg != blended_g {
            g = if tg > blended_g { g + 1. } else { g - 1. };
        }
        if tb <= bb && tb != blended_b {
            b = if tb > blended_b { b + 1. } else { b - 1. };
        }
    }

    if desired_rgb == rgb_precision {
        if tr >= br && tr != blended_r {
            r = if tr > blended_r { r + 1. } else { r - 1. };
        }
        if tg >= bg && tg != blended_g {
            g = if tg > blended_g { g + 1. } else { g - 1. };
        }
        if tb >= bb && tb != blended_b {
            b = if tb > blended_b { b + 1. } else { b - 1. };
        }
    }

    let hex = rgba_to_hex(Rgba::new(r, g, b, a));
    hex_to_hsla(hex)
}

pub fn color_diff(color: &Hsla, color2: &Hsla) -> f32 {
    let lab = Lab::from_color_unclamped(*color);
    let lab2 = Lab::from_color_unclamped(*color2);

    Ciede2000::difference(lab, lab2)
}
