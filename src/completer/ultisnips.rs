use crate::{
    core::query::filter_and_sort_generic_candidates,
    ycmd_types::{Candidate, SimpleRequest},
};

use super::{Completer, CompleterInner, CompletionConfig};

pub struct UltisnipsCompleter {
    config: CompletionConfig,
    candidates: Vec<Candidate>,
}

impl UltisnipsCompleter {
    pub fn new(config: CompletionConfig) -> Self {
        Self {
            config,
            candidates: vec![],
        }
    }
}

impl CompleterInner for UltisnipsCompleter {
    fn get_settings(&self) -> &CompletionConfig {
        &self.config
    }

    fn get_settings_mut(&mut self) -> &mut CompletionConfig {
        &mut self.config
    }
}

impl Completer for UltisnipsCompleter {
    fn on_event(&mut self, event: &crate::ycmd_types::EventNotification) {
        if let crate::ycmd_types::Event::BufferVisit = event.event_name {
            match &event.ultisnips_snippets {
                Some(s) => {
                    self.candidates = s
                        .iter()
                        .map(|s| Candidate {
                            insertion_text: s.trigger.clone(),
                            menu_text: Some(String::from("<snip> ") + &s.description),
                            extra_menu_info: None,
                            detailed_info: None,
                            kind: None,
                            extra_data: None,
                        })
                        .collect();
                }
                None => {}
            }
        }
    }

    fn should_use_now(&self, request: &SimpleRequest) -> bool {
        self.query_length_above_min_threshold(request.start_column(), request.column_num)
    }

    fn compute_candidates(&self, request: &crate::ycmd_types::SimpleRequest) -> Vec<Candidate> {
        // Here be cache and some other stuff
        filter_and_sort_generic_candidates(
            self.candidates.clone(),
            request.query(),
            self.get_settings().max_candidates,
            |c| &c.insertion_text,
        )
    }
}
