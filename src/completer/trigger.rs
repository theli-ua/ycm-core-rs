use std::collections::{HashMap, HashSet};

use regex::{escape, RegexSet};

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_triggers() {
        let input = vec![
            ("c".into(), vec![".".into(), "->".into()]),
            (
                "objc,objcpp".into(),
                vec!["->".into(), ".".into(), r"re!\[[_a-zA-Z]+\w*\s".into()],
            ),
            ("objc".into(), vec!["foo".into()]),
        ]
        .into_iter()
        .collect::<HashMap<String, Vec<String>>>();

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
}
