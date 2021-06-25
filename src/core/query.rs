use std::cmp::Ordering;

use unicode_segmentation::UnicodeSegmentation;

use partial_sort::PartialSort;

use super::{candidate::Candidate, character::Character};

#[derive(PartialEq, Debug)]
pub struct QueryResult<'a, 'b> {
    pub is_subsequence: bool,
    pub query_is_prefix: bool,
    pub first_char_is_same: bool,
    pub char_match_index_sum: usize,
    pub num_wb_matches: usize,
    pub candidate: &'a Candidate<'a>,
    pub query: &'b Word<'b>,
}

#[derive(PartialEq, Debug)]
pub struct Word<'a> {
    pub characters: Vec<Character>,
    pub text: &'a str,
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
