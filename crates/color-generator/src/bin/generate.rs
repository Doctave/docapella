use color_generator::{ColorGenerator, Appearance};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <accent-color> [gray-color]", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} \"#3b82f6\"", args[0]);
        eprintln!("  {} \"#3b82f6\" \"#6b7280\"", args[0]);
        eprintln!("  {} \"#10b981\" \"#64748b\"", args[0]);
        std::process::exit(1);
    }
    
    let accent_color = &args[1];
    let gray_color = if args.len() == 3 {
        &args[2]
    } else {
        "#6b7280" // Default gray
    };
    
    // Validate hex color formats
    for (name, color) in [("accent", accent_color.as_str()), ("gray", gray_color)] {
        if !color.starts_with('#') || (color.len() != 7 && color.len() != 4) {
            eprintln!("Error: Invalid {} color format '{}'. Use #RRGGBB or #RGB", name, color);
            std::process::exit(1);
        }
    }
    
    // Create the color generator
    let generator = ColorGenerator::new();
    
    // Generate palettes for both light and dark modes
    if gray_color == "#6b7280" {
        println!("🎨 Generating color palette for: {}", accent_color);
    } else {
        println!("🎨 Generating color palette for accent: {} with gray: {}", accent_color, gray_color);
    }
    println!();
    
    // Light mode
    println!("🌅 LIGHT MODE");
    let light_palette = generator.generate_radix_colors(
        Appearance::Light,
        accent_color,   // accent color
        gray_color,     // gray color  
        "#ffffff",      // background
    );
    
    print_palette(&light_palette, "Light");
    
    println!();
    
    // Dark mode
    println!("🌙 DARK MODE");
    let dark_palette = generator.generate_radix_colors(
        Appearance::Dark,
        accent_color,   // accent color
        gray_color,     // gray color
        "#0f0f0f",      // background
    );
    
    print_palette(&dark_palette, "Dark");
}

fn print_palette(palette: &color_generator::RadixColors, mode: &str) {
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
    println!("    Accent Contrast: {} {}", color_swatch(&palette.accent_contrast), palette.accent_contrast);
    println!("    Background:      {} {}", color_swatch(&palette.background), palette.background);
    println!("    Gray Surface:    {} {}", color_swatch(&palette.gray_surface), palette.gray_surface);
    println!("    Accent Surface:  {} {}", color_swatch(&palette.accent_surface), palette.accent_surface);
    
    if mode == "Light" {
        println!("  Wide Gamut (P3) Preview:");
        println!("    Accent Step 9:   {} {}", color_swatch_from_oklch(), palette.accent_scale_wide_gamut[8]);
        println!("    Gray Step 12:    {} {}", color_swatch_from_oklch(), palette.gray_scale_wide_gamut[11]);
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