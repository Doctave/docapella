use crate::open_api::ast::PageAst;
use crate::NodeKind;
use crate::Project;
use crate::Result;

pub struct SearchIndex {
    index: elasticlunr::Index,
    doc_id: u64,
}

impl SearchIndex {
    pub fn new(project: &Project) -> Result<Self> {
        let eindex = elasticlunr::IndexBuilder::new()
            .add_fields(&[
                "title",
                "lvl0",
                "lvl1",
                "lvl2",
                "lvl3",
                "lvl4",
                "lvl5",
                "text",
                "code",
                "alt",
                "openapi_tag",
                "openapi_summary",
                "openapi_description",
                "openapi_path",
                "openapi_method",
                "page_kind",
            ])
            .save_docs(true)
            .build();

        let mut index = Self {
            index: eindex,
            doc_id: 0,
        };

        for page in project.pages() {
            match page.ast(None) {
                Ok(ast) => {
                    match ast {
                        crate::Ast::Markdown(ast) => {
                            index_markdown(&mut index, ast, page.uri_path());
                        }
                        crate::Ast::OpenApi(ast) => {
                            index_openapi(&mut index, ast, page.uri_path());
                        }
                    };
                }
                Err(_) => {
                    // Ignore pages that can't be rendered
                }
            }
        }

        Ok(index)
    }

    fn add_doc(&mut self, doc: &[&str]) {
        self.index.add_doc(&format!("{}", self.doc_id), doc);
        self.doc_id += 1;
    }

    pub fn to_json(&self) -> String {
        self.index.to_json()
    }
}

#[derive(Debug)]
struct DocumentBuilder {
    title: String,
    lvl0: String,
    lvl1: String,
    lvl2: String,
    lvl3: String,
    lvl4: String,
    lvl5: String,
    text: String,
    code: String,
    alt: String,
    openapi_tag: String,
    openapi_summary: String,
    openapi_description: String,
    openapi_path: String,
    openapi_method: String,
    page_kind: String,
}

impl DocumentBuilder {
    fn markdown() -> Self {
        Self {
            title: String::new(),
            lvl0: String::new(),
            lvl1: String::new(),
            lvl2: String::new(),
            lvl3: String::new(),
            lvl4: String::new(),
            lvl5: String::new(),
            text: String::new(),
            code: String::new(),
            alt: String::new(),
            openapi_tag: String::new(),
            openapi_summary: String::new(),
            openapi_description: String::new(),
            openapi_path: String::new(),
            openapi_method: String::new(),
            page_kind: "markdown".to_string(),
        }
    }

    fn openapi() -> Self {
        Self {
            title: String::new(),
            lvl0: String::new(),
            lvl1: String::new(),
            lvl2: String::new(),
            lvl3: String::new(),
            lvl4: String::new(),
            lvl5: String::new(),
            text: String::new(),
            code: String::new(),
            alt: String::new(),
            openapi_tag: String::new(),
            openapi_summary: String::new(),
            openapi_description: String::new(),
            openapi_path: String::new(),
            openapi_method: String::new(),
            page_kind: "openapi".to_string(),
        }
    }

    fn as_elasticlunr_document(&self) -> Vec<&str> {
        vec![
            &self.lvl0,
            &self.lvl1,
            &self.lvl2,
            &self.lvl3,
            &self.lvl4,
            &self.lvl5,
            &self.text,
            &self.code,
            &self.alt,
            &self.openapi_tag,
            &self.openapi_summary,
            &self.openapi_description,
            &self.openapi_path,
            &self.openapi_method,
            &self.page_kind,
        ]
    }
}

fn index_markdown(index: &mut SearchIndex, ast: crate::markdown::Node, title: &str) {
    fn index_node(node: &crate::markdown::Node, doc: &mut DocumentBuilder) {
        match &node.kind {
            NodeKind::Heading { level, .. } => match level {
                1 => {
                    doc.lvl0.push_str(&node.inner_text());
                    doc.lvl0.push(' ');
                }
                2 => {
                    doc.lvl1.push_str(&node.inner_text());
                    doc.lvl1.push(' ');
                }
                3 => {
                    doc.lvl2.push_str(&node.inner_text());
                    doc.lvl2.push(' ');
                }
                4 => {
                    doc.lvl3.push_str(&node.inner_text());
                    doc.lvl3.push(' ');
                }
                5 => {
                    doc.lvl4.push_str(&node.inner_text());
                    doc.lvl4.push(' ');
                }
                6 => {
                    doc.lvl5.push_str(&node.inner_text());
                    doc.lvl5.push(' ');
                }
                _ => {}
            },
            NodeKind::Text { value } => {
                doc.text.push_str(value);
                doc.text.push(' ');
            }
            NodeKind::Image { alt, .. } => {
                doc.alt.push_str(alt);
                doc.alt.push(' ');
            }
            NodeKind::Code { value, .. } => {
                doc.text.push_str(value);
                doc.text.push(' ');
            }
            _ => {
                for child in &node.children {
                    index_node(child, doc);
                }
            }
        }
    }

    let mut doc = DocumentBuilder::markdown();
    doc.title = title.to_string();

    index_node(&ast, &mut doc);

    index.add_doc(&doc.as_elasticlunr_document());
}

fn index_openapi(index: &mut SearchIndex, ast: PageAst, title: &str) {
    for operation in &ast.operations {
        let mut doc = DocumentBuilder::openapi();
        doc.title = title.to_string();
        doc.openapi_tag = ast.tag.name.clone();
        doc.openapi_path = operation.route_pattern.clone();
        doc.openapi_method = operation.method.clone();
        doc.openapi_summary = operation.summary.clone().unwrap_or_default();
        if let Some(description) = &operation.description_ast {
            doc.openapi_description = description.inner_text();
        }

        index.add_doc(&doc.as_elasticlunr_document());
    }
}
