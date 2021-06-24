use std::cmp::Ordering;

use unicode_linebreak::{break_property, BreakClass};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use partial_sort::PartialSort;

#[derive(Debug, Clone, Eq)]
pub struct Character {
    normal: Vec<char>,
    base: Vec<char>,
    folded_case: Vec<char>,
    swapped_case: Vec<char>,
    is_base: bool,
    is_uppercase: bool,
    is_punctuation: bool,
    is_letter: bool,
}

impl Character {
    pub fn new(character: &str) -> Self {
        let mut is_base = false;
        let mut normal = Vec::default();
        let mut folded_case = Vec::default();
        let mut swapped_case = Vec::default();
        let mut base = Vec::default();
        let mut is_uppercase = false;
        let mut is_punctuation = false;
        let mut is_letter = false;
        for c in character.nfd() {
            normal.push(c);
            match break_property(c as u32) {
                BreakClass::Before
                | BreakClass::After
                | BreakClass::BeforeAndAfter
                | BreakClass::Space => {
                    is_base = false;
                }
                _ => {
                    base.append(&mut c.to_lowercase().collect::<Vec<_>>());
                }
            }
            is_uppercase |= c.is_uppercase();
            is_punctuation |= c.is_ascii_punctuation() | c.is_whitespace();
            is_letter |= c.is_alphabetic();
            folded_case.append(&mut c.to_lowercase().collect::<Vec<_>>());
            swapped_case.append(
                &mut (if c.is_lowercase() {
                    c.to_uppercase().collect::<Vec<_>>()
                } else {
                    c.to_lowercase().collect::<Vec<_>>()
                }),
            );
        }

        Self {
            is_base,
            normal,
            base,
            folded_case,
            swapped_case,
            is_uppercase,
            is_punctuation,
            is_letter,
        }
    }
    /// Smart base matching on top of smart case matching, e.g.:
    ///  - e matches e, é, E, É;
    ///  - E matches E, É but not e, é;
    ///  - é matches é, É but not e, E;
    pub fn smartcaseeq(&self, other: &Self) -> bool {
        (self.is_base && self.base.eq(&other.base) && (!self.is_uppercase || other.is_uppercase))
            || (!self.is_uppercase && self.folded_case.eq(&other.folded_case))
            || self.normal == other.normal
    }
}

impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        //self.smartcaseeq(other)
        self.base.eq(&other.base)
    }
}

#[derive(Debug, PartialEq)]
pub struct Candidate<'a> {
    characters: Vec<Character>,
    word_boundary_chars: Vec<Character>,
    text_is_lowercase: bool,
    case_swapped: Vec<Vec<char>>,
    pub text: &'a str,
}

impl<'a> Candidate<'a> {
    pub fn new(s: &'a str) -> Self {
        let characters: Vec<Character> = s.graphemes(true).map(Character::new).collect();
        let mut word_boundary_chars = characters
            .windows(2)
            .filter_map(|chars| {
                let prev = &chars[0];
                let current = &chars[1];
                if (prev.is_punctuation && !current.is_punctuation)
                    | (!prev.is_uppercase && current.is_uppercase)
                {
                    Some(current.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if !characters.is_empty() && !characters[0].is_punctuation {
            word_boundary_chars.insert(0, characters[0].clone());
        }
        let text_is_lowercase = characters.iter().all(|c| !c.is_uppercase);
        let case_swapped = characters.iter().map(|c| c.swapped_case.clone()).collect();

        Self {
            characters,
            word_boundary_chars,
            text_is_lowercase,
            case_swapped,
            text: s,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }

    pub fn matches_query<'c, 'b>(&'c self, q: &'b Word<'b>) -> QueryResult<'c, 'b> {
        let mut q_iter = q.characters.iter();
        let mut last_q = q_iter.next();
        let mut match_index_sum = 0;
        let mut is_prefix = true;
        for (i, g) in self.characters.iter().enumerate() {
            match last_q {
                Some(c) => {
                    if c.smartcaseeq(g) {
                        last_q = q_iter.next();
                        match_index_sum += i;
                    } else {
                        is_prefix = false;
                    }
                }
                None => return QueryResult::new(true, is_prefix, match_index_sum, &self, q),
            }
        }
        if last_q.is_none() {
            return QueryResult::new(true, is_prefix, match_index_sum, &self, q);
        }
        QueryResult::default()
    }
}

#[derive(PartialEq, Debug)]
pub struct QueryResult<'a, 'b> {
    is_subsequence: bool,
    query_is_prefix: bool,
    first_char_is_same: bool,
    char_match_index_sum: usize,
    num_wb_matches: usize,
    pub candidate: &'a Candidate<'a>,
    query: &'b Word<'b>,
}

#[derive(PartialEq, Debug)]
pub struct Word<'a> {
    characters: Vec<Character>,
    text: &'a str,
}

impl<'a> Word<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            characters: text.graphemes(true).map(Character::new).collect(),
            text,
        }
    }
}

lazy_static::lazy_static! {
    static ref EMPTY_CANDIDATE: Candidate<'static> = Candidate::new("");
    static ref EMPTY_WORD: Word<'static> = Word::new("");
}

impl Default for QueryResult<'_, '_> {
    fn default() -> Self {
        Self {
            is_subsequence: false,
            query_is_prefix: false,
            first_char_is_same: false,
            char_match_index_sum: 0,
            num_wb_matches: 0,
            candidate: &EMPTY_CANDIDATE,
            query: &EMPTY_WORD,
        }
    }
}

impl<'a, 'b> QueryResult<'a, 'b> {
    pub fn new(
        is_subsequence: bool,
        query_is_prefix: bool,
        char_match_index_sum: usize,
        candidate: &'a Candidate,
        query: &'b Word,
    ) -> Self {
        let (num_wb_matches, first_char_is_same) =
            if candidate.is_empty() | query.characters.is_empty() {
                (0, false)
            } else {
                let first_char_is_same = candidate.characters[0].base == query.characters[0].base;
                let num_wb_matches =
                    lcs::LcsTable::new(&candidate.word_boundary_chars, &query.characters)
                        .longest_common_subsequence()
                        .len();
                (num_wb_matches, first_char_is_same)
            };

        Self {
            is_subsequence,
            query_is_prefix,
            first_char_is_same,
            char_match_index_sum,
            num_wb_matches,
            candidate,
            query,
        }
    }
}

impl PartialOrd for QueryResult<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if !self.query.text.is_empty() {
            match self.first_char_is_same.cmp(&other.first_char_is_same) {
                Ordering::Less => return Some(Ordering::Greater),
                Ordering::Greater => return Some(Ordering::Less),
                Ordering::Equal => {}
            }

            if self.num_wb_matches == self.query.characters.len()
                || other.num_wb_matches == other.query.characters.len()
            {
                match self.num_wb_matches.cmp(&other.num_wb_matches) {
                    Ordering::Less => return Some(Ordering::Greater),
                    Ordering::Greater => return Some(Ordering::Less),
                    Ordering::Equal => {}
                };
                match self
                    .candidate
                    .word_boundary_chars
                    .len()
                    .cmp(&other.candidate.word_boundary_chars.len())
                {
                    o @ (Ordering::Less | Ordering::Greater) => return Some(o),
                    Ordering::Equal => {}
                };
            }

            match self.query_is_prefix.cmp(&other.query_is_prefix) {
                Ordering::Less => return Some(Ordering::Greater),
                Ordering::Greater => return Some(Ordering::Less),
                Ordering::Equal => {}
            }

            match self.num_wb_matches.cmp(&other.num_wb_matches) {
                Ordering::Less => return Some(Ordering::Greater),
                Ordering::Greater => return Some(Ordering::Less),
                Ordering::Equal => {}
            };

            match self
                .candidate
                .word_boundary_chars
                .len()
                .cmp(&other.candidate.word_boundary_chars.len())
            {
                o @ (Ordering::Less | Ordering::Greater) => return Some(o),
                Ordering::Equal => {}
            };

            match self.char_match_index_sum.cmp(&other.char_match_index_sum) {
                o @ (Ordering::Less | Ordering::Greater) => return Some(o),
                Ordering::Equal => {}
            };

            match self
                .candidate
                .characters
                .len()
                .cmp(&other.candidate.characters.len())
            {
                o @ (Ordering::Less | Ordering::Greater) => return Some(o),
                Ordering::Equal => {}
            }

            match self
                .candidate
                .text_is_lowercase
                .cmp(&other.candidate.text_is_lowercase)
            {
                Ordering::Less => return Some(Ordering::Greater),
                Ordering::Greater => return Some(Ordering::Less),
                Ordering::Equal => {}
            };
        }
        Some(
            self.candidate
                .case_swapped
                .cmp(&other.candidate.case_swapped),
        )
    }
}

pub fn filter_and_sort_candidates<'a, 'b>(
    candidates: &'a Vec<Candidate>,
    query: &'b Word,
    max_candidates: usize,
) -> Vec<QueryResult<'a, 'b>> {
    let mut results = candidates
        .iter()
        .map(|c| c.matches_query(query))
        .filter(|r| r.is_subsequence)
        .collect::<Vec<_>>();

    let max_candidates = max_candidates.min(results.len());
    results.partial_sort(max_candidates, |a, b| a.partial_cmp(b).unwrap());
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_query_match() {
        let s = "acb";
        let c = Candidate::new(&s);
        let q = Word::new("ab");
        assert_eq!(
            c.matches_query(&q),
            QueryResult::new(true, false, 2, &c, &q)
        );
        let q = Word::new("ba");
        assert_eq!(
            c.matches_query(&q),
            QueryResult::new(false, false, 0, &EMPTY_CANDIDATE, &Word::new(""))
        );
    }

    #[test]
    fn test_filter_and_sort() {
        let candidates = std::array::IntoIter::new(["acb", "ab", "Ab", "bab", "A , B", "BA"])
            .map(Candidate::new)
            .collect::<Vec<_>>();
        let q = Word::new("ab");

        let results = filter_and_sort_candidates(&candidates, &q, usize::MAX);
        let expected_candidates = vec!["A , B", "ab", "Ab", "acb", "bab"];
        let result_strings = results
            .into_iter()
            .map(|r| r.candidate.text)
            .collect::<Vec<_>>();
        assert_eq!(expected_candidates, result_strings);
    }
}
