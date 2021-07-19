use std::{collections::HashMap, time::Duration};

use std::sync::Mutex;

use crate::completer::{
    ultisnips::UltisnipsCompleter, Completer, CompletionConfig, GenericCompleters,
};

use super::ycmd_types::*;

#[derive(serde::Deserialize)]
pub struct Options {
    pub hmac_secret: String,
    pub max_num_candidates: usize,
    pub min_num_of_chars_for_completion: usize,
    pub max_num_candidates_to_detail: isize,
    pub max_diagnostics_to_display: usize,
}

pub struct ServerState {
    generic_completers: Mutex<GenericCompleters>,
}

impl ServerState {
    pub fn new(opt: Options) -> Self {
        let config = CompletionConfig {
            min_num_chars: opt.min_num_of_chars_for_completion,
            max_diagnostics_to_display: opt.max_num_candidates,
            completion_triggers: HashMap::default(),
            signature_triggers: HashMap::default(),
            max_candidates: opt.max_num_candidates,
            max_candidates_to_detail: opt.max_num_candidates_to_detail,
        };
        Self {
            generic_completers: Mutex::new(GenericCompleters {
                completers: vec![Box::new(UltisnipsCompleter::new(config.clone()))],
                config,
            }),
        }
    }

    pub fn is_ready(&self) -> bool {
        true
    }

    pub fn is_healthy(&self) -> bool {
        true
    }

    pub fn completions(&self, request: SimpleRequest) -> CompletionResponse {
        let candidates = self
            .generic_completers
            .lock()
            .unwrap()
            .compute_candidates(&request);
        CompletionResponse {
            completions: candidates,
            completion_start_column: request.start_column(),
            errors: vec![],
        }
    }

    pub fn debug_info(&self, _request: SimpleRequest) -> DebugInfo {
        DebugInfo {
            python: PythonInfo {
                executable: "/dev/null".into(),
                version: "0".into(),
            },
            clang: ClangInfo {
                has_support: false,
                version: None,
            },
            extra_conf: ExtraInfo {
                path: "/dev/null".into(),
                is_loaded: false,
            },
            completer: DebugInfoResponse {
                name: "Rust YCMD".into(),
                servers: vec![],
                items: vec![],
            },
        }
    }

    pub fn defined_subcommands(&self, _request: SimpleRequest) -> Vec<String> {
        unimplemented!()
    }

    pub fn semantic_completer_available(&self, _request: SimpleRequest) -> bool {
        false
    }

    pub fn signature_help_available(&self, _request: Subserver) -> Available {
        Available::NO
    }

    pub fn event_notification(&self, request: EventNotification) -> Vec<DiagnosticData> {
        self.generic_completers.lock().unwrap().on_event(&request);
        Vec::new()
    }

    pub async fn get_messages(&self, _request: SimpleRequest) -> MessagePollResponse {
        tokio::time::sleep(Duration::from_secs(30)).await;
        MessagePollResponse::MessagePollResponse(true)
    }
}
