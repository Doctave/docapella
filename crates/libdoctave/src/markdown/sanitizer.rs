use std::collections::{HashMap, HashSet};

use url::Url;

use super::Attribute;

pub struct Sanitizer<'a> {
    pub tags: &'a TAGS,
    pub tag_attributes: &'a TAG_ATTRIBUTES,
    pub generic_attribute_prefixes: &'a GENERIC_ATTRIBUTE_PREFIXES,
    pub generic_attributes: &'a GENERIC_ATTRIBUTES,
    pub url_schemes: &'a URL_SCHEMES,
}

static ALLOWED_IFRAME_HOSTS: [(&str, &str); 6] = [
    ("www.openstreetmap.org", "/export/"),
    ("youtube.com", "/embed/"),
    ("www.youtube.com", "/embed/"),
    ("loom.com", "/embed/"),
    ("www.loom.com", "/embed/"),
    ("embed.api.video", "/vod/"),
];

// These tags are from
// - ammonia: https://github.com/rust-ammonia/ammonia/blob/25017a4c461563c26ff994ba0eede2dcb8b4f235/src/lib.rs#L354
// - doctave specific
lazy_static! {
    pub(crate) static ref TAG_ATTRIBUTES: HashMap<&'static str, HashSet<&'static str>> =
        HashMap::from_iter([
            // ammonia specific
            ("a", HashSet::from_iter([
              // ammonia specific
              "href", "hreflang",
              // doctave specific
              "target", "download"
            ])),
            ("bdo", HashSet::from_iter(["dir"])),
            ("blockquote", HashSet::from_iter(["cite"])),
            ("col", HashSet::from_iter(["align", "char", "charoff", "span"])),
            (
                "colgroup",
                HashSet::from_iter(["align", "char", "charoff", "span"])
            ),
            ("del", HashSet::from_iter(["cite", "datetime"])),
            ("hr", HashSet::from_iter(["align", "size", "width"])),
            (
                "img",
                HashSet::from_iter(["align", "alt", "height", "src", "width"])
            ),
            ("ins", HashSet::from_iter(["cite", "datetime"])),
            ("ol", HashSet::from_iter(["start"])),
            ("q", HashSet::from_iter(["cite"])),
            (
                "table",
                HashSet::from_iter(["align", "char", "charoff", "summary", "width"])
            ),
            (
                "tbody",
                HashSet::from_iter(["align", "char", "charoff", "valign"])
            ),
            (
                "td",
                HashSet::from_iter([
                    "abbr", "align", "axis", "char", "charoff", "colspan", "headers", "rowspan",
                    "scope", "valign"
                ])
            ),
            (
                "tfoot",
                HashSet::from_iter(["align", "char", "charoff", "valign"])
            ),
            (
                "th",
                HashSet::from_iter([
                    "abbr", "align", "axis", "char", "charoff", "colspan", "headers", "rowspan",
                    "scope", "valign"
                ])
            ),
            (
                "thead",
                HashSet::from_iter(["align", "char", "charoff", "valign"])
            ),
            ("tr", HashSet::from_iter(["align", "char", "charoff", "valign"])),
            // doctave specific
            ("iframe", HashSet::from_iter(["src", "allowfullscreen", "scrolling", "width", "height"])),
            ("input", HashSet::from_iter(["type", "id", "name", "checked", "value", "disabled"])),
            ("label", HashSet::from_iter(["for"])),
            ("fieldset", HashSet::from_iter(["class", "name"]))
        ]);
        pub(crate) static ref GENERIC_ATTRIBUTES: HashSet<&'static str> = HashSet::from_iter([
        // from ammonia
        "lang", "title",
        // doctave specific
        "class", "id", "style", "dir", "tabindex", "role", "hidden", "type", "if",
        "else", "elseif",
    ]);
    pub(crate) static ref TAGS: HashSet<&'static str> = HashSet::from_iter([
        // ammonia specific
        "a", "abbr", "acronym", "area", "article", "aside", "b", "bdi",
        "bdo", "blockquote", "br", "caption", "center", "cite", "code", "col", "colgroup", "data", "dd", "del",
        "details", "dfn", "div", "dl", "dt", "em", "figcaption", "figure",
        "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hgroup", "hr", "i",
        "img", "ins", "kbd", "li", "map", "mark", "nav", "ol",
        "p", "pre", "q", "rp", "rt", "rtc", "ruby", "s", "samp", "small", "span",
        "strike", "strong", "sub", "summary", "sup", "table", "tbody", "td", "th",
        "thead", "time", "tr", "tt", "u", "ul", "var", "wbr",
        // doctave specific
        "iframe", "input", "fieldset", "label", "button", "template", "select",
    ]);
    pub(crate) static ref URL_SCHEMES: HashSet<&'static str> = HashSet::from_iter([
        // ammonia specific
        "bitcoin", "ftp", "ftps", "geo", "http", "https", "im",
        "irc", "ircs", "magnet", "mailto", "mms", "mx", "news",
        "nntp", "openpgp4fpr", "sip", "sms", "smsto", "ssh", "tel",
        "url", "webcal", "wtai", "xmpp",
        // doctave specific
        "http", "https", "mailto", "tel", "asset"
    ]);

    pub(crate) static ref GENERIC_ATTRIBUTE_PREFIXES: HashSet<&'static str> = HashSet::from_iter([
        "data-",
        "aria-",
    ]);

    pub(crate) static ref SANITIZER: Sanitizer<'static> = {
        let tags = &TAGS;
        let tag_attributes = &TAG_ATTRIBUTES;
        let generic_attribute_prefixes = &GENERIC_ATTRIBUTE_PREFIXES;
        let generic_attributes = &GENERIC_ATTRIBUTES;
        let url_schemes = &URL_SCHEMES;

        Sanitizer {
            tags,
            tag_attributes,
            generic_attributes,
            generic_attribute_prefixes,
            url_schemes,
        }
    };
}

impl Sanitizer<'_> {
    pub fn sanitize_html_block(&self, name: &str, attributes: &mut Vec<Attribute>) -> Option<()> {
        if !self.tags.contains(name) {
            return None;
        }

        let mut allowed_attributes: HashSet<&str> = HashSet::new();

        if let Some(tag_attrs) = self.tag_attributes.get(&name) {
            allowed_attributes.extend(tag_attrs);
            allowed_attributes.extend(self.generic_attributes.iter());
        } else {
            allowed_attributes.extend(self.generic_attributes.iter());
        };

        attributes.retain(|a| {
            match (name, a.key.as_str()) {
                ("iframe", "src") => {
                    if let Some(value) = &a.value {
                        if let Ok(url) = Url::parse(value.as_str()) {
                            return ALLOWED_IFRAME_HOSTS.iter().any(|(host, path)| {
                                url.host_str() == Some(*host) && url.path().starts_with(path)
                            });
                        }
                    }
                }
                ("input", "type") => {
                    if let Some(value) = &a.value {
                        return ["radio", "checkbox"].contains(&value.as_str());
                    }
                }
                (_, "src") | (_, "href") => {
                    if let Some(value) = &a.value {
                        if let Ok(url) = Url::parse(value.as_str()) {
                            return self.url_schemes.contains(url.scheme());
                        }
                    }
                }
                _ => {}
            }

            allowed_attributes.contains(&a.key.as_str())
                || self
                    .generic_attribute_prefixes
                    .iter()
                    .any(|prefix| a.key.starts_with(prefix))
        });

        Some(())
    }
}

#[cfg(test)]
mod test {
    mod sanitization {
        use pretty_assertions::assert_str_eq;

        use crate::{ast_mdx, render_context::RenderContext};

        #[test]
        fn removes_script_tags() {
            let markdown = indoc! {r#"
            <script src="foo.js">The text</script>
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(node.debug_string().unwrap(), indoc! {r#""# });
        }

        #[test]
        fn allows_some_tags() {
            let markdown = indoc! {r#"
            <div />
            <span />
            <a />
            <img />
            <iframe />
            <input />
            <fieldset />
            <label />
            <button />
            <template />
            <select />
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <div>
                </div>
                <span>
                </span>
                <a>
                </a>
                <img>
                </img>
                <iframe>
                </iframe>
                <input>
                </input>
                <fieldset>
                </fieldset>
                <label>
                </label>
                <button>
                </button>
                <template>
                </template>
                <select>
                </select>
                "#}
            );
        }

        #[test]
        fn allows_generic() {
            let markdown = indoc! {r#"
            <div class="foo" />
            <div id="foo" />
            <div style="foo" />
            <div dir="foo" />
            <div tabindex="foo" />
            <div role="foo" />
            <div hidden="foo" />
            <div type="foo" />
            <div arbitrary="foo" />

            <div data-arbitrary="foo" />
            <div aria-arbitrary="foo" />

            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <div class={foo}>
                </div>
                <div id={foo}>
                </div>
                <div style={foo}>
                </div>
                <div dir={foo}>
                </div>
                <div tabindex={foo}>
                </div>
                <div role={foo}>
                </div>
                <div hidden={foo}>
                </div>
                <div type={foo}>
                </div>
                <div>
                </div>
                <div data-arbitrary={foo}>
                </div>
                <div aria-arbitrary={foo}>
                </div>
                "# }
            );
        }

        #[test]
        fn allows_url_schemes() {
            let markdown = indoc! {r#"
            <a href="http://www.example.com" />
            <a href="https://www.example.com" />
            <a href="mailto://www.example.com" />
            <a href="tel://www.example.com" />
            <a href="asset://www.example.com" />
            <a href="arbitrary://www.example.com" />

            <img src="http://www.example.com" />
            <img src="https://www.example.com" />
            <img src="mailto://www.example.com" />
            <img src="tel://www.example.com" />
            <img src="asset://www.example.com" />
            <img src="arbitrary://www.example.com" />
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <a href={http://www.example.com}>
                </a>
                <a href={https://www.example.com}>
                </a>
                <a href={mailto://www.example.com}>
                </a>
                <a href={tel://www.example.com}>
                </a>
                <a href={asset://www.example.com}>
                </a>
                <a>
                </a>
                <img src={http://www.example.com}>
                </img>
                <img src={https://www.example.com}>
                </img>
                <img src={mailto://www.example.com}>
                </img>
                <img src={tel://www.example.com}>
                </img>
                <img src={asset://www.example.com}>
                </img>
                <img>
                </img>
                "# }
            );
        }

        #[test]
        fn allows_input_type() {
            let markdown = indoc! {r#"
            <input type="radio" />
            <input type="checkbox" />
            <input type="foo" />

            <input
              type="radio"
              id="foo"
              name="bar"
              checked
              value="baz"
              disabled
              arbitrary="qux"
            />
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <input type={radio}>
                </input>
                <input type={checkbox}>
                </input>
                <input>
                </input>
                <input type={radio} id={foo} name={bar} checked value={baz} disabled>
                </input>
                "# }
            );
        }

        #[test]
        fn allowed_iframes() {
            let markdown = indoc! {r#"
            <iframe sandbox="allow-popups" />

            <iframe src="https://www.openstreetmap.org/export/" />
            <iframe src="https://youtube.com/embed/" />
            <iframe src="https://www.youtube.com/embed/" />
            <iframe src="https://embed.api.video/vod/" />
            <iframe src="https://www.example.com" />

            <iframe
              src="https://embed.api.video/vod/"
              allowfullscreen
              scrolling="no"
              width="100"
              height="100"
              some-arbitrary="value"
            />
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <iframe>
                </iframe>
                <iframe src={https://www.openstreetmap.org/export/}>
                </iframe>
                <iframe src={https://youtube.com/embed/}>
                </iframe>
                <iframe src={https://www.youtube.com/embed/}>
                </iframe>
                <iframe src={https://embed.api.video/vod/}>
                </iframe>
                <iframe>
                </iframe>
                <iframe src={https://embed.api.video/vod/} allowfullscreen scrolling={no} width={100} height={100}>
                </iframe>
                "# }
            );
        }

        #[test]
        fn allowed_link_attributes() {
            let markdown = indoc! {r#"
            <a href="https://www.example.com">
              Example
            </a>
            <a href="https://www.example.com" target="_blank">
              External
            </a>
            <a href="https://www.example.com" download>
              Example
            </a>
            <a href="https://www.example.com" download="some.png">
              Img
            </a>
            <a href="https://www.example.com" arbitrary="foo">
              Img
            </a>
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <a href={https://www.example.com}>
                    <Paragraph>
                        <Text>
                            Example
                        </Text>
                    </Paragraph>
                </a>
                <a href={https://www.example.com} target={_blank}>
                    <Paragraph>
                        <Text>
                            External
                        </Text>
                    </Paragraph>
                </a>
                <a href={https://www.example.com} download>
                    <Paragraph>
                        <Text>
                            Example
                        </Text>
                    </Paragraph>
                </a>
                <a href={https://www.example.com} download={some.png}>
                    <Paragraph>
                        <Text>
                            Img
                        </Text>
                    </Paragraph>
                </a>
                <a href={https://www.example.com}>
                    <Paragraph>
                        <Text>
                            Img
                        </Text>
                    </Paragraph>
                </a>
                "# }
            );
        }

        #[test]
        fn allowed_img_attributes() {
            let markdown = indoc! {r#"
            <img src="https://www.example.com" />
            <img src="https://www.example.com" arbitrary="foobar" />
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <img src={https://www.example.com}>
                </img>
                <img src={https://www.example.com}>
                </img>
                "# }
            );
        }

        #[test]
        fn allowed_label_attributes() {
            let markdown = indoc! {r#"
            <label for="foo" />
            <label for="foo" arbitrary="bar" />
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <label for={foo}>
                </label>
                <label for={foo}>
                </label>
                "# }
            );
        }

        #[test]
        fn allowed_fieldset_attributes() {
            let markdown = indoc! {r#"
            <fieldset class="foo" />
            <fieldset name="bar" />
            <fieldset arbitrary="baz" />
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <fieldset class={foo}>
                </fieldset>
                <fieldset name={bar}>
                </fieldset>
                <fieldset>
                </fieldset>
                "# }
            );
        }
    }
}
