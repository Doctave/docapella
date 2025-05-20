use std::fmt::Display;

use crate::{Map, Set};
use crate::{String, Value};
use thiserror::Error;

use super::schemas::{
    callback, components, contact, encoding, example, external_documentation, info, license, link,
    oauth_flow, oauth_flows, openapi, parameter, request_body, response, responses, schema,
    security_requirement, security_scheme, server, server_variable, tag,
};

#[derive(Debug, Error)]
pub enum Error {
    Info(#[from] info::Error),
    OpenAPI(#[from] openapi::Error),
    Components(#[from] components::Error),
    Schema(#[from] schema::Error),
    Server(#[from] server::Error),
    ServerVariable(#[from] server_variable::Error),
    License(#[from] license::Error),
    Link(#[from] link::Error),
    OAuthFlow(#[from] oauth_flow::Error),
    OAuthFlows(#[from] oauth_flows::Error),
    Example(#[from] example::Error),
    Parameter(#[from] parameter::Error),
    ExternalDocumentation(#[from] external_documentation::Error),
    Encoding(#[from] encoding::Error),
    Response(#[from] response::Error),
    Responses(#[from] responses::Error),
    Tag(#[from] tag::Error),
    SecurityRequirement(#[from] security_requirement::Error),
    SecurityScheme(#[from] security_scheme::Error),
    Contact(#[from] contact::Error),
    RequestBody(#[from] request_body::Error),
    Callback(#[from] callback::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Info(e) => write!(f, "{}", e),
            Error::OpenAPI(e) => write!(f, "{}", e),
            Error::Components(e) => write!(f, "{}", e),
            Error::Schema(e) => write!(f, "{}", e),
            Error::Server(e) => write!(f, "{}", e),
            Error::ServerVariable(e) => write!(f, "{}", e),
            Error::License(e) => write!(f, "{}", e),
            Error::Link(e) => write!(f, "{}", e),
            Error::OAuthFlow(e) => write!(f, "{}", e),
            Error::OAuthFlows(e) => write!(f, "{}", e),
            Error::Example(e) => write!(f, "{}", e),
            Error::Parameter(e) => write!(f, "{}", e),
            Error::ExternalDocumentation(e) => write!(f, "{}", e),
            Error::Encoding(e) => write!(f, "{}", e),
            Error::Response(e) => write!(f, "{}", e),
            Error::Responses(e) => write!(f, "{}", e),
            Error::Tag(e) => write!(f, "{}", e),
            Error::SecurityRequirement(e) => write!(f, "{}", e),
            Error::SecurityScheme(e) => write!(f, "{}", e),
            Error::Contact(e) => write!(f, "{}", e),
            Error::RequestBody(e) => write!(f, "{}", e),
            Error::Callback(e) => write!(f, "{}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default, Debug)]
pub struct ParserContext {
    pub ref_cache: ReferenceCache,
}

impl ParserContext {
    pub fn get(&self, ref_path: &str, visited_refs: &mut Set<String>) -> Option<Value> {
        match &self.ref_cache {
            ReferenceCache::Components(_components) => todo!(),
            ReferenceCache::Value(value) => {
                if visited_refs.insert(String::from(ref_path)).is_some() {
                    let name = ref_path.rsplit_once('/').map(|(_, name)| name)?;
                    let mut schema = Map::new();

                    let mut metadata = Map::new();
                    metadata.insert("recursive".into(), String::from(name).into());
                    schema.insert("doctave_metadata".into(), metadata.into());

                    return Some(Value::Object(schema));
                }

                value
                    .pointer(&ref_path.replace('#', "").replace("/components", ""))
                    .and_then(|val| {
                        let mut val = val
                            .get("$ref")
                            .and_then(Value::as_str)
                            .and_then(|ref_path| self.get(ref_path, visited_refs))
                            .unwrap_or(val.clone());

                        let name = ref_path.rsplit_once('/').map(|(_, name)| name)?;

                        let mut metadata: Map<String, Value> = Map::new();

                        metadata.insert("component_name".into(), Value::String(name.into()));
                        val.insert("doctave_metadata".into(), metadata.into());

                        Some(val)
                    })
            }
        }
    }
}

#[derive(Debug)]
pub enum ReferenceCache {
    Components(Box<components::Components>),
    Value(Value),
}

impl Default for ReferenceCache {
    fn default() -> Self {
        ReferenceCache::Value(Value::Object(Default::default()))
    }
}

impl ReferenceCache {
    pub fn with_value(&mut self, value: Value) {
        *self = ReferenceCache::Value(value);
    }
}

pub fn parse_json(input: &str) -> Result<openapi::OpenAPI> {
    match serde_json_lenient::from_str::<Value>(input) {
        Ok(val) => openapi::OpenAPI::try_parse(val),
        Err(_e) => Err(Error::OpenAPI(openapi::Error::InvalidOpenAPI)),
    }
}

pub fn parse_yaml(input: &str) -> Result<openapi::OpenAPI> {
    match serde_yaml::from_str::<Value>(input) {
        Ok(val) => openapi::OpenAPI::try_parse(val),
        Err(_e) => Err(Error::OpenAPI(openapi::Error::InvalidOpenAPI)),
    }
}
