use std::collections::HashMap;

use regex::RegexSet;

pub mod trigger;

pub struct CompletionConfig {
    min_num_chars: usize,
    max_diagnostics_to_display: usize,
    completion_triggers: HashMap<String, RegexSet>,
    signature_triggers: HashMap<String, RegexSet>,
}

pub trait CompleterInner {
    fn get_settings(&self) -> &CompletionConfig;
    fn get_settings_mut(&mut self) -> &mut CompletionConfig;
}

pub trait Completer: CompleterInner {
    fn foo(&self) {
        let _settings = self.get_settings();
    }
}
