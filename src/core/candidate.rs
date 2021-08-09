use unicode_segmentation::UnicodeSegmentation;

use super::{
    character::Character,
    query::{QueryResult, Word},
};

#[derive(Debug, PartialEq)]
pub struct Candidate<'a> {
    pub characters: Vec<Character>,
    pub word_boundary_chars: Vec<Character>,
    pub text_is_lowercase: bool,
    pub case_swapped: Vec<char>,
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
        let case_swapped = characters
            .iter()
            .map(|c| c.swapped_case.clone())
            .flatten()
            .collect();

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
                None => return QueryResult::new(true, is_prefix, match_index_sum, self, q),
            }
        }
        if last_q.is_none() {
            return QueryResult::new(true, is_prefix, match_index_sum, self, q);
        }
        QueryResult::default()
    }
}

