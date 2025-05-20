use indexmap::IndexMap;
pub use openapi30::parser::ParserContext;
use openapi30::schemas::schema::integer::IntegerSchema;
pub use openapi30::schemas::{
    callback::Callback,
    components::Components,
    contact::Contact,
    encoding::Encoding,
    example::Example,
    external_documentation::ExternalDocumentation,
    header::Header,
    info::Info,
    license::License,
    link::Link,
    media_type::MediaType,
    oauth_flow::OAuthFlow,
    oauth_flows::OAuthFlows,
    openapi::OpenAPI,
    operation::Operation,
    parameter::{
        CookieParameter, HeaderParameter, Parameter, ParameterKind, PathParameter, QueryParameter,
        SchemaOrContent,
    },
    path_item::PathItem,
    request_body::RequestBody,
    response::Response,
    responses::Responses,
    schema::{
        any_of::AnyOfSchema, array::ArraySchema, boolean::BooleanSchema, number::NumberSchema,
        object::ObjectSchema, one_of::OneOfSchema, property::Property, string::StringSchema,
        Metadata, Schema, SchemaKind,
    },
    security_requirement::SecurityRequirement,
    security_scheme::{
        ApiKeySecurityScheme, HttpSecurityScheme, OAuth2SecurityScheme,
        OpenIdConnectSecurityScheme, SecurityScheme, SecuritySchemeKind,
    },
    server::Server,
    server_variable::ServerVariable,
    tag::Tag,
    webhook::Webhook,
};
use serde::Deserialize;
use serde::Serialize;
use std::mem;
use std::{
    collections::hash_map::DefaultHasher,
    fmt::Display,
    hash::{Hash, Hasher},
};

pub type Set<T> = im::HashSet<T>;
pub type String = smartstring::alias::String;
pub type Map<T, K> = IndexMap<T, K>;

pub mod openapi30;

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum Value {
    Null,
    String(String),
    Bool(bool),
    Number(Number),
    Array(Vec<Value>),
    Object(Map<String, Value>),
    Mapping(Map<Value, Value>),
}

impl From<serde_yaml::Value> for Value {
    fn from(value: serde_yaml::Value) -> Self {
        match value {
            serde_yaml::Value::Null => Value::Null,
            serde_yaml::Value::Mapping(m) => {
                let map = m
                    .into_iter()
                    .filter_map(|(k, v)| match k {
                        serde_yaml::Value::String(s) => Some((s.into(), v.into())),
                        _ => None,
                    })
                    .collect::<Map<String, Value>>();

                Value::Object(map)
            }
            serde_yaml::Value::Sequence(v) => {
                let vec = v.into_iter().map(|v| v.into()).collect::<Vec<Value>>();

                Value::Array(vec)
            }
            serde_yaml::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Number(Number::Int(i))
                } else {
                    Value::Null
                }
            }
            serde_yaml::Value::Bool(b) => Value::Bool(b),
            serde_yaml::Value::Tagged(t) => t.value.into(),
            serde_yaml::Value::String(s) => Value::String(s.into()),
        }
    }
}

impl Eq for Value {}
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Value::Null => {}
            Value::Bool(v) => v.hash(state),
            Value::Number(v) => v.hash(state),
            Value::String(v) => v.hash(state),
            Value::Object(v) => {
                let mut xor = 0;
                for (k, v) in v {
                    let mut hasher = DefaultHasher::new();
                    k.hash(&mut hasher);
                    v.hash(&mut hasher);
                    xor ^= hasher.finish();
                }

                xor.hash(state);
            }
            Value::Array(v) => v.hash(state),
            Value::Mapping(v) => {
                let mut xor = 0;
                for (k, v) in v {
                    let mut hasher = DefaultHasher::new();
                    k.hash(&mut hasher);
                    v.hash(&mut hasher);
                    xor ^= hasher.finish();
                }

                xor.hash(state);
            }
        }
    }
}

fn parse_index(s: &str) -> Option<usize> {
    if s.starts_with('+') || (s.starts_with('0') && s.len() != 1) {
        return None;
    }
    s.parse().ok()
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<Number> for Value {
    fn from(n: Number) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

impl From<Map<String, Value>> for Value {
    fn from(o: Map<String, Value>) -> Self {
        Value::Object(o)
    }
}

impl From<Schema> for Value {
    fn from(s: Schema) -> Self {
        let mut map: Map<String, Value> = Map::new();

        if let Some(title) = s.title {
            map.insert("title".into(), title.into());
        }

        if let Some(desc) = s.description {
            map.insert("description".into(), desc.into());
        }

        if let Some(nullable) = s.nullable {
            map.insert("nullable".into(), nullable.into());
        }

        if let Some(not) = s.not {
            map.insert("not".into(), (*not).into());
        }

        let kind_val: Value = s.kind.into();

        if let Some(kind_map) = kind_val.take_object() {
            map.extend(kind_map);
        }

        if s.metadata != Metadata::default() {
            map.insert("doctave_metadata".into(), s.metadata.into());
        }

        if let Some(example) = s.example {
            map.insert("example".into(), example);
        }

        if let Some(default) = s.default {
            map.insert("default".into(), default);
        }

        Value::Object(map)
    }
}

impl From<SchemaKind> for Value {
    fn from(k: SchemaKind) -> Value {
        match k {
            SchemaKind::Object(o) => o.into(),
            SchemaKind::String(s) => s.into(),
            SchemaKind::Boolean(b) => b.into(),
            SchemaKind::Number(n) => n.into(),
            SchemaKind::Array(a) => a.into(),
            SchemaKind::OneOf(o) => o.into(),
            SchemaKind::AnyOf(a) => a.into(),
            _ => Value::String("".into()),
        }
    }
}

impl From<ArraySchema> for Value {
    fn from(a: ArraySchema) -> Value {
        let mut object = Map::new();

        object.insert("type".into(), Value::String("array".into()));

        if let Some(items) = a.items {
            object.insert("items".into(), (*items).into());
        }

        if let Some(max_items) = a.max_items {
            object.insert("maxItems".into(), max_items.into());
        }

        if let Some(min_items) = a.min_items {
            object.insert("minItems".into(), min_items.into());
        }

        if let Some(unique_items) = a.unique_items {
            object.insert("uniqueItems".into(), unique_items.into());
        }

        Value::Object(object)
    }
}

impl From<BooleanSchema> for Value {
    fn from(b: BooleanSchema) -> Value {
        let mut object = Map::new();

        object.insert("type".into(), Value::String("boolean".into()));

        if !b.r#enum.is_empty() {
            let booleans = b.r#enum.into_iter().map(Value::from).collect::<Vec<_>>();

            object.insert("enum".into(), booleans.into());
        }

        Value::Object(object)
    }
}

impl From<NumberSchema> for Value {
    fn from(n: NumberSchema) -> Value {
        let mut object = Map::new();

        object.insert("type".into(), Value::String("number".into()));

        if let Some(max) = n.maximum {
            object.insert("maximum".into(), max.into());
        }

        if let Some(min) = n.minimum {
            object.insert("minimum".into(), min.into());
        }

        if let Some(exclusive_max) = n.exclusive_maximum {
            object.insert("exclusiveMaximum".into(), exclusive_max.into());
        }

        if let Some(exclusive_min) = n.exclusive_minimum {
            object.insert("exclusiveMinimum".into(), exclusive_min.into());
        }

        if let Some(multiple_of) = n.multiple_of {
            object.insert("multipleOf".into(), multiple_of.into());
        }

        if !n.r#enum.is_empty() {
            let numbers = n.r#enum.into_iter().map(Value::from).collect::<Vec<_>>();

            object.insert("enum".into(), numbers.into());
        }

        if let Some(format) = n.format {
            object.insert("format".into(), format.into());
        }

        Value::Object(object)
    }
}

impl From<IntegerSchema> for Value {
    fn from(n: IntegerSchema) -> Value {
        let mut object = Map::new();

        object.insert("type".into(), Value::String("integer".into()));

        if let Some(max) = n.maximum {
            object.insert("maximum".into(), max.into());
        }

        if let Some(min) = n.minimum {
            object.insert("minimum".into(), min.into());
        }

        if let Some(exclusive_max) = n.exclusive_maximum {
            object.insert("exclusiveMaximum".into(), exclusive_max.into());
        }

        if let Some(exclusive_min) = n.exclusive_minimum {
            object.insert("exclusiveMinimum".into(), exclusive_min.into());
        }

        if let Some(multiple_of) = n.multiple_of {
            object.insert("multipleOf".into(), multiple_of.into());
        }

        if !n.r#enum.is_empty() {
            let numbers = n.r#enum.into_iter().map(Value::from).collect::<Vec<_>>();

            object.insert("enum".into(), numbers.into());
        }

        if let Some(format) = n.format {
            object.insert("format".into(), format.into());
        }

        Value::Object(object)
    }
}

impl From<StringSchema> for Value {
    fn from(s: StringSchema) -> Value {
        let mut object = Map::new();

        object.insert("type".into(), Value::String("string".into()));

        if let Some(max_length) = s.max_length {
            object.insert("maxLength".into(), max_length.into());
        }

        if let Some(min_length) = s.min_length {
            object.insert("minLength".into(), min_length.into());
        }

        if let Some(pattern) = s.pattern {
            object.insert("pattern".into(), pattern.into());
        }

        if let Some(format) = s.format {
            object.insert("format".into(), format.into());
        }

        if !s.r#enum.is_empty() {
            let enums = s.r#enum.into_iter().map(Value::from).collect::<Vec<_>>();
            object.insert("enum".into(), enums.into());
        }

        Value::Object(object)
    }
}

impl From<ObjectSchema> for Value {
    fn from(o: ObjectSchema) -> Value {
        let mut object = Map::new();

        object.insert("type".into(), Value::String("object".into()));

        if let Some(max_properties) = o.max_properties {
            object.insert("maxProperties".into(), max_properties.into());
        }

        if let Some(min_properties) = o.min_properties {
            object.insert("minProperties".into(), min_properties.into());
        }

        if !o.properties.is_empty() {
            let required_props = o
                .properties
                .iter()
                .filter_map(|(key, prop)| {
                    if prop.required {
                        Some(Value::String(key.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<Value>>();

            if !required_props.is_empty() {
                object.insert("required".into(), Value::Array(required_props));
            }

            object.insert(
                "properties".into(),
                o.properties
                    .into_iter()
                    .map(|(k, v)| (k, (*v).into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if let Some(additional_properties) = o.additional_properties {
            object.insert(
                "additionalProperties".into(),
                (*additional_properties).into(),
            );
        }

        Value::Object(object)
    }
}

impl From<Property> for Value {
    fn from(value: Property) -> Self {
        let mut object = Map::new();

        let schema: Value = value.schema.into();

        if let Some(obj) = schema.take_object() {
            object.extend(obj);
        }

        Value::Object(object)
    }
}

impl From<Info> for Value {
    fn from(i: Info) -> Self {
        let mut object = Map::new();

        object.insert("title".into(), i.title.into());

        if let Some(desc) = i.description {
            object.insert("description".into(), desc.into());
        }

        if let Some(terms) = i.terms_of_service {
            object.insert("termsOfService".into(), terms.into());
        }

        if let Some(contact) = i.contact {
            object.insert("contact".into(), contact.into());
        }

        if let Some(license) = i.license {
            object.insert("license".into(), license.into());
        }

        object.insert("version".into(), i.version.into());

        Value::Object(object)
    }
}

impl From<OpenAPI> for Value {
    fn from(oapi: OpenAPI) -> Self {
        let mut object = Map::new();

        object.insert("openapi".into(), oapi.openapi.into());
        object.insert("info".into(), oapi.info.into());

        if !oapi.servers.is_empty() {
            let servers = oapi
                .servers
                .into_iter()
                .map(Value::from)
                .collect::<Vec<_>>();

            object.insert("servers".into(), servers.into());
        }

        if !oapi.paths.is_empty() {
            object.insert(
                "paths".into(),
                oapi.paths
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if let Some(comps) = oapi.components {
            object.insert("components".into(), comps.into());
        }

        if !oapi.security.is_empty() {
            let security = oapi
                .security
                .into_iter()
                .map(Value::from)
                .collect::<Vec<_>>();

            object.insert("security".into(), security.into());
        }

        if !oapi.tags.is_empty() {
            let tags = oapi.tags.into_iter().map(Value::from).collect::<Vec<_>>();

            object.insert("tags".into(), tags.into());
        }

        if let Some(external_docs) = oapi.external_docs {
            object.insert("externalDocs".into(), external_docs.into());
        }

        for webhook in oapi.webhooks {
            let mut webhooks_obj = Map::new();

            webhooks_obj.insert(webhook.name, webhook.operation.into());

            object.insert("x-webhooks".into(), webhooks_obj.into());
        }

        Value::Object(object)
    }
}

impl From<Server> for Value {
    fn from(s: Server) -> Self {
        let mut object = Map::new();

        object.insert("url".into(), s.url.into());

        if let Some(desc) = s.description {
            object.insert("description".into(), desc.into());
        }

        if !s.variables.is_empty() {
            object.insert(
                "variables".into(),
                s.variables
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        Value::Object(object)
    }
}

impl From<ServerVariable> for Value {
    fn from(sv: ServerVariable) -> Self {
        let mut object = Map::new();

        if let Some(enums) = sv.r#enum {
            let enums = enums.into_iter().map(Value::from).collect::<Vec<_>>();
            object.insert("enum".into(), enums.into());
        }

        object.insert("default".into(), sv.default.into());

        if let Some(desc) = sv.description {
            object.insert("description".into(), desc.into());
        }

        Value::Object(object)
    }
}

impl From<PathItem> for Value {
    fn from(pi: PathItem) -> Self {
        let mut object = Map::new();

        if let Some(summary) = pi.summary {
            object.insert("summary".into(), summary.into());
        }

        if let Some(desc) = pi.description {
            object.insert("description".into(), desc.into());
        }

        if !pi.servers.is_empty() {
            let servers = pi.servers.into_iter().map(Value::from).collect::<Vec<_>>();
            object.insert("servers".into(), servers.into());
        }

        if let Some(get) = pi.get {
            object.insert("get".into(), get.into());
        }

        if let Some(put) = pi.put {
            object.insert("put".into(), put.into());
        }

        if let Some(post) = pi.post {
            object.insert("post".into(), post.into());
        }

        if let Some(delete) = pi.delete {
            object.insert("delete".into(), delete.into());
        }

        if let Some(options) = pi.options {
            object.insert("options".into(), options.into());
        }

        if let Some(head) = pi.head {
            object.insert("head".into(), head.into());
        }

        if let Some(patch) = pi.patch {
            object.insert("patch".into(), patch.into());
        }

        if let Some(trace) = pi.trace {
            object.insert("trace".into(), trace.into());
        }

        Value::Object(object)
    }
}

impl From<Operation> for Value {
    fn from(o: Operation) -> Self {
        let mut object = Map::new();

        if let Some(summary) = o.summary {
            object.insert("summary".into(), summary.into());
        }

        if let Some(desc) = o.description {
            object.insert("description".into(), desc.into());
        }

        if !o.tags.is_empty() {
            let tags = o.tags.into_iter().map(Value::from).collect::<Vec<_>>();
            object.insert("tags".into(), tags.into());
        }

        if !o.servers.is_empty() {
            let servers = o.servers.into_iter().map(Value::from).collect::<Vec<_>>();
            object.insert("servers".into(), servers.into());
        }

        if let Some(operation_id) = o.operation_id {
            object.insert("operationId".into(), operation_id.into());
        }

        if let Some(deprecated) = o.deprecated {
            object.insert("deprecated".into(), deprecated.into());
        }

        if let Some(responses) = o.responses {
            object.insert("responses".into(), responses.into());
        }

        if let Some(request_body) = o.request_body {
            object.insert("requestBody".into(), request_body.into());
        }

        if !o.parameters.is_empty() {
            let parameters = o
                .parameters
                .into_iter()
                .map(Value::from)
                .collect::<Vec<_>>();
            object.insert("parameters".into(), parameters.into());
        }

        if !o.security.is_empty() {
            let security = o.security.into_iter().map(Value::from).collect::<Vec<_>>();
            object.insert("security".into(), security.into());
        }

        if !o.callbacks.is_empty() {
            let callbacks = o
                .callbacks
                .into_iter()
                .map(|(k, v)| (k, Value::from(v)))
                .collect::<Map<_, _>>();
            object.insert("callbacks".into(), callbacks.into());
        }

        if let Some(external_docs) = o.external_docs {
            object.insert("externalDocs".into(), external_docs.into());
        }

        object.insert("method".into(), o.method.into());

        object.extend(o.extensions);

        Value::Object(object)
    }
}

impl From<License> for Value {
    fn from(l: License) -> Self {
        let mut object = Map::new();

        object.insert("name".into(), l.name.into());

        if let Some(url) = l.url {
            object.insert("url".into(), url.into());
        }

        Value::Object(object)
    }
}

impl From<Link> for Value {
    fn from(l: Link) -> Self {
        let mut object = Map::new();

        if let Some(operation_ref) = l.operation_ref {
            object.insert("operationRef".into(), operation_ref.into());
        }

        if let Some(operation_id) = l.operation_id {
            object.insert("operationId".into(), operation_id.into());
        }

        if !l.parameters.is_empty() {
            object.insert(
                "parameters".into(),
                l.parameters.into_iter().collect::<Map<_, _>>().into(),
            );
        }

        if let Some(request_body) = l.request_body {
            object.insert("requestBody".into(), request_body);
        }

        if let Some(description) = l.description {
            object.insert("description".into(), description.into());
        }

        if let Some(server) = l.server {
            object.insert("server".into(), server.into());
        }

        Value::Object(object)
    }
}

impl From<OAuthFlow> for Value {
    fn from(oaf: OAuthFlow) -> Self {
        let mut object = Map::new();

        match oaf {
            OAuthFlow::AuthorizationCode {
                authorization_url,
                refresh_url,
                token_url,
                scopes,
            } => {
                object.insert("authorizationUrl".into(), authorization_url.into());
                object.insert("tokenUrl".into(), token_url.into());
                if let Some(refresh_url) = refresh_url {
                    object.insert("refreshUrl".into(), refresh_url.into());
                }
                if !scopes.is_empty() {
                    object.insert(
                        "scopes".into(),
                        scopes
                            .into_iter()
                            .map(|(k, v)| (k, v.into()))
                            .collect::<Map<_, _>>()
                            .into(),
                    );
                }
            }
            OAuthFlow::ClientCredentials {
                token_url,
                refresh_url,
                scopes,
            } => {
                object.insert("tokenUrl".into(), token_url.into());
                if let Some(refresh_url) = refresh_url {
                    object.insert("refreshUrl".into(), refresh_url.into());
                }
                if !scopes.is_empty() {
                    object.insert(
                        "scopes".into(),
                        scopes
                            .into_iter()
                            .map(|(k, v)| (k, v.into()))
                            .collect::<Map<_, _>>()
                            .into(),
                    );
                }
            }
            OAuthFlow::Implicit {
                authorization_url,
                refresh_url,
                scopes,
            } => {
                object.insert("authorizationUrl".into(), authorization_url.into());
                if let Some(refresh_url) = refresh_url {
                    object.insert("refreshUrl".into(), refresh_url.into());
                }
                if !scopes.is_empty() {
                    object.insert(
                        "scopes".into(),
                        scopes
                            .into_iter()
                            .map(|(k, v)| (k, v.into()))
                            .collect::<Map<_, _>>()
                            .into(),
                    );
                }
            }
            OAuthFlow::Password {
                token_url,
                refresh_url,
                scopes,
            } => {
                object.insert("tokenUrl".into(), token_url.into());
                if let Some(refresh_url) = refresh_url {
                    object.insert("refreshUrl".into(), refresh_url.into());
                }
                if !scopes.is_empty() {
                    object.insert(
                        "scopes".into(),
                        scopes
                            .into_iter()
                            .map(|(k, v)| (k, v.into()))
                            .collect::<Map<_, _>>()
                            .into(),
                    );
                }
            }
        }

        Value::Object(object)
    }
}

impl From<OAuthFlows> for Value {
    fn from(oafs: OAuthFlows) -> Self {
        let mut object = Map::new();

        if let Some(implicit) = oafs.implicit {
            object.insert("implicit".into(), implicit.into());
        }

        if let Some(password) = oafs.password {
            object.insert("password".into(), password.into());
        }

        if let Some(client_credentials) = oafs.client_credentials {
            object.insert("clientCredentials".into(), client_credentials.into());
        }

        if let Some(authorization_code) = oafs.authorization_code {
            object.insert("authorizationCode".into(), authorization_code.into());
        }

        Value::Object(object)
    }
}

impl From<Example> for Value {
    fn from(e: Example) -> Self {
        let mut object = Map::new();

        if let Some(summary) = e.summary {
            object.insert("summary".into(), summary.into());
        }

        if let Some(description) = e.description {
            object.insert("description".into(), description.into());
        }

        if let Some(value) = e.value {
            object.insert("value".into(), value);
        }

        if let Some(external_value) = e.external_value {
            object.insert("externalValue".into(), external_value.into());
        }

        Value::Object(object)
    }
}

impl From<Parameter> for Value {
    fn from(p: Parameter) -> Self {
        let mut object = Map::new();

        object.insert("name".into(), p.name.into());

        object.insert("in".into(), p.kind.get_type().into());

        if let Some(schema_or_content) = p.schema_or_content {
            match schema_or_content {
                SchemaOrContent::Schema(schema) => {
                    object.insert("schema".into(), schema.into());
                }
                SchemaOrContent::Content(content) => {
                    object.insert(
                        "content".into(),
                        content
                            .into_iter()
                            .map(|(k, v)| (k, v.into()))
                            .collect::<Map<_, _>>()
                            .into(),
                    );
                }
            }
        }

        if let Some(desc) = p.description {
            object.insert("description".into(), desc.into());
        }

        if let Some(deprecated) = p.deprecated {
            object.insert("deprecated".into(), deprecated.into());
        }

        if let Some(style) = p.style {
            object.insert("style".into(), style.into());
        }

        if let Some(explode) = p.explode {
            object.insert("explode".into(), explode.into());
        }

        if let Some(example) = p.example {
            object.insert("example".into(), example);
        }

        if let Some(req) = p.required {
            object.insert("required".into(), req.into());
        }

        if !p.examples.is_empty() {
            object.insert(
                "examples".into(),
                p.examples
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        let kind_val: Value = p.kind.into();

        if let Some(kind_map) = kind_val.take_object() {
            object.extend(kind_map);
        }

        Value::Object(object)
    }
}

impl From<ParameterKind> for Value {
    fn from(k: ParameterKind) -> Value {
        match k {
            ParameterKind::Query(q) => q.into(),
            ParameterKind::Header(h) => h.into(),
            ParameterKind::Cookie(c) => c.into(),
            ParameterKind::Path(p) => p.into(),
        }
    }
}

impl From<QueryParameter> for Value {
    fn from(q: QueryParameter) -> Self {
        let mut object = Map::new();

        if let Some(allow_empty_value) = q.allow_empty_value {
            object.insert("allowEmptyValue".into(), allow_empty_value.into());
        }

        if let Some(allow_reserved) = q.allow_reserved {
            object.insert("allowReserved".into(), allow_reserved.into());
        }

        Value::Object(object)
    }
}

impl From<HeaderParameter> for Value {
    fn from(_h: HeaderParameter) -> Self {
        Value::Object(Map::new())
    }
}

impl From<CookieParameter> for Value {
    fn from(_c: CookieParameter) -> Self {
        Value::Object(Map::new())
    }
}

impl From<PathParameter> for Value {
    fn from(_p: PathParameter) -> Self {
        Value::Object(Map::new())
    }
}

impl From<ExternalDocumentation> for Value {
    fn from(ed: ExternalDocumentation) -> Self {
        let mut object = Map::new();

        object.insert("url".into(), ed.url.into());

        if let Some(desc) = ed.description {
            object.insert("description".into(), desc.into());
        }

        Value::Object(object)
    }
}

impl From<Header> for Value {
    fn from(value: Header) -> Self {
        let mut object = Map::new();

        let value: Value = value.0.into();

        if let Some(map) = value.take_object() {
            object.extend(map);
        }

        Value::Object(object)
    }
}

impl From<Encoding> for Value {
    fn from(e: Encoding) -> Self {
        let mut object = Map::new();

        if let Some(content_type) = e.content_type {
            object.insert("contentType".into(), content_type.into());
        }

        if !e.headers.is_empty() {
            object.insert(
                "headers".into(),
                e.headers
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if let Some(style) = e.style {
            object.insert("style".into(), style.into());
        }

        if let Some(explode) = e.explode {
            object.insert("explode".into(), explode.into());
        }

        if let Some(allow_reserved) = e.allow_reserved {
            object.insert("allowReserved".into(), allow_reserved.into());
        }

        Value::Object(object)
    }
}

impl From<MediaType> for Value {
    fn from(mt: MediaType) -> Self {
        let mut object = Map::new();

        if let Some(schema) = mt.schema {
            object.insert("schema".into(), schema.into());
        }

        if let Some(example) = mt.example {
            object.insert("example".into(), example);
        }

        if !mt.examples.is_empty() {
            object.insert(
                "examples".into(),
                mt.examples
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !mt.encoding.is_empty() {
            object.insert(
                "encoding".into(),
                mt.encoding
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        Value::Object(object)
    }
}

impl From<Response> for Value {
    fn from(r: Response) -> Self {
        let mut object = Map::new();

        object.insert("description".into(), r.description.into());

        if !r.headers.is_empty() {
            object.insert(
                "headers".into(),
                r.headers
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !r.content.is_empty() {
            object.insert(
                "content".into(),
                r.content
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !r.links.is_empty() {
            object.insert(
                "links".into(),
                r.links
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        Value::Object(object)
    }
}

impl From<Responses> for Value {
    fn from(rs: Responses) -> Self {
        let mut object = Map::new();

        object.extend(
            rs.0.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<Map<_, _>>(),
        );

        Value::Object(object)
    }
}

impl From<Tag> for Value {
    fn from(t: Tag) -> Self {
        let mut object = Map::new();

        object.insert("name".into(), t.name.into());

        if let Some(desc) = t.description {
            object.insert("description".into(), desc.into());
        }

        if let Some(external_docs) = t.external_docs {
            object.insert("externalDocs".into(), external_docs.into());
        }

        Value::Object(object)
    }
}

impl From<SecurityRequirement> for Value {
    fn from(sr: SecurityRequirement) -> Self {
        let mut object = Map::new();

        object.extend(
            sr.requirements
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        Value::Array(v.into_iter().map(Value::from).collect::<Vec<_>>()),
                    )
                })
                .collect::<Map<_, _>>(),
        );

        Value::Object(object)
    }
}

impl From<SecurityScheme> for Value {
    fn from(ss: SecurityScheme) -> Self {
        let mut object: Map<String, Value> = Map::new();

        if let Some(description) = ss.description {
            object.insert("description".into(), description.into());
        }

        let kind_val: Value = ss.kind.into();

        if let Some(kind_map) = kind_val.take_object() {
            object.extend(kind_map);
        }

        Value::Object(object)
    }
}

impl From<SecuritySchemeKind> for Value {
    fn from(ssk: SecuritySchemeKind) -> Self {
        match ssk {
            SecuritySchemeKind::ApiKey(a) => a.into(),
            SecuritySchemeKind::Http(h) => h.into(),
            SecuritySchemeKind::OAuth2(o) => (*o).into(),
            SecuritySchemeKind::OpenIdConnect(oidc) => oidc.into(),
        }
    }
}

impl From<ApiKeySecurityScheme> for Value {
    fn from(ass: ApiKeySecurityScheme) -> Self {
        let mut object = Map::new();

        object.insert("type".into(), String::from("apiKey").into());
        object.insert("in".into(), ass.r#in.into());
        object.insert("name".into(), ass.name.into());

        Value::Object(object)
    }
}

impl From<HttpSecurityScheme> for Value {
    fn from(hss: HttpSecurityScheme) -> Self {
        let mut object = Map::new();

        object.insert("type".into(), String::from("http").into());
        object.insert("scheme".into(), hss.scheme.into());

        if let Some(format) = hss.bearer_format {
            object.insert("bearerFormat".into(), format.into());
        }

        Value::Object(object)
    }
}

impl From<OAuth2SecurityScheme> for Value {
    fn from(oss: OAuth2SecurityScheme) -> Self {
        let mut object = Map::new();

        object.insert("type".into(), String::from("oauth2").into());

        let flows: Value = oss.flows.into();

        if let Some(flows) = flows.take_object() {
            object.insert("flows".into(), flows.into());
        }

        Value::Object(object)
    }
}

impl From<OpenIdConnectSecurityScheme> for Value {
    fn from(ocs: OpenIdConnectSecurityScheme) -> Self {
        let mut object = Map::new();

        object.insert("type".into(), String::from("openIdConnect").into());
        object.insert("openIdConnectUrl".into(), ocs.open_id_connect_url.into());

        Value::Object(object)
    }
}

impl From<Contact> for Value {
    fn from(c: Contact) -> Self {
        let mut object = Map::new();

        if let Some(name) = c.name {
            object.insert("name".into(), name.into());
        }

        if let Some(url) = c.url {
            object.insert("url".into(), url.into());
        }

        if let Some(email) = c.email {
            object.insert("email".into(), email.into());
        }

        Value::Object(object)
    }
}

impl From<RequestBody> for Value {
    fn from(rb: RequestBody) -> Self {
        let mut object = Map::new();

        if let Some(description) = rb.description {
            object.insert("description".into(), description.into());
        }

        if !rb.content.is_empty() {
            object.insert(
                "content".into(),
                rb.content
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if let Some(required) = rb.required {
            object.insert("required".into(), required.into());
        }

        Value::Object(object)
    }
}

impl From<Callback> for Value {
    fn from(cb: Callback) -> Self {
        let mut object = Map::new();

        object.extend(
            cb.callbacks
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<Map<_, _>>(),
        );

        Value::Object(object)
    }
}

impl From<Components> for Value {
    fn from(value: Components) -> Self {
        let mut object = Map::new();

        if !value.schemas.is_empty() {
            object.insert(
                "schemas".into(),
                value
                    .schemas
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !value.responses.is_empty() {
            object.insert(
                "responses".into(),
                value
                    .responses
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !value.parameters.is_empty() {
            object.insert(
                "parameters".into(),
                value
                    .parameters
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !value.examples.is_empty() {
            object.insert(
                "examples".into(),
                value
                    .examples
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !value.request_bodies.is_empty() {
            object.insert(
                "requestBodies".into(),
                value
                    .request_bodies
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !value.headers.is_empty() {
            object.insert(
                "headers".into(),
                value
                    .headers
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !value.security_schemes.is_empty() {
            object.insert(
                "securitySchemes".into(),
                value
                    .security_schemes
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !value.links.is_empty() {
            object.insert(
                "links".into(),
                value
                    .links
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        if !value.callbacks.is_empty() {
            object.insert(
                "callbacks".into(),
                value
                    .callbacks
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Map<_, _>>()
                    .into(),
            );
        }

        Value::Object(object)
    }
}

impl From<OneOfSchema> for Value {
    fn from(oo: OneOfSchema) -> Self {
        let mut object = Map::new();

        if !oo.one_of.is_empty() {
            let schemas = oo.one_of.into_iter().map(Value::from).collect::<Vec<_>>();
            object.insert("oneOf".into(), schemas.into());
        }

        Value::Object(object)
    }
}

impl From<AnyOfSchema> for Value {
    fn from(oo: AnyOfSchema) -> Self {
        let mut object = Map::new();

        if !oo.any_of.is_empty() {
            let schemas = oo.any_of.into_iter().map(Value::from).collect::<Vec<_>>();
            object.insert("anyOf".into(), schemas.into());
        }

        Value::Object(object)
    }
}

impl From<Webhook> for Value {
    fn from(w: Webhook) -> Self {
        let mut object = Map::new();

        object.insert(w.name, w.operation.into());

        Value::Object(object)
    }
}

#[allow(dead_code)]
impl Value {
    pub fn pointer(&self, pointer: &str) -> Option<&Value> {
        if pointer.is_empty() {
            return Some(self);
        }
        if !pointer.starts_with('/') {
            return None;
        }
        pointer
            .split('/')
            .skip(1)
            .map(|x| x.replace("~1", "/").replace("~0", "~"))
            .try_fold(self, |target, token| match target {
                Value::Object(map) => map.get(&String::from(token)),
                Value::Array(list) => parse_index(&token).and_then(|x| list.get(x)),
                _ => None,
            })
    }

    fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    fn as_number(&self) -> Option<&Number> {
        match self {
            Value::Number(v) => Some(v),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&Map<String, Value>> {
        match self {
            Value::Object(v) => Some(v),
            _ => None,
        }
    }

    fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match self {
            Value::Object(v) => Some(v),
            _ => None,
        }
    }

    fn take_string(self) -> Option<String> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    fn take_object(self) -> Option<Map<String, Value>> {
        match self {
            Value::Object(v) => Some(v),
            Value::Mapping(m) => Some(
                m.into_iter()
                    .filter_map(|(k, v)| {
                        k.as_str()
                            .map(|v| v.to_string())
                            .or(k.as_number().map(|n| n.to_string()))
                            .map(|k| (k.into(), v))
                    })
                    .collect::<Map<String, Value>>(),
            ),
            _ => None,
        }
    }

    fn take_array(self) -> Option<Vec<Value>> {
        match self {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    fn take_bool(self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(v),
            _ => None,
        }
    }

    fn take_number(self) -> Option<Number> {
        match self {
            Value::Number(v) => Some(v),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(v) => v.get(key),
            _ => None,
        }
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self {
            Value::Object(v) => v.get_mut(key),
            _ => None,
        }
    }

    fn insert(&mut self, key: String, value: Value) {
        if let Value::Object(v) = self {
            v.insert(key, value);
        }
    }

    fn take(&mut self, key: &str) -> Option<Value> {
        match self {
            Value::Object(v) => v.shift_remove(key),
            _ => None,
        }
    }
}

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> serde_json::Value {
        match value {
            Value::Array(a) => {
                serde_json::Value::Array(a.into_iter().map(|v| v.into()).collect::<Vec<_>>())
            }
            Value::Object(o) => serde_json::Value::Object(
                o.into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect::<serde_json::Map<_, _>>(),
            ),
            Value::Number(n) => serde_json::from_str(&n.to_string()).unwrap(), // should be safe?
            Value::String(s) => serde_json::Value::String(s.into()),
            Value::Bool(b) => serde_json::Value::Bool(b),
            Value::Null => serde_json::Value::Null,
            Value::Mapping(m) => serde_json::Value::Object(
                m.into_iter()
                    .filter_map(|(k, v)| {
                        k.as_str()
                            .map(|v| v.to_string())
                            .or(k.as_number().map(|n| n.to_string()))
                            .map(|k| (k, v.into()))
                    })
                    .collect::<serde_json::Map<std::string::String, serde_json::Value>>(),
            ),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq, PartialOrd, Debug)]
#[serde(untagged)]
pub enum Number {
    // TODO: Handle top range of positive u64 numbers
    Int(i64),
    Float(f64),
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{}", i),
            Self::Float(flo) => write!(f, "{}", flo),
        }
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Float(_) => {
                // you should feel bad for using f64 as a map key
                3.hash(state);
            }
            Self::Int(u) => u.hash(state),
        }
    }
}

#[allow(dead_code)]
impl Number {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Number::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Number::Float(v) => Some(*v),
            _ => None,
        }
    }

    fn is_int(&self) -> bool {
        self.as_int().is_some()
    }

    fn is_float(&self) -> bool {
        self.as_float().is_some()
    }
}

/// Macro that uses the serde json macro internally to first create a Value
/// and then serialize it to a string, an finally parse the string to our custom crate::Value.
#[cfg(test)]
macro_rules! json {
    ($($json:tt)*) => {{
        let value = serde_json::json!($($json)*);
        let string = value.to_string();
        crate::Value::from_str(&string).unwrap()
    }};
}

#[cfg(test)]
pub(crate) use json;
