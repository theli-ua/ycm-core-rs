use std::collections::{HashMap, HashSet};

use regex::{escape, Regex, RegexSet};

const REGEX_PREFIX: &str = "re!";

pub fn parse_triggers(
    triggers: Vec<HashMap<String, Vec<String>>>,
    filetypes: &HashSet<String>,
) -> HashMap<String, RegexSet> {
    let mut res = HashMap::new();
    for mut map in triggers.into_iter() {
        for (k, v) in map.drain() {
            for ftype in k
                .split(',')
                .filter(|f| filetypes.is_empty() || filetypes.contains(*f))
            {
                let re = res.entry(ftype.into()).or_insert(RegexSet::empty());
                let mut patterns: Vec<_> = v
                    .iter()
                    .map(|p| {
                        if p.starts_with(REGEX_PREFIX) {
                            String::from(&p[REGEX_PREFIX.len()..])
                        } else {
                            escape(p)
                        }
                    })
                    .collect();
                patterns.extend_from_slice(re.patterns());
                *re = RegexSet::new(&patterns).unwrap();
            }
        }
    }

    res
}

pub trait PatternMatcher {
    fn matches_for_filetype(
        &self,
        filetype: &str,
        line: &str,
        start_codepoint: usize,
        column_codepoint: usize,
    ) -> bool;
}

impl PatternMatcher for HashMap<String, RegexSet> {
    fn matches_for_filetype(
        &self,
        filetype: &str,
        line: &str,
        start_codepoint: usize,
        column_codepoint: usize,
    ) -> bool {
        let line = if column_codepoint < line.len() {
            &line[..column_codepoint]
        } else {
            &line[..]
        };
        match self.get(filetype) {
            None => false,
            Some(re) => {
                for m in re.matches(line) {
                    for m in Regex::new(&re.patterns()[m]).unwrap().find_iter(line) {
                        /*
                            By definition of 'start_codepoint', we know that the character just before
                            'start_codepoint' is not an identifier character but all characters
                            between 'start_codepoint' and 'column_codepoint' are. This means that if
                            our trigger ends with an identifier character, its tail must match between
                            'start_codepoint' and 'column_codepoint', 'start_codepoint' excluded. But
                            if it doesn't, its tail must match exactly at 'start_codepoint'. Both
                            cases are mutually exclusive hence the following condition.
                        */
                        if start_codepoint <= m.end() && m.end() <= column_codepoint {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_default() -> HashMap<String, Vec<String>> {
        vec![
            ("c".into(), vec![".".into(), "->".into()]),
            (
                "objc,objcpp".into(),
                vec!["->".into(), ".".into(), r"re!\[[_a-zA-Z]+\w*\s".into()],
            ),
            ("objc".into(), vec!["foo".into()]),
        ]
        .into_iter()
        .collect::<HashMap<String, Vec<String>>>()
    }
    #[test]
    fn test_triggers() {
        let input = get_default();

        let output = parse_triggers(vec![input], &HashSet::default());

        assert_eq!(3, output.len());
        assert!(output["c"].is_match("."));
        assert!(output["c"].is_match("->"));

        assert!(output["objcpp"].is_match("."));
        assert!(output["objcpp"].is_match("->"));
        assert!(output["objcpp"].is_match("[asdf_asdasFF_FF asdf asdf "));

        assert!(output["objc"].is_match("."));
        assert!(output["objc"].is_match("->"));
        assert!(output["objc"].is_match("[asdf_asdasFF_FF asdf asdf "));

        assert!(output["objc"].is_match("foo"));
        assert!(!output["objcpp"].is_match("foo"));
    }

    #[test]
    fn test_matcher() {
        let triggers = parse_triggers(vec![get_default()], &HashSet::default());
        assert!(triggers.matches_for_filetype("c", "foo->bar", 5, 9));
        assert!(!triggers.matches_for_filetype("c", "foo::bar", 5, 9));
    }
}
