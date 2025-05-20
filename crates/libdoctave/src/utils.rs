/// Capitalize a str
pub(crate) fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(crate) fn cartesian_product<T: Clone>(lists: Vec<Vec<T>>) -> Vec<Vec<T>> {
    match lists.split_first() {
        Some((first, rest)) => {
            let init: Vec<Vec<T>> = first.iter().cloned().map(|n| vec![n]).collect();

            rest.iter()
                .cloned()
                .fold(init, |vec, list| partial_cartesian(vec, list))
        }
        None => {
            vec![]
        }
    }
}

fn partial_cartesian<T: Clone>(a: Vec<Vec<T>>, b: Vec<T>) -> Vec<Vec<T>> {
    a.into_iter()
        .flat_map(|xs| {
            b.iter()
                .cloned()
                .map(|y| {
                    let mut vec = xs.clone();
                    vec.push(y);
                    vec
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Serde helper for checking if a HashMap is empty
#[allow(dead_code)]
pub(crate) fn empty_map<K, V>(map: &std::collections::HashMap<K, V>) -> bool {
    map.is_empty()
}

/// Serde helper method for skipping some serde field serialization for empty lists
pub(crate) fn empty_vector<T>(v: &[T]) -> bool {
    v.is_empty()
}

/// Serde helper for checking if a bool is false
#[allow(dead_code)]
pub(crate) fn is_false(val: &bool) -> bool {
    !(*val)
}
