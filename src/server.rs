use std::time::Duration;

use super::ycmd_types::*;

#[derive(serde::Deserialize)]
pub struct Options {
    pub hmac_secret: String,
}

pub struct ServerState {}

impl ServerState {
    pub fn new(_opt: Options) -> Self {
        Self {}
    }
    pub fn is_ready(&self) -> bool {
        true
    }
    pub fn is_healthy(&self) -> bool {
        true
    }

    pub fn completions(&self, _request: SimpleRequest) -> CompletionResponse {
        unimplemented!()
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

    pub fn event_notification(&self, _request: EventNotification) -> Vec<DiagnosticData> {
        Vec::new()
    }

    pub async fn get_messages(&self, _request: SimpleRequest) -> MessagePollResponse {
        tokio::time::sleep(Duration::from_secs(30)).await;
        MessagePollResponse::MessagePollResponse(true)
    }
}
