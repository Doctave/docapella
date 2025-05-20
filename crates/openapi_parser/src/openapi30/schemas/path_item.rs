use super::{operation::Operation, parameter::Parameter, server::Server};
use crate::{
    openapi30::parser::{self, ParserContext},
    Set, String, Value,
};

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Clone)]
pub struct PathItem {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub get: Option<Operation>,
    pub put: Option<Operation>,
    pub post: Option<Operation>,
    pub delete: Option<Operation>,
    pub options: Option<Operation>,
    pub head: Option<Operation>,
    pub patch: Option<Operation>,
    pub trace: Option<Operation>,
    pub servers: Vec<Server>,
}

impl PathItem {
    #[cfg(test)]
    pub(crate) fn pretty_print(self) -> String {
        let val: Value = self.into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn operations(&self) -> Vec<&Operation> {
        vec![
            self.get.as_ref(),
            self.put.as_ref(),
            self.post.as_ref(),
            self.delete.as_ref(),
            self.options.as_ref(),
            self.head.as_ref(),
            self.patch.as_ref(),
            self.trace.as_ref(),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    pub fn try_parse(
        mut value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
        route_pattern: String,
        root_servers: &[Server],
    ) -> parser::Result<Self> {
        let summary = value.take("summary").and_then(Value::take_string);
        let description = value.take("description").and_then(Value::take_string);

        let mut servers = vec![];
        if let Some(_servers) = value.take("servers").and_then(Value::take_array) {
            for server in _servers {
                servers.push(Server::try_parse(server)?)
            }
        } else {
            servers = root_servers.to_vec();
        }

        let path_params = value
            .take("parameters")
            .and_then(Value::take_array)
            .map(|vec| {
                vec.into_iter()
                    .map(|v| Parameter::try_parse(v, ctx, visited_refs))
                    .collect::<parser::Result<Vec<_>>>()
            })
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default();

        Ok(Self {
            summary,
            description,
            get: value
                .take("get")
                .map(|v| {
                    Operation::try_parse(
                        v,
                        ctx,
                        visited_refs,
                        "get".into(),
                        route_pattern.clone(),
                        path_params.clone(),
                        &servers,
                    )
                })
                .transpose()?,
            put: value
                .take("put")
                .map(|v| {
                    Operation::try_parse(
                        v,
                        ctx,
                        visited_refs,
                        "put".into(),
                        route_pattern.clone(),
                        path_params.clone(),
                        &servers,
                    )
                })
                .transpose()?,
            post: value
                .take("post")
                .map(|v| {
                    Operation::try_parse(
                        v,
                        ctx,
                        visited_refs,
                        "post".into(),
                        route_pattern.clone(),
                        path_params.clone(),
                        &servers,
                    )
                })
                .transpose()?,
            delete: value
                .take("delete")
                .map(|v| {
                    Operation::try_parse(
                        v,
                        ctx,
                        visited_refs,
                        "delete".into(),
                        route_pattern.clone(),
                        path_params.clone(),
                        &servers,
                    )
                })
                .transpose()?,
            options: value
                .take("options")
                .map(|v| {
                    Operation::try_parse(
                        v,
                        ctx,
                        visited_refs,
                        "options".into(),
                        route_pattern.clone(),
                        path_params.clone(),
                        &servers,
                    )
                })
                .transpose()?,
            head: value
                .take("head")
                .map(|v| {
                    Operation::try_parse(
                        v,
                        ctx,
                        visited_refs,
                        "head".into(),
                        route_pattern.clone(),
                        path_params.clone(),
                        &servers,
                    )
                })
                .transpose()?,
            patch: value
                .take("patch")
                .map(|v| {
                    Operation::try_parse(
                        v,
                        ctx,
                        visited_refs,
                        "patch".into(),
                        route_pattern.clone(),
                        path_params.clone(),
                        &servers,
                    )
                })
                .transpose()?,
            trace: value
                .take("trace")
                .map(|v| {
                    Operation::try_parse(
                        v,
                        ctx,
                        visited_refs,
                        "trace".into(),
                        route_pattern.clone(),
                        path_params,
                        &servers,
                    )
                })
                .transpose()?,
            servers,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    use crate::json;

    #[test]
    fn parses_path_item_summary() {
        let value = json!({
            "summary": "summary"
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "summary": "summary"
          }"# }
        )
    }

    #[test]
    fn parses_path_item_description() {
        let value = json!({
            "description": "description"
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "description": "description"
          }"# }
        )
    }

    #[test]
    fn parses_path_item_servers() {
        let value = json!({
            "servers": [
                {
                    "url": "https://api.example.com/v1",
                    "description": "Production server"
                }
            ]
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "servers": [
              {
                "url": "https://api.example.com/v1",
                "description": "Production server"
              }
            ]
          }"# }
        )
    }

    #[test]
    fn parses_get() {
        let value = json!({
            "get": {
                "summary": "summary",
                "description": "description"
            }
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "get": {
              "summary": "summary",
              "description": "description",
              "method": "get"
            }
          }"# }
        )
    }

    #[test]
    fn parses_put() {
        let value = json!({
            "put": {
                "summary": "summary",
                "description": "description"
            }
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "put": {
              "summary": "summary",
              "description": "description",
              "method": "put"
            }
          }"# }
        )
    }

    #[test]
    fn parses_post() {
        let value = json!({
            "post": {
                "summary": "summary",
                "description": "description"
            }
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "post": {
              "summary": "summary",
              "description": "description",
              "method": "post"
            }
          }"# }
        )
    }

    #[test]
    fn parses_delete() {
        let value = json!({
            "delete": {
                "summary": "summary",
                "description": "description"
            }
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "delete": {
              "summary": "summary",
              "description": "description",
              "method": "delete"
            }
          }"# }
        )
    }

    #[test]
    fn parses_options() {
        let value = json!({
            "options": {
                "summary": "summary",
                "description": "description"
            }
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "options": {
              "summary": "summary",
              "description": "description",
              "method": "options"
            }
          }"# }
        )
    }

    #[test]
    fn parses_head() {
        let value = json!({
            "head": {
                "summary": "summary",
                "description": "description"
            }
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "head": {
              "summary": "summary",
              "description": "description",
              "method": "head"
            }
          }"# }
        )
    }

    #[test]
    fn parses_patch() {
        let value = json!({
            "patch": {
                "summary": "summary",
                "description": "description"
            }
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "patch": {
              "summary": "summary",
              "description": "description",
              "method": "patch"
            }
          }"# }
        )
    }

    #[test]
    fn parses_trace() {
        let value = json!({
            "trace": {
                "summary": "summary",
                "description": "description"
            }
        });

        let path_item = PathItem::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print(),
            indoc! {r#"
          {
            "trace": {
              "summary": "summary",
              "description": "description",
              "method": "trace"
            }
          }"# }
        )
    }

    #[test]
    fn uses_servers_from_parent() {
        let server = Server::try_parse(json!(
          {
              "url": "https://api.example.com/v1",
              "description": "Production server"
          }
        ))
        .unwrap();

        let path_item = PathItem::try_parse(
            json!({}),
            &ParserContext::default(),
            &mut Set::new(),
            "/path".into(),
            &[server],
        )
        .unwrap();

        assert_str_eq!(
            path_item.pretty_print().as_str(),
            indoc! {r#"
            {
              "servers": [
                {
                  "url": "https://api.example.com/v1",
                  "description": "Production server"
                }
              ]
            }"#}
        );
    }

    #[test]
    fn overwrites_parent_servers() {
        let server = Server::try_parse(json!(
          {
              "url": "https://api.example.com/v1",
              "description": "Production server"
          }
        ))
        .unwrap();

        let operation = Operation::try_parse(
            json!({
                "servers": [
                    {
                        "url": "https://api.example.com/v2",
                        "description": "Staging server"
                    }
                ]
            }),
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[server],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "servers": [
                {
                  "url": "https://api.example.com/v2",
                  "description": "Staging server"
                }
              ],
              "method": "get"
            }"#}
        );
    }
}
