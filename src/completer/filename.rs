use std::collections::HashMap;

use crate::{core::query::filter_and_sort_generic_candidates, ycmd_types::Candidate};

use super::{Completer, CompleterInner, CompletionConfig};

pub struct FilenameCompleter {
    config: CompletionConfig,
    blacklist: HashMap<String, bool>,
}

impl FilenameCompleter {
    pub fn new(config: CompletionConfig, blacklist: HashMap<String, bool>) -> Self {
        Self { config, blacklist }
    }
}

impl CompleterInner for FilenameCompleter {
    fn get_settings(&self) -> &CompletionConfig {
        &self.config
    }

    fn get_settings_mut(&mut self) -> &mut CompletionConfig {
        &mut self.config
    }
}

impl Completer for FilenameCompleter {
    fn should_use_now(
        &self,
        current_line: &str,
        start_codepoint: usize,
        column_codepoint: usize,
        filetypes: &[String],
    ) -> bool {
        !*self
            .blacklist
            .get(filetypes.get(0).unwrap_or(&String::from("")))
            .unwrap_or(&false)
    }
}
