use crate::{ContentApiResponse, Result};
use minijinja::{self, context, Environment, State, Value};
use serde_json;

pub struct Renderer {
    env: Environment<'static>,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();
        minijinja_embed::load_templates!(&mut env);

        env.add_function("render_markdown_node", render_markdown_node);

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
                    .get_template("layouts/base.html")
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

fn render_markdown_node(
    state: &State,
    node: Value,
) -> std::result::Result<String, minijinja::Error> {
    // Get the node kind to determine which template to use
    let tmp = node
        .get_attr("kind")
        .and_then(|k| k.get_attr("name"))
        .expect("Failed to get node kind");

    let kind = tmp.as_str().expect("Failed to get node kind as string");

    // Get the current environment from context
    let env = state.env();

    // Determine template path based on node kind
    let template_path = match kind {
        "root" => "components/markdown/root.html",
        "text" => "components/markdown/text.html",
        "paragraph" => "components/markdown/paragraph.html",
        "heading" => "components/markdown/heading.html",
        "strong" => "components/markdown/strong.html",
        "emphasis" => "components/markdown/emphasis.html",
        "link" => "components/markdown/link.html",
        "list" => "components/markdown/list.html",
        "list_item" => "components/markdown/list_item.html",
        "code" => "components/markdown/code.html",
        "code_block" => "components/markdown/code_block.html",
        "break" => "components/markdown/break.html",
        "thematic_break" => "components/markdown/thematic_break.html",
        _ => {
            todo!("Unknown node kind: {}", kind);
        }
    };

    // Load and render the template
    let template = env
        .get_template(template_path)
        .expect("Failed to get template");

    let rendered = template
        .render(context! { node => node })
        .expect("Failed to render template");

    Ok(rendered)
}
