use unicode_linebreak::{break_property, BreakClass};
use unicode_normalization::UnicodeNormalization;

use smallvec::SmallVec;

#[derive(Debug, Clone, Eq)]
pub struct Character {
    pub normal: SmallVec<[char; 2]>,
    pub base: SmallVec<[char; 2]>,
    pub folded_case: SmallVec<[char; 2]>,
    pub swapped_case: SmallVec<[char; 2]>,
    pub is_base: bool,
    pub is_uppercase: bool,
    pub is_punctuation: bool,
    pub is_letter: bool,
}

impl Character {
    pub fn new(character: &str) -> Self {
        let mut is_base = false;
        let mut normal = SmallVec::<[char; 2]>::new();
        let mut folded_case = SmallVec::<[char; 2]>::new();
        let mut swapped_case = SmallVec::<[char; 2]>::new();
        let mut base = SmallVec::<[char; 2]>::new();
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
                    for cc in c.to_lowercase() {
                        base.push(cc);
                    }
                }
            }
            is_uppercase |= c.is_uppercase();
            is_punctuation |= c.is_ascii_punctuation() | c.is_whitespace();
            is_letter |= c.is_alphabetic();
            for cc in c.to_lowercase() {
                folded_case.push(cc);
            }
            if c.is_lowercase() {
                for cc in c.to_uppercase() {
                    swapped_case.push(cc);
                }
            } else {
                for cc in c.to_lowercase() {
                    swapped_case.push(cc);
                }
            }
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

