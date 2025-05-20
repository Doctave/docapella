use regex::Regex;
use std::borrow::Cow;
use std::collections::HashSet;

lazy_static! {
    static ref REJECTED_CHARS: Regex = Regex::new(r"[^\p{L}\p{M}\p{N}\p{Pc} -]").unwrap();
}

/// Source, with slight modifications: https://github.com/kivikakk/comrak/blob/120a36cfd519e1490c555c4cf55f97c7a46c272e/src/html.rs#L84
///
/// Converts header Strings to canonical, unique, but still human-readable, anchors.
///
/// To guarantee uniqueness, an anchorizer keeps track of the anchors
/// it has returned.  So, for example, to parse several MarkDown
/// files, use a new anchorizer per file.
///
#[derive(Debug, Default)]
pub struct Anchorizer(HashSet<String>);

impl Anchorizer {
    /// Construct a new anchorizer.
    pub fn new() -> Self {
        Anchorizer(HashSet::new())
    }

    /// Returns a String that has been converted into an anchor using the
    /// GFM algorithm, which involves changing spaces to dashes, removing
    /// problem characters and, if needed, adding a suffix to make the
    /// resultant anchor unique.
    pub fn anchorize(&mut self, header: String) -> String {
        let mut id = header.to_lowercase();
        id = REJECTED_CHARS.replace_all(&id, "").replace(' ', "-");

        let mut uniq = 0;
        id = loop {
            let anchor = if uniq == 0 {
                Cow::from(&id)
            } else {
                Cow::from(format!("{}-{}", id, uniq))
            };

            if !self.0.contains(&*anchor) {
                break anchor.into_owned();
            }

            uniq += 1;
        };
        self.0.insert(id.clone());
        id
    }
}
