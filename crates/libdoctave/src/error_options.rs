use serde::{Deserialize, Serialize};

#[cfg(test)]
use ts_rs::TS;

use crate::vale::{ValeResults, ValeRuntimeError};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Rules for rendering a Doctave page, for rewriting links,
/// prefixing asset URLs, settings user preferences, etc.
///
/// Note that bool fields are `false` by default according to
/// Rust's default rules
#[derive(Default)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct ErrorOptions {
    #[cfg_attr(test, ts(skip))]
    pub external_results: Option<ValeResults>,
    #[cfg_attr(test, ts(skip))]
    pub vale_runtime_error: Option<ValeRuntimeError>,
}
