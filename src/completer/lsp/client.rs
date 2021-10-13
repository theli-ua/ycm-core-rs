use std::{ffi::OsStr, process::Stdio};

use lsp_types;
use tokio::process::Child;

/// Object responsible for spawning an LSP server process
/// and its lifetime
pub struct LspClient {
    transport: super::transport::LspTransport,
    child: Child,
}

impl LspClient {
    pub async fn new<P, S, I>(path: P, args: I, port: Option<u32>) -> Result<Self, anyhow::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
        P: AsRef<OsStr>,
    {
        let mut command = tokio::process::Command::new(path);
        command.args(args);
        if port.is_none() {
            command.stdin(Stdio::piped()).stdin(Stdio::piped());
        }
        let mut child = command.spawn()?;

        let transport = match port {
            None => super::transport::LspTransport::new(
                child.stdout.take().unwrap(),
                child.stdin.take().unwrap(),
            ),
            Some(p) => {
                let stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", p)).await?;
                let (r, w) = tokio::io::split(stream);
                super::transport::LspTransport::new(r, w)
            }
        };

        Ok(Self { child, transport })
    }

    pub async fn request<T: lsp_types::request::Request>(
        &self,
        params: T::Params,
    ) -> Result<T::Result, anyhow::Error> {
        let params = match serde_json::to_value(params)? {
            jsonrpc_core::Value::Null => jsonrpc_core::types::Params::None,
            jsonrpc_core::Value::Array(a) => jsonrpc_core::types::Params::Array(a),
            jsonrpc_core::Value::Object(m) => jsonrpc_core::types::Params::Map(m),
            _ => unreachable!(),
        };
        match self.transport.call(T::METHOD.to_string(), params).await {
            jsonrpc_core::Output::Success(r) => Ok(serde_json::from_value(r.result)?),
            jsonrpc_core::Output::Failure(e) => Err(e.error.into()),
        }
    }

    pub async fn notification<T: lsp_types::notification::Notification>(
        &self,
        params: T::Params,
    ) -> Result<(), anyhow::Error> {
        let params = match serde_json::to_value(params)? {
            jsonrpc_core::Value::Null => jsonrpc_core::types::Params::None,
            jsonrpc_core::Value::Array(a) => jsonrpc_core::types::Params::Array(a),
            jsonrpc_core::Value::Object(m) => jsonrpc_core::types::Params::Map(m),
            _ => unreachable!(),
        };
        self.transport.notify(T::METHOD.to_string(), params).await;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        self.request::<lsp_types::request::Shutdown>(()).await?;
        self.child.wait().await?;
        Ok(())
    }
}
