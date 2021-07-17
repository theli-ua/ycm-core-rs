use std::collections::HashMap;
use std::ops::Deref;

use regex::{Captures, Regex, RegexBuilder};

const C_STYLE_COMMENT: &str = "(/\\*(?:\n|.)*?\\*/)";
const CPP_STYLE_COMMENT: &str = "(//.*?$)";
const PYTHON_STYLE_COMMENT: &str = "(#.*?$)";

// Anything inside single quotes, '...', but mind:
//  1. that the starting single quote is not escaped
//  2. the escaped slash (\\)
//  3. the escaped single quote inside the string
const SINGLE_QUOTE_STRING: &str = r"(?:[^\\])('(?:\\\\|\\'|.)*?')";
// Anything inside double quotes, "...", but mind:
//  1. that the starting double quote is not escaped
//  2. the escaped slash (\\)
//  3. the escaped double quote inside the string
const DOUBLE_QUOTE_STRING: &str = r#"(?:[^\\])("(?:\\\\|\\"|.)*?")"#;
// Anything inside back quotes, `...`, but mind:
//  1. that the starting back quote is not escaped
//  2. the escaped slash (\\)
//  3. the escaped back quote inside the string
const BACK_QUOTE_STRING: &str = r#"(?:[^\\])(`(?:\\\\|\\`|.)*?`)"#;
// Python-style multiline single-quote string
const MULTILINE_SINGLE_QUOTE_STRING: &str = "('''(?:\n|.)*?''')";
// Python-style multiline double-quote string
const MULTILINE_DOUBLE_QUOTE_STRING: &str = r#"("""(?:\n|.)*?""")"#;

type RE = &'static (dyn Deref<Target = Regex> + Sync);

lazy_static::lazy_static! {

static ref DEFAULT_COMMENT_AND_STRING_REGEX: Regex = RegexBuilder::new(&[
    C_STYLE_COMMENT,
    CPP_STYLE_COMMENT,
    PYTHON_STYLE_COMMENT,
    SINGLE_QUOTE_STRING,
    DOUBLE_QUOTE_STRING,
    BACK_QUOTE_STRING,
    MULTILINE_SINGLE_QUOTE_STRING,
    MULTILINE_DOUBLE_QUOTE_STRING
].join("|")).multi_line(true).build().unwrap();

// Spec:
// http://www.open-std.org/jtc1/sc22/wg21/docs/papers/2013/n3690.pdf
static ref CPP_COMMENT_AND_STRING_REGEX: Regex = RegexBuilder::new(&[ C_STYLE_COMMENT,
                                                                CPP_STYLE_COMMENT,
                                                                SINGLE_QUOTE_STRING,
                                                                DOUBLE_QUOTE_STRING
                                                                ].join("|"))
    .multi_line(true).build().unwrap();

// Spec:
// https://golang.org/ref/spec#Comments
// https://golang.org/ref/spec#String_literals
// https://golang.org/ref/spec#Rune_literals
static ref GO_COMMENT_AND_STRING_REGEX: Regex = RegexBuilder::new(&[ C_STYLE_COMMENT,
                                                              CPP_STYLE_COMMENT,
                                                              SINGLE_QUOTE_STRING,
                                                              DOUBLE_QUOTE_STRING,
                                                              BACK_QUOTE_STRING
                                                            ].join("|"))
    .multi_line(true).build().unwrap();

// Spec:
// https://docs.python.org/3.6/reference/lexical_analysis.html#comments
// https://docs.python.org/3.6/reference/lexical_analysis.html#literals
static ref PYTHON_COMMENT_AND_STRING_REGEX: Regex = RegexBuilder::new(&[ PYTHON_STYLE_COMMENT,
                                                                  MULTILINE_SINGLE_QUOTE_STRING,
                                                                  MULTILINE_DOUBLE_QUOTE_STRING,
                                                                  SINGLE_QUOTE_STRING,
                                                                  DOUBLE_QUOTE_STRING
                                                            ].join("|"))
    .multi_line(true).build().unwrap();

// Spec:
// https://doc.rust-lang.org/reference.html#comments
// https://doc.rust-lang.org/reference.html#character-and-string-literals
static ref RUST_COMMENT_AND_STRING_REGEX: Regex = RegexBuilder::new(&[ CPP_STYLE_COMMENT,
                                                                  SINGLE_QUOTE_STRING,
                                                                  DOUBLE_QUOTE_STRING
                                                            ].join("|"))
    .multi_line(true).build().unwrap();

static ref FILETYPE_TO_COMMENT_AND_STRING_REGEX: HashMap<&'static str, RE> = {

    let mut map = HashMap::new();

    map.insert("cpp", &CPP_COMMENT_AND_STRING_REGEX as RE);
    map.insert("c", &CPP_COMMENT_AND_STRING_REGEX);
    map.insert("cuda", &CPP_COMMENT_AND_STRING_REGEX);
    map.insert("objc", &CPP_COMMENT_AND_STRING_REGEX);
    map.insert("objcpp", &CPP_COMMENT_AND_STRING_REGEX);
    map.insert("javascript", &CPP_COMMENT_AND_STRING_REGEX);
    map.insert("typesript", &CPP_COMMENT_AND_STRING_REGEX);

    map.insert("go", &GO_COMMENT_AND_STRING_REGEX);

    map.insert("python", &PYTHON_COMMENT_AND_STRING_REGEX);

    map.insert("rust", &RUST_COMMENT_AND_STRING_REGEX);

    map
};


// At least c++ and javascript support unicode identifiers, and identifiers may
// start with unicode character, e.g. ålpha. So we need to accept any identifier
// starting with an 'alpha' character or underscore. i.e. not starting with a
// 'digit'. The following regex will match:
//   - A character which is alpha or _. That is a character which is NOT:
//     - a digit (\d)
//     - non-alphanumeric
//     - not an underscore
//       (The latter two come from \W which is the negation of \w)
//   - Followed by any alphanumeric or _ characters
static ref DEFAULT_IDENTIFIER_REGEX: Regex = Regex::new(r"[^\W\d]\w*").unwrap();

}

fn get_comments_and_strings_re_for_ftype(filetype: Option<&str>) -> RE {
    match filetype {
        None => &DEFAULT_COMMENT_AND_STRING_REGEX,
        Some(t) => *FILETYPE_TO_COMMENT_AND_STRING_REGEX
            .get(t)
            .unwrap_or(&(&DEFAULT_COMMENT_AND_STRING_REGEX as RE)),
    }
}

fn replace_with_empty_lines(caps: &Captures) -> String {
    if caps.len() == 1 {
        std::iter::repeat("\n")
            .take(caps[0].lines().count() - 1)
            .collect()
    } else {
        let off = caps.get(0).unwrap().start();
        let mut prev = off;
        let whole = &caps[0];
        caps.iter()
            .skip(1)
            .flatten()
            .map(|c| -> String {
                let saved_prev = prev;
                prev = c.end();
                if saved_prev < c.start() {
                    std::iter::once(&whole[saved_prev - off..c.start() - off])
                        .chain(std::iter::repeat("\n").take(c.as_str().lines().count() - 1))
                        .collect()
                } else {
                    std::iter::repeat("\n")
                        .take(c.as_str().lines().count() - 1)
                        .collect()
                }
            })
            .collect()
    }
}

pub fn remove_identifier_free_text(text: &str, filetype: Option<&str>) -> String {
    get_comments_and_strings_re_for_ftype(filetype)
        .replace_all(text, replace_with_empty_lines)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remove_identifier_free_text_cpp_comments_test() {
        assert_eq!(
            "foo \nbar \nqux",
            &remove_identifier_free_text("foo \nbar //foo \nqux", None)
        );
    }

    #[test]
    fn remove_identifier_free_text_python_comments_test() {
        assert_eq!(
            "foo \nbar \nqux",
            &remove_identifier_free_text("foo \nbar #foo \nqux", None)
        );
    }

    #[test]
    fn remove_identifier_free_text_simple_double_quoted() {
        assert_eq!(
            "foo \nbar \nqux",
            &remove_identifier_free_text("foo \nbar \"foo\"\nqux", None)
        );
    }

    //TODO: port remained of freetext tests
}
