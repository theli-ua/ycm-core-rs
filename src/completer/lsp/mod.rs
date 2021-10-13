use std::ffi::OsStr;

use super::{Completer, CompleterInner, CompletionConfig};

pub mod client;
pub mod transport;

pub struct LspCompleter {
    client: client::LspClient,
    config: CompletionConfig,
}

impl CompleterInner for LspCompleter {
    fn get_settings(&self) -> &CompletionConfig {
        &self.config
    }

    fn get_settings_mut(&mut self) -> &mut CompletionConfig {
        &mut self.config
    }
}

impl LspCompleter {
    pub async fn new<P, S, I>(
        path: P,
        args: I,
        port: Option<u32>,
        config: CompletionConfig,
    ) -> Result<Self, anyhow::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
        P: AsRef<OsStr>,
    {
        let client = client::LspClient::new(path, args, port).await?;

        Ok(Self { client, config })
    }
}

impl Completer for LspCompleter {}
