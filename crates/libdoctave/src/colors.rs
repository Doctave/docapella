// Generate a color palette based on a given HEX color.
//
// # Arguments
// * `color` - Color in HEX format
//
// # Result
// A vector of HEX colors
//
// # Example
// ```
// use libdoctave::colors::get_color_palette;
//
// let palette = get_color_palette("#ff0000");
//
// assert_eq!(palette, vec!["FFECEC", "FFDADA", "FFBFBF", "FF9494", "FF5959", "FF2424", "FF0000", "D30000", "AF0101", "910707", "4C0000"])
// ```

use std::{cmp::Ordering, str::FromStr};

use palette::{
    rgb::{Rgb, Rgba},
    FromColor, Hsl, IntoColor, Srgb,
};

#[allow(dead_code)]
pub fn get_color_palette(color: &str) -> Vec<String> {
    let color = hex_to_hsl(color);
    let factors = color_factors(color);

    factors
        .iter()
        .map(|(l, h, s)| {
            let mut c = color;

            c.lightness += l;
            c.hue += *h;
            c.saturation += s;

            let new_color: Rgba = Srgb::from_color(c).into();

            let r = (new_color.red * 255.) as u32;
            let g = (new_color.green * 255.) as u32;
            let b = (new_color.blue * 255.) as u32;

            let hex = format!("{:02X}", Rgb::<Srgb, u32>::new(r, g, b));

            hex
        })
        .collect::<Vec<_>>()
}

fn find_closest_value(vec: Vec<f32>, target: f32) -> f32 {
    let mut closest = vec[0];
    let mut min_distance = (target - closest).abs();

    for &val in &vec[1..] {
        let distance = (target - val).abs();
        if distance < min_distance {
            min_distance = distance;
            closest = val;
        }
    }

    closest
}

pub fn color_factors(color: Hsl) -> Vec<(f32, f32, f32)> {
    let f = FAMILIES
        .iter()
        .min_by(|f1, f2| {
            if f1.diff(color).abs() > f2.diff(color).abs() {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        })
        .unwrap();

    let c = f.closest_color_to(color);

    f.factors(c)
}

#[derive(Debug)]
struct ColorFamily {
    #[allow(dead_code)]
    title: &'static str,
    c_50: &'static str,
    c_100: &'static str,
    c_200: &'static str,
    c_300: &'static str,
    c_400: &'static str,
    c_500: &'static str,
    c_600: &'static str,
    c_700: &'static str,
    c_800: &'static str,
    c_900: &'static str,
    c_950: &'static str,
}

impl ColorFamily {
    fn closest_color_to(&self, color: Hsl) -> Hsl {
        let closest_l = find_closest_value(self.lightnesses(), color.lightness);
        let colors = self.colors_vec();

        *colors
            .iter()
            .find(|hsl| hsl.lightness == closest_l)
            .unwrap()
    }

    fn diff(&self, color: Hsl) -> f32 {
        let closest_color_by_l = self.closest_color_to(color);

        ((color.lightness - closest_color_by_l.lightness)
            + (color.hue - closest_color_by_l.hue)
            + (color.saturation - closest_color_by_l.saturation))
            .into()
    }

    fn factors(&self, hsl: Hsl) -> Vec<(f32, f32, f32)> {
        self.colors_vec()
            .iter()
            .map(|c| {
                (
                    c.lightness - hsl.lightness,
                    (c.hue - hsl.hue).into(),
                    c.saturation - hsl.saturation,
                )
            })
            .collect()
    }

    fn lightnesses(&self) -> Vec<f32> {
        self.colors_vec().iter().map(|hsl| hsl.lightness).collect()
    }

    fn colors_vec(&self) -> Vec<Hsl> {
        vec![
            hex_to_hsl(self.c_50),
            hex_to_hsl(self.c_100),
            hex_to_hsl(self.c_200),
            hex_to_hsl(self.c_300),
            hex_to_hsl(self.c_400),
            hex_to_hsl(self.c_500),
            hex_to_hsl(self.c_600),
            hex_to_hsl(self.c_700),
            hex_to_hsl(self.c_800),
            hex_to_hsl(self.c_900),
            hex_to_hsl(self.c_950),
        ]
    }
}

pub fn hex_to_hsl(hex: &str) -> Hsl {
    // TODO: Make better. I couldn't quickly find a way to
    // convert hex to HSL so it's first created as RGB. This
    // might cause small inaccuracies.
    let rgb: Rgb<Srgb, u8> = Rgb::from_str(hex).unwrap();

    let r = rgb.red as f32 / 255.;
    let g = rgb.green as f32 / 255.;
    let b = rgb.blue as f32 / 255.;

    Srgb::new(r, g, b).into_color()
}

static FAMILIES: [ColorFamily; 22] = [
    ColorFamily {
        title: "Slate",
        c_50: "#f8fafc",
        c_100: "#f1f5f9",
        c_200: "#e2e8f0",
        c_300: "#cbd5e1",
        c_400: "#94a3b8",
        c_500: "#64748b",
        c_600: "#475569",
        c_700: "#334155",
        c_800: "#1e293b",
        c_900: "#0f172a",
        c_950: "#020617",
    },
    ColorFamily {
        title: "Gray",
        c_50: "#f9fafb",
        c_100: "#f3f4f6",
        c_200: "#e5e7eb",
        c_300: "#d1d5db",
        c_400: "#9ca3af",
        c_500: "#6b7280",
        c_600: "#4b5563",
        c_700: "#374151",
        c_800: "#1f2937",
        c_900: "#111827",
        c_950: "#030712",
    },
    ColorFamily {
        title: "Zinc",
        c_50: "#fafafa",
        c_100: "#f4f4f5",
        c_200: "#e4e4e7",
        c_300: "#d4d4d8",
        c_400: "#a1a1aa",
        c_500: "#71717a",
        c_600: "#52525b",
        c_700: "#3f3f46",
        c_800: "#27272a",
        c_900: "#18181b",
        c_950: "#09090b",
    },
    ColorFamily {
        title: "Neutral",
        c_50: "#fafafa",
        c_100: "#f5f5f5",
        c_200: "#e5e5e5",
        c_300: "#d4d4d4",
        c_400: "#a3a3a3",
        c_500: "#737373",
        c_600: "#525252",
        c_700: "#404040",
        c_800: "#262626",
        c_900: "#171717",
        c_950: "#0a0a0a",
    },
    ColorFamily {
        title: "Stone",
        c_50: "#fafaf9",
        c_100: "#f5f5f4",
        c_200: "#e7e5e4",
        c_300: "#d6d3d1",
        c_400: "#a8a29e",
        c_500: "#78716c",
        c_600: "#57534e",
        c_700: "#44403c",
        c_800: "#292524",
        c_900: "#1c1917",
        c_950: "#0c0a09",
    },
    ColorFamily {
        title: "Red",
        c_50: "#fef2f2",
        c_100: "#fee2e2",
        c_200: "#fecaca",
        c_300: "#fca5a5",
        c_400: "#f87171",
        c_500: "#ef4444",
        c_600: "#dc2626",
        c_700: "#b91c1c",
        c_800: "#991b1b",
        c_900: "#7f1d1d",
        c_950: "#450a0a",
    },
    ColorFamily {
        title: "Orange",
        c_50: "#fff7ed",
        c_100: "#ffedd5",
        c_200: "#fed7aa",
        c_300: "#fdba74",
        c_400: "#fb923c",
        c_500: "#f97316",
        c_600: "#ea580c",
        c_700: "#c2410c",
        c_800: "#9a3412",
        c_900: "#7c2d12",
        c_950: "#431407",
    },
    ColorFamily {
        title: "Amber",
        c_50: "#fffbeb",
        c_100: "#fef3c7",
        c_200: "#fde68a",
        c_300: "#fcd34d",
        c_400: "#fbbf24",
        c_500: "#f59e0b",
        c_600: "#d97706",
        c_700: "#b45309",
        c_800: "#92400e",
        c_900: "#78350f",
        c_950: "#451a03",
    },
    ColorFamily {
        title: "Yellow",
        c_50: "#fefce8",
        c_100: "#fef9c3",
        c_200: "#fef08a",
        c_300: "#fde047",
        c_400: "#facc15",
        c_500: "#eab308",
        c_600: "#ca8a04",
        c_700: "#a16207",
        c_800: "#854d0e",
        c_900: "#713f12",
        c_950: "#422006",
    },
    ColorFamily {
        title: "Lime",
        c_50: "#f7fee7",
        c_100: "#ecfccb",
        c_200: "#d9f99d",
        c_300: "#bef264",
        c_400: "#a3e635",
        c_500: "#84cc16",
        c_600: "#65a30d",
        c_700: "#4d7c0f",
        c_800: "#3f6212",
        c_900: "#365314",
        c_950: "#1a2e05",
    },
    ColorFamily {
        title: "Green",
        c_50: "#f0fdf4",
        c_100: "#dcfce7",
        c_200: "#bbf7d0",
        c_300: "#86efac",
        c_400: "#4ade80",
        c_500: "#22c55e",
        c_600: "#16a34a",
        c_700: "#15803d",
        c_800: "#166534",
        c_900: "#14532d",
        c_950: "#052e16",
    },
    ColorFamily {
        title: "Emerald",
        c_50: "#ecfdf5",
        c_100: "#d1fae5",
        c_200: "#a7f3d0",
        c_300: "#6ee7b7",
        c_400: "#34d399",
        c_500: "#10b981",
        c_600: "#059669",
        c_700: "#047857",
        c_800: "#065f46",
        c_900: "#064e3b",
        c_950: "#022c22",
    },
    ColorFamily {
        title: "Teal",
        c_50: "#f0fdfa",
        c_100: "#ccfbf1",
        c_200: "#99f6e4",
        c_300: "#5eead4",
        c_400: "#2dd4bf",
        c_500: "#14b8a6",
        c_600: "#0d9488",
        c_700: "#0f766e",
        c_800: "#115e59",
        c_900: "#134e4a",
        c_950: "#042f2e",
    },
    ColorFamily {
        title: "Cyan",
        c_50: "#ecfeff",
        c_100: "#cffafe",
        c_200: "#a5f3fc",
        c_300: "#67e8f9",
        c_400: "#22d3ee",
        c_500: "#06b6d4",
        c_600: "#0891b2",
        c_700: "#0e7490",
        c_800: "#155e75",
        c_900: "#164e63",
        c_950: "#083344",
    },
    ColorFamily {
        title: "Sky",
        c_50: "#f0f9ff",
        c_100: "#e0f2fe",
        c_200: "#bae6fd",
        c_300: "#7dd3fc",
        c_400: "#38bdf8",
        c_500: "#0ea5e9",
        c_600: "#0284c7",
        c_700: "#0369a1",
        c_800: "#075985",
        c_900: "#0c4a6e",
        c_950: "#082f49",
    },
    ColorFamily {
        title: "Blue",
        c_50: "#eff6ff",
        c_100: "#dbeafe",
        c_200: "#bfdbfe",
        c_300: "#93c5fd",
        c_400: "#60a5fa",
        c_500: "#3b82f6",
        c_600: "#2563eb",
        c_700: "#1d4ed8",
        c_800: "#1e40af",
        c_900: "#1e3a8a",
        c_950: "#172554",
    },
    ColorFamily {
        title: "Indigo",
        c_50: "#eef2ff",
        c_100: "#e0e7ff",
        c_200: "#c7d2fe",
        c_300: "#a5b4fc",
        c_400: "#818cf8",
        c_500: "#6366f1",
        c_600: "#4f46e5",
        c_700: "#4338ca",
        c_800: "#3730a3",
        c_900: "#312e81",
        c_950: "#1e1b4b",
    },
    ColorFamily {
        title: "Violet",
        c_50: "#f5f3ff",
        c_100: "#ede9fe",
        c_200: "#ddd6fe",
        c_300: "#c4b5fd",
        c_400: "#a78bfa",
        c_500: "#8b5cf6",
        c_600: "#7c3aed",
        c_700: "#6d28d9",
        c_800: "#5b21b6",
        c_900: "#4c1d95",
        c_950: "#2e1065",
    },
    ColorFamily {
        title: "Purple",
        c_50: "#faf5ff",
        c_100: "#f3e8ff",
        c_200: "#e9d5ff",
        c_300: "#d8b4fe",
        c_400: "#c084fc",
        c_500: "#a855f7",
        c_600: "#9333ea",
        c_700: "#7e22ce",
        c_800: "#6b21a8",
        c_900: "#581c87",
        c_950: "#3b0764",
    },
    ColorFamily {
        title: "Fuchsia",
        c_50: "#fdf4ff",
        c_100: "#fae8ff",
        c_200: "#f5d0fe",
        c_300: "#f0abfc",
        c_400: "#e879f9",
        c_500: "#d946ef",
        c_600: "#c026d3",
        c_700: "#a21caf",
        c_800: "#86198f",
        c_900: "#701a75",
        c_950: "#4a044e",
    },
    ColorFamily {
        title: "Pink",
        c_50: "#fdf2f8",
        c_100: "#fce7f3",
        c_200: "#fbcfe8",
        c_300: "#f9a8d4",
        c_400: "#f472b6",
        c_500: "#ec4899",
        c_600: "#db2777",
        c_700: "#be185d",
        c_800: "#9d174d",
        c_900: "#831843",
        c_950: "#500724",
    },
    ColorFamily {
        title: "Rose",
        c_50: "#fff1f2",
        c_100: "#ffe4e6",
        c_200: "#fecdd3",
        c_300: "#fda4af",
        c_400: "#fb7185",
        c_500: "#f43f5e",
        c_600: "#e11d48",
        c_700: "#be123c",
        c_800: "#9f1239",
        c_900: "#881337",
        c_950: "#4c0519",
    },
];
