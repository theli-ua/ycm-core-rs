use std::collections::HashMap;

use regex::RegexSet;

pub mod trigger;
use trigger::PatternMatcher;

pub struct CompletionConfig {
    min_num_chars: usize,
    max_diagnostics_to_display: usize,
    completion_triggers: HashMap<String, RegexSet>,
    signature_triggers: HashMap<String, RegexSet>,
    max_candidates: usize,
    max_candidates_to_detail: usize,
}

pub trait CompleterInner {
    fn get_settings(&self) -> &CompletionConfig;
    fn get_settings_mut(&mut self) -> &mut CompletionConfig;
}

pub trait Completer: CompleterInner {
    fn supported_filetypes(&self) -> &[String];
    fn should_use_now(
        &self,
        current_line: &str,
        start_codepoint: usize,
        column_codepoint: usize,
        filetypes: &[String],
    ) -> bool {
        let filetype = filetypes
            .iter()
            .find(|f| self.supported_filetypes().contains(f))
            .or(Some(&filetypes[0]))
            .unwrap();
        // Here be cache?
        self.should_use_now_inner(current_line, start_codepoint, column_codepoint, filetype)
    }

    fn should_use_now_inner(
        &self,
        current_line: &str,
        start_codepoint: usize,
        column_codepoint: usize,
        filetype: &str,
    ) -> bool {
        self.get_settings()
            .completion_triggers
            .matches_for_filetype(filetype, current_line, start_codepoint, column_codepoint)
    }
}
