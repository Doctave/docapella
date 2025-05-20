use crate::render_context::RenderContext;
use serde_json::Value;

use super::model;

/// A single page that we are going to render.
///
/// Pages are grouped by tag, and all operations will match this tag.
#[derive(Debug, Clone)]
pub(crate) struct PageView<'a, 'c> {
    pub page: &'a model::Page,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> PageView<'a, 'c> {
    pub(crate) fn new(page: &'a model::Page, ctx: &'c RenderContext<'c>) -> Self {
        PageView { page, ctx }
    }

    pub(crate) fn operations(&self) -> Vec<Operation<'a, 'c>> {
        self.page
            .operations
            .iter()
            .map(|operation| Operation {
                inner: operation,
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn spec_download_link(&self) -> Option<String> {
        self.ctx.options.download_url_prefix.as_ref().map(|prefix| {
            let mut s = String::new();
            s.push_str(prefix);
            s.push_str(&format!("{}", self.page.fs_path.display()));
            s
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Operation<'a, 'c> {
    pub inner: &'a model::Operation,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> Operation<'a, 'c> {
    pub(crate) fn summary(&self) -> Option<&str> {
        self.inner.summary.as_deref()
    }

    pub(crate) fn anchor_tag(&self) -> &str {
        self.inner.anchor_tag.as_str()
    }

    pub(crate) fn method(&self) -> &str {
        self.inner.method.as_str()
    }

    pub(crate) fn route_pattern(&self) -> &str {
        self.inner.route_pattern.as_str()
    }

    pub(crate) fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    pub(crate) fn security_requirements(&'a self) -> Vec<SecurityRequirement<'a, 'c>> {
        self.inner
            .security_requirements
            .iter()
            .map(|req| SecurityRequirement {
                inner: req,
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn query_parameters(&'a self) -> Vec<Parameter<'a, 'c>> {
        self.inner
            .query_parameters
            .iter()
            .map(|param| Parameter {
                parent_view: self,
                inner: param,
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn path_parameters(&'a self) -> Vec<Parameter<'a, 'c>> {
        self.inner
            .path_parameters
            .iter()
            .map(|param| Parameter {
                parent_view: self,
                inner: param,
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn header_parameters(&'a self) -> Vec<Parameter<'a, 'c>> {
        self.inner
            .header_parameters
            .iter()
            .map(|param| Parameter {
                parent_view: self,
                inner: param,
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    #[allow(dead_code)]
    pub(crate) fn cookie_parameters(&'a self) -> Vec<Parameter<'a, 'c>> {
        self.inner
            .cookie_parameters
            .iter()
            .map(|param| Parameter {
                parent_view: self,
                inner: param,
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn request_examples(&'a self) -> RequestExamples<'a, 'c> {
        RequestExamples {
            _body: self.request_body(),
            code_samples: self.code_samples(),
            _ctx: self.ctx,
        }
    }

    pub(crate) fn response_examples(&'a self) -> ResponseExamples<'a, 'c> {
        ResponseExamples {
            statuses: self
                .inner
                .responses
                .iter()
                .map(|r| Status {
                    inner: r,
                    ctx: self.ctx,
                })
                .collect(),
            _ctx: self.ctx,
        }
    }

    pub(crate) fn request_body(&'a self) -> Option<RequestBody<'a, 'c>> {
        self.inner.request_body.as_ref().map(|rq| RequestBody {
            parent_view: self,
            inner: rq,
            ctx: self.ctx,
        })
    }

    pub(crate) fn code_samples(&'a self) -> Vec<CodeSample<'a, 'c>> {
        self.inner
            .code_examples
            .iter()
            .map(|inner| CodeSample {
                inner,
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn identifier(&self) -> String {
        format!(
            "{}-{}-{}",
            self.method(),
            self.route_pattern(),
            self.summary()
                .as_ref()
                .map(|s| s.trim().replace(' ', "-"))
                .unwrap_or_else(|| String::from("operation"))
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SecurityRequirement<'a, 'c> {
    pub inner: &'a model::SecurityRequirement,
    pub(crate) ctx: &'c RenderContext<'c>,
}

#[derive(Debug, Clone)]
pub(crate) struct Parameter<'a, 'c> {
    pub parent_view: &'a Operation<'a, 'c>,
    pub inner: &'a model::Parameter,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> Parameter<'a, 'c> {
    pub(crate) fn schema(&'a self) -> Option<Schema<'a, 'c>> {
        self.inner.schema.as_ref().map(|inner| Schema {
            inner,
            parent_view: SchemaParent::Parameter(self),
            ctx: self.ctx,
        })
    }

    pub(crate) fn identifier(&self) -> String {
        format!("param-{}", self.parent_view.identifier())
    }

    pub(crate) fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Status<'a, 'c> {
    inner: &'a model::Response,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> Status<'a, 'c> {
    pub(crate) fn code(&self) -> &str {
        self.inner.status.as_str()
    }

    pub(crate) fn description(&self) -> &str {
        &self.inner.description
    }

    pub(crate) fn headers(&'a self) -> Vec<Header<'a, 'c>> {
        self.inner
            .headers
            .iter()
            .map(|inner| Header {
                inner,
                parent_view: self,
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn media_types(&'a self) -> Vec<MediaType<'a, 'c>> {
        self.inner
            .content
            .iter()
            .map(|inner| MediaType {
                inner,
                parent_view: MediaTypeParent::Status(self),
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResponseExamples<'a, 'c> {
    pub statuses: Vec<Status<'a, 'c>>,
    pub(crate) _ctx: &'c RenderContext<'c>,
}

#[derive(Debug, Clone)]
pub(crate) struct RequestExamples<'a, 'c> {
    _body: Option<RequestBody<'a, 'c>>,
    code_samples: Vec<CodeSample<'a, 'c>>,
    _ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> RequestExamples<'a, 'c> {
    pub(crate) fn code_samples(&'a self) -> &'a Vec<CodeSample<'a, 'c>> {
        &self.code_samples
    }

    pub(crate) fn _prettify_language(lang: &str) -> String {
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

    pub(crate) fn language_aliases(lang: &str) -> String {
        match lang {
            "node" => "js".to_string(),
            "android" => "java".to_string(),
            _ => lang.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RequestBody<'a, 'c> {
    pub parent_view: &'a Operation<'a, 'c>,
    pub inner: &'a model::RequestBody,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> RequestBody<'a, 'c> {
    pub(crate) fn media_types(&'a self) -> Vec<MediaType<'a, 'c>> {
        self.inner
            .content
            .iter()
            .map(|inner| MediaType {
                inner,
                parent_view: MediaTypeParent::RequestBody(self),
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    pub(crate) fn identifier(&self) -> String {
        format!("request-body-{}", self.parent_view.identifier())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SchemaParent<'a, 'c> {
    Parameter(&'a Parameter<'a, 'c>),
    MediaType(&'a MediaType<'a, 'c>),
    Schema(&'a Schema<'a, 'c>),
    Header(&'a Header<'a, 'c>),
}

impl SchemaParent<'_, '_> {
    pub(crate) fn identifier(&self) -> String {
        match &self {
            SchemaParent::Parameter(p) => p.identifier(),
            SchemaParent::MediaType(p) => p.identifier(),
            SchemaParent::Schema(p) => p.identifier(),
            SchemaParent::Header(p) => p.identifier(),
        }
    }

    pub(crate) fn example_string(&self) -> Option<&str> {
        match &self {
            SchemaParent::Parameter(p) => p.inner.example_string.as_deref(),
            SchemaParent::MediaType(_p) => None,
            SchemaParent::Schema(_p) => None,
            SchemaParent::Header(_p) => None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Schema<'a, 'c> {
    pub parent_view: SchemaParent<'a, 'c>,
    pub inner: &'a model::Schema,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl std::fmt::Debug for Schema<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Schema")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'a, 'c> Schema<'a, 'c> {
    pub(crate) fn has_child_schemas(&self) -> bool {
        self.inner.is_nested()
    }

    pub(crate) fn identifier(&self) -> String {
        format!(
            "{}-{}",
            self.title().unwrap_or(""),
            self.parent_view.identifier()
        )
    }

    pub(crate) fn schemas(&'a self) -> Vec<Schema<'a, 'c>> {
        let mut schemas = self
            .inner
            .nested_schemas()
            .iter()
            .map(|inner| Schema {
                inner,
                parent_view: SchemaParent::Schema(self),
                ctx: self.ctx,
            })
            .collect::<Vec<_>>();

        // Array types have a single nested schema. Here we unify things
        // so that the template is easier to write.
        if let Some(single_nested) = self.inner.single_nested_schema() {
            schemas.push(Schema {
                inner: single_nested,
                parent_view: SchemaParent::Schema(self),
                ctx: self.ctx,
            });
        }

        schemas
    }

    pub(crate) fn media_type(&self) -> Option<&str> {
        self.inner.mediatype.as_deref()
    }

    pub(crate) fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    pub(crate) fn title(&self) -> Option<&str> {
        self.inner.title.as_deref()
    }

    pub(crate) fn type_name(&self) -> String {
        self.inner.type_name()
    }

    pub(crate) fn format(&self) -> Option<&str> {
        self.inner.format()
    }

    pub(crate) fn pattern(&self) -> Option<&str> {
        self.inner.pattern()
    }

    pub(crate) fn required(&self) -> Option<bool> {
        self.inner.required
    }

    pub(crate) fn default(&self) -> Option<&Value> {
        self.inner.default()
    }

    pub(crate) fn deprecated(&self) -> bool {
        self.inner.deprecated
    }

    pub(crate) fn minimum(&self) -> Option<String> {
        self.inner.minimum()
    }

    pub(crate) fn maximum(&self) -> Option<String> {
        self.inner.maximum()
    }

    pub(crate) fn multiple_of(&self) -> Option<String> {
        self.inner.multiple_of()
    }

    pub(crate) fn min_length(&self) -> Option<usize> {
        self.inner.min_length()
    }

    pub(crate) fn max_length(&self) -> Option<usize> {
        self.inner.max_length()
    }

    pub(crate) fn enumeration(&self) -> Option<&[Option<String>]> {
        self.inner.enumeration()
    }

    pub(crate) fn example_string(&self) -> Option<&str> {
        self.inner
            .example
            .as_ref()
            .map(|e| e.value.as_ref())
            .or_else(|| self.parent_view.example_string())
    }

    pub(crate) fn combination_explanation(&self) -> &str {
        match self.inner.schema_kind {
            model::SchemaKind::OneOf { .. } => "Must match one of the following",
            model::SchemaKind::AnyOf { .. } => "Must match any of the following",
            model::SchemaKind::AllOf { .. } => "Must match all of the following",
            model::SchemaKind::Not { .. } => "Cannot match any of the following",
            _ => "Child attributes",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MediaTypeParent<'a, 'c> {
    RequestBody(&'a RequestBody<'a, 'c>),
    Status(&'a Status<'a, 'c>),
}

impl MediaTypeParent<'_, '_> {
    pub(crate) fn identifier(&self) -> String {
        match &self {
            MediaTypeParent::RequestBody(p) => p.identifier(),
            MediaTypeParent::Status(s) => s.code().to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MediaType<'a, 'c> {
    pub parent_view: MediaTypeParent<'a, 'c>,
    pub inner: &'a model::MediaType,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> MediaType<'a, 'c> {
    pub(crate) fn name(&self) -> &str {
        self.inner.name.as_str()
    }

    pub(crate) fn schemas(&'a self) -> Vec<Schema<'a, 'c>> {
        self.inner
            .schemas
            .iter()
            .map(|inner| Schema {
                inner,
                parent_view: SchemaParent::MediaType(self),
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn examples(&'a self) -> Vec<Example<'a, 'c>> {
        self.inner
            .examples
            .iter()
            .map(|inner| Example {
                inner,
                parent_view: ExampleParent::MediaType(self),
                _syntax: "json",
                ctx: self.ctx,
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn identifier(&self) -> String {
        format!("{}-{}", self.name(), self.parent_view.identifier())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CodeSample<'a, 'c> {
    pub inner: &'a model::Example,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> CodeSample<'a, 'c> {
    pub(crate) fn identifier(&self) -> String {
        "code".to_string()
    }

    pub(crate) fn language(&self) -> &str {
        &self.inner.name
    }

    /// Code sample as renderable example
    pub(crate) fn example(&'a self) -> Example<'a, 'c> {
        Example {
            inner: self.inner,
            parent_view: ExampleParent::Request(self),
            _syntax: &self.inner.name,
            ctx: self.ctx,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ExampleParent<'a, 'c> {
    MediaType(&'a MediaType<'a, 'c>),
    Request(&'a CodeSample<'a, 'c>),
    #[allow(dead_code)]
    // TODO(Nik): Really?
    Schema(&'a Schema<'a, 'c>),
}

impl ExampleParent<'_, '_> {
    pub(crate) fn identifier(&self) -> String {
        match &self {
            ExampleParent::MediaType(e) => e.identifier(),
            ExampleParent::Request(e) => e.identifier(),
            ExampleParent::Schema(e) => e.identifier(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Example<'a, 'c> {
    pub parent_view: ExampleParent<'a, 'c>,
    pub inner: &'a model::Example,
    _syntax: &'a str,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl Example<'_, '_> {
    pub(crate) fn identifier(&self) -> String {
        format!("example-{}-{}", self.name(), self.parent_view.identifier())
    }

    pub(crate) fn name(&self) -> String {
        self.inner.name.to_owned()
    }

    pub(crate) fn summary(&self) -> String {
        self.inner
            .summary
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.name())
    }

    pub(crate) fn has_summary(&self) -> bool {
        self.inner.summary.is_some()
    }

    pub(crate) fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    #[allow(dead_code)]
    pub(crate) fn value(&self) -> &str {
        &self.inner.value
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Header<'a, 'c> {
    pub parent_view: &'a Status<'a, 'c>,
    pub inner: &'a model::Header,
    pub(crate) ctx: &'c RenderContext<'c>,
}

impl<'a, 'c> Header<'a, 'c> {
    pub(crate) fn identifier(&self) -> String {
        format!("header-{}-{}", self.name(), self.parent_view.code())
    }

    pub(crate) fn name(&self) -> &str {
        self.inner.name.as_str()
    }

    pub(crate) fn schema(&'a self) -> Schema<'a, 'c> {
        Schema {
            inner: &self.inner.schema,
            parent_view: SchemaParent::Header(self),
            ctx: self.ctx,
        }
    }
}
