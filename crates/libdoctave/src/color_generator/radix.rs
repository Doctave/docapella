use std::cmp::Ordering;

use palette::{rgb::Rgba, Hsla};

use crate::color_generator::{
    color_utils::{alpha_convert, hex_to_hsla, hsla_to_hex},
    stylesheets::{GRAY, MAUVE, OLIVE, SAGE, SAND, SLATE},
};

use super::color_utils::color_diff;

#[derive(Debug, PartialEq, Clone)]
pub struct ColorFamily {
    pub title: RadixFamilyName,
    pub light: ColorScale,
    pub dark: ColorScale,
    pub light_alpha: ColorScale,
    pub dark_alpha: ColorScale,
    pub gray_family: Option<Box<ColorFamily>>,
    pub ref_family: RadixFamilyName,
    pub original_color: Option<Hsla>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RadixFamilyName {
    Amber,
    Blue,
    Bronze,
    Brown,
    Crimson,
    Cyan,
    Gold,
    Grass,
    Green,
    Indigo,
    Iris,
    Jade,
    Lime,
    Mint,
    Orange,
    Pink,
    Plum,
    Purple,
    Red,
    Ruby,
    Sky,
    Teal,
    Tomato,
    Violet,
    Yellow,

    // grays
    Gray,
    Mauve,
    Olive,
    Sage,
    Sand,
    Slate,

    // this is a generated family
    Custom,
}

impl std::fmt::Display for RadixFamilyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match &self {
            RadixFamilyName::Amber => "amber",
            RadixFamilyName::Blue => "blue",
            RadixFamilyName::Bronze => "bronze",
            RadixFamilyName::Brown => "brown",
            RadixFamilyName::Crimson => "crimson",
            RadixFamilyName::Cyan => "cyan",
            RadixFamilyName::Gold => "gold",
            RadixFamilyName::Grass => "grass",
            RadixFamilyName::Green => "green",
            RadixFamilyName::Indigo => "indigo",
            RadixFamilyName::Iris => "iris",
            RadixFamilyName::Jade => "jade",
            RadixFamilyName::Lime => "lime",
            RadixFamilyName::Mint => "mint",
            RadixFamilyName::Orange => "orange",
            RadixFamilyName::Pink => "pink",
            RadixFamilyName::Plum => "plum",
            RadixFamilyName::Purple => "purple",
            RadixFamilyName::Red => "red",
            RadixFamilyName::Ruby => "ruby",
            RadixFamilyName::Sky => "sky",
            RadixFamilyName::Teal => "teal",
            RadixFamilyName::Tomato => "tomato",
            RadixFamilyName::Violet => "violet",
            RadixFamilyName::Yellow => "yellow",
            RadixFamilyName::Gray => "gray",
            RadixFamilyName::Mauve => "mauve",
            RadixFamilyName::Olive => "olive",
            RadixFamilyName::Sage => "sage",
            RadixFamilyName::Sand => "sand",
            RadixFamilyName::Slate => "slate",
            RadixFamilyName::Custom => "custom",
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct ColorScale {
    pub c_1: Hsla,
    pub c_2: Hsla,
    pub c_3: Hsla,
    pub c_4: Hsla,
    pub c_5: Hsla,
    pub c_6: Hsla,
    pub c_7: Hsla,
    pub c_8: Hsla,
    pub c_9: Hsla,
    pub c_10: Hsla,
    pub c_11: Hsla,
    pub c_12: Hsla,
}

impl ColorScale {
    pub fn as_vec(&self) -> Vec<&Hsla> {
        vec![
            &self.c_1, &self.c_2, &self.c_3, &self.c_4, &self.c_5, &self.c_6, &self.c_7, &self.c_8,
            &self.c_9, &self.c_10, &self.c_11, &self.c_12,
        ]
    }
}

pub fn hsla_alpha_conversion(hsla: Hsla, background: Rgba) -> Result<Hsla, String> {
    let hex = hsla_to_hex(hsla);

    alpha_convert(&hex, background)
}

pub fn get_palette(hex: &str, families: Vec<ColorFamily>) -> Result<ColorFamily, String> {
    let color = hex_to_hsla(hex.to_string())?;

    let closest_family = families
        .iter()
        .min_by(|f1, f2| {
            if f1.diff(color).abs() > f2.diff(color).abs() {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        })
        .unwrap();

    let (variation, closest_color) = closest_family.closest_color_to(color);

    let (light, dark) = if variation == "dark" {
        let dark_color_palette = closest_family.color_palette(closest_color, &color, "dark");
        let light_color_palette = closest_family.color_palette(
            &closest_family.light.c_9,
            &dark_color_palette[8],
            "light",
        );

        (light_color_palette, dark_color_palette)
    } else {
        let light_color_palette = closest_family.color_palette(closest_color, &color, "light");
        let dark_color_palette =
            closest_family.color_palette(&closest_family.dark.c_9, &light_color_palette[8], "dark");

        (light_color_palette, dark_color_palette)
    };

    Ok(ColorFamily {
        title: RadixFamilyName::Custom,
        ref_family: closest_family.title.clone(),
        original_color: Some(color),
        light: ColorScale {
            c_1: light[0],
            c_2: light[1],
            c_3: light[2],
            c_4: light[3],
            c_5: light[4],
            c_6: light[5],
            c_7: light[6],
            c_8: light[7],
            c_9: light[8],
            c_10: light[9],
            c_11: light[10],
            c_12: light[11],
        },
        dark: ColorScale {
            c_1: dark[0],
            c_2: dark[1],
            c_3: dark[2],
            c_4: dark[3],
            c_5: dark[4],
            c_6: dark[5],
            c_7: dark[6],
            c_8: dark[7],
            c_9: dark[8],
            c_10: dark[9],
            c_11: dark[10],
            c_12: dark[11],
        },
        light_alpha: ColorScale {
            c_1: hsla_alpha_conversion(light[0], Rgba::new(255., 255., 255., 1.))?,
            c_2: hsla_alpha_conversion(light[1], Rgba::new(255., 255., 255., 1.))?,
            c_3: hsla_alpha_conversion(light[2], Rgba::new(255., 255., 255., 1.))?,
            c_4: hsla_alpha_conversion(light[3], Rgba::new(255., 255., 255., 1.))?,
            c_5: hsla_alpha_conversion(light[4], Rgba::new(255., 255., 255., 1.))?,
            c_6: hsla_alpha_conversion(light[5], Rgba::new(255., 255., 255., 1.))?,
            c_7: hsla_alpha_conversion(light[6], Rgba::new(255., 255., 255., 1.))?,
            c_8: hsla_alpha_conversion(light[7], Rgba::new(255., 255., 255., 1.))?,
            c_9: hsla_alpha_conversion(light[8], Rgba::new(255., 255., 255., 1.))?,
            c_10: hsla_alpha_conversion(light[9], Rgba::new(255., 255., 255., 1.))?,
            c_11: hsla_alpha_conversion(light[10], Rgba::new(255., 255., 255., 1.))?,
            c_12: hsla_alpha_conversion(light[11], Rgba::new(255., 255., 255., 1.))?,
        },
        dark_alpha: ColorScale {
            c_1: hsla_alpha_conversion(dark[0], Rgba::new(0., 0., 0., 1.))?,
            c_2: hsla_alpha_conversion(dark[1], Rgba::new(0., 0., 0., 1.))?,
            c_3: hsla_alpha_conversion(dark[2], Rgba::new(0., 0., 0., 1.))?,
            c_4: hsla_alpha_conversion(dark[3], Rgba::new(0., 0., 0., 1.))?,
            c_5: hsla_alpha_conversion(dark[4], Rgba::new(0., 0., 0., 1.))?,
            c_6: hsla_alpha_conversion(dark[5], Rgba::new(0., 0., 0., 1.))?,
            c_7: hsla_alpha_conversion(dark[6], Rgba::new(0., 0., 0., 1.))?,
            c_8: hsla_alpha_conversion(dark[7], Rgba::new(0., 0., 0., 1.))?,
            c_9: hsla_alpha_conversion(dark[8], Rgba::new(0., 0., 0., 1.))?,
            c_10: hsla_alpha_conversion(dark[9], Rgba::new(0., 0., 0., 1.))?,
            c_11: hsla_alpha_conversion(dark[10], Rgba::new(0., 0., 0., 1.))?,
            c_12: hsla_alpha_conversion(dark[11], Rgba::new(0., 0., 0., 1.))?,
        },
        gray_family: closest_family.gray_family.clone(),
    })
}

impl ColorFamily {
    pub fn get_gray_family(&self) -> ColorFamily {
        use RadixFamilyName::*;

        match self.title {
            Tomato | Red | Ruby | Crimson | Pink | Plum | Purple | Violet => MAUVE.clone(),
            Iris | Indigo | Blue | Sky | Cyan => SLATE.clone(),
            Mint | Teal | Jade | Green => SAGE.clone(),
            Grass | Lime => OLIVE.clone(),
            Yellow | Amber | Orange | Brown => SAND.clone(),
            Custom => self.gray_family.as_deref().unwrap_or(&GRAY).clone(),
            _ => GRAY.clone(),
        }
    }

    /// Finds family color's palette and creates a new palette for reference color
    /// by applying the found factors
    fn color_palette(
        &self,
        family_color: &Hsla,
        reference_color: &Hsla,
        flavor: &str,
    ) -> Vec<Hsla> {
        let factors = self.factors(family_color, flavor);

        factors
            .iter()
            .map(|(h, s, l, a)| {
                let mut c = *reference_color;

                c.hue += *h;
                c.saturation += s;
                c.lightness += l;
                c.alpha += *a;

                c
            })
            .collect::<Vec<_>>()
    }

    /// Finds closest family color to reference
    fn closest_color_to(&self, reference_color: Hsla) -> (&str, &Hsla) {
        let light_colors = self.light.as_vec();
        let dark_colors = self.dark.as_vec();

        let l = *light_colors
            .iter()
            .min_by(|f1, f2| {
                if color_diff(f1, &reference_color).abs() > color_diff(f2, &reference_color).abs() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .unwrap();

        let d = *dark_colors
            .iter()
            .min_by(|f1, f2| {
                if color_diff(f1, &reference_color).abs() > color_diff(f2, &reference_color).abs() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
            .unwrap();

        if color_diff(l, &reference_color) < color_diff(d, &reference_color) {
            ("light", l)
        } else {
            ("dark", d)
        }
    }

    fn diff(&self, color: Hsla) -> f32 {
        let (_, closest_color_by_l) = self.closest_color_to(color);

        color_diff(closest_color_by_l, &color)
    }

    /// Find difference in hue, saturation and lightness between the reference color and
    /// each family color
    fn factors(&self, family_color: &Hsla, flavor: &str) -> Vec<(f32, f32, f32, f32)> {
        let colors = if flavor == "dark" {
            self.dark.as_vec()
        } else {
            self.light.as_vec()
        };

        colors
            .iter()
            .map(|c| {
                // We lose some precision due to f32 arithmetics
                (
                    (c.hue - family_color.hue).into(),
                    c.saturation - family_color.saturation,
                    c.lightness - family_color.lightness,
                    c.alpha - family_color.alpha,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    mod radix_colors {
        use crate::color_generator::{
            color_utils::hsla_to_hex,
            radix::get_palette,
            stylesheets::{get_radix_families, AMBER},
        };

        #[test]
        fn get_palette_with_a_radix_color() {
            // We should match to the right scale if the color is exactly on the scale
            let hex = hsla_to_hex(AMBER.light.c_9);
            let family = get_palette(&hex, get_radix_families()).unwrap();

            assert_eq!(hsla_to_hex(family.light.c_1), hsla_to_hex(AMBER.light.c_1));
            assert_eq!(hsla_to_hex(family.light.c_2), hsla_to_hex(AMBER.light.c_2));
            assert_eq!(hsla_to_hex(family.light.c_3), hsla_to_hex(AMBER.light.c_3));
            assert_eq!(hsla_to_hex(family.light.c_4), hsla_to_hex(AMBER.light.c_4));
            assert_eq!(hsla_to_hex(family.light.c_5), hsla_to_hex(AMBER.light.c_5));
            assert_eq!(hsla_to_hex(family.light.c_6), hsla_to_hex(AMBER.light.c_6));
            assert_eq!(hsla_to_hex(family.light.c_7), hsla_to_hex(AMBER.light.c_7));
            assert_eq!(hsla_to_hex(family.light.c_8), hsla_to_hex(AMBER.light.c_8));
            assert_eq!(hsla_to_hex(family.light.c_9), hsla_to_hex(AMBER.light.c_9));
            assert_eq!(
                hsla_to_hex(family.light.c_10),
                hsla_to_hex(AMBER.light.c_10)
            );
            assert_eq!(
                hsla_to_hex(family.light.c_11),
                hsla_to_hex(AMBER.light.c_11)
            );
            assert_eq!(
                hsla_to_hex(family.light.c_12),
                hsla_to_hex(AMBER.light.c_12)
            );

            assert_eq!(hsla_to_hex(family.dark.c_1), hsla_to_hex(AMBER.dark.c_1));
            assert_eq!(hsla_to_hex(family.dark.c_2), hsla_to_hex(AMBER.dark.c_2));
            assert_eq!(hsla_to_hex(family.dark.c_3), hsla_to_hex(AMBER.dark.c_3));
            assert_eq!(hsla_to_hex(family.dark.c_4), hsla_to_hex(AMBER.dark.c_4));
            assert_eq!(hsla_to_hex(family.dark.c_5), hsla_to_hex(AMBER.dark.c_5));
            assert_eq!(hsla_to_hex(family.dark.c_6), hsla_to_hex(AMBER.dark.c_6));
            assert_eq!(hsla_to_hex(family.dark.c_7), hsla_to_hex(AMBER.dark.c_7));
            assert_eq!(hsla_to_hex(family.dark.c_8), hsla_to_hex(AMBER.dark.c_8));
            assert_eq!(hsla_to_hex(family.dark.c_9), hsla_to_hex(AMBER.dark.c_9));
            assert_eq!(hsla_to_hex(family.dark.c_10), hsla_to_hex(AMBER.dark.c_10));
            assert_eq!(hsla_to_hex(family.dark.c_11), hsla_to_hex(AMBER.dark.c_11));
            assert_eq!(hsla_to_hex(family.dark.c_12), hsla_to_hex(AMBER.dark.c_12));
        }

        #[test]
        fn get_palette_with_a_low_radix_color() {
            // We should match to the right scale, even if the color is not the primary one (c_9)
            let hex = hsla_to_hex(AMBER.light.c_4);
            let family = get_palette(&hex, get_radix_families()).unwrap();

            assert_eq!(hsla_to_hex(family.light.c_1), hsla_to_hex(AMBER.light.c_1));
            assert_eq!(hsla_to_hex(family.light.c_2), hsla_to_hex(AMBER.light.c_2));
            assert_eq!(hsla_to_hex(family.light.c_3), hsla_to_hex(AMBER.light.c_3));
            assert_eq!(hsla_to_hex(family.light.c_4), hsla_to_hex(AMBER.light.c_4));
            assert_eq!(hsla_to_hex(family.light.c_5), hsla_to_hex(AMBER.light.c_5));
            assert_eq!(hsla_to_hex(family.light.c_6), hsla_to_hex(AMBER.light.c_6));
            assert_eq!(hsla_to_hex(family.light.c_7), hsla_to_hex(AMBER.light.c_7));
            assert_eq!(hsla_to_hex(family.light.c_8), hsla_to_hex(AMBER.light.c_8));
            assert_eq!(hsla_to_hex(family.light.c_9), hsla_to_hex(AMBER.light.c_9));
            assert_eq!(
                hsla_to_hex(family.light.c_10),
                hsla_to_hex(AMBER.light.c_10)
            );
            assert_eq!(
                hsla_to_hex(family.light.c_11),
                hsla_to_hex(AMBER.light.c_11)
            );
            assert_eq!(
                hsla_to_hex(family.light.c_12),
                hsla_to_hex(AMBER.light.c_12)
            );

            assert_eq!(hsla_to_hex(family.dark.c_1), hsla_to_hex(AMBER.dark.c_1));
            assert_eq!(hsla_to_hex(family.dark.c_2), hsla_to_hex(AMBER.dark.c_2));
            assert_eq!(hsla_to_hex(family.dark.c_3), hsla_to_hex(AMBER.dark.c_3));
            assert_eq!(hsla_to_hex(family.dark.c_4), hsla_to_hex(AMBER.dark.c_4));
            assert_eq!(hsla_to_hex(family.dark.c_5), hsla_to_hex(AMBER.dark.c_5));
            assert_eq!(hsla_to_hex(family.dark.c_6), hsla_to_hex(AMBER.dark.c_6));
            assert_eq!(hsla_to_hex(family.dark.c_7), hsla_to_hex(AMBER.dark.c_7));
            assert_eq!(hsla_to_hex(family.dark.c_8), hsla_to_hex(AMBER.dark.c_8));
            assert_eq!(hsla_to_hex(family.dark.c_9), hsla_to_hex(AMBER.dark.c_9));
            assert_eq!(hsla_to_hex(family.dark.c_10), hsla_to_hex(AMBER.dark.c_10));
            assert_eq!(hsla_to_hex(family.dark.c_11), hsla_to_hex(AMBER.dark.c_11));
            assert_eq!(hsla_to_hex(family.dark.c_12), hsla_to_hex(AMBER.dark.c_12));
        }
    }
}
