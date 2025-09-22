use serde::{Deserialize, Serialize};

use crate::vale::{ValeResults, ValeRuntimeError};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Rules for rendering a Doctave page, for rewriting links,
/// prefixing asset URLs, settings user preferences, etc.
///
/// Note that bool fields are `false` by default according to
/// Rust's default rules
#[derive(Default)]
pub struct ErrorOptions {
    pub external_results: Option<ValeResults>,
    pub vale_runtime_error: Option<ValeRuntimeError>,
}
