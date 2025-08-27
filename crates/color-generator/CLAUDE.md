# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

The `color-generator` crate is a Rust library that generates Radix-compatible color scales from custom input colors. It implements sophisticated color theory algorithms to create cohesive design system palettes that maintain accessibility and visual consistency across both light and dark themes.

## Core Architecture

### Main Components

- **ColorGenerator**: The primary interface that orchestrates color scale generation using pre-loaded Radix color scales as reference data
- **RadixColors**: Output struct containing all generated color formats (hex, alpha variants, wide-gamut P3)
- **Color Processing Pipeline**: Multi-step process involving color space conversion, scale interpolation, and accessibility optimization

### Key Color Science Features

- **OKLCH Color Space**: All internal calculations use OKLCH for perceptually uniform color operations
- **Radix Scale Interpolation**: Uses triangulation algorithms to blend between closest reference scales
- **Delta-E Color Matching**: Implements OK Lab color difference calculations for accurate color similarity
- **APCA Contrast**: Advanced Perceptual Contrast Algorithm for accessibility compliance
- **Multi-format Output**: Generates sRGB hex, P3 wide-gamut, and alpha-blended variants

### Dependencies

- `palette = "0.7.6"`: Core color space conversions and color theory operations

## Development Commands

### Building
```bash
cargo build                    # Build the library
cargo build --release          # Release build
```

### Testing
```bash
cargo test                     # Run all tests
```

### Code Quality
```bash
cargo check                    # Fast syntax checking
cargo clippy                   # Linting
cargo fmt                      # Format code
```

## Key Implementation Details

### Color Scale Generation Process

1. **Reference Scale Loading**: Pre-computed Radix scales are loaded (currently unimplemented placeholders)
2. **Source Color Analysis**: Input colors are parsed and converted to OKLCH color space
3. **Scale Matching**: Uses Delta-E calculations to find the two closest Radix scales
4. **Triangulation Mixing**: Calculates optimal blend ratio between reference scales using geometric triangulation
5. **Hue/Chroma Adjustment**: Applies source color's hue and chroma characteristics to the mixed scale
6. **Lightness Progression**: Applies appearance-specific lightness adjustments using cubic bezier easing
7. **Accessibility Optimization**: Ensures proper contrast ratios and text color generation

### Critical Algorithms

- **`get_scale_from_color()`**: Core scale generation logic with triangulation-based mixing
- **`transpose_progression_start()`**: Lightness curve adjustment for appearance modes
- **`get_alpha_color()`**: Browser-accurate alpha blending calculations
- **`contrast_apca()`**: APCA contrast calculation for accessibility
- **`bezier_ease()`**: Cubic bezier implementation for smooth color transitions

### Current Status

The crate contains a comprehensive implementation framework but has unimplemented placeholder functions in the `radix_scales` module. The actual Radix color data needs to be loaded from external sources or hardcoded.

## Testing Strategy

- Basic smoke test exists (`it_works()` - needs implementation)
- Color generation accuracy testing needed
- Accessibility compliance verification required
- Cross-appearance mode consistency testing