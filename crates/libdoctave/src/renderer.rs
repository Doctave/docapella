use crate::{ContentApiResponse, Result};
use color_generator::{Appearance, ColorGenerator};
use minijinja::{self, context, Environment, Error, Value};
use serde_json;

pub struct Renderer {
    env: Environment<'static>,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();
        minijinja_embed::load_templates!(&mut env);

        env.add_function("initial_openapi_tab_index", initial_openapi_tab_index);
        env.add_function("actual_type_name", actual_type_name);
        env.add_function("startswith", startswith);
        env.add_function("color_scale_css", color_scale_css);
        env.add_function("flatten_examples", flatten_examples);
        env.add_function("flatten_response_examples", flatten_response_examples);

        Ok(Renderer { env })
    }

    pub fn render_page(&self, response: ContentApiResponse) -> Result<String> {
        match response {
            ContentApiResponse::Content {
                page,
                project,
                build,
                view_mode,
                sign_assets,
                debug_info,
            } => {
                // Convert to Value for template rendering
                let page_value = serde_json::to_value(&page).expect("Failed to serialize page");
                let project_value =
                    serde_json::to_value(&project).expect("Failed to serialize project");

                let template = self
                    .env
                    .get_template("layouts/base.html.jinja")
                    .expect("Failed to get template");

                let rendered = template
                    .render(context! {
                        page => page_value,
                        project => project_value,
                        build => build,
                        view_mode => view_mode,
                        sign_assets => sign_assets,
                        debug_info => debug_info,
                    })
                    .expect("Failed to render template");

                Ok(rendered)
            }
            _ => todo!(),
        }
    }
}

/// Returns the index of the initial openapi operation tab
fn initial_openapi_tab_index(value: &Value) -> std::result::Result<usize, Error> {
    if value.get_attr("request_body")? != Value::UNDEFINED {
        Ok(0)
    } else if value.get_attr("query_params")? != Value::UNDEFINED {
        Ok(1)
    } else if value.get_attr("header_params")? != Value::UNDEFINED {
        Ok(2)
    } else if value.get_attr("path_params")? != Value::UNDEFINED {
        Ok(3)
    } else if value.get_attr("cookie_params")? != Value::UNDEFINED {
        Ok(4)
    } else {
        Ok(0)
    }
}

/// Returns the actual type name for display, handling array and object types with metadata
fn actual_type_name(schema: &Value) -> std::result::Result<String, Error> {
    let type_name = schema
        .get_attr("type_name")?
        .as_str()
        .unwrap_or("")
        .to_string();

    let mut result_type_name = type_name.clone();
    let is_array = type_name.starts_with("array");
    let is_object = type_name.starts_with("object");

    if is_array {
        result_type_name = "array".to_string();

        // Check for metadata title or component_name
        if let Ok(metadata) = schema.get_attr("metadata") {
            if metadata != Value::UNDEFINED {
                let title = metadata.get_attr("title").ok().and_then(|v| {
                    if v == Value::UNDEFINED {
                        None
                    } else {
                        v.as_str().map(|s| s.to_string())
                    }
                });
                let component_name = metadata.get_attr("component_name").ok().and_then(|v| {
                    if v == Value::UNDEFINED {
                        None
                    } else {
                        v.as_str().map(|s| s.to_string())
                    }
                });

                if let Some(name) = title.or(component_name) {
                    result_type_name = format!("array [{}]", name);
                }
            }
        }
    } else if is_object {
        result_type_name = "object".to_string();

        // Check for metadata title or component_name
        if let Ok(metadata) = schema.get_attr("metadata") {
            if metadata != Value::UNDEFINED {
                let title = metadata.get_attr("title").ok().and_then(|v| {
                    if v == Value::UNDEFINED {
                        None
                    } else {
                        v.as_str().map(|s| s.to_string())
                    }
                });
                let component_name = metadata.get_attr("component_name").ok().and_then(|v| {
                    if v == Value::UNDEFINED {
                        None
                    } else {
                        v.as_str().map(|s| s.to_string())
                    }
                });

                if let Some(name) = title.or(component_name) {
                    result_type_name = format!("object ({})", name);
                }
            }
        }
    }

    Ok(result_type_name)
}

fn color_scale_css(accent: &Value, gray_scale: &Value) -> std::result::Result<String, Error> {
    let accent = accent.as_str().ok_or_else(|| {
        Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "color_scale_css first argument must be a string",
        )
    })?;
    let gray_scale = gray_scale.as_str().ok_or_else(|| {
        Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "color_scale_css second argument must be a string",
        )
    })?;

    let generator = ColorGenerator::new();

    let light_palette = generator.generate_scale(Appearance::Light, accent, gray_scale, "#ffffff");
    let dark_palette = generator.generate_scale(Appearance::Dark, accent, gray_scale, "#000000");

    let light_css = light_palette.generate_css("accent", ":root, .light, .light-theme");
    let dark_css = dark_palette.generate_css("accent", ".dark, .dark-theme");

    Ok(format!("{}\n\n{}", light_css, dark_css))
}

/// Returns whether a string starts with a given prefix
fn startswith(string: &Value, prefix: &Value) -> std::result::Result<bool, Error> {
    let Some(string_str) = string.as_str() else {
        return Err(Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "startswith first argument must be a string",
        ));
    };

    let Some(prefix_str) = prefix.as_str() else {
        return Err(Error::new(
            minijinja::ErrorKind::InvalidOperation,
            "startswith second argument must be a string",
        ));
    };

    Ok(string_str.starts_with(prefix_str))
}

/// Flattens examples from request_examples and media_type examples into a single array
/// Matches V2 logic: (operation.request_body?.media_types || []).flatMap((m) => m.examples).concat(operation.request_examples || [])
fn flatten_examples(operation: &Value) -> std::result::Result<Value, Error> {
    let mut all_examples: Vec<Value> = Vec::new();

    // Add request_examples first
    if let Ok(request_examples) = operation.get_attr("request_examples") {
        if request_examples != Value::UNDEFINED {
            if let Ok(examples_iter) = request_examples.try_iter() {
                for example in examples_iter {
                    all_examples.push(example);
                }
            }
        }
    }

    // Add media type examples
    if let Ok(request_body) = operation.get_attr("request_body") {
        if request_body != Value::UNDEFINED {
            if let Ok(media_types) = request_body.get_attr("media_types") {
                if media_types != Value::UNDEFINED {
                    if let Ok(media_types_iter) = media_types.try_iter() {
                        for media_type in media_types_iter {
                            if let Ok(examples) = media_type.get_attr("examples") {
                                if examples != Value::UNDEFINED {
                                    if let Ok(examples_iter) = examples.try_iter() {
                                        for example in examples_iter {
                                            all_examples.push(example);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Value::from_serialize(all_examples))
}

/// Flattens examples from all media types of a response into a single array
/// Matches V2 logic: (response?.media_types || []).flatMap((m) => m.examples)
fn flatten_response_examples(response: &Value) -> std::result::Result<Value, Error> {
    let mut all_examples: Vec<Value> = Vec::new();

    // Add all examples from all media types
    if let Ok(media_types) = response.get_attr("media_types") {
        if media_types != Value::UNDEFINED {
            if let Ok(media_types_iter) = media_types.try_iter() {
                for media_type in media_types_iter {
                    if let Ok(examples) = media_type.get_attr("examples") {
                        if examples != Value::UNDEFINED {
                            if let Ok(examples_iter) = examples.try_iter() {
                                for example in examples_iter {
                                    all_examples.push(example);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Value::from_serialize(all_examples))
}
