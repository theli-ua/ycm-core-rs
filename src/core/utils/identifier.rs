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
// Spec:
// http://www.ecma-international.org/ecma-262/6.0/#sec-names-and-keywords
// Default identifier plus the dollar sign.
static ref JS_IDENTIFIER_REGEX: Regex= Regex::new( r"(?:[^\W\d]|\$)[\w$]*").unwrap();
// Spec: https://www.w3.org/TR/css-syntax-3/#ident-token-diagram
static ref CSS_IDENTIFIER_REGEX: Regex = Regex::new( r"-?[^\W\d][\w-]*").unwrap();

// Spec: http://www.w3.org/TR/html5/syntax.html#tag-name-state
// But not quite since not everything we want to pull out is a tag name. We
// also want attribute names (and probably unquoted attribute values).
// And we also want to ignore common template chars like `}` and `{`.
static ref HTML_IDENTIFIER_REGEX: Regex = Regex::new( r#"[a-zA-Z][^\s/>='\\"}{\.]*"#).unwrap();

// Spec: http://cran.r-project.org/doc/manuals/r-release/R-lang.pdf
// Section 10.3.2.
// Can be any sequence of '.', '_' and alphanum BUT can't start with:
//   - '.' followed by digit
//   - digit
//   - '_'
static ref R_IDENTIFIER_REGEX: Regex = Regex::new( r"(?:\.\d|\d|_)?(?P<id>[\.\w]*)").unwrap();

// Spec: http://clojure.org/reader
// Section: Symbols
static ref CLOJURE_IDENTIFIER_REGEX: Regex =  Regex::new(
     r"[-\*\+!_\?:\.a-zA-Z][-\*\+!_\?:\.\w]*/?[-\*\+!_\?:\.\w]*").unwrap();

// Spec: http://www.haskell.org/onlinereport/lexemes.html
// Section 2.4
static ref HASKELL_IDENTIFIER_REGEX: Regex = Regex::new( r"[_a-zA-Z][\w']+").unwrap();

// Spec: ?
// Colons are often used in labels (e.g. \label{fig:foobar}) so we accept
// them in the middle of an identifier but not at its extremities. We also
// accept dashes for compound words.
static ref TEX_IDENTIFIER_REFEX: Regex = Regex::new( r"[^\W\d](?:[\w:-]*\w)?").unwrap();

// Spec: http://doc.perl6.org/language/syntax
static ref PERL6_IDENTIFIER_REGEX: Regex = Regex::new( r"[_a-zA-Z](?:\w|[-'](?:[_a-zA-Z]))*",).unwrap();


// https://www.scheme.com/tspl4/grammar.html#grammar:symbols
static ref SCHEME_IDENTIFIER_REGEX: Regex = Regex::new( r"\+|\-|\.\.\.|(?:->|(?:\\x[0-9A-Fa-f]+;|[!$%&*/:<=>?~^]|[^\W\d]))(?:\\x[0-9A-Fa-f]+;|[-+.@!$%&*/:<=>?~^\w])*").unwrap();


static ref FILETYPE_TO_IDENTIFIER_REGEX: HashMap<&'static str, RE> = {

    let mut map = HashMap::new();

    map.insert("javascript", &JS_IDENTIFIER_REGEX as RE);
    map.insert("typescript", &JS_IDENTIFIER_REGEX as RE);

    map.insert("css", &CSS_IDENTIFIER_REGEX);
    map.insert("scss", &CSS_IDENTIFIER_REGEX);
    map.insert("sass", &CSS_IDENTIFIER_REGEX);
    map.insert("less", &CSS_IDENTIFIER_REGEX);

    map.insert("html", &HTML_IDENTIFIER_REGEX);

    map.insert("r", &R_IDENTIFIER_REGEX);

    map.insert("clojure", &CLOJURE_IDENTIFIER_REGEX);
    map.insert("elisp", &CLOJURE_IDENTIFIER_REGEX);
    map.insert("lisp", &CLOJURE_IDENTIFIER_REGEX);

    map.insert("haskell", &HASKELL_IDENTIFIER_REGEX);

    map.insert("tex", &TEX_IDENTIFIER_REFEX);

    map.insert("perl6", &PERL6_IDENTIFIER_REGEX);

    map.insert("scheme", &SCHEME_IDENTIFIER_REGEX);

    map
};
}

fn get_comments_and_strings_re_for_ftype(filetype: Option<&str>) -> RE {
    match filetype {
        None => &DEFAULT_COMMENT_AND_STRING_REGEX,
        Some(t) => *FILETYPE_TO_COMMENT_AND_STRING_REGEX
            .get(t)
            .unwrap_or(&(&DEFAULT_COMMENT_AND_STRING_REGEX as RE)),
    }
}

fn get_identifier_re_for_ftype(filetype: Option<&str>) -> RE {
    match filetype {
        None => &DEFAULT_IDENTIFIER_REGEX,
        Some(t) => *FILETYPE_TO_IDENTIFIER_REGEX
            .get(t)
            .unwrap_or(&(&DEFAULT_IDENTIFIER_REGEX as RE)),
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

pub fn is_identifier(text: &str, filetype: Option<&str>) -> bool {
    if text.is_empty() {
        return false;
    }

    let re = get_identifier_re_for_ftype(filetype);
    if let Some(c) = re.captures(text) {
        if c.len() == 1 {
            c.get(0).unwrap().range() == (0..text.len())
        } else {
            c.name("id").unwrap().range() == (0..text.len())
        }
    } else {
        false
    }
}

// index is 0-based and EXCLUSIVE, so ("foo.", 3) -> 0
// Returns the index on bad input.
// Note: its different from python ycmd as its both expects and returns byte position
pub fn start_of_longest_identifier_ending_at_index(
    text: &str,
    index: usize,
    filetype: Option<&str>,
) -> usize {
    if text.len() < index || !text.is_char_boundary(index) {
        return index;
    }

    for i in 0..index {
        if text.is_char_boundary(i) && is_identifier(&text[i..index], filetype) {
            return i;
        }
    }
    index
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

    #[test]
    fn is_identifier_generic() {
        assert!(is_identifier("foo", None));
        assert!(is_identifier("foo129", None));
        assert!(is_identifier("f12", None));
        assert!(is_identifier("f12", None));

        assert!(is_identifier("_foo", None));
        assert!(is_identifier("_foo129", None));
        assert!(is_identifier("_f12", None));
        assert!(is_identifier("_f12", None));

        assert!(is_identifier("uniçode", None));
        assert!(is_identifier("uç", None));
        assert!(is_identifier("ç", None));
        assert!(is_identifier("çode", None));

        assert!(!is_identifier("1foo129", None));
        assert!(!is_identifier("-foo", None));
        assert!(!is_identifier("foo-", None));
        assert!(!is_identifier("font-face", None));
        assert!(!is_identifier("", None));
    }

    #[test]
    fn is_identifier_js() {
        assert!(is_identifier("_føo1", Some("javascript")));
        assert!(is_identifier("fø_o1", Some("javascript")));
        assert!(is_identifier("$føo1", Some("javascript")));
        assert!(is_identifier("fø$o1", Some("javascript")));

        assert!(!is_identifier("1føo", Some("javascript")));
    }

    #[test]
    fn is_identifier_ts() {
        assert!(is_identifier("_føo1", Some("typescript")));
        assert!(is_identifier("fø_o1", Some("typescript")));
        assert!(is_identifier("$føo1", Some("typescript")));
        assert!(is_identifier("fø$o1", Some("typescript")));

        assert!(!is_identifier("1føo", Some("typescript")));
    }

    #[test]
    fn is_identifier_css() {
        assert!(is_identifier("foo", Some("css")));
        assert!(is_identifier("a", Some("css")));
        assert!(is_identifier("a1", Some("css")));
        assert!(is_identifier("a-", Some("css")));
        assert!(is_identifier("a-b", Some("css")));
        assert!(is_identifier("_b", Some("css")));
        assert!(is_identifier("-ms-foo", Some("css")));
        assert!(is_identifier("-_o", Some("css")));
        assert!(is_identifier("font-face", Some("css")));
        assert!(is_identifier("αβγ", Some("css")));

        assert!(!is_identifier("-3b", Some("css")));
        assert!(!is_identifier("-3", Some("css")));
        assert!(!is_identifier("--", Some("css")));
        assert!(!is_identifier("3", Some("css")));
        assert!(!is_identifier("€", Some("css")));
        assert!(!is_identifier("", Some("css")));
    }

    #[test]
    fn is_identifier_r() {
        assert!(is_identifier("a", Some("r")));
        assert!(is_identifier("a.b", Some("r")));
        assert!(is_identifier("a.b.c", Some("r")));
        assert!(is_identifier("a_b", Some("r")));
        assert!(is_identifier("a1", Some("r")));
        assert!(is_identifier("a_1", Some("r")));
        assert!(is_identifier(".a", Some("r")));
        assert!(is_identifier(".a_b", Some("r")));
        assert!(is_identifier(".a1", Some("r")));
        assert!(is_identifier("...", Some("r")));
        assert!(is_identifier("..1", Some("r")));

        assert!(!is_identifier(".1a", Some("r")));
        assert!(!is_identifier(".1", Some("r")));
        assert!(!is_identifier("1a", Some("r")));
        assert!(!is_identifier("123", Some("r")));
        assert!(!is_identifier("_1a", Some("r")));
        assert!(!is_identifier("_a", Some("r")));
        assert!(!is_identifier("", Some("r")));
    }

    #[test]
    fn is_identifier_clojure() {
        assert!(is_identifier("foo", Some("clojure")));
        assert!(is_identifier("f9", Some("clojure")));
        assert!(is_identifier("a.b.c", Some("clojure")));
        assert!(is_identifier("a.c", Some("clojure")));
        assert!(is_identifier("a/c", Some("clojure")));
        assert!(is_identifier("*", Some("clojure")));
        assert!(is_identifier("a*b", Some("clojure")));
        assert!(is_identifier("?", Some("clojure")));
        assert!(is_identifier("a?b", Some("clojure")));
        assert!(is_identifier(":", Some("clojure")));
        assert!(is_identifier("a:b", Some("clojure")));
        assert!(is_identifier("+", Some("clojure")));
        assert!(is_identifier("a+b", Some("clojure")));
        assert!(is_identifier("-", Some("clojure")));
        assert!(is_identifier("a-b", Some("clojure")));
        assert!(is_identifier("!", Some("clojure")));
        assert!(is_identifier("a!b", Some("clojure")));

        assert!(!is_identifier("9f", Some("clojure")));
        assert!(!is_identifier("9", Some("clojure")));
        assert!(!is_identifier("a/b/c", Some("clojure")));
        assert!(!is_identifier("(a)", Some("clojure")));
        assert!(!is_identifier("", Some("clojure")));
    }

    #[test]
    fn is_identifier_elisp() {
        // elisp is using the clojure regexes, so we're testing this more lightly
        assert!(is_identifier("foo", Some("elisp")));
        assert!(is_identifier("f9", Some("elisp")));
        assert!(is_identifier("a.b.c", Some("elisp")));
        assert!(is_identifier("a/c", Some("elisp")));

        assert!(!is_identifier("9f", Some("elisp")));
        assert!(!is_identifier("9", Some("elisp")));
        assert!(!is_identifier("a/b/c", Some("elisp")));
        assert!(!is_identifier("(a)", Some("elisp")));
        assert!(!is_identifier("", Some("elisp")));
    }

    #[test]
    fn is_identifier_haskell() {
        assert!(is_identifier("foo", Some("haskell")));
        assert!(is_identifier("foo'", Some("haskell")));
        assert!(is_identifier("x'", Some("haskell")));
        assert!(is_identifier("_x'", Some("haskell")));
        assert!(is_identifier("_x", Some("haskell")));
        assert!(is_identifier("x9", Some("haskell")));

        assert!(!is_identifier("'x", Some("haskell")));
        assert!(!is_identifier("9x", Some("haskell")));
        assert!(!is_identifier("9", Some("haskell")));
        assert!(!is_identifier("", Some("haskell")));
    }

    #[test]
    fn is_identifier_tex() {
        assert!(is_identifier("foo", Some("tex")));
        assert!(is_identifier("fig:foo", Some("tex")));
        assert!(is_identifier("fig:foo-bar", Some("tex")));
        assert!(is_identifier("sec:summary", Some("tex")));
        assert!(is_identifier("eq:bar_foo", Some("tex")));
        assert!(is_identifier("fōo", Some("tex")));
        assert!(is_identifier("some8", Some("tex")));

        assert!(!is_identifier("\\section", Some("tex")));
        assert!(!is_identifier("foo:", Some("tex")));
        assert!(!is_identifier("-bar", Some("tex")));
        assert!(!is_identifier("", Some("tex")));
    }

    #[test]
    fn is_identifier_perl() {
        assert!(is_identifier("foo", Some("perl6")));
        assert!(is_identifier("f-o", Some("perl6")));
        assert!(is_identifier("x'y", Some("perl6")));
        assert!(is_identifier("_x-y", Some("perl6")));
        assert!(is_identifier("x-y'a", Some("perl6")));
        assert!(is_identifier("x-_", Some("perl6")));
        assert!(is_identifier("x-_7", Some("perl6")));
        assert!(is_identifier("_x", Some("perl6")));
        assert!(is_identifier("x9", Some("perl6")));

        assert!(!is_identifier("'x", Some("perl6")));
        assert!(!is_identifier("x'", Some("perl6")));
        assert!(!is_identifier("-x", Some("perl6")));
        assert!(!is_identifier("x-", Some("perl6")));
        assert!(!is_identifier("x-1", Some("perl6")));
        assert!(!is_identifier("x--", Some("perl6")));
        assert!(!is_identifier("x--a", Some("perl6")));
        assert!(!is_identifier("x-'", Some("perl6")));
        assert!(!is_identifier("x-'a", Some("perl6")));
        assert!(!is_identifier("x-a-", Some("perl6")));
        assert!(!is_identifier("x+", Some("perl6")));
        assert!(!is_identifier("9x", Some("perl6")));
        assert!(!is_identifier("9", Some("perl6")));
        assert!(!is_identifier("", Some("perl6")));
    }

    #[test]
    fn is_identifier_scheme() {
        assert!(is_identifier("λ", Some("scheme")));
        assert!(is_identifier("_", Some("scheme")));
        assert!(is_identifier("+", Some("scheme")));
        assert!(is_identifier("-", Some("scheme")));
        assert!(is_identifier("...", Some("scheme")));
        assert!(is_identifier(r"\x01;", Some("scheme")));
        assert!(is_identifier(r"h\x65;lle", Some("scheme")));
        assert!(is_identifier("foo", Some("scheme")));
        assert!(is_identifier("foo+-*/1-1", Some("scheme")));
        assert!(is_identifier("call/cc", Some("scheme")));

        assert!(!is_identifier(".", Some("scheme")));
        assert!(!is_identifier("..", Some("scheme")));
        assert!(!is_identifier("--", Some("scheme")));
        assert!(!is_identifier("++", Some("scheme")));
        assert!(!is_identifier("+1", Some("scheme")));
        assert!(!is_identifier("-1", Some("scheme")));
        assert!(!is_identifier("-abc", Some("scheme")));
        assert!(!is_identifier("-<abc", Some("scheme")));
        assert!(!is_identifier("@", Some("scheme")));
        assert!(!is_identifier("@a", Some("scheme")));
        assert!(!is_identifier("-@a", Some("scheme")));
        assert!(!is_identifier("-12a", Some("scheme")));
        assert!(!is_identifier("12a", Some("scheme")));
        assert!(!is_identifier("\\", Some("scheme")));
        assert!(!is_identifier(r"\x", Some("scheme")));
        assert!(!is_identifier(r"\x123", Some("scheme")));
        assert!(!is_identifier(r"aa\x123;cc\x", Some("scheme")));
    }

    #[test]
    fn start_of_longest_identifier_ending_at_index_simple() {
        assert_eq!(
            0,
            start_of_longest_identifier_ending_at_index("foo", 3, None)
        );
        assert_eq!(
            0,
            start_of_longest_identifier_ending_at_index("f12", 3, None)
        );
    }
    #[test]
    fn start_of_longest_identifier_ending_at_index_badinput() {
        assert_eq!(0, start_of_longest_identifier_ending_at_index("", 0, None));
        assert_eq!(1, start_of_longest_identifier_ending_at_index("", 1, None));
        assert_eq!(5, start_of_longest_identifier_ending_at_index("", 5, None));
        assert_eq!(
            usize::MAX,
            start_of_longest_identifier_ending_at_index("foo", usize::MAX, None)
        );
        assert_eq!(
            10,
            start_of_longest_identifier_ending_at_index("foo", 10, None)
        );
    }

    #[test]
    fn start_of_longest_identifier_ending_at_index_punctuation() {
        assert_eq!(
            1,
            start_of_longest_identifier_ending_at_index("(foo", 4, None)
        );
        assert_eq!(
            6,
            start_of_longest_identifier_ending_at_index("      foo", 9, None)
        );
        assert_eq!(
            4,
            start_of_longest_identifier_ending_at_index("gar;foo", 7, None)
        );
        assert_eq!(
            2,
            start_of_longest_identifier_ending_at_index("...", 2, None)
        );
    }

    #[test]
    fn start_of_longest_identifier_ending_at_index_unicode() {
        assert_eq!(
            1,
            start_of_longest_identifier_ending_at_index("(fäö", 4, None)
        );
        assert_eq!(
            6,
            start_of_longest_identifier_ending_at_index("fäö(fäö", 11, None)
        );
        assert_eq!(
            2,
            start_of_longest_identifier_ending_at_index("  fäö", 5, None)
        );
    }

    //TODO: port all other tests
}
