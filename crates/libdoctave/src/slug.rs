//! Module for converting a string into a URL safe slug.
//!
//! This module is based on the following function from another crate:
//! https://docs.rs/slug/latest/slug/fn.slugify.html
//!
//! The reason I did not bring in the crate as a dependency were:
//!
//! 1. The original function lowercased all slugs, which goes against Doctave
//!    current behavior.
//! 2. It's just a single function (+ one other unicode crate) and easy to port
//!
//! Nik 22/11/2022

pub fn slugify(s: &str) -> String {
    let mut slug: Vec<u8> = Vec::with_capacity(s.len());
    // Starts with true to avoid leading -
    let mut prev_is_dash = true;
    {
        let mut push_char = |x: char| match x {
            ' ' => {
                if !prev_is_dash {
                    prev_is_dash = true;
                    slug.push(b'-');
                }
            }
            '-' => {
                if !prev_is_dash {
                    prev_is_dash = true;
                    slug.push(b'-');
                }
            }
            'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '~' => {
                prev_is_dash = false;
                slug.push(x as u8);
            }
            _ => {
                if !prev_is_dash {
                    slug.push(b'-');
                    prev_is_dash = true;
                }
            }
        };

        for c in s.chars() {
            if c.is_ascii() {
                (push_char)(c);
            } else {
                for cx in deunicode::deunicode_char(c).unwrap_or("-").chars() {
                    (push_char)(cx);
                }
            }
        }
    }

    let mut string = String::from_utf8(slug).expect("Generated non-utf8 slug");
    if string.ends_with('-') {
        string.pop();
    }
    // We likely reserved more space than needed.
    string.shrink_to_fit();
    string
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn does_not_lowercase_caps() {
        assert_eq!(&slugify("FOO"), "FOO");
        assert_eq!(&slugify("foo"), "foo");
    }

    #[test]
    fn does_not_update_underscors_to_dashes() {
        assert_eq!(&slugify("foo_bar"), "foo_bar");
    }

    #[test]
    fn original_tests_from_crate() {
        assert_eq!(slugify("My Test String!!!1!1"), "My-Test-String-1-1");
        assert_eq!(slugify("test\nit   now!"), "test-it-now");
        assert_eq!(slugify("  --test-cool"), "test-cool");
        assert_eq!(slugify("Æúű--cool?"), "AEuu-cool");
        assert_eq!(slugify("You & Me"), "You-Me");
        assert_eq!(slugify("user@example.com"), "user-example.com");
    }

    #[test]
    fn handles_finnish_aakkoset() {
        assert_eq!(slugify("äÄöÖ"), "aAoO");
    }
}
