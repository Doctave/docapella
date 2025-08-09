use crate::{ContentApiResponse, Result};
use minijinja::{self, context, value, Environment, Error, Value};
use serde_json;

pub struct Renderer {
    env: Environment<'static>,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();
        minijinja_embed::load_templates!(&mut env);

        env.add_function("initial_openapi_tab_index", initial_openapi_tab_index);

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
