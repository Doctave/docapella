use tl::VDom;

fn parse(html: &str) -> VDom {
    tl::parse(html, tl::ParserOptions::default()).unwrap()
}

pub fn htmls(html: &str, selector: &str) -> Vec<String> {
    let dom = parse(html);

    let handle = dom.query_selector(selector).unwrap();

    handle
        .map(|node| {
            node.get(dom.parser())
                .unwrap()
                .inner_html(dom.parser())
                .to_string()
        })
        .collect()
}

#[allow(dead_code)]
pub fn count_children(html: &str, selector: &str) -> Vec<usize> {
    let dom = parse(html);

    let handle = dom.query_selector(selector).unwrap();

    handle
        .map(|node| {
            node.get(dom.parser())
                .unwrap()
                .children()
                .unwrap()
                .top()
                .len()
        })
        .collect()
}

pub fn inner_texts(html: &str, selector: &str) -> Vec<String> {
    // tl lib has a bug and does not support child selector
    if let Some((parent, child)) = selector.split_once(" > ") {
        let htmls = htmls(html, parent);

        htmls
            .iter()
            .flat_map(|html| inner_texts(html, child))
            .collect()
    } else {
        let dom = parse(html);

        let handle = dom.query_selector(selector).unwrap();

        let inner_texts: Vec<String> = handle
            .map(|node| {
                node.get(dom.parser())
                    .unwrap()
                    .inner_text(dom.parser())
                    .into_owned()
            })
            .collect();

        inner_texts
    }
}
