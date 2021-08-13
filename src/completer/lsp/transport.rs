use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, error};

use bytes::{Bytes, BytesMut};
use sharded_slab::Slab;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot};

use jsonrpc_core::types as jrpc_types;

pub struct LspTransport<W: AsyncWrite> {
    stream: W,
    response_channels: Arc<Slab<oneshot::Sender<jrpc_types::Output>>>,
    server_requests: mpsc::Receiver<jrpc_types::Call>,
}

impl<W: AsyncWrite + Unpin + Send> LspTransport<W> {
    pub fn new<R: AsyncRead + Unpin + Send + 'static>(mut stream_in: R, stream_out: W) -> Self {
        // Notifications channel
        let (sender, receiver) = mpsc::channel(1024);

        let response_channels = Arc::default();

        let result = Self {
            server_requests: receiver,
            response_channels,
            stream: stream_out,
        };

        let response_channels = result.response_channels.clone();

        tokio::spawn(async move {
            let mut buf = BytesMut::with_capacity(16535);
            #[allow(clippy::mutable_key_type)]
            let mut headers: HashMap<Bytes, Bytes> = HashMap::default();
            let content_len_key = Bytes::from("Content-Length".as_bytes());
            loop {
                /* each message */
                let mut last_checked_index = 0;
                loop {
                    /* each header */
                    let newline_offset = buf[last_checked_index..].iter().position(|b| *b == b'\n');

                    if let Some(n) = newline_offset {
                        let newline_index = last_checked_index + n;
                        last_checked_index = 0;
                        let mut value = buf.split_to(newline_index + 1).split_to(newline_index - 1);

                        if value.is_empty() {
                            // This is `/r/n` line, end of headers
                            break;
                        }
                        let sep_index = value.iter().position(|b| *b == b':').unwrap();
                        let name = value.split_to(sep_index + 1).split_to(sep_index);

                        headers.insert(name.freeze(), value.freeze());
                    } else {
                        last_checked_index = buf.len();
                    }

                    if last_checked_index >= buf.len()
                        && stream_in.read_buf(&mut buf).await.unwrap() == 0
                    {
                        return;
                    }
                }
                let content_len: usize =
                    std::str::from_utf8(headers.get(&content_len_key).unwrap())
                        .unwrap()
                        .trim()
                        .parse()
                        .unwrap();

                if buf.capacity() < content_len {
                    buf.reserve(content_len - buf.capacity());
                }

                while buf.len() < content_len {
                    stream_in.read_buf(&mut buf).await.unwrap();
                }

                headers.clear();
                let content = buf.split_to(content_len);
                let output: serde_json::Result<jrpc_types::Output> =
                    serde_json::from_slice(&content[..]);
                match output {
                    Ok(output) => match output.id() {
                        jsonrpc_core::Id::Num(n) => {
                            //response
                            match response_channels.take(*n as usize) {
                                Some(c) => {
                                    c.send(output).unwrap();
                                }
                                None => {
                                    error!(
                                    "Got response from lsp with unknown id: '{}', response: {:?}",
                                    n, output
                                );
                                }
                            }
                        }
                        _ => {
                            error!(
                                "Got response from lsp with unsupported id, response: {:?}",
                                output
                            );
                        }
                    },

                    Err(_) => {
                        let call: serde_json::Result<jrpc_types::Call> =
                            serde_json::from_slice(&content[..]);
                        match call {
                            Ok(call) => {
                                debug!("Sending call from server from bg task: {:?}", call);
                                sender.send(call).await.unwrap()
                            }
                            Err(_) => {
                                error!(
                                    "Failed to decode message from server: {:?}",
                                    std::str::from_utf8(&content[..])
                                );
                            }
                        }
                    }
                };
            }
        });

        result
    }

    async fn write_request(&mut self, request: &jsonrpc_core::types::Call) {
        let bytes = serde_json::to_vec(request).unwrap();
        let headers = format!("Content-Length: {}\r\n\r\n", bytes.len());
        self.stream.write_all(headers.as_bytes()).await.unwrap();
        self.stream.write_all(&bytes).await.unwrap();
    }

    /// Read next notification
    pub async fn read_requests_from_server(&mut self) -> Option<jrpc_types::Call> {
        self.server_requests.recv().await
    }

    /// Send request returning awaitable result
    pub async fn call(&mut self, method: String, params: jrpc_types::Params) -> jrpc_types::Output {
        let (sender, receiver) = oneshot::channel();
        let id = self.response_channels.insert(sender).unwrap();

        let request = jrpc_types::Call::MethodCall(jrpc_types::MethodCall {
            jsonrpc: None,
            method,
            params,
            id: jrpc_types::Id::Num(id as u64),
        });

        self.write_request(&request).await;
        receiver.await.unwrap()
    }

    /// Notify server
    pub async fn notify(&mut self, method: String, params: jrpc_types::Params) {
        let request = jrpc_types::Call::Notification(jrpc_types::Notification {
            jsonrpc: None,
            method,
            params,
        });

        self.write_request(&request).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_notifications() {
        env_logger::init();
        let (client, mut server) = tokio::io::duplex(32768);
        let (client_r, client_w) = tokio::io::split(client);
        let mut lsp = LspTransport::new(client_r, client_w);

        let notification = jrpc_types::Notification {
            jsonrpc: None,
            method: "method".to_string(),
            params: jrpc_types::Params::None,
        };

        // Server notifies client
        let notification_bytes = serde_json::to_vec(&notification).unwrap();
        let headers_str = dbg!(format!(
            "Content-Length: {}\r\n\r\n",
            notification_bytes.len()
        ));

        server.write_all(headers_str.as_bytes()).await.unwrap();
        server.write_all(&notification_bytes[..]).await.unwrap();

        assert_eq!(
            jrpc_types::Call::Notification(notification),
            lsp.read_requests_from_server().await.unwrap()
        );
    }
}
