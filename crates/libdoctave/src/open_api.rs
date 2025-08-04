pub mod ast;
pub mod model;
pub mod overview;

use std::path::PathBuf;

use model::Components;
use model::Operation;
use model::Page;
use model::Tag;

use crate::page_kind::PageKind;
use crate::slug;

#[derive(Clone)]
/// Represents currently a v3 openapi spec. Can be used to generate pages based on
/// tags that can then be rendered out.
pub(crate) struct OpenApi {}

impl OpenApi {
    pub fn pages_from_parsed_spec(
        spec: &openapi_parser::OpenAPI,
        source: PathBuf,
        uri_path: String,
    ) -> crate::Result<Vec<PageKind>> {
        let mut tag_pages = vec![];
        let mut pages = vec![];

        let security_schemes = spec.components.as_ref().map(|c| c.security_schemes.clone());

        let mut all_tags = vec![];

        all_tags.extend(&spec.tags);

        let inline_tags = spec
            .operations()
            .iter()
            .flat_map(|op| {
                op.tags
                    .iter()
                    .map(|t| openapi_parser::Tag {
                        name: t.to_owned(),
                        description: None,
                        external_docs: None,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        for inline_tag in &inline_tags {
            if !all_tags.iter().any(|t| t.name == inline_tag.name) {
                all_tags.push(inline_tag);
            }
        }

        for tag in all_tags {
            let mut tag_operations = vec![];

            for (pattern, path_item) in spec.paths.iter() {
                if let Some(get) = &path_item.get {
                    if get.tags.iter().any(|t| &tag.name == t) {
                        let desc = get.description.as_ref().map(|v| v.to_string());
                        tag_operations.push(Operation::from_parsed(
                            get.clone(),
                            "get".to_string(),
                            pattern.to_string(),
                            desc,
                            &security_schemes,
                        )?);
                    }
                }

                if let Some(post) = &path_item.post {
                    if post.tags.iter().any(|t| &tag.name == t) {
                        let desc = post.description.as_ref().map(|v| v.to_string());
                        tag_operations.push(Operation::from_parsed(
                            post.clone(),
                            "post".to_string(),
                            pattern.to_string(),
                            desc,
                            &security_schemes,
                        )?);
                    }
                }

                if let Some(put) = &path_item.put {
                    if put.tags.iter().any(|t| &tag.name == t) {
                        let desc = put.description.as_ref().map(|v| v.to_string());
                        tag_operations.push(Operation::from_parsed(
                            put.clone(),
                            "put".to_string(),
                            pattern.to_string(),
                            desc,
                            &security_schemes,
                        )?);
                    }
                }

                if let Some(delete) = &path_item.delete {
                    if delete.tags.iter().any(|t| &tag.name == t) {
                        let desc = delete.description.as_ref().map(|v| v.to_string());
                        tag_operations.push(Operation::from_parsed(
                            delete.clone(),
                            "delete".to_string(),
                            pattern.to_string(),
                            desc,
                            &security_schemes,
                        )?);
                    }
                }

                if let Some(patch) = &path_item.patch {
                    if patch.tags.iter().any(|t| &tag.name == t) {
                        let desc = patch.description.as_ref().map(|v| v.to_string());
                        tag_operations.push(Operation::from_parsed(
                            patch.clone(),
                            "patch".to_string(),
                            pattern.to_string(),
                            desc,
                            &security_schemes,
                        )?);
                    }
                }

                if let Some(head) = &path_item.head {
                    if head.tags.iter().any(|t| &tag.name == t) {
                        let desc = head.description.as_ref().map(|v| v.to_string());
                        tag_operations.push(Operation::from_parsed(
                            head.clone(),
                            "head".to_string(),
                            pattern.to_string(),
                            desc,
                            &security_schemes,
                        )?);
                    }
                }

                if let Some(options) = &path_item.options {
                    if options.tags.iter().any(|t| &tag.name == t) {
                        let desc = options.description.as_ref().map(|v| v.to_string());
                        tag_operations.push(Operation::from_parsed(
                            options.clone(),
                            "options".to_string(),
                            pattern.to_string(),
                            desc,
                            &security_schemes,
                        )?);
                    }
                }

                if let Some(trace) = &path_item.trace {
                    if trace.tags.iter().any(|t| &tag.name == t) {
                        let desc = trace.description.as_ref().map(|v| v.to_string());
                        tag_operations.push(Operation::from_parsed(
                            trace.clone(),
                            "trace".to_string(),
                            pattern.to_string(),
                            desc,
                            &security_schemes,
                        )?);
                    }
                }
            }
            tag_pages.push(Page {
                tag: Tag {
                    name: tag.name.to_string(),
                    description: tag.description.as_ref().map(|d| d.to_string()),
                },
                operations: tag_operations,
                uri_path: format!(
                    "/{}/{}",
                    uri_path.strip_prefix('/').unwrap_or(&uri_path),
                    slug::slugify(tag.name.strip_prefix('/').unwrap_or(&tag.name))
                ),
                fs_path: source.clone(),
            });
        }

        for webhook in &spec.webhooks {
            let webhook_op =
                Operation::from_parsed_webhook(webhook.operation.clone(), &security_schemes)?;

            for page in &mut tag_pages {
                if webhook_op.tags.iter().any(|t| &page.tag.name == t) {
                    page.operations.push(webhook_op.clone());
                }
            }
        }

        for model in tag_pages {
            if model.operations.is_empty() {
                continue;
            }
            pages.push(PageKind::OpenApi(crate::OpenApiPage::new(model)));
        }

        let overview_page_markdown = overview::build_parsed(
            &spec.info,
            spec.servers.as_slice(),
            spec.external_docs.as_ref(),
        );

        let mut overview = crate::MarkdownPage::new(&source, overview_page_markdown.into_bytes());
        // Override the URI path
        overview.uri_path = uri_path;
        pages.push(PageKind::Markdown(overview));

        Ok(pages)
    }

    pub fn components_parsed(spec: &mut openapi_parser::OpenAPI) -> crate::Result<Components> {
        let components = spec
            .components
            .take()
            .map(Components::from_parsed)
            .transpose()?
            .unwrap_or_default();

        Ok(components)
    }
}
