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

    pub fn debug_info(&self, _request: SimpleRequest) -> DebugInfoResponse {
        unimplemented!()
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
}
