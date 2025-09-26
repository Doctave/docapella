use super::ast::PageAst;
use indexmap::IndexMap;
use openapi_parser::openapi30::schemas::parameter::ParameterKind;
use serde::Serialize;
use serde_json::{json, Value};

use crate::{markdown, page_kind::OutgoingLink, render_context::RenderContext, Error};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub(crate) struct Components {
    pub schemas: HashMap<String, Schema>,
}

impl Components {
    pub(crate) fn from_parsed(spec: openapi_parser::Components) -> crate::Result<Self> {
        let mut schemas = HashMap::new();

        for (name, schema) in spec.schemas {
            schemas.insert(
                name.into(),
                Schema::from_parsed(schema, None, None, None, None, false)?,
            );
        }

        Ok(Self { schemas })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Tag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Page {
    pub uri_path: String,
    pub fs_path: PathBuf,
    pub tag: Tag,
    pub operations: Vec<Operation>,
}

impl Page {
    pub fn ast(&self, ctx: &RenderContext) -> crate::Result<PageAst> {
        PageAst::from_page(self, ctx)
    }

    pub fn spec_download_link(&self, ctx: &RenderContext) -> Option<String> {
        ctx.options.download_url_prefix.as_ref().map(|prefix| {
            let mut s = String::new();
            s.push_str(prefix);
            s.push_str(&format!("{}", self.fs_path.display()));
            s
        })
    }

    pub fn outgoing_links(&self, ctx: &mut RenderContext) -> crate::Result<Vec<OutgoingLink>> {
        ctx.with_url_base_by_page_uri(&self.uri_path);

        // Step 1: Concat all descriptions together.
        let mut all_markdown = String::new();

        fn concat(out: &mut String, s: &str) {
            out.push_str(s);
            out.push_str("\n\n");
        }

        for op in &self.operations {
            if let Some(desc) = op.description.as_deref() {
                concat(&mut all_markdown, desc);
            }

            for param in &op.query_parameters {
                if let Some(desc) = param.description.as_deref() {
                    concat(&mut all_markdown, desc);
                }
            }

            for param in &op.header_parameters {
                if let Some(desc) = param.description.as_deref() {
                    concat(&mut all_markdown, desc);
                }
            }

            for param in &op.path_parameters {
                if let Some(desc) = param.description.as_deref() {
                    concat(&mut all_markdown, desc);
                }
            }

            for param in &op.cookie_parameters {
                if let Some(desc) = param.description.as_deref() {
                    concat(&mut all_markdown, desc);
                }
            }
        }

        // Step 2: Parse it into Markdown ast
        markdown::parser::extract_links(&all_markdown, ctx)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum SecurityRequirement {
    Http {
        name: String,
        description: Option<String>,
        scheme: String,
        bearer_format: Option<String>,
    },
    ApiKey {
        name: String,
        description: Option<String>,
        key_name: String,
        key_location: String,
    },
    OAuth2 {
        name: String,
        description: Option<String>,
        all_scopes: Vec<(String, String)>,
        required_scopes: Vec<String>,
        flows: Vec<OAuth2Flow>,
    },
    OpenID {
        name: String,
        description: Option<String>,
        open_id_connect_url: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum OAuth2Flow {
    Implicit {
        authorization_url: String,
        refresh_url: Option<String>,
        #[serde(skip)]
        scopes: IndexMap<String, String>,
    },
    Password {
        refresh_url: Option<String>,
        token_url: String,
        #[serde(skip)]
        scopes: IndexMap<String, String>,
    },
    ClientCredentials {
        refresh_url: Option<String>,
        token_url: String,
        #[serde(skip)]
        scopes: IndexMap<String, String>,
    },
    AuthorizationCode {
        authorization_url: String,
        token_url: String,
        refresh_url: Option<String>,
        #[serde(skip)]
        scopes: IndexMap<String, String>,
    },
}

impl OAuth2Flow {
    pub(crate) fn scopes(&self) -> &IndexMap<String, String> {
        match self {
            Self::Implicit { scopes, .. } => scopes,
            Self::Password { scopes, .. } => scopes,
            Self::ClientCredentials { scopes, .. } => scopes,
            Self::AuthorizationCode { scopes, .. } => scopes,
        }
    }
}

impl From<openapi_parser::OAuthFlow> for OAuth2Flow {
    fn from(other: openapi_parser::OAuthFlow) -> Self {
        match other {
            openapi_parser::OAuthFlow::Implicit {
                authorization_url,
                refresh_url,
                scopes,
            } => Self::Implicit {
                authorization_url: authorization_url.to_string(),
                refresh_url: refresh_url.map(|s| s.into()),
                scopes: scopes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            openapi_parser::OAuthFlow::Password {
                refresh_url,
                token_url,
                scopes,
            } => Self::Password {
                refresh_url: refresh_url.map(|s| s.into()),
                token_url: token_url.to_string(),
                scopes: scopes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            openapi_parser::OAuthFlow::ClientCredentials {
                refresh_url,
                token_url,
                scopes,
            } => Self::ClientCredentials {
                refresh_url: refresh_url.map(|s| s.into()),
                token_url: token_url.to_string(),
                scopes: scopes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            openapi_parser::OAuthFlow::AuthorizationCode {
                authorization_url,
                token_url,
                refresh_url,
                scopes,
            } => Self::AuthorizationCode {
                authorization_url: authorization_url.to_string(),
                token_url: token_url.to_string(),
                refresh_url: refresh_url.map(|s| s.into()),
                scopes: scopes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
    #[serde(skip)]
    pub variables: IndexMap<String, ServerVariable>,
}

impl Server {
    fn from_parsed(spec: openapi_parser::Server) -> Self {
        Self {
            url: spec.url.into(),
            description: spec.description.map(Into::into),
            variables: spec
                .variables
                .into_iter()
                .map(|(k, v)| (k.into(), ServerVariable::from_parsed(v)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ServerVariable {
    pub r#enum: Option<Vec<String>>,
    pub default: String,
    pub description: Option<String>,
}

impl ServerVariable {
    fn from_parsed(spec: openapi_parser::ServerVariable) -> Self {
        Self {
            r#enum: spec.r#enum.map(|v| v.into_iter().map(Into::into).collect()),
            default: spec.default.into(),
            description: spec.description.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Operation {
    pub method: String,
    pub route_pattern: String,
    pub operation_id: Option<String>,
    pub tags: Vec<String>,
    /// These are code examples provided by `x-doctave > code-samples`
    pub code_examples: Vec<Example>,
    pub summary: Option<String>,
    pub anchor_tag: String,
    pub description: Option<String>,
    pub query_parameters: Vec<Parameter>,
    pub header_parameters: Vec<Parameter>,
    pub path_parameters: Vec<Parameter>,
    pub cookie_parameters: Vec<Parameter>,

    pub request_body: Option<RequestBody>,
    pub responses: Vec<Response>,

    /// By the spec, the operation has to be authorized by _one_
    /// of these.
    pub security_requirements: Vec<SecurityRequirement>,
    pub servers: Vec<Server>,
}

impl Operation {
    pub fn from_parsed(
        spec: openapi_parser::Operation,
        method: String,
        route_pattern: String,
        description: Option<String>,
        security_schemes: &Option<
            openapi_parser::Map<openapi_parser::String, openapi_parser::SecurityScheme>,
        >,
    ) -> crate::Result<Self> {
        let code_examples = code_examples_from_parsed(&spec);
        let tags = spec
            .tags
            .iter()
            .map(openapi_parser::String::to_string)
            .collect();

        let anchor_tag = spec
            .summary
            .to_owned()
            .unwrap_or(format!("{}-{}", method, route_pattern).into())
            .replace([' ', '/', ',', '.', '\\', '-', '_', '=', '?'], "-")
            .to_ascii_lowercase();

        let query_parameters = spec
            .parameters
            .iter()
            .filter(|param| matches!(param.kind, ParameterKind::Query(_)))
            .map(|param| Parameter::from_parsed(param.clone(), "query".into()))
            .collect::<crate::Result<Vec<_>>>()?;

        let header_parameters = spec
            .parameters
            .iter()
            .filter(|param| matches!(param.kind, ParameterKind::Header(_)))
            .map(|param| Parameter::from_parsed(param.clone(), "header".into()))
            .collect::<crate::Result<Vec<_>>>()?;

        let path_parameters = spec
            .parameters
            .iter()
            .filter(|param| matches!(param.kind, ParameterKind::Path(_)))
            .map(|param| Parameter::from_parsed(param.clone(), "path".into()))
            .collect::<crate::Result<Vec<_>>>()?;

        let cookie_parameters = spec
            .parameters
            .iter()
            .filter(|param| matches!(param.kind, ParameterKind::Cookie(_)))
            .map(|param| Parameter::from_parsed(param.clone(), "cookie".into()))
            .collect::<crate::Result<Vec<_>>>()?;

        let responses = spec
            .responses
            .map(|r| {
                r.0.into_iter()
                    .map(|(k, v)| Response::from_parsed(v, k.to_string()))
                    .collect::<crate::Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();

        let request_body = spec
            .request_body
            .map(RequestBody::from_parsed)
            .transpose()?;

        let security_requirements = {
            let mut final_requirements = vec![];
            let requirements = spec.security;
            for req in requirements {
                for (key, scopes) in req.requirements {
                    if let Some(scheme) = security_schemes.as_ref().and_then(|s| s.get(&key)) {
                        match &scheme.kind {
                            openapi_parser::SecuritySchemeKind::Http(kind) => {
                                final_requirements.push(SecurityRequirement::Http {
                                    name: key.to_string(),
                                    description: description.clone(),
                                    scheme: kind.scheme.to_string(),
                                    bearer_format: kind
                                        .bearer_format
                                        .as_ref()
                                        .map(|v| v.to_string()),
                                });
                            }
                            openapi_parser::SecuritySchemeKind::ApiKey(kind) => {
                                final_requirements.push(SecurityRequirement::ApiKey {
                                    name: key.to_string(),
                                    description: description.clone(),
                                    key_location: kind.r#in.to_string(),
                                    key_name: kind.name.to_string(),
                                });
                            }
                            openapi_parser::SecuritySchemeKind::OpenIdConnect(kind) => {
                                final_requirements.push(SecurityRequirement::OpenID {
                                    name: key.to_string(),
                                    description: description.clone(),
                                    open_id_connect_url: kind.open_id_connect_url.to_string(),
                                });
                            }
                            openapi_parser::SecuritySchemeKind::OAuth2(kind) => {
                                let mut all_flows = vec![];

                                if let Some(flow) = &kind.flows.implicit {
                                    all_flows.push(OAuth2Flow::from(flow.clone()));
                                }

                                if let Some(flow) = &kind.flows.password {
                                    all_flows.push(OAuth2Flow::from(flow.clone()));
                                }

                                if let Some(flow) = &kind.flows.client_credentials {
                                    all_flows.push(OAuth2Flow::from(flow.clone()));
                                }

                                if let Some(flow) = &kind.flows.authorization_code {
                                    all_flows.push(OAuth2Flow::from(flow.clone()));
                                }

                                let mut required_scopes = vec![];
                                let mut all_scopes = HashSet::new();

                                for flow in &all_flows {
                                    for scope in flow.scopes() {
                                        all_scopes.insert((scope.0.to_owned(), scope.1.to_owned()));
                                    }
                                }

                                let mut all_scopes = all_scopes.into_iter().collect::<Vec<_>>();
                                all_scopes.sort_by(|a, b| a.0.cmp(&b.0));

                                for scope in scopes {
                                    if all_flows
                                        .iter()
                                        .find_map(|f| f.scopes().get(&scope.to_string()))
                                        .is_some()
                                    {
                                        required_scopes.push(scope.into());
                                    }
                                }

                                final_requirements.push(SecurityRequirement::OAuth2 {
                                    name: key.to_string(),
                                    description: description.clone(),
                                    required_scopes,
                                    all_scopes,
                                    flows: all_flows,
                                });
                            }
                        }
                    }
                }
            }

            final_requirements
        };

        let servers = spec
            .servers
            .into_iter()
            .map(Server::from_parsed)
            .collect::<Vec<_>>();

        Ok(Self {
            method,
            route_pattern,
            operation_id: spec.operation_id.map(|s| s.into()),
            tags,
            code_examples,
            summary: spec.summary.map(|s| s.to_string()),
            anchor_tag,
            description,
            query_parameters,
            header_parameters,
            path_parameters,
            cookie_parameters,
            request_body,
            responses,
            security_requirements,
            servers,
        })
    }

    pub fn from_parsed_webhook(
        spec: openapi_parser::Operation,
        security_schemes: &Option<
            openapi_parser::Map<openapi_parser::String, openapi_parser::SecurityScheme>,
        >,
    ) -> crate::Result<Self> {
        Operation::from_parsed(spec, "webhook".into(), "".into(), None, security_schemes)
    }

    pub fn identifier(&self) -> String {
        format!(
            "{}-{}-{}",
            self.method,
            self.route_pattern,
            self.summary
                .as_ref()
                .map(|s| s.trim().replace(' ', "-"))
                .unwrap_or_else(|| String::from("operation"))
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Response {
    pub status: String,
    pub description: String,
    pub content: Vec<MediaType>,
    pub headers: Vec<Header>,
}

impl Response {
    pub fn from_parsed(
        spec: openapi_parser::openapi30::schemas::response::Response,
        status: String,
    ) -> crate::Result<Self> {
        let description = spec.description.to_string();
        let content = spec
            .content
            .into_iter()
            .map(|(k, v)| MediaType::from_parsed(v, k.into()))
            .collect::<crate::Result<Vec<_>>>()?;
        let headers = spec
            .headers
            .into_iter()
            .map(|(k, v)| Header::from_parsed(v, k.into()))
            .collect::<crate::Result<Vec<_>>>()?;

        Ok(Response {
            status,
            description,
            content,
            headers,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Header {
    pub name: String,
    pub description: Option<String>,
    pub schema: Schema,
    pub required: bool,
}

impl Header {
    pub fn from_parsed(
        spec: openapi_parser::openapi30::schemas::header::Header,
        name: String,
    ) -> crate::Result<Self> {
        let param = Parameter::from_parsed(spec.0, "header".into())?;

        let schema = param.schema.ok_or(Error {
            code: Error::OPENAPI_REFERENCE,
            message: "Parameter must have a schema".to_string(),
            description: "".to_string(),
            file: Some(PathBuf::from("")),
            position: None,
        })?;

        Ok(Header {
            name,
            description: param.description.map(|d| d.to_string()),
            schema,
            required: param.required,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Parameter {
    pub name: String,
    pub contained_in: String,
    pub style: String,
    pub required: bool,
    pub deprecated: Option<bool>,
    pub description: Option<String>,
    pub example_string: Option<String>,
    pub explode: Option<bool>,
    pub schema: Option<Schema>,
}

impl Parameter {
    pub fn from_parsed(
        spec: openapi_parser::openapi30::schemas::parameter::Parameter,
        contained_in: String,
    ) -> crate::Result<Self> {
        let example_string = spec.example.as_ref().map(|e| {
            serde_json::to_string_pretty(e).expect("Failed to serialize example to JSON string")
        });

        let schema = match spec.schema_or_content {
            Some(openapi_parser::SchemaOrContent::Schema(s)) => Some(Schema::from_parsed(
                s,
                spec.deprecated,
                Some(spec.name.to_string()),
                spec.required,
                None,
                false,
            )?),
            Some(openapi_parser::SchemaOrContent::Content(c)) => {
                Some(Schema::from_parsed_parameter_content(
                    c,
                    spec.deprecated,
                    Some(spec.name.to_string()),
                    spec.required,
                    false,
                )?)
            }
            None => None,
        };

        Ok(Parameter {
            name: spec.name.to_string(),
            contained_in,
            style: spec.style.unwrap_or_default().into(),
            required: spec.required.unwrap_or_default(),
            deprecated: spec.deprecated,
            description: spec.description.map(|d| d.into()),
            example_string,
            explode: spec.explode,
            schema,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) enum SchemaKind {
    SingleType(Type),
    OneOf {
        schemas: Vec<Schema>,
    },
    #[allow(dead_code)]
    AllOf {
        schemas: Vec<Schema>,
    },
    AnyOf {
        schemas: Vec<Schema>,
    },
    Not {
        /// Technically I believe this can only ever be len() == 1,
        /// but some code becomes much easier to write when we allocate
        /// this as an array instead.
        schemas: Vec<Schema>,
    },
    Any {
        typ: Option<String>,
        pattern: Option<String>,
        minimum: Option<f64>,
        maximum: Option<f64>,
        // TODO(Nik): Missing fields
    },
}

impl Default for SchemaKind {
    fn default() -> Self {
        SchemaKind::Any {
            typ: None,
            pattern: None,
            minimum: None,
            maximum: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct SchemaOpts {
    title: Option<String>,
    description: Option<String>,
    required: Option<bool>,
    deprecated: Option<bool>,
    nullable: Option<bool>,
    /// Schema might be part of a `content` block in certain cases. See "schema vs content":
    /// https://swagger.io/docs/specification/describing-parameters/
    mediatype: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub(crate) struct Schema {
    pub schema_kind: SchemaKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub deprecated: bool,
    pub example: Option<Example>,
    pub required: Option<bool>,
    pub default: Option<Value>,
    pub mediatype: Option<String>,
    pub nullable: bool,
    #[serde(skip)]
    pub metadata: Option<openapi_parser::Metadata>,
    pub expanded: bool,
}

impl Schema {
    fn empty(opts: SchemaOpts) -> Self {
        Schema {
            title: opts.title,
            description: opts.description,
            deprecated: opts.deprecated.unwrap_or(false),
            required: opts.required,
            ..Default::default()
        }
    }

    /// If we get the content case, attempt to parse the mediatype's schema.
    ///
    /// Few things to note:
    /// - We may have more than one content type. That's not supported yet, because the
    ///   template would have to drastically change to support it. No time now.
    /// - Nothing about the types guarantee there actually is a spec in the mediatype,
    ///   which means we have to make these "empty" schemas in the cases where it
    ///   doesn't exist.
    fn from_parsed_parameter_content(
        content: openapi_parser::Map<openapi_parser::String, openapi_parser::MediaType>,
        deprecated: Option<bool>,
        title: Option<String>,
        required: Option<bool>,
        expanded: bool,
    ) -> crate::Result<Self> {
        if let Some((mediatype_name, mediatype)) = content.into_iter().next() {
            if let Some(schema) = mediatype.schema.to_owned() {
                Schema::from_parsed(
                    schema,
                    deprecated,
                    title,
                    required,
                    Some(mediatype_name.to_string()),
                    expanded,
                )
            } else {
                Ok(Schema::empty(SchemaOpts::default()))
            }
        } else {
            Ok(Schema::empty(SchemaOpts::default()))
        }
    }

    fn from_parsed(
        spec: openapi_parser::Schema,
        deprecated: Option<bool>,
        title: Option<String>,
        required: Option<bool>,
        mediatype: Option<String>,
        expanded: bool,
    ) -> crate::Result<Self> {
        use openapi_parser::openapi30::schemas::schema::SchemaKind as ExpSchemaKind;

        let nullable = spec.nullable.unwrap_or_default();
        let description = spec.description.clone().map(|t| t.into());
        let title = title.or(spec.title.as_ref().map(|t| t.to_string()));

        let metadata = spec.metadata.clone();

        let example = spec
            .example
            .as_ref()
            .map(|v| Example::from_parsed_raw(v.clone(), String::new(), mediatype.clone()))
            .transpose()?;

        let default = spec.default.as_ref().map(|v| v.clone().into());
        let this_deprecated = deprecated.or(spec.deprecated).unwrap_or_default();
        let schema_kind = match spec.kind {
            ExpSchemaKind::AnyOf(any_of) => {
                let mut schemas = vec![];
                for schema in any_of.any_of {
                    schemas.push(Schema::from_parsed(
                        schema, deprecated, None, None, None, true,
                    )?);
                }

                SchemaKind::AnyOf { schemas }
            }
            ExpSchemaKind::OneOf(one_of) => {
                let mut schemas = vec![];
                for schema in one_of.one_of {
                    schemas.push(Schema::from_parsed(
                        schema,
                        deprecated,
                        None,
                        None,
                        mediatype.clone(),
                        true,
                    )?);
                }

                SchemaKind::OneOf { schemas }
            }
            ExpSchemaKind::String(_)
            | ExpSchemaKind::Integer(_)
            | ExpSchemaKind::Number(_)
            | ExpSchemaKind::Boolean(_)
            | ExpSchemaKind::Object(_)
            | ExpSchemaKind::Array(_) => {
                SchemaKind::SingleType(Type::from_parsed(spec, deprecated)?)
            }
            ExpSchemaKind::Unknown => {
                if let Some(not) = spec.not {
                    SchemaKind::Not {
                        schemas: vec![Schema::from_parsed(
                            *not,
                            deprecated,
                            None,
                            None,
                            mediatype.clone(),
                            true,
                        )?],
                    }
                } else {
                    SchemaKind::default()
                }
            }
        };

        Ok(Schema {
            schema_kind,
            title,
            description,
            deprecated: this_deprecated,
            example,
            required,
            default,
            mediatype,
            nullable,
            metadata: Some(metadata),
            expanded,
        })
    }

    pub fn is_nested(&self) -> bool {
        match &self.schema_kind {
            SchemaKind::SingleType(ref t) => match t {
                Type::Object { properties, .. } => !properties.is_empty(),
                Type::Array { items, .. } => items.is_some(),
                _ => false,
            },
            SchemaKind::OneOf { .. } => true,
            SchemaKind::AnyOf { .. } => true,
            SchemaKind::AllOf { .. } => true,
            SchemaKind::Not { .. } => true,
            SchemaKind::Any { .. } => false,
        }
    }

    pub fn combination_explanation(&self) -> &str {
        match self.schema_kind {
            SchemaKind::OneOf { .. } => "Must match one of the following",
            SchemaKind::AnyOf { .. } => "Must match any of the following",
            SchemaKind::AllOf { .. } => "Must match all of the following",
            SchemaKind::Not { .. } => "Cannot match any of the following",
            _ => "Child attributes",
        }
    }

    pub fn nested_schemas(&self) -> &[Schema] {
        match &self.schema_kind {
            SchemaKind::SingleType(ref t) => match t {
                Type::Object { properties, .. } => properties.as_slice(),
                _ => &[],
            },
            SchemaKind::AnyOf { schemas } => schemas.as_slice(),
            SchemaKind::OneOf { schemas } => schemas.as_slice(),
            SchemaKind::AllOf { schemas } => schemas.as_slice(),
            SchemaKind::Any { .. } => &[],
            SchemaKind::Not { schemas } => schemas.as_slice(),
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(
            self.schema_kind,
            SchemaKind::SingleType(Type::Object { .. })
        )
    }

    pub fn type_name(&self) -> String {
        let type_name = match &self.schema_kind {
            SchemaKind::OneOf { .. } => String::from("One Of"),
            SchemaKind::AnyOf { .. } => String::from("Any Of"),
            SchemaKind::AllOf { .. } => String::from("All Of"),
            SchemaKind::SingleType(t) => match t {
                Type::String { .. } => String::from("string"),
                Type::Integer { .. } => String::from("int"),
                Type::Number { .. } => String::from("number"),
                Type::Boolean { .. } => String::from("boolean"),
                Type::File { .. } => String::from("file"),
                Type::Object { .. } => {
                    if let Some(title) = &self.title {
                        format!("object ({})", title.clone())
                    } else {
                        String::from("object")
                    }
                }
                Type::Array { items, .. } => {
                    format!(
                        "array[{}]",
                        items
                            .as_ref()
                            .map(|s| s.type_name())
                            .unwrap_or_else(|| "unknown".to_string())
                    )
                }
            },
            SchemaKind::Any { ref typ, .. } => typ.as_ref().cloned().unwrap_or_default(),
            SchemaKind::Not { .. } => String::from("Must not match"),
        };

        if self.nullable {
            format!("{} or null", type_name)
        } else {
            type_name
        }
    }

    pub fn default(&self) -> Option<&Value> {
        self.default.as_ref()
    }

    pub fn minimum(&self) -> Option<String> {
        match self.schema_kind {
            SchemaKind::SingleType(ref t) => match t {
                Type::Integer { minimum, .. } => minimum.map(|m| format!("{}", m)),
                Type::Number { minimum, .. } => minimum.map(|m| format!("{}", m)),
                _ => None,
            },
            SchemaKind::AllOf { ref schemas } => schemas.first().and_then(|s| s.minimum()),
            _ => None,
        }
    }

    pub fn maximum(&self) -> Option<String> {
        match self.schema_kind {
            SchemaKind::SingleType(ref t) => match t {
                Type::Integer { maximum, .. } => maximum.map(|m| format!("{}", m)),
                Type::Number { maximum, .. } => maximum.map(|m| format!("{}", m)),
                _ => None,
            },
            SchemaKind::AllOf { ref schemas } => schemas.first().and_then(|s| s.maximum()),
            _ => None,
        }
    }

    pub fn multiple_of(&self) -> Option<String> {
        match self.schema_kind {
            SchemaKind::SingleType(ref t) => match t {
                Type::Integer { multiple_of, .. } => multiple_of.map(|m| format!("{}", m)),
                Type::Number { multiple_of, .. } => multiple_of.map(|m| format!("{}", m)),
                _ => None,
            },
            SchemaKind::AllOf { ref schemas } => schemas.first().and_then(|s| s.multiple_of()),
            _ => None,
        }
    }

    pub fn min_length(&self) -> Option<usize> {
        match self.schema_kind {
            SchemaKind::SingleType(Type::String { min_length, .. }) => min_length,
            SchemaKind::AllOf { ref schemas } => schemas.first().and_then(|s| s.min_length()),
            _ => None,
        }
    }

    pub fn max_length(&self) -> Option<usize> {
        match self.schema_kind {
            SchemaKind::SingleType(Type::String { max_length, .. }) => max_length,
            SchemaKind::AllOf { ref schemas } => schemas.first().and_then(|s| s.max_length()),
            _ => None,
        }
    }

    pub fn enumeration(&self) -> Option<&[Option<String>]> {
        match &self.schema_kind {
            SchemaKind::SingleType(Type::String { enumeration, .. }) => {
                Some(enumeration.as_slice())
            }
            SchemaKind::AllOf { ref schemas } => schemas.first().and_then(|s| s.enumeration()),
            _ => None,
        }
    }

    pub fn format(&self) -> Option<&str> {
        match self.schema_kind {
            SchemaKind::SingleType(ref t) => match t {
                Type::String { format, .. } => format.as_deref(),
                Type::Integer { format, .. } => format.as_deref(),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn pattern(&self) -> Option<&str> {
        match &self.schema_kind {
            SchemaKind::SingleType(Type::String { pattern, .. }) => pattern.as_deref(),
            _ => None,
        }
    }

    pub fn example_string(&self) -> Option<&str> {
        self.example.as_ref().map(|e| e.value.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// TODO: Missing a few
/// https://docs.rs/openapiv3/1.0.1/openapiv3/struct.NumberType.html
pub(crate) enum Type {
    Integer {
        format: Option<String>,
        minimum: Option<i64>,
        maximum: Option<i64>,
        multiple_of: Option<i64>,
        enumeration: Vec<Option<i64>>,
    },
    Number {
        format: Option<String>,
        minimum: Option<f64>,
        maximum: Option<f64>,
        multiple_of: Option<f64>,
        enumeration: Vec<Option<f64>>,
    },
    String {
        format: Option<String>,
        pattern: Option<String>,
        min_length: Option<usize>,
        max_length: Option<usize>,
        enumeration: Vec<Option<String>>,
    },
    Object {
        properties: Vec<Schema>,
        required: Vec<String>,
    },
    Array {
        items: Option<Box<Schema>>,
        min_items: Option<usize>,
        unique: bool,
    },
    Boolean,
    File,
}

impl Type {
    fn from_parsed(
        spec: openapi_parser::openapi30::schemas::schema::Schema,
        deprectated: Option<bool>,
    ) -> crate::Result<Self> {
        use openapi_parser::openapi30::schemas::schema::SchemaKind as ExpSchemaKind;

        match spec.kind {
            ExpSchemaKind::String(s) => {
                if s.format == Some("binary".into()) {
                    Ok(Type::File)
                } else {
                    Ok(Type::String {
                        format: s.format.map(|f| f.into()),
                        pattern: s.pattern.map(|p| p.into()),
                        min_length: s.min_length.and_then(|m| m.as_int().map(|m| m as usize)),
                        max_length: s.max_length.and_then(|m| m.as_int().map(|m| m as usize)),
                        enumeration: s
                            .r#enum
                            .iter()
                            .map(|e| Some(e.to_string()))
                            .collect::<Vec<_>>(),
                    })
                }
            }
            ExpSchemaKind::Number(n) => Ok(Type::Number {
                format: n.format.map(|f| f.into()),
                minimum: n.minimum.and_then(|m| m.as_float()),
                maximum: n.maximum.and_then(|m| m.as_float()),
                multiple_of: n.multiple_of.and_then(|m| m.as_float()),
                enumeration: n.r#enum.iter().map(|e| e.as_float()).collect::<Vec<_>>(),
            }),
            ExpSchemaKind::Integer(i) => Ok(Type::Integer {
                format: i.format.map(|f| f.into()),
                minimum: i.minimum.and_then(|m| m.as_int()),
                maximum: i.maximum.and_then(|m| m.as_int()),
                multiple_of: i.multiple_of.and_then(|m| m.as_int()),
                enumeration: i.r#enum.iter().map(|e| e.as_int()).collect::<Vec<_>>(),
            }),
            ExpSchemaKind::Boolean(_) => Ok(Type::Boolean),
            ExpSchemaKind::Object(o) => {
                let mut properties = vec![];
                let mut required_props = vec![];
                for (title, schema) in o.properties {
                    if schema.required {
                        required_props.push(title.to_string());
                    }

                    properties.push(Schema::from_parsed(
                        schema.schema,
                        deprectated,
                        Some(title.to_string()),
                        Some(schema.required),
                        None,
                        false,
                    )?);
                }

                Ok(Type::Object {
                    properties,
                    required: required_props,
                })
            }
            ExpSchemaKind::Array(a) => Ok(Type::Array {
                items: a
                    .items
                    .map(|r| {
                        Schema::from_parsed(*r, deprectated, None, Some(false), None, false)
                            .map(Box::new)
                    })
                    .transpose()?,
                min_items: a.min_items.and_then(|m| m.as_int().map(|m| m as usize)),
                unique: a.unique_items.unwrap_or_default(),
            }),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct RequestBody {
    pub required: bool,
    pub description: Option<String>,
    pub content: Vec<MediaType>,
}

impl RequestBody {
    pub fn from_parsed(spec: openapi_parser::RequestBody) -> crate::Result<Self> {
        let mut content = vec![];
        for (key, mediatype) in spec.content.into_iter() {
            content.push(MediaType::from_parsed(mediatype, key.into())?);
        }

        Ok(RequestBody {
            required: spec.required.unwrap_or_default(),
            description: spec.description.map(|v| v.into()),
            content,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct MediaType {
    pub name: String,
    pub schemas: Vec<Schema>,
    pub examples: Vec<Example>,
}

impl MediaType {
    pub fn from_parsed(
        spec: openapi_parser::openapi30::schemas::media_type::MediaType,
        name: String,
    ) -> crate::Result<Self> {
        let schema = spec
            .schema
            .map(|s| Schema::from_parsed(s, None, None, None, Some(name.clone()), false))
            .transpose()?;

        let mut examples = vec![];

        if spec.examples.is_empty() {
            if let Some(example) = spec.example {
                examples.push(Example::from_parsed_raw(
                    example,
                    String::new(),
                    Some(name.to_string()),
                )?);
            }
        } else {
            for (key, example) in spec.examples.into_iter() {
                examples.push(Example::from_parsed(
                    example,
                    key.into(),
                    Some(name.clone()),
                )?);
            }
        }

        if examples.is_empty() && name == "application/json" {
            // If we didn't have any examples, generate one from the schema
            if let Some(s) = schema.as_ref() {
                match &s.schema_kind {
                    SchemaKind::AnyOf { schemas } | SchemaKind::OneOf { schemas } => {
                        for s in schemas {
                            examples.push(Example::autogenerate_from_v3_schema(s));
                        }
                    }
                    _ => {
                        examples.push(Example::autogenerate_from_v3_schema(s));
                    }
                }
            }
        }

        Ok(MediaType {
            name,
            schemas: schema
                .map(|s| {
                    // If the top most schema for the mediatype is an object, we want to flatten
                    // it. Otherwise you get this anonymous object in the UI that is not helpful
                    // to the reader at all. So just show the fields of the top level object
                    // directly instead.
                    if s.is_object() {
                        s.nested_schemas().to_vec()
                    } else {
                        vec![s]
                    }
                })
                .unwrap_or_default(),
            examples,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Example {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub value: String,
}

impl Example {
    pub fn from_parsed(
        spec: openapi_parser::Example,
        name: String,
        summary: Option<String>,
    ) -> crate::Result<Self> {
        Ok(Example {
            name,
            summary: summary.or(spec.summary.map(|s| s.into())),
            description: spec.description.map(|s| s.into()),
            value: serde_json::to_string_pretty(&spec.value)
                .expect("Error serializing JSON example"),
        })
    }

    pub fn from_parsed_raw(
        value: openapi_parser::Value,
        name: String,
        summary: Option<String>,
    ) -> crate::Result<Self> {
        Ok(Example {
            name,
            summary,
            description: None,
            // Our examples have already gone from JSON string -> Serde Values, so going back
            // should never be a problem.
            value: serde_json::to_string_pretty(&value).expect("Error serializing JSON example"),
        })
    }

    pub fn autogenerate_from_v3_schema(schema: &Schema) -> Self {
        fn value(schema: &Schema) -> serde_json::Value {
            if let Some(example) = schema.example.as_ref() {
                if let Ok(val) = serde_json::from_str(example.value.as_str()) {
                    return val;
                }
            }

            match &schema.schema_kind {
                SchemaKind::SingleType(Type::Object { properties, .. }) => properties
                    .iter()
                    .map(|s| {
                        (
                            s.title.clone().unwrap_or_else(|| "field".to_string()),
                            value(s),
                        )
                    })
                    .collect(),
                SchemaKind::SingleType(Type::Array { items, .. }) => {
                    if let Some(s) = items {
                        json!(vec![value(s)])
                    } else {
                        // This just needs _a_ type annotation. Bool is arbitrary
                        json!(Vec::new() as Vec<bool>)
                    }
                }
                SchemaKind::SingleType(Type::Integer {
                    enumeration,
                    minimum,
                    maximum,
                    multiple_of,
                    ..
                }) => {
                    if let Some(e) = enumeration.first() {
                        json!(e)
                    } else if let Some(multiple) = multiple_of {
                        match (minimum, maximum) {
                            (Some(min), Some(_max)) => json!(min + multiple),
                            (Some(min), None) => json!(min + multiple),
                            (None, Some(max)) => json!(max - multiple),
                            (None, None) => json!(multiple),
                        }
                    } else {
                        match (minimum, maximum) {
                            (Some(min), Some(_max)) => json!(min),
                            (Some(min), None) => json!(min),
                            (None, Some(max)) => json!(max),
                            (None, None) => json!(123),
                        }
                    }
                }
                SchemaKind::SingleType(Type::Number {
                    enumeration,
                    minimum,
                    maximum,
                    multiple_of,
                    ..
                }) => {
                    if let Some(e) = enumeration.first() {
                        json!(e)
                    } else if let Some(multiple) = multiple_of {
                        match (minimum, maximum) {
                            (Some(min), Some(_max)) => json!(min + multiple),
                            (Some(min), None) => json!(min + multiple),
                            (None, Some(max)) => json!(max - multiple),
                            (None, None) => json!(multiple),
                        }
                    } else {
                        match (minimum, maximum) {
                            (Some(min), Some(_max)) => json!(min),
                            (Some(min), None) => json!(min),
                            (None, Some(max)) => json!(max),
                            (None, None) => json!(123.0),
                        }
                    }
                }
                SchemaKind::SingleType(Type::String {
                    enumeration,
                    format,
                    ..
                }) => {
                    if let Some(e) = enumeration.first() {
                        json!(e)
                    } else if let Some(f) = format {
                        match f.as_str() {
                            "date" => json!("2023-06-07"),
                            "date-time" => json!("2023-06-07T17:32:28Z"),
                            "email" => json!("alice@example.com"),
                            "uuid" => json!("5bf77342-221c-11ee-be56-0242ac120002"),
                            "uri" => json!("https://www.example.com"),
                            "url" => json!("https://www.example.com"),
                            "ipv4" => json!("192.168.1.1"),
                            "ipv6" => json!("fd00:3559:f175:8ffb:e190:1a98:0ca5:3692"),
                            _ => json!("string"),
                        }
                    } else {
                        json!("string")
                    }
                }
                SchemaKind::SingleType(Type::Boolean) => {
                    json!(true)
                }
                SchemaKind::OneOf { schemas } => schemas
                    .first()
                    .map(value)
                    .unwrap_or(serde_json::Value::Null),
                SchemaKind::AnyOf { schemas } => schemas
                    .first()
                    .map(value)
                    .unwrap_or(serde_json::Value::Null),
                _ => serde_json::Value::Null,
            }
        }

        Example {
            name: schema
                .metadata
                .as_ref()
                .and_then(|m| m.component_name.as_ref().map(|v| v.to_string()))
                .unwrap_or("".to_string()),
            summary: schema.mediatype.as_ref().cloned(),
            description: None,
            // Our examples have already gone from JSON string -> Serde Values, so going back
            // should never be a problem.
            value: serde_json::to_string_pretty(&value(schema))
                .expect("Error serializing JSON example"),
        }
    }

    pub fn identifier(&self, parent_id: &str) -> String {
        format!("example-{}-{}", self.name, parent_id)
    }
}

fn code_examples_from_parsed(spec: &openapi_parser::Operation) -> Vec<Example> {
    if let Some(openapi_parser::Value::Array(samples)) = spec
        .extensions
        .get("x-doctave")
        .and_then(|v| v.get("code-samples"))
    {
        samples
            .iter()
            .filter_map(|value| {
                let mut language = None;
                let mut code = None;

                if let openapi_parser::Value::Object(map) = value {
                    for (key, val) in map {
                        match key.as_str() {
                            "language" => {
                                if let openapi_parser::Value::String(s) = val {
                                    language = Some(s);
                                }
                            }
                            "code" => {
                                if let openapi_parser::Value::String(s) = val {
                                    code = Some(s);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if let (Some(language), Some(code)) = (language, code) {
                    return Some(Example {
                        name: language.to_string(),
                        summary: None,
                        description: None,
                        value: code.to_string(),
                    });
                }
                None
            })
            .collect()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod test {
    use crate::open_api::OpenApi;
    use crate::page_kind::PageKind;

    use super::*;

    fn into_example(schema: &str) -> Example {
        let schema_val: openapi_parser::Value = serde_yaml::from_str(schema).unwrap();

        let parsed = openapi_parser::Schema::try_parse(
            schema_val,
            &openapi_parser::ParserContext::default(),
            &mut openapi_parser::Set::new(),
            None,
        )
        .unwrap();

        let schema = Schema::from_parsed(parsed, None, None, None, None, false).unwrap();

        Example::autogenerate_from_v3_schema(&schema)
    }

    #[test]
    fn creates_a_default_example_for_a_media_type_for_basic_types() {
        let schema = indoc! {r#"
        type: object
        properties:
          id:
            type: integer
          name:
            type: string
          fullTime:
            type: boolean
          ages:
            type: array
            items:
              type: integer
          nested:
            type: object
            properties:
              id:
                type: integer
              name:
                type: string
        "#};

        let example = into_example(schema);

        assert_eq!(
            example.value.as_str(),
            indoc! {r#"
            {
              "id": 123,
              "name": "string",
              "fullTime": true,
              "ages": [
                123
              ],
              "nested": {
                "id": 123,
                "name": "string"
              }
            }"#}
        );
    }

    #[test]
    fn preserves_order_on_generated_example() {
        let schema = indoc! {r#"
        type: object
        properties:
          D:
            type: integer
          E:
            type: string
          A:
            type: boolean
          B:
            type: array
            items:
              type: integer
          C:
            type: object
            properties:
              id:
                type: integer
              name:
                type: string
        "#};

        let example = into_example(schema);

        assert_eq!(
            example.value.as_str(),
            indoc! {r#"
            {
              "D": 123,
              "E": "string",
              "A": true,
              "B": [
                123
              ],
              "C": {
                "id": 123,
                "name": "string"
              }
            }"#}
        );
    }

    #[test]
    fn creates_a_default_example_for_a_media_type_with_special_string_formats() {
        let schema = indoc! {r#"
        type: object
        properties:
          birthday:
            type: string
            format: date
          birthtime:
            type: string
            format: date-time
          email:
            type: string
            format: email
          identifier:
            type: string
            format: uuid
        "#};

        let example = into_example(schema);

        assert_eq!(
            example.value.as_str(),
            indoc! {r#"
            {
              "birthday": "2023-06-07",
              "birthtime": "2023-06-07T17:32:28Z",
              "email": "alice@example.com",
              "identifier": "5bf77342-221c-11ee-be56-0242ac120002"
            }"#}
        );
    }

    #[test]
    fn creates_a_default_example_for_a_media_type_with_special_existing_sub_schema_examples() {
        let schema = indoc! {r#"
        type: object
        properties:
          pet:
            type: string
            example: dog
        "#};

        let example = into_example(schema);

        assert_eq!(
            example.value.as_str(),
            indoc! {r#"
            {
              "pet": "dog"
            }"#}
        );
    }

    #[test]
    fn uses_enums_for_integer_and_number_values() {
        let schema = indoc! {r#"
        type: object
        properties:
          status:
            type: integer
            enum: [400]
          percent:
            type: number
            enum: [40.1]
        "#};

        let example = into_example(schema);

        assert_eq!(
            example.value.as_str(),
            indoc! {r#"
            {
              "status": 400,
              "percent": 40.1
            }"#}
        );
    }

    #[test]
    #[ignore = "WE SHOULD THINK ABOUT HOW TO HANDLE NULLS IN OPENAPI_PARSER"]
    fn enums_can_have_null_values() {
        let schema = indoc! {r#"
        type: object
        properties:
          status:
            type: integer
            enum: [null]
          percent:
            type: number
            enum: [null]
          other:
            type: string
            enum: [null]
        "#};

        let example = into_example(schema);

        assert_eq!(
            example.value.as_str(),
            indoc! {r#"
            {
              "status": null,
              "percent": null,
              "other": null
            }"#}
        );
    }

    #[test]
    fn integer_and_number_examples_support_min_and_max() {
        let schema = indoc! {r#"
        type: object
        properties:
          status:
            type: integer
            minimum: 100
            maximum: 110
          percent:
            type: number
            minimum: 100.0
            maximum: 110.0
        "#};

        let example = into_example(schema);

        assert_eq!(
            example.value.as_str(),
            indoc! {r#"
            {
              "status": 100,
              "percent": 100.0
            }"#}
        );
    }

    #[test]
    fn integer_and_number_examples_support_min_and_max_with_multiple_of() {
        let schema = indoc! {r#"
        type: object
        properties:
          status:
            type: integer
            minimum: 100
            multipleOf: 10
          percent:
            type: number
            minimum: 100.0
            multipleOf: 10.1
          withMax:
            type: integer
            maximum: 100
            multipleOf: 10
          withMaxNum:
            type: number
            maximum: 100.0
            multipleOf: 5.5
        "#};

        let example = into_example(schema);

        assert_eq!(
            example.value.as_str(),
            indoc! {r#"
            {
              "status": 110,
              "percent": 110.1,
              "withMax": 90,
              "withMaxNum": 94.5
            }"#}
        );
    }

    #[test]
    fn operation_uses_common_parameters_from_ref() {
        let base = indoc! {r##"
        openapi: 3.0.0
        info:
          version: 1.0.0
          title: Sample API
          description: A sample API to illustrate OpenAPI concepts
        tags:
          - name: Users
            description: Some **info** about users
        paths:
          /api:
            parameters:
              - $ref: "#/components/parameters/TitleParam"
            get:
              tags:
                - Users
              parameters:
                - name: page
                  in: query
                  description: "Page number"
                  required: false
                  schema:
                    type: integer
              responses:
                '200':
                  description: A JSON array of user names
                  content:
                    application/json:
                      schema:
                        type: array
                        items:
                          type: string
        components:
          parameters:
            TitleParam:
              name: title
              in: query
              description: "Title"
              required: false
              schema:
                type: string
        "##};

        let spec = openapi_parser::openapi30::parser::parse_yaml(base).unwrap();
        let pages =
            OpenApi::pages_from_parsed_spec(&spec, "openapi.yaml".into(), "/api".into()).unwrap();

        let tag_pages = pages
            .into_iter()
            .filter_map(|p| match p {
                PageKind::OpenApi(p) => Some(p.get_page().clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        let page = tag_pages.first().unwrap();
        let operation = page.operations.first().unwrap();

        assert!(operation.query_parameters.len() == 2);

        let param_1 = operation.query_parameters.first().unwrap();
        let param_2 = operation.query_parameters.last().unwrap();

        assert_eq!(param_1.name, "title");
        assert_eq!(param_2.name, "page");
    }
}
