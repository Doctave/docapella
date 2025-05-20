pub(crate) fn build_parsed(
    info: &openapi_parser::Info,
    servers: &[openapi_parser::Server],
    external_docs: Option<&openapi_parser::ExternalDocumentation>,
) -> String {
    formatdoc! {"
    # {}

    Current Version: `{}`

    {}

    ## Servers

    {}

    {}
    ",
    info.title,
    info.version,
    info.description.as_deref().unwrap_or(""),
    format_server_parsed(servers),
    format_external_docs_parsed(external_docs)
    }
}

fn format_server_parsed(servers: &[openapi_parser::Server]) -> String {
    servers
        .iter()
        .map(|s| {
            format!(
                "* [{}]({})\n\n{}\n\n",
                s.url,
                s.url,
                s.description
                    .as_ref()
                    .map(|s| s.lines().fold(String::new(), |mut acc, l| {
                        acc.push_str(&format!("  {}\n", l));
                        acc
                    }))
                    .as_deref()
                    .unwrap_or("")
            )
        })
        .fold(String::new(), |mut acc, s| {
            acc.push_str(s.as_str());
            acc
        })
}

fn format_external_docs_parsed(docs: Option<&openapi_parser::ExternalDocumentation>) -> String {
    if let Some(doc) = docs {
        formatdoc! {"
        ## External Docs

        [{}]({})
        ", doc.description.as_deref().unwrap_or("View addition documentation here"), doc.url
        }
    } else {
        String::new()
    }
}
