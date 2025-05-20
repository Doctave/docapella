use std::collections::HashMap;

pub fn parse_attributes(input: &str) -> HashMap<String, Option<String>> {
    let mut map: HashMap<String, Option<String>> = HashMap::new();

    let mut key = vec![];
    let mut value = vec![];

    let mut in_double_quotes = false;
    let mut in_single_quotes = false;

    for token in input.trim().chars() {
        match token {
            ' ' => {
                if in_double_quotes || in_single_quotes {
                    value.push(token);
                } else if key.is_empty() {
                    let key_s: String = value.drain(..).collect();

                    map.insert(key_s, None);
                } else {
                    let key_s: String = key.drain(..).collect();
                    let value_s: String = value.drain(..).collect();

                    map.insert(key_s, Some(value_s));
                }
            }
            '=' => {
                key = std::mem::take(&mut value);
            }
            '"' => {
                if in_single_quotes {
                    value.push(token);
                } else {
                    in_double_quotes = !in_double_quotes;
                }
            }
            '\'' => {
                if in_double_quotes {
                    value.push(token);
                } else {
                    in_single_quotes = !in_single_quotes;
                }
            }
            _ => {
                value.push(token);
            }
        }
    }

    if key.is_empty() {
        let key_s: String = value.drain(..).collect();

        map.insert(key_s, None);
    } else {
        let key_s: String = key.drain(..).collect();
        let value_s: String = value.drain(..).collect();

        map.insert(key_s, Some(value_s));
    }

    map
}

#[cfg(test)]
mod test {
    #[test]
    fn basic_attribute_parse() {
        let input = indoc! {r#"
        title="foo" bar="baz"
        "#};

        let map = super::parse_attributes(input);

        assert_eq!(map.get("title"), Some(&Some("foo".to_string())));
        assert_eq!(map.get("bar"), Some(&Some("baz".to_string())));
    }

    #[test]
    fn no_value() {
        let input = indoc! {r#"
        title some= bar="baz" asdf=
        "#};

        let map = super::parse_attributes(input);

        assert_eq!(map.get("title"), Some(&None));
        assert_eq!(map.get("bar"), Some(&Some("baz".to_string())));
        assert_eq!(map.get("some"), Some(&Some("".to_string())));
        assert_eq!(map.get("asdf"), Some(&Some("".to_string())));
    }

    #[test]
    fn no_value_last() {
        let input = indoc! {r#"
        title bar="baz" some
        "#};

        let map = super::parse_attributes(input);

        assert_eq!(map.get("title"), Some(&None));
        assert_eq!(map.get("bar"), Some(&Some("baz".to_string())));
        assert_eq!(map.get("some"), Some(&None));
    }
}
