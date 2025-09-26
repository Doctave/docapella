use std::{collections::BTreeSet, path::PathBuf};

use super::shared_ast::{Point, Position};
use super::{Node, NodeKind};
use crate::{
    content_ast, interpreter::Interpreter, page_kind::OutgoingLink, project::Asset,
    render_context::RenderContext, AttributeValue, Error, RenderOptions, Result,
};

use unix_path::{self as upath};

pub(crate) fn to_ast_mdx(input: &str, ctx: &RenderContext) -> Result<Node> {
    let content_ast = content_ast::build_mdx(input, ctx)?;
    let mut interpreter = Interpreter::new(ctx, input);
    let mut renderable_ast = interpreter.interpret(content_ast)?;

    rewrite_links(&mut renderable_ast, ctx);

    Ok(renderable_ast)
}

pub(crate) fn to_ast_mdx_fault_tolerant(
    input: &str,
    ctx: &RenderContext,
) -> std::result::Result<Node, (Option<Node>, Vec<Error>)> {
    let (content_ast, errors) = fault_tolerant_parse(input, ctx);
    let result = interpret_and_rewrite(content_ast, ctx, input);

    match result {
        Ok(renderable_ast) => {
            if errors.is_empty() {
                Ok(renderable_ast)
            } else {
                Err((Some(renderable_ast), errors))
            }
        }
        Err(e) => {
            let mut all_errors = errors;
            all_errors.push(e);
            Err((None, all_errors))
        }
    }
}

fn fault_tolerant_parse(content: &str, ctx: &RenderContext) -> (content_ast::Node, Vec<Error>) {
    match fault_tolerant_parse_inner(content, 0, content.len(), ctx, 0) {
        Ok(result) => result,
        Err(_) => (
            content_ast::Node {
                kind: content_ast::NodeKind::Root,
                children: vec![],
                pos: Position {
                    start: Point {
                        row: 0,
                        col: 0,
                        byte_offset: 0,
                    },
                    end: Point {
                        row: 0,
                        col: 0,
                        byte_offset: 0,
                    },
                },
            },
            vec![crate::Error {
                code: crate::Error::INVALID_MARKDOWN_TEMPLATE,
                message: "Unable to parse Markdown".to_string(),
                description: "Could not parse Markdown template. Please check the syntax to ensure you have a valid Markdown file.".to_string(),
                file: None,
                position: None,
            }],
        ),
    }
}

/// Parses the content in a fault-tolerant way.
///
/// If it encounters an error, tries to recover by parsing the content before
/// and after the line with the error, and merging the two ASTs. This is done
/// recursively until the whole content is parsed.
///
fn fault_tolerant_parse_inner(
    content: &str,
    start_offset: usize,
    end_offset: usize,
    ctx: &RenderContext,
    depth: u8,
) -> std::result::Result<(content_ast::Node, Vec<Error>), ()> {
    if depth > 64 {
        // Depth limit reached
        return Err(());
    }

    match content_ast::build_mdx(&content[start_offset..end_offset], ctx) {
        Ok(ast) => Ok((bump_offsets(ast, start_offset, content), vec![])),
        Err(mut e) => {
            let error_line = e.position.as_ref().map(|p| p.start.row).unwrap_or(0);

            let begining_of_error_line = line_number_to_byte_offset(
                content,
                lines_before_offset(content, start_offset) + error_line,
            );
            let end_of_error_line = std::cmp::min(
                line_number_to_byte_offset(
                    content,
                    lines_before_offset(content, start_offset) + error_line + 1,
                ),
                end_offset,
            );

            let (before_ast, mut before_errors) = fault_tolerant_parse_inner(
                content,
                start_offset,
                begining_of_error_line,
                ctx,
                depth + 1,
            )?;
            let (after_ast, mut after_errors) =
                fault_tolerant_parse_inner(content, end_of_error_line, end_offset, ctx, depth + 1)?;

            if let Some(p) = e.position.as_mut() {
                // First we bump the error position based on the slice of the file we're in
                p.bump_by_byte_offset(start_offset, content);
                // Then, if we had a frontmatter, bump based on the size of it

                if let Some(file_context) = ctx.file_context.as_ref() {
                    p.bump_by_byte_and_line_offset(
                        file_context.error_lines_offset,
                        file_context.error_bytes_offset,
                    );
                }
            }

            let mut errors = vec![e];

            errors.append(&mut before_errors);
            errors.append(&mut after_errors);

            Ok((combine_asts(before_ast, after_ast), errors))
        }
    }
}

fn combine_asts(before_ast: content_ast::Node, after_ast: content_ast::Node) -> content_ast::Node {
    let mut combined_ast = content_ast::Node {
        kind: content_ast::NodeKind::Root,
        pos: Position {
            start: before_ast.pos.start,
            end: after_ast.pos.end,
        },
        children: Vec::new(),
    };

    combined_ast.children.extend(before_ast.children);
    combined_ast.children.extend(after_ast.children);

    combined_ast
}

fn lines_before_offset(input: &str, byte_offset: usize) -> usize {
    input[..byte_offset].lines().count()
}

fn line_number_to_byte_offset(input: &str, line_number: usize) -> usize {
    if line_number <= 1 {
        return 0;
    }

    input
        .char_indices()
        .filter(|(_byte_offset, v)| v == &'\n')
        .map(|(byte_offset, _)| byte_offset + 1)
        .nth(line_number - 2) // Subtract 2 to get the correct newline (i.e, the second line, is actually the 0th `\n` character in the string)
        .unwrap_or(input.len())
}

fn bump_offsets(ast: content_ast::Node, start_offset: usize, input: &str) -> content_ast::Node {
    let mut ast = ast;
    iter_content_nodes_mut(&mut ast, &|node| {
        node.pos.bump_by_byte_offset(start_offset, input);
    });
    ast
}

fn interpret_and_rewrite(ast: content_ast::Node, ctx: &RenderContext, input: &str) -> Result<Node> {
    let mut interpreter = Interpreter::new(ctx, input);
    let mut renderable_ast = interpreter.interpret(ast)?;
    rewrite_links(&mut renderable_ast, ctx);
    Ok(renderable_ast)
}

fn rewrite_links(renderable_ast: &mut Node, ctx: &RenderContext) {
    iter_nodes_mut(renderable_ast, &|node| match node.kind {
        NodeKind::Link { ref mut url, .. } => {
            if url.starts_with('#') {
                return;
            }

            *url = to_final_link(url, ctx);
        }
        NodeKind::HtmlBlock {
            ref name,
            ref mut attributes,
            ..
        } => {
            for attr in attributes {
                if attr.key == "href" {
                    if let Some(AttributeValue::Literal(ref mut href)) = attr.value {
                        if href.starts_with('#') {
                            return;
                        }

                        *href = to_final_link(href, ctx);
                    }
                }
                // Handle raw `<img>` tags
                if name == "img" && attr.key == "src" {
                    if let Some(AttributeValue::Literal(ref mut href)) = attr.value {
                        if href.starts_with('#') {
                            return;
                        }

                        *href = rewrite_image_src(href, ctx.options, ctx.assets);
                    }
                }
            }
        }
        NodeKind::Image { .. } => {
            if !ctx.options.link_rewrites.is_empty()
                || ctx.options.prefix_asset_urls.is_some()
                || ctx.options.prefix_link_urls.is_some()
                || ctx.options.bust_image_caches
            {
                rewrite_image_node(node, ctx)
            }
        }
        _ => {}
    });
}

pub(crate) fn to_ast(input: &str, ctx: &RenderContext) -> Result<Node> {
    let content_ast = content_ast::build_gfm(input, ctx)?;
    let mut interpreter = Interpreter::new(ctx, input);
    let mut renderable_ast = interpreter.interpret(content_ast)?;

    iter_nodes_mut(&mut renderable_ast, &|node| match node.kind {
        NodeKind::Link { ref mut url, .. } => {
            if url.starts_with('#') {
                return;
            }

            *url = to_final_link(url, ctx);
        }
        NodeKind::Image { .. } => {
            if !ctx.options.link_rewrites.is_empty()
                || ctx.options.prefix_asset_urls.is_some()
                || ctx.options.prefix_link_urls.is_some()
                || ctx.options.bust_image_caches
            {
                rewrite_image_node(node, ctx)
            }
        }
        _ => {}
    });

    Ok(renderable_ast)
}

/// Extracts the outgoing **internal** links out of a blob of markdown
pub(crate) fn extract_links(input: &str, ctx: &RenderContext) -> crate::Result<Vec<OutgoingLink>> {
    let mut links = vec![];

    // Render without expanding links. Copy the context but override any relative URL base.
    let default_ctx = RenderContext {
        relative_url_base: None,
        ..ctx.clone()
    };

    let ast = to_ast_mdx(input, &default_ctx)?;

    fn _iter_nodes<F>(node: &Node, acc: &mut Vec<OutgoingLink>, f: &F)
    where
        F: Fn(&Node, &mut Vec<OutgoingLink>),
    {
        f(node, acc);
        for c in &node.children {
            _iter_nodes(c, acc, f);
        }
    }

    _iter_nodes(&ast, &mut links, &|node, acc| match &node.kind {
        NodeKind::Link { url, .. } => {
            let mut link_split = url.split('#');
            let link_without_fragment = link_split.next().unwrap_or(url);

            if let Some(url) = expand_paths_in_local_links(link_without_fragment, ctx) {
                acc.push(OutgoingLink {
                    uri: link_without_fragment.to_string(),
                    expanded_uri: Some(url.clone()),
                });
            }
        }
        NodeKind::HtmlBlock { attributes, .. } => {
            if !attributes.iter().any(|a| a.key == "download") {
                for attr in attributes {
                    if attr.key == "href" {
                        if let Some(AttributeValue::Literal(url)) = &attr.value {
                            let mut link_split = url.split('#');
                            let link_without_fragment = link_split.next().unwrap_or(url);

                            if let Some(url) =
                                expand_paths_in_local_links(link_without_fragment, ctx)
                            {
                                acc.push(OutgoingLink {
                                    uri: link_without_fragment.to_string(),
                                    expanded_uri: Some(url.clone()),
                                });
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    });

    Ok(links)
}

/// Extracts the outgoing **internal** links out of a blob of markdown
pub(crate) fn extract_asset_links(
    input: &str,
    ctx: &RenderContext,
) -> crate::Result<Vec<OutgoingLink>> {
    let mut links = vec![];

    // Render without expanding links. Copy the context but override any relative URL base.
    let default_ctx = RenderContext {
        relative_url_base: None,
        ..ctx.clone()
    };

    let ast = to_ast_mdx(input, &default_ctx)?;

    fn _iter_nodes<F>(node: &Node, acc: &mut Vec<OutgoingLink>, f: &F)
    where
        F: Fn(&Node, &mut Vec<OutgoingLink>),
    {
        f(node, acc);
        for c in &node.children {
            _iter_nodes(c, acc, f);
        }
    }

    _iter_nodes(&ast, &mut links, &|node, acc| match &node.kind {
        NodeKind::Image { url, .. } => {
            if let Some(expanded_url) = expand_paths_in_local_links(url, ctx) {
                acc.push(OutgoingLink {
                    uri: url.to_string(),
                    expanded_uri: Some(expanded_url.clone()),
                });
            }
        }
        NodeKind::HtmlBlock { attributes, .. } => {
            if attributes.iter().any(|a| a.key == "download") {
                for attr in attributes {
                    if attr.key == "href" {
                        if let Some(AttributeValue::Literal(url)) = &attr.value {
                            if let Some(expanded_url) = expand_paths_in_local_links(url, ctx) {
                                acc.push(OutgoingLink {
                                    uri: url.to_string(),
                                    expanded_uri: Some(expanded_url.clone()),
                                });
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    });

    Ok(links)
}

pub(crate) fn extract_external_links(
    input: &str,
    ctx: &RenderContext,
) -> crate::Result<Vec<String>> {
    println!("Extracting external links from {}", input);
    let ast = to_ast_mdx(input, ctx).map_err(|e| {
        println!("Error: {:#?}", e);
        e
    })?;
    println!("AST: {:#?}", ast);

    let unique_links = ast
        .walk()
        .filter_map(|node| {
            if let NodeKind::Link { url, .. } = &node.kind {
                if uriparse::URI::try_from(url.as_str()).is_ok() {
                    Some(url.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<BTreeSet<String>>();

    Ok(Vec::from_iter(unique_links))
}

/// Webbifies, expands, and rewrites links
pub(crate) fn to_final_link(url: &str, ctx: &RenderContext) -> String {
    let mut link_split = url.split('#');

    let link_without_fragment = link_split.next().unwrap_or(url).to_string();
    let maybe_fragment = link_split.next();

    let mut modified_link = link_without_fragment;

    if ctx.should_expand_relative_uris() {
        modified_link = expand_paths_in_local_links(&modified_link, ctx).unwrap_or(modified_link);
    }

    if ctx.options.webbify_internal_urls && !ctx.options.fsify_internal_urls {
        modified_link = webbify_node(&modified_link, ctx)
    }

    if ctx.options.fsify_internal_urls && !ctx.options.webbify_internal_urls {
        modified_link = fsify_node(&modified_link, ctx)
    }

    if !ctx.options.link_rewrites.is_empty()
        || ctx.options.prefix_asset_urls.is_some()
        || ctx.options.prefix_link_urls.is_some()
    {
        modified_link = rewrite_link(&modified_link, ctx)
    }

    // Re-add fragment if any
    if let Some(fragment) = maybe_fragment {
        modified_link = format!("{}#{}", modified_link, fragment);
    }

    modified_link
}

fn iter_nodes_mut<F>(node: &mut Node, f: &F)
where
    F: Fn(&mut Node),
{
    f(node);
    for c in &mut node.children {
        iter_nodes_mut(c, f);
    }
}

fn iter_content_nodes_mut<F>(node: &mut content_ast::Node, f: &F)
where
    F: Fn(&mut content_ast::Node),
{
    f(node);
    for c in &mut node.children {
        iter_content_nodes_mut(c, f);
    }
}

fn webbify_node(link: &str, _ctx: &RenderContext) -> String {
    webbify_url(link)
}

fn fsify_node(link: &str, ctx: &RenderContext) -> String {
    if is_internal_link(link) {
        let without_hash = link.split('#').collect::<Vec<_>>()[0];

        let l = if let Some(page) = ctx.pages.iter().find(|p| p.uri_path() == without_hash) {
            page.fs_path().display().to_string()
        } else if let Some(page) = ctx.pages.iter().find(|p| {
            p.fs_path().display().to_string().as_str()
                == without_hash.strip_prefix('/').unwrap_or(without_hash)
        }) {
            page.fs_path().display().to_string()
        } else {
            format!("{}.md", link.strip_suffix(".md").unwrap_or(link))
        };

        format!("/{}", l.strip_prefix('/').unwrap_or(&l))
    } else {
        link.to_string()
    }
}

/// Returns the expanded local link if it was a local link. Otherwise return None.
fn expand_paths_in_local_links(link: &str, ctx: &RenderContext) -> Option<String> {
    match parse_internal_link(link) {
        Some(relative_path) if relative_path.is_relative() => Some(prefix_and_expand_path(
            &relative_path,
            ctx.relative_url_base.as_deref().unwrap_or_default(),
        )),
        Some(absolute_path) => Some(expand_path(&absolute_path)),
        None => None,
    }
}

pub fn is_external_link(link: &str) -> bool {
    parse_internal_link(link).is_none()
}

pub fn is_internal_link(link: &str) -> bool {
    parse_internal_link(link).is_some()
}

pub fn parse_external_link(link: &str) -> Option<String> {
    match parse_internal_link(link) {
        Some(_internal_link) => None,
        None => Some(link.to_string()),
    }
}

pub fn parse_internal_link(link: &str) -> Option<upath::PathBuf> {
    if link.is_empty() {
        return None;
    }

    let has_hostname = uriparse::URI::try_from(link).is_ok();
    if !has_hostname {
        return Some(upath::PathBuf::from(link));
    }
    None
}

pub(crate) fn expand_path(path: &upath::Path) -> String {
    let mut result = Vec::<&str>::new();

    for component in path.components() {
        match component {
            upath::Component::RootDir => {}
            upath::Component::CurDir => {}
            upath::Component::ParentDir => {
                result.pop();
            }
            upath::Component::Normal(part) => {
                if let Some(part) = part.to_str() {
                    result.push(part)
                }
            }
        }
    }
    format!("/{}", result.join("/"))
}

fn prefix_path(uri: &upath::Path, base: &str) -> upath::PathBuf {
    upath::Path::new(base).join(uri)
}

pub(crate) fn prefix_and_expand_path(uri: &upath::Path, base: &str) -> String {
    expand_path(&prefix_path(uri, base))
}

pub fn webbify_url(url: &str) -> String {
    if is_internal_link(url) && url.ends_with(".md") {
        // TODO how can this work?
        crate::fs_to_uri_path(&PathBuf::from(url))
    } else {
        url.to_owned()
    }
}

fn rewrite_image_node(node: &mut Node, ctx: &RenderContext) {
    if let NodeKind::Image { ref mut url, .. } = node.kind {
        *url = rewrite_image_src(url, ctx.options, ctx.assets);
    }
}

pub(crate) fn rewrite_image_src(src: &str, opts: &RenderOptions, assets: &[Asset]) -> String {
    let new_url = if opts.bust_image_caches {
        let cache_key = if let Some(asset) = assets
            .iter()
            .find(|a| a == &&PathBuf::from(src.trim_start_matches('/')))
        {
            asset.signature.to_string()
        } else {
            let now = std::time::SystemTime::now();
            now.duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis()
                .to_string()
        };

        format!("{}?c={}", src, cache_key)
    } else {
        src.to_string()
    };

    if let Some(rewrite) = opts.link_rewrites.get(src) {
        rewrite.to_owned()
    } else if let Some(prefix) = &opts.prefix_asset_urls {
        if new_url.starts_with("/_assets/") {
            let mut new = prefix.clone();
            new.push_str(src);

            new.to_owned()
        } else {
            new_url
        }
    } else {
        new_url
    }
}

pub(crate) fn rewrite_link(link: &str, ctx: &RenderContext) -> String {
    if let Some(rewrite) = ctx.options.link_rewrites.get(link) {
        return rewrite.to_owned();
    } else if let Some(prefix) = &ctx.options.prefix_link_urls {
        if parse_internal_link(link).is_some() {
            let mut rewrite = String::from(prefix);
            rewrite.push('/');
            rewrite.push_str(link.strip_prefix('/').unwrap_or(link));

            return rewrite.to_owned();
        }
    }

    link.to_owned()
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_str_eq;

    use crate::{settings::Settings, RenderOptions};

    use super::*;

    #[test]
    fn parses_markdown() {
        let input = indoc! {"
        # Foo

        In the bar
        "};

        let ctx = RenderContext::new();

        assert_str_eq!(
            to_ast(input, &ctx).unwrap().debug_string().unwrap(),
            indoc! { r#"
            <Heading1>
                <Text>
                    Foo
                </Text>
            </Heading1>
            <Paragraph>
                <Text>
                    In the bar
                </Text>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn expands_relative_path_according_to_base() {
        assert_eq!(
            prefix_and_expand_path(&upath::PathBuf::from("../baz"), "/foo/bar"),
            String::from("/foo/baz")
        );

        assert_eq!(
            prefix_and_expand_path(&upath::PathBuf::from("../../baz"), "/foo/bar"),
            String::from("/baz")
        );

        assert_eq!(
            prefix_and_expand_path(&upath::PathBuf::from("../../../baz"), "/foo/bar"),
            String::from("/baz")
        );

        assert_eq!(
            prefix_and_expand_path(&upath::PathBuf::from("./baz"), "/foo/bar"),
            String::from("/foo/bar/baz")
        );
    }

    #[test]
    fn finds_local_links_when_extracting_links() {
        let mut ctx = RenderContext::new();
        ctx.with_url_base_by_page_uri("/");

        assert_eq!(
            extract_links("[link](/foo/bar)", &ctx),
            Ok(vec![crate::page_kind::OutgoingLink {
                expanded_uri: Some("/foo/bar".to_owned()),
                uri: "/foo/bar".to_owned()
            }])
        );

        assert_eq!(
            extract_links("[link](foo.md)", &ctx),
            Ok(vec![crate::page_kind::OutgoingLink {
                expanded_uri: Some("/foo.md".to_owned()),
                uri: "foo.md".to_owned()
            }])
        );
    }

    #[test]
    fn skips_external_links_when_extracting_links() {
        let mut ctx = RenderContext::new();
        ctx.with_url_base_by_page_uri("/");

        assert_eq!(
            extract_links("[other link](https://www.example.com)", &ctx),
            Ok(vec![])
        );
    }

    #[test]
    fn skips_download_links_when_extracting_links() {
        let mut ctx = RenderContext::new();
        ctx.with_url_base_by_page_uri("/");

        assert_eq!(
            extract_links(r#"<a download href="/_assets/foo/bar.png"></a>"#, &ctx),
            Ok(vec![])
        );
    }

    #[test]
    fn extracts_download_links_when_extracting_assets() {
        let mut ctx = RenderContext::new();
        let settings = Settings::default();
        ctx.with_url_base_by_page_uri("/");
        ctx.settings = &settings;

        assert_eq!(
            extract_asset_links(r#"<a download href="/_assets/foo/bar.png">Foo</a>"#, &ctx),
            Ok(vec![OutgoingLink {
                expanded_uri: Some("/_assets/foo/bar.png".to_owned()),
                uri: "/_assets/foo/bar.png".to_owned()
            }])
        );
    }

    #[test]
    fn extracts_image_links_when_extracting_assets() {
        let mut ctx = RenderContext::new();
        let settings = Settings::default();
        ctx.with_url_base_by_page_uri("/");
        ctx.settings = &settings;

        assert_eq!(
            extract_asset_links("![bar image](/_assets/foo/bar.png)", &ctx),
            Ok(vec![OutgoingLink {
                expanded_uri: Some("/_assets/foo/bar.png".to_owned()),
                uri: "/_assets/foo/bar.png".to_owned()
            }])
        );
    }

    #[test]
    fn skips_links_when_extracting_assets() {
        let mut ctx = RenderContext::new();
        let settings = Settings::default();
        ctx.with_url_base_by_page_uri("/");
        ctx.settings = &settings;

        assert_eq!(
            extract_asset_links("[foo link](/_assets/foo/bar)", &ctx),
            Ok(vec![])
        );
    }

    #[test]
    fn skips_html_blocks_without_download_when_extracting_assets() {
        let mut ctx = RenderContext::new();
        let settings = Settings::default();
        ctx.with_url_base_by_page_uri("/");
        ctx.settings = &settings;

        assert_eq!(
            extract_asset_links(r#"<a href="/_assets/foo/bar">Foo</a>"#, &ctx),
            Ok(vec![])
        );
    }

    #[test]
    fn skips_external_links_with_paths_and_anchors_when_extracting_links() {
        let mut ctx = RenderContext::new();
        ctx.with_url_base_by_page_uri("/");

        assert_eq!(
            extract_links(
                indoc! {r#"
                    [external link](https://www.example.com/with/path)
                    [other link](https://www.example.com/with/path#and-id)
                "#},
                &ctx
            ),
            Ok(vec![])
        );
    }

    #[test]
    fn skips_mailto_links_when_extracting_links() {
        let mut ctx = RenderContext::new();
        ctx.with_url_base_by_page_uri("/");

        assert_eq!(
            extract_links("[mail us](mailto:hi@example.com)", &ctx),
            Ok(vec![])
        );
    }

    #[test]
    fn it_skips_plain_fragment_links_when_extracting_links() {
        let ctx = RenderContext::new();
        let input = "[hi](#fizz)";

        assert_eq!(extract_links(input, &ctx), Ok(vec![]));
    }

    #[test]
    fn it_webbifies_fragment_links() {
        let opts = RenderOptions {
            webbify_internal_urls: true,
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let input = "[hi](/foo.md#fizz)";

        assert_str_eq!(
            to_ast(input, &ctx).unwrap().debug_string().unwrap(),
            indoc! { r#"
            <Paragraph>
                <Link url={/foo#fizz}>
                    <Text>
                        hi
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn it_does_not_webbify_external_links_that_end_in_dot_md() {
        let opts = RenderOptions {
            webbify_internal_urls: true,
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let input = "[hi](https://www.example.com/foo.md)";

        assert_str_eq!(
            to_ast(input, &ctx).unwrap().debug_string().unwrap(),
            indoc! { r#"
            <Paragraph>
                <Link url={https://www.example.com/foo.md}>
                    <Text>
                        hi
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn it_does_not_prefix_fragment_links() {
        let opts = RenderOptions {
            prefix_link_urls: Some(String::from("#/project")),
            webbify_internal_urls: true,
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let input = "[hi](#fizz)";

        assert_str_eq!(
            to_ast(input, &ctx).unwrap().debug_string().unwrap(),
            indoc! { r#"
            <Paragraph>
                <Link url={#fizz}>
                    <Text>
                        hi
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn it_considers_internal_fragment_links_internal() {
        let ctx = RenderContext::new();
        let input = "[hi](/foo.md#fizz)";

        assert_eq!(
            extract_links(input, &ctx),
            Ok(vec![OutgoingLink {
                uri: "/foo.md".to_owned(),
                expanded_uri: Some("/foo.md".to_owned()),
            }])
        );
    }

    #[test]
    fn it_expands_relative_links_when_extracting_links() {
        let ctx = RenderContext::new();
        let input = "[hi](/bar/../foo.md)";

        assert_eq!(
            extract_links(input, &ctx),
            Ok(vec![OutgoingLink {
                uri: "/bar/../foo.md".to_owned(),
                expanded_uri: Some("/foo.md".to_owned())
            }])
        );
    }

    #[test]
    fn it_expands_relative_links_to_parent_directory_when_extracting_links() {
        let mut ctx = RenderContext::new();
        ctx.relative_url_base = Some("/fizz/buzz".to_owned());
        let input = "[hi](../foo.md)";

        assert_eq!(
            extract_links(input, &ctx),
            Ok(vec![OutgoingLink {
                uri: "../foo.md".to_owned(),
                expanded_uri: Some("/fizz/foo.md".to_owned())
            }])
        );
    }

    #[test]
    fn it_expands_relative_links_from_current_directory_when_extracting_links() {
        let ctx = RenderContext::new();
        let input = "[hi](./foo.md)";

        assert_eq!(
            extract_links(input, &ctx),
            Ok(vec![OutgoingLink {
                uri: "./foo.md".to_owned(),
                expanded_uri: Some("/foo.md".to_owned())
            }])
        );
    }

    #[test]
    fn it_recognizes_absolute_link_as_internal() {
        let uri = "/foobar.md";

        let expected = upath::PathBuf::from("/foobar.md");

        assert_eq!(parse_internal_link(uri), Some(expected));
    }

    #[test]
    fn it_recognizes_relative_link_as_internal() {
        let uri = "foobar.md";

        let expected = upath::PathBuf::from("foobar.md");

        assert_eq!(parse_internal_link(uri), Some(expected));
    }

    #[test]
    fn it_recognizes_absolute_link_without_suffix_as_internal() {
        let uri = "/foobar";

        let expected = upath::PathBuf::from("/foobar");

        assert_eq!(parse_internal_link(uri), Some(expected));
    }

    #[test]
    fn it_recognizes_relative_link_with_parent_as_internal() {
        let uri = "../foobar.md";

        let expected = upath::PathBuf::from("../foobar.md");

        assert_eq!(parse_internal_link(uri), Some(expected));
    }

    #[test]
    fn it_recognizes_external_urls_as_not_internal() {
        let uri = "https://www.google.com";

        assert_eq!(parse_internal_link(uri), None);
    }

    #[test]
    fn it_recognizes_external_urls_without_base_as_internal() {
        let uri = "google.com";

        let expected = upath::PathBuf::from("google.com");

        assert_eq!(parse_internal_link(uri), Some(expected));
    }

    #[test]
    fn it_recognizes_email_urls_as_not_internal() {
        let uri = "mailto:foobar@google.com";

        assert_eq!(parse_internal_link(uri), None);
    }

    #[test]
    fn to_final_link_returns_correct_links() {
        let mut ctx = RenderContext::new();
        ctx.relative_url_base = Some("/fizz/buzz".to_owned());
        let input = "./foo.md";

        assert_eq!(to_final_link(input, &ctx), "/fizz/buzz/foo.md".to_string());
    }

    #[test]
    fn to_final_link_returns_correct_links_with_prefix() {
        let opts = RenderOptions {
            prefix_link_urls: Some(String::from("/v1")),
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.relative_url_base = Some("/fizz/buzz".to_owned());
        ctx.with_options(&opts);
        let input = "foo.md";

        assert_eq!(
            to_final_link(input, &ctx),
            "/v1/fizz/buzz/foo.md".to_string()
        );
    }

    #[test]
    fn prefix_and_expand_path_returns_correct_links() {
        use upath::Path;
        let base = "/";
        assert_eq!(
            prefix_and_expand_path(Path::new("foo.md"), base),
            "/foo.md".to_string()
        );
        assert_eq!(
            prefix_and_expand_path(Path::new("./foo.md"), base),
            "/foo.md".to_string()
        );
        assert_eq!(
            prefix_and_expand_path(Path::new("bar/foo.md"), base),
            "/bar/foo.md".to_string()
        );
        assert_eq!(
            prefix_and_expand_path(Path::new("bar/./foo.md"), base),
            "/bar/foo.md".to_string()
        );
        assert_eq!(
            prefix_and_expand_path(Path::new("bar/../foo.md"), base),
            "/foo.md".to_string()
        );
    }

    #[test]
    fn prefix_and_expand_path_returns_correct_links_with_base() {
        use upath::Path;
        let base = "/fizz/buzz";
        assert_eq!(
            prefix_and_expand_path(Path::new("foo.md"), base),
            "/fizz/buzz/foo.md".to_string()
        );
        assert_eq!(
            prefix_and_expand_path(Path::new("./foo.md"), base),
            "/fizz/buzz/foo.md".to_string()
        );
        assert_eq!(
            prefix_and_expand_path(Path::new("bar/foo.md"), base),
            "/fizz/buzz/bar/foo.md".to_string()
        );
        assert_eq!(
            prefix_and_expand_path(Path::new("bar/./foo.md"), base),
            "/fizz/buzz/bar/foo.md".to_string()
        );
        assert_eq!(
            prefix_and_expand_path(Path::new("bar/../foo.md"), base),
            "/fizz/buzz/foo.md".to_string()
        );
    }

    #[test]
    fn it_extracts_external_links() {
        let input = indoc! { r#"
        [internal link](./foo.md)
        [external link 1](https://www.example.com)
        [external link 2](https://api.example.com)
        "# };

        assert_eq!(
            extract_external_links(input, &RenderContext::new()),
            Ok(vec![
                "https://api.example.com".to_string(),
                "https://www.example.com".to_string()
            ])
        )
    }

    #[test]
    fn it_extracts_links_as_unique() {
        let input = indoc! { r#"
        [internal link](./foo.md)
        [external link 1](https://www.example.com)
        [external link 2](https://api.example.com)
        [external link 3](https://api.example.com)
        "# };

        assert_eq!(
            extract_external_links(input, &RenderContext::new()),
            Ok(vec![
                "https://api.example.com".to_string(),
                "https://www.example.com".to_string()
            ])
        )
    }

    #[test]
    fn it_tells_you_if_something_is_an_internal_link() {
        assert!(is_internal_link("/foo"));
        assert!(is_internal_link("/../foo"));
        assert!(is_internal_link("./foo"));
        assert!(is_internal_link("/foo/bar/baz"));
        assert!(is_internal_link("foo/bar/baz"));
        assert!(is_internal_link("foo"));
        assert!(is_internal_link("foo.md"));
        assert!(is_internal_link("./foo.md"));
        assert!(is_internal_link("/foo.md"));
        assert!(is_internal_link("/foo/bar.md"));
        assert!(is_internal_link("/../foo/bar.md"));
        assert!(is_internal_link("../foo/bar.md"));

        // Some not-obvious cases:
        assert!(is_internal_link("example.com/"));
        assert!(is_internal_link("www.example.com/"));
        assert!(is_internal_link("www.example.com/"));
    }

    #[test]
    fn it_tells_you_if_something_is_an_external_link() {
        assert!(is_external_link("http://example.com"));
        assert!(is_external_link("https://example.com"));
        assert!(is_external_link("https://example.com/foo.md"));
    }

    mod fsify_internal_urls {
        use crate::project::{InputContent, InputFile, Project};

        use super::*;

        fn basic_project(pages: &[(PathBuf, String)]) -> Project {
            let mut files = vec![
                crate::project::InputFile {
                    path: PathBuf::from(crate::NAVIGATION_FILE_NAME),
                    content: InputContent::Text(
                        indoc! {r#"
                    ---
                    - heading: Something
                    "#}
                        .to_string(),
                    ),
                },
                InputFile {
                    path: PathBuf::from(crate::SETTINGS_FILE_NAME),
                    content: InputContent::Text(
                        indoc! { r#"
                    ---
                    title: An Project
                    "# }
                        .to_string(),
                    ),
                },
            ];

            for (path, content) in pages {
                files.push(InputFile {
                    path: PathBuf::from(path),
                    content: InputContent::Text(content.to_string()),
                });
            }

            Project::from_file_list(files).unwrap()
        }

        #[test]
        fn it_fsifies_internal_links() {
            let project = basic_project(&[
                ("README.md".into(), "[hi](/foo)".to_string()),
                ("foo.md".into(), "".to_string()),
            ]);

            let page = project.get_page_by_uri_path("/").unwrap();
            let ast = page
                .ast(Some(&RenderOptions {
                    fsify_internal_urls: true,
                    ..Default::default()
                }))
                .unwrap();

            // Expect the correct path to be used
            assert_str_eq!(
                ast.as_markdown().unwrap().debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Link url={/foo.md}>
                        <Text>
                            hi
                        </Text>
                    </Link>
                </Paragraph>
                "# }
            );

            // Check we don't have broken links
            assert!(
                project.verify(None, None).is_ok(),
                "Project failed to verify:\n{:#?}",
                project.verify(None, None)
            );
        }

        #[test]
        fn it_fsifies_relative_internal_links() {
            let project = basic_project(&[
                ("README.md".into(), "".to_string()),
                ("foo.md".into(), "".to_string()),
                ("bar/baz.md".into(), "[hi](../foo)".to_string()),
            ]);

            let page = project.get_page_by_uri_path("/bar/baz").unwrap();
            let ast = page
                .ast(Some(&RenderOptions {
                    fsify_internal_urls: true,
                    ..Default::default()
                }))
                .unwrap();

            // Expect the correct path to be used
            assert_str_eq!(
                ast.as_markdown().unwrap().debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Link url={/foo.md}>
                        <Text>
                            hi
                        </Text>
                    </Link>
                </Paragraph>
                "# }
            );

            // Check we don't have broken links
            assert!(
                project.verify(None, None).is_ok(),
                "Project failed to verify:\n{:#?}",
                project.verify(None, None)
            );
        }

        #[test]
        fn it_fsifies_links_that_might_be_directory_or_file() {
            let project = basic_project(&[
                ("README.md".into(), "[one](/foo) [two](/bar)".to_string()),
                ("foo.md".into(), "".to_string()),
                ("bar/README.md".into(), "".to_string()),
            ]);

            let page = project.get_page_by_uri_path("/").unwrap();
            let ast = page
                .ast(Some(&RenderOptions {
                    fsify_internal_urls: true,
                    ..Default::default()
                }))
                .unwrap();

            // Expect the correct path to be used
            assert_str_eq!(
                ast.as_markdown().unwrap().debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Link url={/foo.md}>
                        <Text>
                            one
                        </Text>
                    </Link>
                    <Text>

                    </Text>
                    <Link url={/bar/README.md}>
                        <Text>
                            two
                        </Text>
                    </Link>
                </Paragraph>
                "# }
            );

            // Check we don't have broken links
            assert!(
                project.verify(None, None).is_ok(),
                "Project failed to verify:\n{:#?}",
                project.verify(None, None)
            );
        }

        #[test]
        fn it_fsifies_internal_links_with_md_links() {
            let project = basic_project(&[
                ("README.md".into(), "[hi](/foo.md)".to_string()),
                ("foo.md".into(), "".to_string()),
            ]);

            let page = project.get_page_by_uri_path("/").unwrap();
            let ast = page
                .ast(Some(&RenderOptions {
                    fsify_internal_urls: true,
                    ..Default::default()
                }))
                .unwrap();

            // Expect the correct path to be used
            assert_str_eq!(
                ast.as_markdown().unwrap().debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Link url={/foo.md}>
                        <Text>
                            hi
                        </Text>
                    </Link>
                </Paragraph>
                "# }
            );

            // Check we don't have broken links
            assert!(
                project.verify(None, None).is_ok(),
                "Project failed to verify:\n{:#?}",
                project.verify(None, None)
            );
        }

        #[test]
        fn it_fsifies_relative_internal_links_with_md_links() {
            let project = basic_project(&[
                ("README.md".into(), "".to_string()),
                ("foo.md".into(), "".to_string()),
                ("bar/baz.md".into(), "[hi](../foo.md)".to_string()),
            ]);

            let page = project.get_page_by_uri_path("/bar/baz").unwrap();
            let ast = page
                .ast(Some(&RenderOptions {
                    fsify_internal_urls: true,
                    ..Default::default()
                }))
                .unwrap();

            // Expect the correct path to be used
            assert_str_eq!(
                ast.as_markdown().unwrap().debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Link url={/foo.md}>
                        <Text>
                            hi
                        </Text>
                    </Link>
                </Paragraph>
                "# }
            );

            // Check we don't have broken links
            assert!(
                project.verify(None, None).is_ok(),
                "Project failed to verify:\n{:#?}",
                project.verify(None, None)
            );
        }

        #[test]
        fn it_fsifies_links_that_might_be_directory_or_file_with_md_links() {
            let project = basic_project(&[
                (
                    "README.md".into(),
                    "[one](/foo.md) [two](/bar/README.md)".to_string(),
                ),
                ("foo.md".into(), "".to_string()),
                ("bar/README.md".into(), "".to_string()),
            ]);

            let page = project.get_page_by_uri_path("/").unwrap();
            let ast = page
                .ast(Some(&RenderOptions {
                    fsify_internal_urls: true,
                    ..Default::default()
                }))
                .unwrap();

            // Expect the correct path to be used
            assert_str_eq!(
                ast.as_markdown().unwrap().debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Link url={/foo.md}>
                        <Text>
                            one
                        </Text>
                    </Link>
                    <Text>

                    </Text>
                    <Link url={/bar/README.md}>
                        <Text>
                            two
                        </Text>
                    </Link>
                </Paragraph>
                "# }
            );

            // Check we don't have broken links
            assert!(
                project.verify(None, None).is_ok(),
                "Project failed to verify:\n{:#?}",
                project.verify(None, None)
            );
        }

        #[test]
        fn it_doesnt_fsify_external_links() {
            let project = basic_project(&[(
                "README.md".into(),
                "[external link](https://www.example.com)".to_string(),
            )]);

            let page = project.get_page_by_uri_path("/").unwrap();
            let ast = page
                .ast(Some(&RenderOptions {
                    fsify_internal_urls: true,
                    ..Default::default()
                }))
                .unwrap();

            // Expect the correct path to be used
            assert_str_eq!(
                ast.as_markdown().unwrap().debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Link url={https://www.example.com}>
                        <Text>
                            external link
                        </Text>
                    </Link>
                </Paragraph>
                "# }
            );

            // Check we don't have broken links
            assert!(
                project.verify(None, None).is_ok(),
                "Project failed to verify:\n{:#?}",
                project.verify(None, None)
            );
        }
    }

    mod line_number_to_byte_offset {
        use super::*;

        #[test]
        fn test_single_line() {
            let input = "Hello, world!";
            assert_eq!(line_number_to_byte_offset(input, 1), 0);
            assert_eq!(line_number_to_byte_offset(input, 2), input.len());
        }

        #[test]
        fn test_multiple_lines() {
            let input = "First line\nSecond line\nThird line";
            assert_eq!(line_number_to_byte_offset(input, 1), 0);
            assert_eq!(line_number_to_byte_offset(input, 2), 11);
            assert_eq!(line_number_to_byte_offset(input, 3), 23);
            assert_eq!(line_number_to_byte_offset(input, 4), input.len());
        }

        #[test]
        fn test_empty_lines() {
            let input = "\n\nThird line\n\nFifth line";
            assert_eq!(line_number_to_byte_offset(input, 1), 0);
            assert_eq!(line_number_to_byte_offset(input, 2), 1);
            assert_eq!(line_number_to_byte_offset(input, 3), 2);
            assert_eq!(line_number_to_byte_offset(input, 4), 13);
            assert_eq!(line_number_to_byte_offset(input, 5), 14);
            assert_eq!(line_number_to_byte_offset(input, 6), input.len());
        }

        #[test]
        fn test_unicode_characters() {
            let input = "Hello\n世界\nГолос";
            assert_eq!(line_number_to_byte_offset(input, 1), 0);
            assert_eq!(line_number_to_byte_offset(input, 2), 6);
            assert_eq!(line_number_to_byte_offset(input, 3), 13);
            assert_eq!(line_number_to_byte_offset(input, 4), input.len());
        }

        #[test]
        fn test_out_of_bounds() {
            let input = "First\nSecond\nThird";
            assert_eq!(line_number_to_byte_offset(input, 10), input.len());
        }

        #[test]
        fn test_empty_string() {
            let input = "";
            assert_eq!(line_number_to_byte_offset(input, 1), 0);
            assert_eq!(line_number_to_byte_offset(input, 2), 0);
        }
    }
}
