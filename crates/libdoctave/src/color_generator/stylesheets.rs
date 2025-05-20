use lightningcss::{
    stylesheet::{ParserOptions, StyleSheet},
    values::color::HSL,
};
use palette::Hsla;

use super::radix::{ColorFamily, ColorScale, RadixFamilyName};

pub fn get_radix_families() -> Vec<ColorFamily> {
    vec![
        AMBER.clone(),
        BLUE.clone(),
        BRONZE.clone(),
        BROWN.clone(),
        CRIMSON.clone(),
        CYAN.clone(),
        GOLD.clone(),
        GRASS.clone(),
        GREEN.clone(),
        INDIGO.clone(),
        IRIS.clone(),
        JADE.clone(),
        LIME.clone(),
        MINT.clone(),
        ORANGE.clone(),
        PINK.clone(),
        PLUM.clone(),
        PURPLE.clone(),
        RED.clone(),
        RUBY.clone(),
        SKY.clone(),
        TEAL.clone(),
        TOMATO.clone(),
        VIOLET.clone(),
        YELLOW.clone(),
    ]
}

/// NOTE: DO NOT CHANGE THE ORDER OF THESE.
///
/// SettingsV2 relies on them.
pub fn get_gray_families() -> Vec<ColorFamily> {
    vec![
        GRAY.clone(),
        MAUVE.clone(),
        OLIVE.clone(),
        SAGE.clone(),
        SAND.clone(),
        SLATE.clone(),
    ]
}

pub fn get_color_family(
    family_name: RadixFamilyName,
    light_content: &str,
    dark_content: &str,
    light_alpha_content: &str,
    dark_alpha_content: &str,
) -> ColorFamily {
    let hsls_light = scale_from_css(light_content);
    let hsls_dark = scale_from_css(dark_content);
    let hsls_light_alpha = scale_from_css(light_alpha_content);
    let hsls_dark_alpha = scale_from_css(dark_alpha_content);

    let color_scale = ColorScale {
        c_1: hsls_light[0],
        c_2: hsls_light[1],
        c_3: hsls_light[2],
        c_4: hsls_light[3],
        c_5: hsls_light[4],
        c_6: hsls_light[5],
        c_7: hsls_light[6],
        c_8: hsls_light[7],
        c_9: hsls_light[8],
        c_10: hsls_light[9],
        c_11: hsls_light[10],
        c_12: hsls_light[11],
    };

    let dark_color_scale = ColorScale {
        c_1: hsls_dark[0],
        c_2: hsls_dark[1],
        c_3: hsls_dark[2],
        c_4: hsls_dark[3],
        c_5: hsls_dark[4],
        c_6: hsls_dark[5],
        c_7: hsls_dark[6],
        c_8: hsls_dark[7],
        c_9: hsls_dark[8],
        c_10: hsls_dark[9],
        c_11: hsls_dark[10],
        c_12: hsls_dark[11],
    };

    let light_alpha_color_scale = ColorScale {
        c_1: hsls_light_alpha[0],
        c_2: hsls_light_alpha[1],
        c_3: hsls_light_alpha[2],
        c_4: hsls_light_alpha[3],
        c_5: hsls_light_alpha[4],
        c_6: hsls_light_alpha[5],
        c_7: hsls_light_alpha[6],
        c_8: hsls_light_alpha[7],
        c_9: hsls_light_alpha[8],
        c_10: hsls_light_alpha[9],
        c_11: hsls_light_alpha[10],
        c_12: hsls_light_alpha[11],
    };

    let dark_alpha_color_scale = ColorScale {
        c_1: hsls_dark_alpha[0],
        c_2: hsls_dark_alpha[1],
        c_3: hsls_dark_alpha[2],
        c_4: hsls_dark_alpha[3],
        c_5: hsls_dark_alpha[4],
        c_6: hsls_dark_alpha[5],
        c_7: hsls_dark_alpha[6],
        c_8: hsls_dark_alpha[7],
        c_9: hsls_dark_alpha[8],
        c_10: hsls_dark_alpha[9],
        c_11: hsls_dark_alpha[10],
        c_12: hsls_dark_alpha[11],
    };

    let gray_family = match family_name {
        RadixFamilyName::Tomato
        | RadixFamilyName::Red
        | RadixFamilyName::Ruby
        | RadixFamilyName::Crimson
        | RadixFamilyName::Pink
        | RadixFamilyName::Plum
        | RadixFamilyName::Purple
        | RadixFamilyName::Violet => Some(MAUVE.clone()),
        RadixFamilyName::Iris
        | RadixFamilyName::Indigo
        | RadixFamilyName::Blue
        | RadixFamilyName::Sky
        | RadixFamilyName::Cyan => Some(SLATE.clone()),
        RadixFamilyName::Mint
        | RadixFamilyName::Teal
        | RadixFamilyName::Jade
        | RadixFamilyName::Green => Some(SAGE.clone()),
        RadixFamilyName::Grass | RadixFamilyName::Lime => Some(OLIVE.clone()),
        RadixFamilyName::Yellow
        | RadixFamilyName::Amber
        | RadixFamilyName::Orange
        | RadixFamilyName::Brown => Some(SAND.clone()),
        RadixFamilyName::Sand
        | RadixFamilyName::Mauve
        | RadixFamilyName::Sage
        | RadixFamilyName::Slate
        | RadixFamilyName::Olive
        | RadixFamilyName::Gray
        | RadixFamilyName::Custom => None,
        _ => Some(GRAY.clone()),
    };

    ColorFamily {
        title: family_name.clone(),
        ref_family: family_name,
        original_color: None,
        light: color_scale,
        dark: dark_color_scale,
        light_alpha: light_alpha_color_scale,
        dark_alpha: dark_alpha_color_scale,
        gray_family: gray_family.map(Box::new),
    }
}

fn scale_from_css(content: &str) -> Vec<Hsla> {
    let stylesheet = StyleSheet::parse(content, ParserOptions::default())
        .expect("This is a bug: we have some faulty css in radix css files");

    let rules = stylesheet
        .rules
        .0
        .iter()
        .map(|rule| match rule {
            lightningcss::rules::CssRule::Style(style_rule) => {
                let vec = style_rule
                    .declarations
                    .iter()
                    .map(|dec| match dec.0 {
                        lightningcss::properties::Property::Custom(prop) => {
                            let hsl = match prop.value.0.first().unwrap() {
                                lightningcss::properties::custom::TokenOrValue::Color(
                                    lightningcss::values::color::CssColor::RGBA(color),
                                ) => {
                                    let hsl: HSL = (*color).into();

                                    let hue = if hsl.h.is_nan() { 0.0 } else { hsl.h };
                                    let saturation = if hsl.s.is_nan() { 0.0 } else { hsl.s };
                                    let lightness = if hsl.l.is_nan() { 0.0 } else { hsl.l };

                                    Hsla::new(hue, saturation, lightness, hsl.alpha)
                                }
                                _ => panic!(
                                    "This is a bug: we have some faulty css in radix css files"
                                ),
                            };

                            hsl
                        }
                        _ => panic!("This is a bug: we have some faulty css in radix css files"),
                    })
                    .collect::<Vec<_>>();
                vec
            }
            _ => {
                vec![]
            }
        })
        .collect::<Vec<_>>();

    let hsls = rules.first().unwrap();

    hsls.to_vec()
}

lazy_static! {
    pub(crate) static ref AMBER: ColorFamily = get_color_family(
        RadixFamilyName::Amber,
        include_str!("./css/radix/amber.css"),
        include_str!("./css/radix/amber-dark.css"),
        include_str!("./css/radix/amber-alpha.css"),
        include_str!("./css/radix/amber-dark-alpha.css")
    );
    pub(crate) static ref BLUE: ColorFamily = get_color_family(
        RadixFamilyName::Blue,
        include_str!("./css/radix/blue.css"),
        include_str!("./css/radix/blue-dark.css"),
        include_str!("./css/radix/blue-alpha.css"),
        include_str!("./css/radix/blue-dark-alpha.css")
    );
    pub(crate) static ref BRONZE: ColorFamily = get_color_family(
        RadixFamilyName::Bronze,
        include_str!("./css/radix/bronze.css"),
        include_str!("./css/radix/bronze-dark.css"),
        include_str!("./css/radix/bronze-alpha.css"),
        include_str!("./css/radix/bronze-dark-alpha.css")
    );
    pub(crate) static ref BROWN: ColorFamily = get_color_family(
        RadixFamilyName::Brown,
        include_str!("./css/radix/brown.css"),
        include_str!("./css/radix/brown-dark.css"),
        include_str!("./css/radix/brown-alpha.css"),
        include_str!("./css/radix/brown-dark-alpha.css")
    );
    pub(crate) static ref CRIMSON: ColorFamily = get_color_family(
        RadixFamilyName::Crimson,
        include_str!("./css/radix/crimson.css"),
        include_str!("./css/radix/crimson-dark.css"),
        include_str!("./css/radix/crimson-alpha.css"),
        include_str!("./css/radix/crimson-dark-alpha.css")
    );
    pub(crate) static ref CYAN: ColorFamily = get_color_family(
        RadixFamilyName::Cyan,
        include_str!("./css/radix/cyan.css"),
        include_str!("./css/radix/cyan-dark.css"),
        include_str!("./css/radix/cyan-alpha.css"),
        include_str!("./css/radix/cyan-dark-alpha.css")
    );
    pub(crate) static ref GOLD: ColorFamily = get_color_family(
        RadixFamilyName::Gold,
        include_str!("./css/radix/gold.css"),
        include_str!("./css/radix/gold-dark.css"),
        include_str!("./css/radix/gold-alpha.css"),
        include_str!("./css/radix/gold-dark-alpha.css")
    );
    pub(crate) static ref GRASS: ColorFamily = get_color_family(
        RadixFamilyName::Grass,
        include_str!("./css/radix/grass.css"),
        include_str!("./css/radix/grass-dark.css"),
        include_str!("./css/radix/grass-alpha.css"),
        include_str!("./css/radix/grass-dark-alpha.css")
    );
    pub(crate) static ref GRAY: ColorFamily = get_color_family(
        RadixFamilyName::Gray,
        include_str!("./css/radix/gray.css"),
        include_str!("./css/radix/gray-dark.css"),
        include_str!("./css/radix/gray-alpha.css"),
        include_str!("./css/radix/gray-dark-alpha.css")
    );
    pub(crate) static ref GREEN: ColorFamily = get_color_family(
        RadixFamilyName::Green,
        include_str!("./css/radix/green.css"),
        include_str!("./css/radix/green-dark.css"),
        include_str!("./css/radix/green-alpha.css"),
        include_str!("./css/radix/green-dark-alpha.css")
    );
    pub(crate) static ref INDIGO: ColorFamily = get_color_family(
        RadixFamilyName::Indigo,
        include_str!("./css/radix/indigo.css"),
        include_str!("./css/radix/indigo-dark.css"),
        include_str!("./css/radix/indigo-alpha.css"),
        include_str!("./css/radix/indigo-dark-alpha.css")
    );
    pub(crate) static ref IRIS: ColorFamily = get_color_family(
        RadixFamilyName::Iris,
        include_str!("./css/radix/iris.css"),
        include_str!("./css/radix/iris-dark.css"),
        include_str!("./css/radix/iris-alpha.css"),
        include_str!("./css/radix/iris-dark-alpha.css")
    );
    pub(crate) static ref JADE: ColorFamily = get_color_family(
        RadixFamilyName::Jade,
        include_str!("./css/radix/jade.css"),
        include_str!("./css/radix/jade-dark.css"),
        include_str!("./css/radix/jade-alpha.css"),
        include_str!("./css/radix/jade-dark-alpha.css")
    );
    pub(crate) static ref LIME: ColorFamily = get_color_family(
        RadixFamilyName::Lime,
        include_str!("./css/radix/lime.css"),
        include_str!("./css/radix/lime-dark.css"),
        include_str!("./css/radix/lime-alpha.css"),
        include_str!("./css/radix/lime-dark-alpha.css")
    );
    pub(crate) static ref MAUVE: ColorFamily = get_color_family(
        RadixFamilyName::Mauve,
        include_str!("./css/radix/mauve.css"),
        include_str!("./css/radix/mauve-dark.css"),
        include_str!("./css/radix/mauve-alpha.css"),
        include_str!("./css/radix/mauve-dark-alpha.css")
    );
    pub(crate) static ref MINT: ColorFamily = get_color_family(
        RadixFamilyName::Mint,
        include_str!("./css/radix/mint.css"),
        include_str!("./css/radix/mint-dark.css"),
        include_str!("./css/radix/mint-alpha.css"),
        include_str!("./css/radix/mint-dark-alpha.css")
    );
    pub(crate) static ref OLIVE: ColorFamily = get_color_family(
        RadixFamilyName::Olive,
        include_str!("./css/radix/olive.css"),
        include_str!("./css/radix/olive-dark.css"),
        include_str!("./css/radix/olive-alpha.css"),
        include_str!("./css/radix/olive-dark-alpha.css")
    );
    pub(crate) static ref ORANGE: ColorFamily = get_color_family(
        RadixFamilyName::Orange,
        include_str!("./css/radix/orange.css"),
        include_str!("./css/radix/orange-dark.css"),
        include_str!("./css/radix/orange-alpha.css"),
        include_str!("./css/radix/orange-dark-alpha.css")
    );
    pub(crate) static ref PINK: ColorFamily = get_color_family(
        RadixFamilyName::Pink,
        include_str!("./css/radix/pink.css"),
        include_str!("./css/radix/pink-dark.css"),
        include_str!("./css/radix/pink-alpha.css"),
        include_str!("./css/radix/pink-dark-alpha.css")
    );
    pub(crate) static ref PLUM: ColorFamily = get_color_family(
        RadixFamilyName::Plum,
        include_str!("./css/radix/plum.css"),
        include_str!("./css/radix/plum-dark.css"),
        include_str!("./css/radix/plum-alpha.css"),
        include_str!("./css/radix/plum-dark-alpha.css")
    );
    pub(crate) static ref PURPLE: ColorFamily = get_color_family(
        RadixFamilyName::Purple,
        include_str!("./css/radix/purple.css"),
        include_str!("./css/radix/purple-dark.css"),
        include_str!("./css/radix/purple-alpha.css"),
        include_str!("./css/radix/purple-dark-alpha.css")
    );
    pub(crate) static ref RED: ColorFamily = get_color_family(
        RadixFamilyName::Red,
        include_str!("./css/radix/red.css"),
        include_str!("./css/radix/red-dark.css"),
        include_str!("./css/radix/red-alpha.css"),
        include_str!("./css/radix/red-dark-alpha.css")
    );
    pub(crate) static ref RUBY: ColorFamily = get_color_family(
        RadixFamilyName::Ruby,
        include_str!("./css/radix/ruby.css"),
        include_str!("./css/radix/ruby-dark.css"),
        include_str!("./css/radix/ruby-alpha.css"),
        include_str!("./css/radix/ruby-dark-alpha.css")
    );
    pub(crate) static ref SAGE: ColorFamily = get_color_family(
        RadixFamilyName::Sage,
        include_str!("./css/radix/sage.css"),
        include_str!("./css/radix/sage-dark.css"),
        include_str!("./css/radix/sage-alpha.css"),
        include_str!("./css/radix/sage-dark-alpha.css")
    );
    pub(crate) static ref SAND: ColorFamily = get_color_family(
        RadixFamilyName::Sand,
        include_str!("./css/radix/sand.css"),
        include_str!("./css/radix/sand-dark.css"),
        include_str!("./css/radix/sand-alpha.css"),
        include_str!("./css/radix/sand-dark-alpha.css")
    );
    pub(crate) static ref SKY: ColorFamily = get_color_family(
        RadixFamilyName::Sky,
        include_str!("./css/radix/sky.css"),
        include_str!("./css/radix/sky-dark.css"),
        include_str!("./css/radix/sky-alpha.css"),
        include_str!("./css/radix/sky-dark-alpha.css")
    );
    pub(crate) static ref SLATE: ColorFamily = get_color_family(
        RadixFamilyName::Slate,
        include_str!("./css/radix/slate.css"),
        include_str!("./css/radix/slate-dark.css"),
        include_str!("./css/radix/slate-alpha.css"),
        include_str!("./css/radix/slate-dark-alpha.css")
    );
    pub(crate) static ref TEAL: ColorFamily = get_color_family(
        RadixFamilyName::Teal,
        include_str!("./css/radix/teal.css"),
        include_str!("./css/radix/teal-dark.css"),
        include_str!("./css/radix/teal-alpha.css"),
        include_str!("./css/radix/teal-dark-alpha.css")
    );
    pub(crate) static ref TOMATO: ColorFamily = get_color_family(
        RadixFamilyName::Tomato,
        include_str!("./css/radix/tomato.css"),
        include_str!("./css/radix/tomato-dark.css"),
        include_str!("./css/radix/tomato-alpha.css"),
        include_str!("./css/radix/tomato-dark-alpha.css")
    );
    pub(crate) static ref VIOLET: ColorFamily = get_color_family(
        RadixFamilyName::Violet,
        include_str!("./css/radix/violet.css"),
        include_str!("./css/radix/violet-dark.css"),
        include_str!("./css/radix/violet-alpha.css"),
        include_str!("./css/radix/violet-dark-alpha.css")
    );
    pub(crate) static ref YELLOW: ColorFamily = get_color_family(
        RadixFamilyName::Yellow,
        include_str!("./css/radix/yellow.css"),
        include_str!("./css/radix/yellow-dark.css"),
        include_str!("./css/radix/yellow-alpha.css"),
        include_str!("./css/radix/yellow-dark-alpha.css")
    );
}
