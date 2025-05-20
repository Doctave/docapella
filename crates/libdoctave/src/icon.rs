#[cfg(feature = "rustler")]
use rustler::{NifStruct, NifUntaggedEnum};

#[cfg(test)]
use schemars::JsonSchema;

use include_dir::{include_dir, Dir};

#[cfg(test)]
use ts_rs::TS;

static LUCIDE: Dir = include_dir!("./crates/libdoctave/icon_sets/lucide");

#[derive(Debug, Clone, PartialEq, Serialize)]
/// NOTE: Remember to update `available_sets()` if you add an icon set here.
///
/// NOTE: We cannot actually use the ts-rs derived types, because we manually implement Serialize,
/// so we override the type manually here.
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifUntaggedEnum))]
#[serde(tag = "set")]
pub enum Icon {
    #[serde(rename = "lucide")]
    Lucide(LucideIcon),
    #[serde(rename = "devicon")]
    Devicon(DeviconIcon),
    /// Since the user can give an incorrect set name,
    /// we have this fallback variant so that we can
    /// give nice error messages to the user.
    #[serde(rename = "unknown")]
    Unknown(UnknownIcon),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.Structure.Icon")]
pub struct LucideIcon {
    pub name: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.Structure.Icon")]
pub struct DeviconIcon {
    pub name: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.Structure.Icon")]
pub struct UnknownIcon {
    pub unknown_set_name: String,
    pub name: String,
    pub html: String,
}

impl Icon {
    pub fn available_sets() -> &'static [&'static str] {
        &["lucide", "devicon"][..]
    }

    pub fn name(&self) -> &str {
        use Icon::*;
        match self {
            Lucide(LucideIcon { name, .. }) => name,
            Devicon(DeviconIcon { name, .. }) => name,
            Unknown(UnknownIcon { name, .. }) => name,
        }
    }

    pub fn set(&self) -> &str {
        use Icon::*;
        match self {
            Lucide { .. } => "lucide",
            Devicon { .. } => "devicon",
            Unknown(UnknownIcon {
                unknown_set_name, ..
            }) => unknown_set_name.as_str(),
        }
    }

    pub fn html(&self) -> &str {
        use Icon::*;
        match self {
            Lucide(LucideIcon { html, .. }) => html,
            Devicon(DeviconIcon { html, .. }) => html,
            Unknown(UnknownIcon { html, .. }) => html.as_str(),
        }
    }

    pub fn is_valid(&self) -> bool {
        !matches!(self, Icon::Unknown { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, derive(JsonSchema))]
#[cfg_attr(test, ts(export))]
pub struct IconDescription {
    set: String,
    name: String,
}

impl IconDescription {
    pub(crate) fn resolve(self) -> Icon {
        match self.set.to_lowercase().as_str() {
            "lucide" => Icon::Lucide(LucideIcon {
                html: LUCIDE
                    .get_file(format!("{}.svg", self.name))
                    .and_then(|icon| icon.contents_utf8())
                    .map(|i| i.to_owned())
                    .unwrap_or_default(),
                name: self.name,
            }),
            "devicon" => Icon::Devicon(DeviconIcon {
                html: if self.name.chars().all(|c| c.is_alphanumeric() || c == '-') {
                    format!("<i class='devicon-{}-plain'></i>", self.name)
                } else {
                    String::new()
                },
                name: self.name,
            }),
            _other => Icon::Unknown(UnknownIcon {
                unknown_set_name: self.set,
                name: self.name,
                html: String::new(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanitizes_devicons_class_names() {
        let icon: Icon = IconDescription {
            set: "devicon".to_string(),
            name: "<script>console.log('asd')</script>".to_owned(),
        }
        .resolve();

        assert_eq!(icon.html(), "");

        let icon = IconDescription {
            set: "devicon".to_string(),
            name: "tailwindcss".to_owned(),
        }
        .resolve();

        assert_eq!(icon.html(), "<i class='devicon-tailwindcss-plain'></i>");

        let icon = IconDescription {
            set: "devicon".to_string(),
            name: "k3s".to_owned(),
        }
        .resolve();

        assert_eq!(icon.html(), "<i class='devicon-k3s-plain'></i>");
    }

    #[test]
    fn other_icon_variants_are_invalid() {
        assert!(!IconDescription {
            set: "foo".to_owned(),
            name: "foo".to_owned()
        }
        .resolve()
        .is_valid());

        assert!(IconDescription {
            set: "devicon".to_string(),
            name: "foo".to_owned()
        }
        .resolve()
        .is_valid());

        assert!(IconDescription {
            set: "lucide".to_string(),
            name: "foo".to_owned()
        }
        .resolve()
        .is_valid());
    }
}
