use itertools::Itertools;
use serde::Serialize;

use thiserror::Error;

#[cfg(test)]
use ts_rs::TS;

#[cfg(feature = "rustler")]
use rustler::{NifStruct, NifTaggedEnum};

use crate::{
    ast_mdx,
    error_renderer::{self, Highlight, Location},
    expressions::Value,
    markdown::ast,
    open_api::view::{Header, MediaType, Parameter, RequestBody, Schema, Status},
    primitive_components::{OPENAPI_PATH_KEY, TITLE_KEY},
    render_context::RenderContext,
    renderable_ast::Node,
    shared_ast::Position,
    Result,
};

use super::{
    model::Schema as SchemaModel,
    view::{
        Example, ExampleParent, MediaTypeParent, Operation, PageView, RequestExamples,
        SecurityRequirement,
    },
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Missing required attribute {TITLE_KEY}"#)]
    MissingTitle,
    #[error(r#"Missing required attribute {OPENAPI_PATH_KEY}"#)]
    MissingOpenApiPath,
    #[error(r#"Can't find an OpenAPI file with path "{0}""#)]
    ComponentsNotFound(String),
    #[error(r#"OpenAPI schema with title "{0}" not found"#)]
    SchemaNotFound(String),
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext, node_pos: &Position) -> String {
        let mut highlights = vec![];
        match self {
            Error::MissingTitle => {
                let location = Location::Point(node_pos.start.row, node_pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some(format!("Missing {TITLE_KEY}")),
                };

                highlights.push(highlight);
            }
            Error::MissingOpenApiPath => {
                let location = Location::Point(node_pos.start.row, node_pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some(format!("Missing {OPENAPI_PATH_KEY}")),
                };

                highlights.push(highlight);
            }
            Error::ComponentsNotFound(openapi_path) => {
                let pos = error_renderer::offset_attribute_error_pos(
                    md,
                    OPENAPI_PATH_KEY,
                    openapi_path,
                    node_pos,
                );

                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: openapi_path.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
            Error::SchemaNotFound(schema_title) => {
                let pos = error_renderer::offset_attribute_error_pos(
                    md,
                    TITLE_KEY,
                    schema_title,
                    node_pos,
                );

                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: schema_title.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
        }

        error_renderer::render(md, &self.to_string(), highlights, ctx)
    }
}

fn ast_for_openapi(md: &str, ctx: &RenderContext) -> Result<Node> {
    if ctx.settings.is_v2() {
        ast_mdx(md, ctx)
    } else {
        ast(md, ctx)
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.OpenApiAst.Tag")]
pub struct Tag {
    pub name: String,
    pub description_ast: Option<Node>,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.OpenApiAst.Page")]
pub struct PageAst {
    pub tag: Tag,
    pub operations: Vec<OperationAst>,
    pub download_url: Option<String>,
}

impl PageAst {
    pub(crate) fn from_view(view: &PageView) -> Result<Self> {
        let description_ast = if let Some(desc) = view.page.tag.description.as_ref() {
            Some(ast_for_openapi(desc, view.ctx)?)
        } else {
            None
        };

        let download_url = view.spec_download_link();

        let mut operations = vec![];
        for op in view.operations() {
            operations.push(OperationAst::from_view(&op)?);
        }

        Ok(PageAst {
            tag: Tag {
                name: view.page.tag.name.to_owned(),
                description_ast,
            },
            operations,
            download_url,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(
    feature = "rustler",
    module = "Doctave.Libdoctave.OpenApiAst.Operation"
)]
pub struct OperationAst {
    pub summary: Option<String>,
    pub description_ast: Option<Node>,
    pub method: String,
    pub anchor_tag: String,
    pub route_pattern: String,
    pub server_route_patterns: Vec<String>, // has server url prepended
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub header_params: Vec<ParameterAst>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub query_params: Vec<ParameterAst>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub path_params: Vec<ParameterAst>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cookie_params: Vec<ParameterAst>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBodyAst>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub responses: Vec<StatusAst>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub security_requirements: Vec<SecurityRequirementAst>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    /// These are code examples provided by `x-doctave > code-samples`
    pub request_examples: Vec<ExampleAst>,
}

impl OperationAst {
    pub(crate) fn from_view(view: &Operation) -> Result<Self> {
        let description_ast = view
            .description()
            .as_ref()
            .and_then(|description| ast_for_openapi(description, view.ctx).ok());

        let mut header_params = vec![];
        for param in view.header_parameters() {
            header_params.push(ParameterAst::from_view(&param)?);
        }

        let mut query_params = vec![];
        for param in view.query_parameters() {
            query_params.push(ParameterAst::from_view(&param)?);
        }

        let mut path_params = vec![];
        for param in view.path_parameters() {
            path_params.push(ParameterAst::from_view(&param)?);
        }

        let mut cookie_params = vec![];
        for param in view.cookie_parameters() {
            cookie_params.push(ParameterAst::from_view(&param)?);
        }

        let mut responses = vec![];
        for status in view.response_examples().statuses {
            responses.push(StatusAst::from_view(&status)?);
        }

        let request_body = if let Some(req_body) = view.request_body() {
            Some(RequestBodyAst::from_view(&req_body)?)
        } else {
            None
        };

        let mut security_requirements = vec![];
        for req in view.security_requirements() {
            security_requirements.push(SecurityRequirementAst::from_view(&req)?);
        }

        let mut request_examples = vec![];
        let _request_examples = view.request_examples();

        for code_sample in _request_examples.code_samples() {
            request_examples.push(ExampleAst::from_view(&code_sample.example())?);
        }

        let server_route_patterns = view
            .inner
            .servers
            .iter()
            .map(|s| {
                let url = s.url.trim_end_matches('/');
                let route = view.route_pattern().trim_start_matches('/');
                format!("{}/{}", url, route)
            })
            .collect::<Vec<String>>();

        Ok(OperationAst {
            summary: view.summary().map(|s| s.to_owned()),
            description_ast,
            method: view.method().to_owned(),
            anchor_tag: view.anchor_tag().to_owned(),
            route_pattern: view.route_pattern().to_owned(),
            header_params,
            query_params,
            path_params,
            cookie_params,
            request_body,
            responses,
            security_requirements,
            request_examples,
            server_route_patterns,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.OpenApiAst.Example")]
pub struct ExampleAst {
    name: String,
    summary: Option<String>,
    description_ast: Option<Node>,
    identifier: String,
    value: String,
    rendered_value: Option<String>,
    group_name: String,
    language: Option<String>,
}

impl ExampleAst {
    pub(crate) fn from_view(view: &Example) -> Result<Self> {
        let description_ast = view
            .description()
            .as_ref()
            .and_then(|description| ast_for_openapi(description, view.ctx).ok());

        let summary = if view.has_summary() {
            Some(view.summary().to_owned())
        } else {
            None
        };

        match view.parent_view {
            ExampleParent::MediaType(m) => match m.parent_view {
                MediaTypeParent::Status(status) => Ok(ExampleAst {
                    name: view.inner.name.to_owned(),
                    summary,
                    description_ast,
                    identifier: view.identifier().to_owned(),
                    value: view.value().to_owned(),
                    group_name: RequestExamples::_prettify_language(status.code()),
                    language: Some("json".to_string()),
                    rendered_value: None,
                }),
                MediaTypeParent::RequestBody(_) => Ok(ExampleAst {
                    name: view.inner.name.clone(),
                    summary,
                    description_ast,
                    identifier: view.identifier().to_owned(),
                    value: view.value().to_owned(),
                    group_name: RequestExamples::_prettify_language(m.name()),
                    language: Some("json".to_string()),
                    rendered_value: None,
                }),
            },
            ExampleParent::Request(code) => Ok(ExampleAst {
                name: view.name().to_owned(),
                summary,
                description_ast: None,
                identifier: view.identifier().to_owned(),
                value: view.value().to_owned(),
                group_name: RequestExamples::_prettify_language(code.language()),
                language: Some(RequestExamples::language_aliases(code.language())),
                rendered_value: None,
            }),
            ExampleParent::Schema(schema) => Ok(ExampleAst {
                name: view.name().to_owned(),
                summary,
                description_ast,
                identifier: view.identifier().to_owned(),
                value: view.value().to_owned(),
                group_name: RequestExamples::_prettify_language(&schema.type_name()),
                language: None,
                rendered_value: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(
    feature = "rustler",
    module = "Doctave.Libdoctave.OpenApiAst.RequestBody"
)]
pub struct RequestBodyAst {
    pub description_ast: Option<Node>,
    pub media_types: Vec<MediaTypeAst>,
}

impl RequestBodyAst {
    pub(crate) fn from_view(view: &RequestBody) -> Result<Self> {
        let description_ast = view
            .description()
            .as_ref()
            .and_then(|description| ast_for_openapi(description, view.ctx).ok());

        let mut media_types = vec![];
        for val in view.media_types() {
            media_types.push(MediaTypeAst::from_view(&val)?);
        }

        Ok(RequestBodyAst {
            description_ast,
            media_types,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.OpenApiAst.Status")]
pub struct StatusAst {
    pub code: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub media_types: Vec<MediaTypeAst>,
    pub description_ast: Node,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<HeaderAst>,
}

impl StatusAst {
    pub(crate) fn from_view(view: &Status) -> Result<Self> {
        let description_ast = ast_for_openapi(view.description(), view.ctx)
            .unwrap_or(ast_for_openapi("", view.ctx).unwrap());

        let mut media_types = vec![];
        for val in view.media_types() {
            media_types.push(MediaTypeAst::from_view(&val)?);
        }

        let mut headers = vec![];
        for val in view.headers() {
            headers.push(HeaderAst::from_view(&val)?);
        }

        Ok(StatusAst {
            code: view.code().to_owned(),
            media_types,
            headers,
            description_ast,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.OpenApiAst.Header")]
pub struct HeaderAst {
    pub name: String,
    pub schema: SchemaAst,
}

impl HeaderAst {
    pub(crate) fn from_view(view: &Header) -> Result<Self> {
        Ok(HeaderAst {
            name: view.name().to_owned(),
            schema: SchemaAst::from_view(&view.schema())?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(
    feature = "rustler",
    module = "Doctave.Libdoctave.OpenApiAst.MediaType"
)]
pub struct MediaTypeAst {
    pub name: String,
    pub schemas: Vec<SchemaAst>,
    pub examples: Vec<ExampleAst>,
}

impl MediaTypeAst {
    pub(crate) fn from_view(view: &MediaType) -> Result<Self> {
        let mut schemas = vec![];
        for val in view.schemas() {
            schemas.push(SchemaAst::from_view(&val)?);
        }

        let mut examples = vec![];
        for val in view.examples() {
            examples.push(ExampleAst::from_view(&val)?);
        }

        Ok(MediaTypeAst {
            name: view.name().to_owned(),
            schemas,
            examples,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(
    feature = "rustler",
    module = "Doctave.Libdoctave.OpenApiAst.Parameter"
)]
pub struct ParameterAst {
    pub schema: Option<SchemaAst>,
    pub description_ast: Option<Node>,
}

impl ParameterAst {
    pub(crate) fn from_view(view: &Parameter) -> Result<Self> {
        let description_ast = view
            .description()
            .as_ref()
            .and_then(|description| ast_for_openapi(description, view.ctx).ok());

        let schema = if let Some(s) = view.schema().as_ref() {
            Some(SchemaAst::from_view(s)?)
        } else {
            None
        };

        Ok(ParameterAst {
            schema,
            description_ast,
        })
    }
}

fn is_false(input: &bool) -> bool {
    !input
}

fn no_enums(input: &Option<Vec<Option<String>>>) -> bool {
    if let Some(enums) = input {
        enums.is_empty()
    } else {
        true
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.OpenApiAst.Schema")]
pub struct SchemaAst {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub schemas: Vec<SchemaAst>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_ast: Option<Box<Node>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub deprecated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "no_enums")]
    pub enumeration: Option<Vec<Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub combination_explanation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub expanded: bool,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.OpenApiAst.Metadata")]
pub struct Metadata {
    pub field_name: Option<String>,
    pub component_name: Option<String>,
    pub recursive: Option<bool>,
    pub title: Option<String>,
}

impl From<openapi_parser::Metadata> for Metadata {
    fn from(value: openapi_parser::Metadata) -> Self {
        Metadata {
            field_name: value.field_name.map(|s| s.to_string()),
            component_name: value.component_name.as_ref().map(|s| s.to_string()),
            recursive: if value.component_name.is_some() {
                Some(value.recursive.is_some())
            } else {
                None
            },
            title: value.title.as_ref().map(|s| s.to_string()),
        }
    }
}

impl SchemaAst {
    pub(crate) fn try_new_schema<'ctx>(
        title: Option<Value>,
        path: Option<Value>,
        ctx: &'ctx RenderContext,
    ) -> std::result::Result<&'ctx SchemaModel, Error> {
        let openapi_path = path.ok_or(Error::MissingOpenApiPath)?.to_string();
        let title = title.ok_or(Error::MissingTitle)?.to_string();

        let components = ctx
            .openapi_components
            .get(&openapi_path)
            .ok_or(Error::ComponentsNotFound(openapi_path))?;

        components
            .schemas
            .get(&title)
            .ok_or(Error::SchemaNotFound(title))
    }

    pub(crate) fn from_model(
        model: &SchemaModel,
        ctx: &RenderContext,
        expanded: bool,
    ) -> Result<Self> {
        let description_ast = model
            .description
            .as_ref()
            .and_then(|description| ast_for_openapi(description, ctx).ok());

        let schemas: Vec<SchemaAst> = model
            .nested_schemas()
            .iter()
            .map(|s| SchemaAst::from_model(s, ctx, s.expanded))
            .try_collect()?;

        let mut schema = SchemaAst {
            schemas,
            required: model.required,
            description_ast: description_ast.map(Box::new),
            title: model.title.as_ref().map(|v| v.to_owned()),
            type_name: model.type_name(),
            format: model.format().map(|v| v.to_owned()),
            pattern: model.pattern().map(|v| v.to_owned()),
            default: model
                .default()
                .map(|v| serde_json::to_string_pretty(v).unwrap()),
            deprecated: model.deprecated,
            minimum: model.minimum(),
            maximum: model.maximum(),
            multiple_of: model.multiple_of(),
            min_length: model.min_length(),
            max_length: model.max_length(),
            enumeration: model.enumeration().map(|v| v.to_owned()),
            example_string: model.example_string().map(|v| v.to_owned()),
            combination_explanation: None,
            media_type: model.mediatype.as_ref().map(|v| v.to_owned()),
            metadata: model.metadata.as_ref().map(|m| m.clone().into()),
            expanded,
        };

        if model.is_nested() {
            schema.combination_explanation = Some(model.combination_explanation().to_owned());
        }

        Ok(schema)
    }

    pub(crate) fn from_view(view: &Schema) -> Result<Self> {
        let description_ast = view
            .description()
            .as_ref()
            .and_then(|description| ast_for_openapi(description, view.ctx).ok());

        let mut schemas = vec![];
        for schema in view.schemas() {
            schemas.push(SchemaAst::from_view(&schema)?);
        }

        let mut schema = SchemaAst {
            schemas,
            required: view.required(),
            description_ast: description_ast.map(Box::new),
            title: view.title().map(|v| v.to_owned()),
            type_name: view.type_name(),
            format: view.format().map(|v| v.to_owned()),
            pattern: view.pattern().map(|v| v.to_owned()),
            default: view
                .default()
                .map(|v| serde_json::to_string_pretty(v).unwrap()),
            deprecated: view.deprecated(),
            minimum: view.minimum(),
            maximum: view.maximum(),
            multiple_of: view.multiple_of(),
            min_length: view.min_length(),
            max_length: view.max_length(),
            enumeration: view.enumeration().map(|v| v.to_owned()),
            example_string: view.example_string().map(|v| v.to_owned()),
            combination_explanation: None,
            media_type: view.media_type().map(|v| v.to_owned()),
            metadata: view.inner.metadata.as_ref().map(|m| m.clone().into()),
            expanded: view.inner.expanded,
        };

        if view.has_child_schemas() {
            schema.combination_explanation = Some(view.combination_explanation().to_owned());
        }

        Ok(schema)
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifTaggedEnum))]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum SecurityRequirementAst {
    Http {
        name: String,
        description_ast: Option<Node>,
        scheme: String,
        bearer_format: Option<String>,
    },
    ApiKey {
        name: String,
        description_ast: Option<Node>,
        key_name: String,
        key_location: String,
    },
    #[cfg_attr(test, ts(rename = "o_auth2"))]
    OAuth2 {
        name: String,
        description_ast: Option<Node>,
        all_scopes: Vec<(String, String)>,
        required_scopes: Vec<String>,
        flows: Vec<OAuth2FlowAst>,
    },
    OpenID {
        name: String,
        description_ast: Option<Node>,
        open_id_connect_url: String,
    },
}

impl SecurityRequirementAst {
    pub(crate) fn from_view(view: &SecurityRequirement) -> Result<Self> {
        use super::model::SecurityRequirement::*;

        match &view.inner {
            Http {
                name,
                description,
                scheme,
                bearer_format,
            } => {
                let description_ast = if let Some(desc) = description.as_ref() {
                    ast_for_openapi(desc, view.ctx).ok()
                } else {
                    None
                };

                Ok(SecurityRequirementAst::Http {
                    name: name.clone(),
                    description_ast,
                    scheme: scheme.clone(),
                    bearer_format: bearer_format.clone(),
                })
            }
            ApiKey {
                name,
                description,
                key_name,
                key_location,
            } => {
                let description_ast = if let Some(desc) = description.as_ref() {
                    ast_for_openapi(desc, view.ctx).ok()
                } else {
                    None
                };

                Ok(SecurityRequirementAst::ApiKey {
                    name: name.clone(),
                    description_ast,
                    key_name: key_name.clone(),
                    key_location: key_location.clone(),
                })
            }
            OAuth2 {
                name,
                description,
                all_scopes,
                required_scopes,
                flows,
            } => {
                let description_ast = if let Some(desc) = description.as_ref() {
                    ast_for_openapi(desc, view.ctx).ok()
                } else {
                    None
                };

                let mut flows_copy = vec![];
                for flow in flows {
                    flows_copy.push(OAuth2FlowAst::from_model(flow)?);
                }

                Ok(SecurityRequirementAst::OAuth2 {
                    name: name.clone(),
                    description_ast,
                    all_scopes: all_scopes.clone(),
                    required_scopes: required_scopes.clone(),
                    flows: flows_copy,
                })
            }
            OpenID {
                name,
                description,
                open_id_connect_url,
            } => {
                let description_ast = if let Some(desc) = description.as_ref() {
                    ast_for_openapi(desc, view.ctx).ok()
                } else {
                    None
                };

                Ok(SecurityRequirementAst::OpenID {
                    name: name.clone(),
                    description_ast,
                    open_id_connect_url: open_id_connect_url.clone(),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifTaggedEnum))]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum OAuth2FlowAst {
    Implicit {
        authorization_url: String,
        refresh_url: Option<String>,
        scopes: Vec<(String, String)>,
    },
    Password {
        refresh_url: Option<String>,
        token_url: String,
        scopes: Vec<(String, String)>,
    },
    ClientCredentials {
        refresh_url: Option<String>,
        token_url: String,
        scopes: Vec<(String, String)>,
    },
    AuthorizationCode {
        authorization_url: String,
        token_url: String,
        refresh_url: Option<String>,
        scopes: Vec<(String, String)>,
    },
}

impl OAuth2FlowAst {
    /// Yes, this is different from the rest. The `View` version is not very ergonimic for this
    /// translation, let's use the model underneath.
    fn from_model(model: &super::model::OAuth2Flow) -> Result<OAuth2FlowAst> {
        use super::model::OAuth2Flow::*;
        match model {
            Implicit {
                authorization_url,
                refresh_url,
                scopes,
            } => Ok(OAuth2FlowAst::Implicit {
                authorization_url: authorization_url.clone(),
                refresh_url: refresh_url.clone(),
                scopes: scopes.clone().into_iter().collect(),
            }),
            Password {
                refresh_url,
                token_url,
                scopes,
            } => Ok(OAuth2FlowAst::Password {
                token_url: token_url.clone(),
                refresh_url: refresh_url.clone(),
                scopes: scopes.clone().into_iter().collect(),
            }),
            ClientCredentials {
                refresh_url,
                token_url,
                scopes,
            } => Ok(OAuth2FlowAst::ClientCredentials {
                token_url: token_url.clone(),
                refresh_url: refresh_url.clone(),
                scopes: scopes.clone().into_iter().collect(),
            }),
            AuthorizationCode {
                authorization_url,
                token_url,
                refresh_url,
                scopes,
            } => Ok(OAuth2FlowAst::AuthorizationCode {
                authorization_url: authorization_url.clone(),
                token_url: token_url.clone(),
                refresh_url: refresh_url.clone(),
                scopes: scopes.clone().into_iter().collect(),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{page_kind::PageKind, render_context::RenderContext, settings::Settings};

    use super::super::view::PageView;
    use super::super::OpenApi;
    use super::PageAst;

    use indoc::indoc;
    use pretty_assertions::{assert_eq, assert_str_eq};
    use serde_json::{json, Value};

    static SPEC: &str = indoc! {r#"
        openapi: 3.0.0
        info:
          title: Sample API
          description: Optional multiline or single-line description in [CommonMark](http://commonmark.org/help/) or HTML.
          version: 0.1.9
        servers:
          - url: http://api.example.com/v1
            description: Optional server description, e.g. Main (production) server
          - url: http://staging-api.example.com
            description: Optional server description, e.g. Internal staging server for testing
        tags:
          - name: Users
            description: Some **info** about users
        paths:
          /users/{userId}:
            get:
              summary: Returns a list of users.
              description: Optional extended description in CommonMark or HTML.
              tags:
                - Users
              parameters:
                - name: userId
                  in: path
                  schema:
                    type: integer
                  required: true
                  description: Numeric ID of the user to get
              x-doctave:
                code-samples:
                  - language: rust
                    code: |
                      // Rust code example
                  - language: javascript
                    code: |
                      // JavaScript code example
                  - language: python
                    code: |
                      # Python code example
                  - language: ruby
                    code: |
                      # Ruby code example
                  - language: go
                    code: |
                      // Go code example
                  - language: node
                    code: |
                      // Node (js) code example
                  - language: android
                    code: |
                      // Android (java) code example
              responses:
                '200':    # status code
                  description: A JSON array of user names
                  content:
                    application/json:
                      schema:
                        type: array
                        items:
                          type: string
          /projects/{projectId}/timeDilation:
            post:
              tags:
                - Users
              summary: Create a new time-dilation instance for a project
              description: Creates a new time-dilation instance for a specific project. This instance affects the perceived timeline for the code files in the project.
              parameters:
                - name: projectId
                  in: path
                  required: true
                  schema:
                    type: string
              requestBody:
                required: true
                description: Description of request body
                content:
                  application/json:
                    schema:
                      $ref: '#/components/schemas/TimeDilation'
                    examples:
                      default:
                        summary: Some Summary
                        description: Some Fake Description
                        value:
                          dilationFactor: 2.0
              responses:
                '200':
                  description: Time-dilation instance updated
                  headers:
                    X-RateLimit-Limit:
                      schema:
                        type: integer
                      description: Request limit per hour.
                  content:
                    application/json:
                      schema:
                        $ref: '#/components/schemas/TimeDilation'
                      examples:
                        default:
                          value:
                            id: "td123"
                            projectId: "p456"
                            state: "ACTIVE"
                            dilationFactor: 3.0
                            creationDate: "2023-07-24T18:00:00Z"
                '400':
                  description: Bad request
                '404':
                  description: Time-dilation instance not found
          /users/:
            post:
              summary: Returns a list of users.
              description: <bad> html inside </markdown>
              tags:
                - Users
              responses:
                '200':    # status code
                  description: A JSON array of user names
                  content:
                    application/json:
                      schema:
                        type: array
                        items:
                          type: string
        components:
          schemas:
            TimeDilation:
              type: object
              properties:
                id:
                  type: string
                projectId:
                  type: string
                state:
                  type: string
                  enum: [ACTIVE, INACTIVE]
                dilationFactor:
                  type: number
                creationDate:
                  type: string
                  format: date-time

    "#};

    fn parse_into_value() -> serde_json::Value {
        let spec = openapi_parser::openapi30::parser::parse_yaml(SPEC).unwrap();
        let pages =
            OpenApi::pages_from_parsed_spec(&spec, "openapi.yaml".into(), "/api".into()).unwrap();

        let tag_pages = pages
            .into_iter()
            .filter_map(|p| match p {
                PageKind::OpenApi(p) => Some(p.get_page().clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let page = &tag_pages[0];

        let ctx = &RenderContext::new();
        let view = PageView::new(page, ctx);
        let page_ast = PageAst::from_view(&view).unwrap();

        serde_json::to_value(page_ast).unwrap()
    }

    #[test]
    fn tag_name_and_description_ast() {
        let json = parse_into_value();

        assert_eq!(json["tag"]["name"].as_str().unwrap(), "Users");
        assert_eq!(json["tag"]["description_ast"]["kind"]["name"], "root");
        assert_eq!(
            json["tag"]["description_ast"]["children"][0]["children"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
    }

    #[test]
    fn operations() {
        let json = parse_into_value();

        assert_eq!(json["operations"][0]["summary"], "Returns a list of users.");
        assert_eq!(
            json["operations"][0]["description_ast"]["children"][0]["children"][0]["kind"]["data"]
                ["value"],
            "Optional extended description in CommonMark or HTML."
        );
        assert_eq!(json["operations"][0]["method"], "get");
        assert_eq!(json["operations"][0]["route_pattern"], "/users/{userId}");
    }

    #[test]
    fn param_schema() {
        let mut json = parse_into_value();

        let param = &mut json["operations"][0]["path_params"][0];
        let schema = &mut param["schema"];

        // Check description separately because the type is annoying to write out
        let description = schema["description_ast"].take();
        assert_eq!(
            description["children"][0]["children"][0]["kind"]["data"]["value"],
            "Numeric ID of the user to get"
        );

        assert_eq!(
            *schema,
            json!({"required": true, "description_ast": null, "title": "userId", "type_name": "int", "metadata": {"field_name": "userId", "component_name": null, "recursive": null, "title": null}, "expanded": false})
        );
    }

    #[test]
    fn request_body() {
        let json = parse_into_value();

        let req_body = &json["operations"][1]["request_body"];
        assert_eq!(
            req_body["description_ast"]["children"][0]["children"][0]["kind"]["data"]["value"],
            "Description of request body"
        );

        let media_type = &req_body["media_types"][0];

        assert_eq!(media_type["name"], "application/json");

        assert_eq!(
            media_type["schemas"],
            json!([
              {
                "required": false,
                "title": "id",
                "type_name": "string",
                "metadata": {"field_name": "id", "component_name": null, "recursive": null, "title": null},
                "expanded": false,
              },
              {
                "required": false,
                "title": "projectId",
                "type_name": "string",
                "metadata": {"field_name": "projectId", "component_name": null, "recursive": null, "title": null},
                "expanded": false,
              },
              {
                "required": false,
                "title": "state",
                "type_name": "string",
                "enumeration": [
                  "ACTIVE",
                  "INACTIVE"
                ],
                "metadata": {"field_name": "state", "component_name": null, "recursive": null, "title": null},
                "expanded": false,
              },
              {
                "required": false,
                "title": "dilationFactor",
                "type_name": "number",
                "metadata": {"field_name": "dilationFactor", "component_name": null, "recursive": null, "title": null},
                "expanded": false,
              },
              {
                "required": false,
                "title": "creationDate",
                "type_name": "string",
                "format": "date-time",
                "metadata": {"field_name": "creationDate", "component_name": null, "recursive": null, "title": null},
                "expanded": false,
              }
            ])
        );
    }

    #[test]
    fn resposes() {
        let mut json = parse_into_value();

        let response_200 = &mut json["operations"][1]["responses"][0];
        // Check description separately because the type is annoying to write out
        let description = response_200["description_ast"].take();
        assert_eq!(
            description["children"][0]["children"][0]["kind"]["data"]["value"],
            "Time-dilation instance updated"
        );

        assert_eq!(response_200["code"], "200");
        assert_eq!(
            response_200["media_types"],
            json!([{
               "name": "application/json",
               "schemas": [
                 {
                   "required": false,
                   "title": "id",
                   "type_name": "string",
                   "metadata": {"field_name": "id", "component_name": null, "recursive": null, "title": null},
                   "expanded": false,
                 },
                 {
                   "required": false,
                   "title": "projectId",
                   "type_name": "string",
                   "metadata": {"field_name": "projectId", "component_name": null, "recursive": null, "title": null},
                   "expanded": false,
                 },
                 {
                   "required": false,
                   "title": "state",
                   "type_name": "string",
                   "enumeration": [
                     "ACTIVE",
                     "INACTIVE"
                   ],
                   "metadata": {"field_name": "state", "component_name": null, "recursive": null, "title": null},
                   "expanded": false,
                 },
                 {
                   "required": false,
                   "title": "dilationFactor",
                   "type_name": "number",
                   "metadata": {"field_name": "dilationFactor", "component_name": null, "recursive": null, "title": null},
                   "expanded": false,
                 },
                 {
                   "required": false,
                   "title": "creationDate",
                   "type_name": "string",
                   "format": "date-time",
                   "metadata": {"field_name": "creationDate", "component_name": null, "recursive": null, "title": null},
                   "expanded": false,
                 }
               ],
               "examples": [
                  {
                    "name": "default",
                    "summary": "application/json",
                    "description_ast": null,
                    "identifier": "example-default-application/json-200",
                    "value": "{\n  \"id\": \"td123\",\n  \"projectId\": \"p456\",\n  \"state\": \"ACTIVE\",\n  \"dilationFactor\": 3.0,\n  \"creationDate\": \"2023-07-24T18:00:00Z\"\n}", "rendered_value": null, "group_name": "200",
                    "language": "json"
                  }
                ]
            }])
        );

        let response_400 = &mut json["operations"][1]["responses"][1];
        assert_eq!(response_400["code"], "400");
        let description = response_400["description_ast"].take();
        assert_eq!(
            description["children"][0]["children"][0]["kind"]["data"]["value"],
            "Bad request"
        );

        let response_404 = &mut json["operations"][1]["responses"][2];
        assert_eq!(response_404["code"], "404");
        let description = response_404["description_ast"].take();
        assert_eq!(
            description["children"][0]["children"][0]["kind"]["data"]["value"],
            "Time-dilation instance not found"
        );
    }

    #[test]
    fn respose_headers() {
        let mut json = parse_into_value();

        let response_200 = &mut json["operations"][1]["responses"][0];
        assert_eq!(response_200["code"], "200");
        assert_eq!(
            response_200["headers"],
            json!([{
                 "name": "X-RateLimit-Limit",
                 "schema":{
                 "description_ast": {
                   "kind": {
                     "name": "root"
                   },
                   "children": [
                     {
                       "kind": {
                         "name": "paragraph"
                       },
                       "children": [
                         {
                           "kind": {
                             "name": "text",
                             "data": {
                               "value": "Request limit per hour."
                             }
                           },
                           "children": [],
                         }
                       ]
                     }
                   ]
                 },
                 "title": "X-RateLimit-Limit",
                 "type_name": "int",
                 "metadata": {
                    "field_name": "X-RateLimit-Limit",
                    "component_name": null,
                    "recursive": null,
                    "title": null,
                 },
                 "expanded": false,
               }
            }])
        );
    }

    #[test]
    fn temporarily_ignore_markdown_errors_in_openapi_docs() {
        let json = parse_into_value();

        assert_eq!(
            json["operations"][2]["description_ast"]["kind"]["name"],
            "root"
        );
    }

    #[test]
    fn http_request_examples() {
        let mut json = parse_into_value();

        let request_examples =
            &mut json["operations"][1]["request_body"]["media_types"][0]["examples"];

        assert_eq!(request_examples[0]["name"], "default");
        assert_eq!(request_examples[0]["group_name"], "application/json");

        assert_eq!(request_examples[0]["summary"], "application/json");
        let description = request_examples[0]["description_ast"].take();

        assert_eq!(
            description["children"][0]["children"][0]["kind"]["data"]["value"],
            "Some Fake Description"
        );
        assert_eq!(request_examples[0]["identifier"], "example-default-application/json-request-body-post-/projects/{projectId}/timeDilation-Create-a-new-time-dilation-instance-for-a-project");
        assert_eq!(
            request_examples[0]["value"],
            "{\n  \"dilationFactor\": 2.0\n}"
        );
    }

    #[test]
    fn code_request_examples() {
        let mut json = parse_into_value();

        let request_examples = &mut json["operations"][0]["request_examples"];

        assert_eq!(request_examples[0]["name"], "rust");
        assert_eq!(request_examples[0]["group_name"], "Rust");

        assert_eq!(request_examples[0]["summary"], Value::Null);

        assert_eq!(request_examples[0]["identifier"], "example-rust-code");
        assert_eq!(request_examples[0]["value"], "// Rust code example\n");
        assert_eq!(request_examples[0]["language"], "rust");
    }

    #[test]
    fn code_request_examples_aliases() {
        let mut json = parse_into_value();

        let request_examples = &mut json["operations"][0]["request_examples"];

        let node_example = request_examples
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["name"] == "node")
            .unwrap();

        assert_eq!(node_example["name"], "node");
        assert_eq!(node_example["language"], "js");

        let android_example = request_examples
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["name"] == "android")
            .unwrap();

        assert_eq!(android_example["name"], "android");
        assert_eq!(android_example["language"], "java");
    }

    #[test]
    fn v2_markdown_descriptions() {
        let spec = indoc! {r#"
          openapi: 3.0.0
          info:
            title: Sample API
            description: <Card>Some **info** about the API</Card>
            version: 0.1.9
          servers:
            - url: http://api.example.com/v1
              description: Optional server description, e.g. Main (production) server
            - url: http://staging-api.example.com
              description: Optional server description, e.g. Internal staging server for testing
          tags:
            - name: Users
              description: <Callout>Tag description</Callout>
          paths:
            /users/:
              post:
                summary: Returns a list of users.
                description: <Box>Operation description</Box>
                tags:
                  - Users
                responses:
                  '200':    # status code
                    description: <Card>Response description</Card>
                    content:
                      application/json:
                        examples:
                          default:
                            description: <Card>Example description</Card>
                            value: |
                              {
                                "dilationFactor": 2.0
                              }
                        schema:
                          type: array
                          items:
                            type: string
      "#};

        let spec = openapi_parser::openapi30::parser::parse_yaml(spec).unwrap();
        let pages =
            OpenApi::pages_from_parsed_spec(&spec, "openapi.yaml".into(), "/api".into()).unwrap();

        let tag_pages = pages
            .into_iter()
            .filter_map(|p| match p {
                PageKind::OpenApi(p) => Some(p.get_page().clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let page = &tag_pages[0];

        let settings = Settings::V2(Box::default());
        let mut ctx = RenderContext::new();
        ctx.with_settings(&settings);

        let view = PageView::new(page, &ctx);
        let page_ast = PageAst::from_view(&view).unwrap();

        let tag_desc = page_ast.tag.description_ast.unwrap();

        assert_str_eq!(
            tag_desc.debug_string().unwrap(),
            indoc! { r#"
            <Box padding={2} max_width={full} class={d-callout d-callout-info} height={Auto}>
                <Text>
                    Tag description
                </Text>
            </Box>
            "# }
        );

        let operation = page_ast.operations.first().unwrap();

        let op_desc = operation.description_ast.as_ref().unwrap();

        assert_str_eq!(
            op_desc.debug_string().unwrap(),
            indoc! { r#"
            <Box padding={0} max_width={full} class={} height={Auto}>
                <Text>
                    Operation description
                </Text>
            </Box>
            "# }
        );

        let response = operation.responses.first().unwrap();

        assert_str_eq!(
            response.description_ast.debug_string().unwrap(),
            indoc! { r#"
            <Box padding={3} max_width={full} class={d-card } height={Auto}>
                <Text>
                    Response description
                </Text>
            </Box>
            "# }
        );

        let json_media_type = response.media_types.first().unwrap();
        let example = json_media_type.examples.first().unwrap();

        assert_str_eq!(
            example
                .description_ast
                .as_ref()
                .unwrap()
                .debug_string()
                .unwrap(),
            indoc! { r#"
            <Box padding={3} max_width={full} class={d-card } height={Auto}>
                <Text>
                    Example description
                </Text>
            </Box>
            "# }
        );
    }
}
