use color_generator::{Appearance, ColorGenerator};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments and flags
    let mut accent_color = "";
    let mut gray_scale = "gray"; // Default gray scale name
    let mut show_css = false;
    let mut color_name = "accent";

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--css" => show_css = true,
            "--name" => {
                i += 1;
                if i < args.len() {
                    color_name = &args[i];
                } else {
                    eprintln!("Error: --name requires a value");
                    std::process::exit(1);
                }
            }
            arg if arg.starts_with('#') => {
                accent_color = arg;
            }
            arg if ["gray", "mauve", "slate", "sage", "olive", "sand"].contains(&arg) => {
                gray_scale = arg;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if accent_color.is_empty() {
        eprintln!(
            "Usage: {} <accent-color> [gray-scale] [--css] [--name <color-name>]",
            args[0]
        );
        eprintln!("Examples:");
        eprintln!("  {} \"#3b82f6\"", args[0]);
        eprintln!("  {} \"#3b82f6\" sand", args[0]);
        eprintln!("  {} \"#10b981\" slate --css", args[0]);
        eprintln!("  {} \"#3b82f6\" --css --name blue", args[0]);
        eprintln!("");
        eprintln!("Gray scale options: gray, mauve, slate, sage, olive, sand (default: gray)");
        eprintln!("");
        eprintln!("Flags:");
        eprintln!("  --css              Output raw CSS instead of color preview");
        eprintln!("  --name <name>      Set the CSS variable prefix (default: accent)");
        std::process::exit(1);
    }

    // Validate accent color format
    if !accent_color.starts_with('#') || (accent_color.len() != 7 && accent_color.len() != 4) {
        eprintln!(
            "Error: Invalid accent color format '{}'. Use #RRGGBB or #RGB",
            accent_color
        );
        std::process::exit(1);
    }

    // Create the color generator
    let generator = ColorGenerator::new();

    if show_css {
        // Output CSS for both light and dark modes
        let light_palette =
            generator.generate_scale(Appearance::Light, accent_color, gray_scale, "#ffffff");

        let dark_palette =
            generator.generate_scale(Appearance::Dark, accent_color, gray_scale, "#0f0f0f");

        // Generate CSS for light mode
        let light_css = light_palette.generate_css(color_name, ":root, .light, .light-theme");
        let dark_css = dark_palette.generate_css(color_name, ".dark, .dark-theme");

        println!("{}", light_css);
        println!("{}", dark_css);
    } else {
        // Generate palettes for both light and dark modes (original behavior)
        if gray_scale == "gray" {
            println!("🎨 Generating color palette for: {}", accent_color);
        } else {
            println!(
                "🎨 Generating color palette for accent: {} with gray scale: {}",
                accent_color, gray_scale
            );
        }
        println!();

        // Light mode
        println!("🌅 LIGHT MODE");
        let light_palette =
            generator.generate_scale(Appearance::Light, accent_color, gray_scale, "#ffffff");

        print_palette(&light_palette, "Light");

        println!();

        // Dark mode
        println!("🌙 DARK MODE");
        let dark_palette =
            generator.generate_scale(Appearance::Dark, accent_color, gray_scale, "#0f0f0f");

        print_palette(&dark_palette, "Dark");
    }
}

fn print_palette(palette: &color_generator::Scale, mode: &str) {
    println!("  Accent Scale ({}):", mode);
    for (i, color) in palette.accent_scale.iter().enumerate() {
        println!("    Step {:<2}: {} {}", i + 1, color_swatch(color), color);
    }

    println!("  Accent Scale Alpha ({}):", mode);
    for (i, color) in palette.accent_scale_alpha.iter().enumerate() {
        println!("    Step {:<2}: {} {}", i + 1, color_swatch(color), color);
    }

    println!("  Gray Scale ({}):", mode);
    for (i, color) in palette.gray_scale.iter().enumerate() {
        println!("    Step {:<2}: {} {}", i + 1, color_swatch(color), color);
    }

    println!("  Special Colors ({}):", mode);
    println!(
        "    Accent Contrast: {} {}",
        color_swatch(&palette.accent_contrast),
        palette.accent_contrast
    );
    println!(
        "    Background:      {} {}",
        color_swatch(&palette.background),
        palette.background
    );
    println!(
        "    Gray Surface:    {} {}",
        color_swatch(&palette.gray_surface),
        palette.gray_surface
    );
    println!(
        "    Accent Surface:  {} {}",
        color_swatch(&palette.accent_surface),
        palette.accent_surface
    );

    if mode == "Light" {
        println!("  Wide Gamut (P3) Preview:");
        println!(
            "    Accent Step 9:   {} {}",
            color_swatch_from_oklch(),
            palette.accent_scale_wide_gamut[8]
        );
        println!(
            "    Gray Step 12:    {} {}",
            color_swatch_from_oklch(),
            palette.gray_scale_wide_gamut[11]
        );
    }
}

fn color_swatch(hex_color: &str) -> String {
    // Parse hex color (handles both #RRGGBB and #RRGGBBAA formats)
    let hex = hex_color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    // Use ANSI 24-bit RGB escape codes with block characters
    format!("\x1b[38;2;{};{};{}m██\x1b[0m", r, g, b)
}

fn color_swatch_from_oklch() -> String {
    // For OKLCH strings, we'll just show a placeholder since parsing would be complex
    // In a real implementation, you'd convert OKLCH back to RGB
    "🎨".to_string()
}
