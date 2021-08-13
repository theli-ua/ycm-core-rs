use std::collections::HashMap;

use regex::RegexSet;

pub mod filename;
pub mod lsp;
pub mod trigger;
pub mod ultisnips;

use crate::core::query::filter_and_sort_generic_candidates;

use filename::FilenameCompleter;

use super::ycmd_types::{Candidate, EventNotification, SimpleRequest};
use trigger::PatternMatcher;

#[derive(Clone)]
pub struct CompletionConfig {
    pub min_num_chars: usize,
    pub max_diagnostics_to_display: usize,
    pub completion_triggers: HashMap<String, RegexSet>,
    pub signature_triggers: HashMap<String, RegexSet>,
    pub max_candidates: usize,
    pub max_candidates_to_detail: isize,
}

// This is something to store state/settings for default Completer impl
pub trait CompleterInner {
    fn get_settings(&self) -> &CompletionConfig;
    fn get_settings_mut(&mut self) -> &mut CompletionConfig;
}

pub trait Completer: CompleterInner {
    fn supported_filetypes(&self) -> &[String] {
        &[]
    }

    fn should_use_now(&self, request: &SimpleRequest) -> bool {
        let filetypes = request.filetypes();
        if filetypes.is_empty() {
            false
        } else {
            let filetype = request
                .filetypes()
                .iter()
                .find(|f| self.supported_filetypes().contains(f))
                .or(Some(&filetypes[0]))
                .unwrap();
            // Here be cache?
            self.should_use_now_inner(filetype, request)
        }
    }

    fn should_use_now_inner(&self, filetype: &str, request: &SimpleRequest) -> bool {
        self.get_settings()
            .completion_triggers
            .matches_for_filetype(
                filetype,
                request.line_value(),
                request.start_column(),
                request.column_num,
            )
    }

    fn on_event(&mut self, _event: &EventNotification) {}

    fn compute_candidates(&self, request: &mut SimpleRequest) -> Vec<Candidate> {
        // Here be cache and some other stuff
        let candidates = self.compute_candidates_inner(request);
        filter_and_sort_generic_candidates(
            candidates,
            request.query(),
            self.get_settings().max_candidates,
            |c| &c.insertion_text,
        )
    }

    fn compute_candidates_inner(&self, _request: &SimpleRequest) -> Vec<Candidate> {
        vec![]
    }

    fn query_length_above_min_threshold(
        &self,
        start_codepoint: usize,
        column_codepoint: usize,
    ) -> bool {
        column_codepoint - start_codepoint >= self.get_settings().min_num_chars
    }
}

pub struct GenericCompleters {
    pub completers: Vec<Box<dyn Completer + Send>>,
    pub fname_completer: FilenameCompleter,
    pub config: CompletionConfig,
}

impl CompleterInner for GenericCompleters {
    fn get_settings(&self) -> &CompletionConfig {
        &self.config
    }

    fn get_settings_mut(&mut self) -> &mut CompletionConfig {
        &mut self.config
    }
}

impl Completer for GenericCompleters {
    fn compute_candidates(&self, request: &mut SimpleRequest) -> Vec<Candidate> {
        let candidates = self.fname_completer.compute_candidates(request);
        if !candidates.is_empty() {
            candidates
        } else {
            self.completers
                .iter()
                .map(|c| c.compute_candidates(request))
                .flatten()
                .collect()
        }
    }

    fn on_event(&mut self, event: &EventNotification) {
        self.completers.iter_mut().for_each(|c| c.on_event(event))
    }
}

