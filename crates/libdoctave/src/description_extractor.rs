use crate::utils::capitalize;
use crate::Ast;

pub struct DescriptionExtractor {}

impl DescriptionExtractor {
    pub fn extract(ast: &Ast) -> String {
        match ast {
            Ast::OpenApi(openapi_ast) => {
                format!(
                    "API Documentation for {}",
                    capitalize(&openapi_ast.tag.name)
                )
            }
            Ast::Markdown(root) => {
                let inner_text = root.inner_text().replace('\n', " ");

                trim_string_at_word_boundary(&inner_text, 160)
            }
        }
    }
}

fn trim_string_at_word_boundary(s: &str, max_length: usize) -> String {
    // Check if the string is already within the desired length.
    if s.len() <= max_length {
        return s.to_string();
    }

    let mut last_space_index = None;
    let mut current_length = 0;

    for (i, c) in s.char_indices() {
        if c.is_whitespace() {
            last_space_index = Some(i);
        }

        current_length += c.len_utf8();
        if current_length > max_length {
            // If we have seen a space, we trim at the last space. Otherwise, we trim at the current position.
            return s[..last_space_index.unwrap_or(i)].to_string();
        }
    }

    // If the loop completes without returning, it means the string is shorter than max_length.
    s.to_string()
}
