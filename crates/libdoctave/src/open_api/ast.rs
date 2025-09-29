use itertools::Itertools;
use serde::Serialize;

use thiserror::Error;

use crate::{
    ast_mdx,
    error_renderer::{self, Highlight, Location},
    expressions::Value,
    primitive_components::{OPENAPI_PATH_KEY, TITLE_KEY},
    render_context::RenderContext,
    renderable_ast::Node,
    shared_ast::Position,
    Result,
};

use super::model::Schema as SchemaModel;

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
    ast_mdx(md, ctx)
}

#[derive(Debug, Clone, Serialize)]
pub struct Tag {
    pub name: String,
    pub description_ast: Option<Node>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageAst {
    pub tag: Tag,
    pub operations: Vec<OperationAst>,
    pub download_url: Option<String>,
}

impl PageAst {
    pub(crate) fn from_page(
        page: &super::model::Page,
        ctx: &crate::render_context::RenderContext,
    ) -> Result<Self> {
        let description_ast = if let Some(desc) = page.tag.description.as_ref() {
            Some(ast_for_openapi(desc, ctx)?)
        } else {
            None
        };

        let download_url = page.spec_download_link(ctx);

        let mut operations = vec![];
        for op in &page.operations {
            operations.push(OperationAst::from_model(op, ctx)?);
        }

        Ok(PageAst {
            tag: Tag {
                name: page.tag.name.to_owned(),
                description_ast,
            },
            operations,
            download_url,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
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
    pub(crate) fn from_model(
        operation: &super::model::Operation,
        ctx: &crate::render_context::RenderContext,
    ) -> Result<Self> {
        let description_ast = operation
            .description
            .as_ref()
            .and_then(|description| ast_for_openapi(description, ctx).ok());

        let mut header_params = vec![];
        for param in &operation.header_parameters {
            header_params.push(ParameterAst::from_model(
                param,
                ctx,
                &operation.identifier(),
            )?);
        }

        let mut query_params = vec![];
        for param in &operation.query_parameters {
            query_params.push(ParameterAst::from_model(
                param,
                ctx,
                &operation.identifier(),
            )?);
        }

        let mut path_params = vec![];
        for param in &operation.path_parameters {
            path_params.push(ParameterAst::from_model(
                param,
                ctx,
                &operation.identifier(),
            )?);
        }

        let mut cookie_params = vec![];
        for param in &operation.cookie_parameters {
            cookie_params.push(ParameterAst::from_model(
                param,
                ctx,
                &operation.identifier(),
            )?);
        }

        let mut responses = vec![];
        for response in &operation.responses {
            responses.push(StatusAst::from_model(response)?);
        }

        let request_body = if let Some(req_body) = &operation.request_body {
            Some(RequestBodyAst::from_model(
                req_body,
                &operation.identifier(),
            )?)
        } else {
            None
        };

        let mut security_requirements = vec![];
        for req in &operation.security_requirements {
            security_requirements.push(SecurityRequirementAst::from_model(req)?);
        }

        let mut request_examples = vec![];
        for code_sample in &operation.code_examples {
            request_examples.push(ExampleAst::from_model(
                code_sample,
                &prettify_language(&code_sample.name),
            )?);
        }

        let server_route_patterns = operation
            .servers
            .iter()
            .map(|s| {
                let url = s.url.trim_end_matches('/');
                let route = operation.route_pattern.trim_start_matches('/');
                format!("{}/{}", url, route)
            })
            .collect::<Vec<String>>();

        Ok(OperationAst {
            summary: operation.summary.clone(),
            description_ast,
            method: operation.method.clone(),
            anchor_tag: operation.anchor_tag.clone(),
            route_pattern: operation.route_pattern.clone(),
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

// Helper functions restored from the original view layer
fn prettify_language(lang: &str) -> String {
    if !lang.starts_with("application/")
        && !lang.starts_with("text/")
        && !lang.starts_with("audio/")
        && !lang.starts_with("video/")
        && !lang.starts_with("message/")
        && !lang.starts_with("multipart/")
    {
        crate::pretty_language_name(lang)
    } else {
        lang.to_string()
    }
}

fn language_aliases(lang: &str) -> String {
    match lang {
        "node" => "js".to_string(),
        "android" => "java".to_string(),
        _ => lang.to_string(),
    }
}

impl ExampleAst {
    pub(crate) fn from_model(example: &super::model::Example, parent_id: &str) -> Result<Self> {
        let language =
            if parent_id.starts_with("requestBody-") || parent_id.starts_with("response-") {
                // Split on the `application/` prefix
                parent_id.split('/').nth(1).unwrap_or("json").to_owned()
            } else {
                language_aliases(&example.name)
            };

        Ok(ExampleAst {
            name: example.name.clone(),
            summary: example.summary.clone(),
            description_ast: example.description.as_ref().and_then(|d| {
                ast_for_openapi(d, &crate::render_context::RenderContext::new()).ok()
            }),
            identifier: example.identifier(parent_id),
            value: example.value.clone(),
            group_name: parent_id.split('-').last().unwrap_or("default").to_string(),
            language: Some(language),
            rendered_value: None,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestBodyAst {
    pub description_ast: Option<Node>,
    pub media_types: Vec<MediaTypeAst>,
}

impl RequestBodyAst {
    pub(crate) fn from_model(
        request_body: &super::model::RequestBody,
        _operation_id: &str,
    ) -> Result<Self> {
        let mut media_types = vec![];
        for media_type in &request_body.content {
            media_types.push(MediaTypeAst::from_model(media_type, "requestBody")?);
        }

        Ok(RequestBodyAst {
            description_ast: request_body.description.as_ref().and_then(|d| {
                ast_for_openapi(d, &crate::render_context::RenderContext::new()).ok()
            }),
            media_types,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct StatusAst {
    pub code: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub media_types: Vec<MediaTypeAst>,
    pub description_ast: Node,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<HeaderAst>,
}

impl StatusAst {
    pub(crate) fn from_model(response: &super::model::Response) -> Result<Self> {
        let description_ast = ast_for_openapi(
            &response.description,
            &crate::render_context::RenderContext::new(),
        )
        .unwrap_or(ast_for_openapi("", &crate::render_context::RenderContext::new()).unwrap());

        let mut media_types = vec![];
        for media_type in &response.content {
            media_types.push(MediaTypeAst::from_model(
                media_type,
                &format!("response-{}", &response.status),
            )?);
        }

        let mut headers = vec![];
        for header in &response.headers {
            headers.push(HeaderAst::from_model(header)?);
        }

        Ok(StatusAst {
            code: response.status.clone(),
            media_types,
            headers,
            description_ast,
        })
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct HeaderAst {
    pub name: String,
    pub schema: SchemaAst,
}

impl HeaderAst {
    pub(crate) fn from_model(header: &super::model::Header) -> Result<Self> {
        Ok(HeaderAst {
            name: header.name.clone(),
            schema: SchemaAst::from_model(
                &header.schema,
                &crate::render_context::RenderContext::new(),
                false,
            )?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaTypeAst {
    pub name: String,
    pub schemas: Vec<SchemaAst>,
    pub examples: Vec<ExampleAst>,
}

impl MediaTypeAst {
    pub(crate) fn from_model(
        media_type: &super::model::MediaType,
        parent_id: &str,
    ) -> Result<Self> {
        let mut schemas = vec![];
        for schema in &media_type.schemas {
            schemas.push(SchemaAst::from_model(
                schema,
                &crate::render_context::RenderContext::new(),
                false,
            )?);
        }

        let mut examples = vec![];
        for example in &media_type.examples {
            examples.push(ExampleAst::from_model(
                example,
                &format!("{}-{}", parent_id, &media_type.name),
            )?);
        }

        Ok(MediaTypeAst {
            name: media_type.name.clone(),
            schemas,
            examples,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ParameterAst {
    pub schema: Option<SchemaAst>,
    pub description_ast: Option<Node>,
}

impl ParameterAst {
    pub(crate) fn from_model(
        parameter: &super::model::Parameter,
        ctx: &crate::render_context::RenderContext,
        _operation_id: &str,
    ) -> Result<Self> {
        let description_ast = parameter
            .description
            .as_ref()
            .and_then(|description| ast_for_openapi(description, ctx).ok());

        let schema = if let Some(s) = parameter.schema.as_ref() {
            Some(SchemaAst::from_model(s, ctx, false)?)
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
}

#[derive(Debug, Clone, Serialize)]
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
    pub(crate) fn from_model(requirement: &super::model::SecurityRequirement) -> Result<Self> {
        use super::model::SecurityRequirement::*;

        match requirement {
            Http {
                name,
                description,
                scheme,
                bearer_format,
            } => Ok(SecurityRequirementAst::Http {
                name: name.clone(),
                description_ast: description.as_ref().and_then(|d| {
                    ast_for_openapi(d, &crate::render_context::RenderContext::new()).ok()
                }),
                scheme: scheme.clone(),
                bearer_format: bearer_format.clone(),
            }),
            ApiKey {
                name,
                description,
                key_name,
                key_location,
            } => Ok(SecurityRequirementAst::ApiKey {
                name: name.clone(),
                description_ast: description.as_ref().and_then(|d| {
                    ast_for_openapi(d, &crate::render_context::RenderContext::new()).ok()
                }),
                key_name: key_name.clone(),
                key_location: key_location.clone(),
            }),
            OAuth2 {
                name,
                description,
                all_scopes,
                required_scopes,
                flows: _,
            } => Ok(SecurityRequirementAst::OAuth2 {
                name: name.clone(),
                description_ast: description.as_ref().and_then(|d| {
                    ast_for_openapi(d, &crate::render_context::RenderContext::new()).ok()
                }),
                all_scopes: all_scopes.clone(),
                required_scopes: required_scopes.clone(),
                flows: vec![], // TODO: Convert OAuth2Flow to OAuth2FlowAst
            }),
            OpenID {
                name,
                description,
                open_id_connect_url,
            } => Ok(SecurityRequirementAst::OpenID {
                name: name.clone(),
                description_ast: description.as_ref().and_then(|d| {
                    ast_for_openapi(d, &crate::render_context::RenderContext::new()).ok()
                }),
                open_id_connect_url: open_id_connect_url.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
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

#[cfg(test)]
mod test {
    use crate::{page_kind::PageKind, render_context::RenderContext, settings::Settings};

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
        let page_ast = PageAst::from_page(page, ctx).unwrap();

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
    fn responses() {
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
                    "identifier": "example-default-response-200-application/json",
                    "value": "{\n  \"id\": \"td123\",\n  \"projectId\": \"p456\",\n  \"state\": \"ACTIVE\",\n  \"dilationFactor\": 3.0,\n  \"creationDate\": \"2023-07-24T18:00:00Z\"\n}",
                    "rendered_value": null,
                    "group_name": "application/json",
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
        // NOTE: This needs better error handling/propagation to support correctly
        let json = parse_into_value();

        assert_eq!(
            json["operations"][2]["description_ast"]["kind"]["name"],
            json!(null)
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
        assert_eq!(
            request_examples[0]["identifier"],
            "example-default-requestBody-application/json"
        );
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

        assert_eq!(request_examples[0]["identifier"], "example-rust-Rust");
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

        let settings = Settings::default();
        let mut ctx = RenderContext::new();
        ctx.with_settings(&settings);

        let page_ast = PageAst::from_page(page, &ctx).unwrap();

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
