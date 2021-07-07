use super::ycmd_types::*;

#[derive(serde::Deserialize)]
pub struct Options {
    pub hmac_secret: String,
}

pub struct ServerState {}

impl ServerState {
    pub fn new(opt: Options) -> Self {
        Self {}
    }
    pub fn is_ready(&self) -> bool {
        true
    }
    pub fn is_healthy(&self) -> bool {
        true
    }

    pub fn completions(&self, request: SimpleRequest) -> CompletionResponse {
        unimplemented!()
    }
}
