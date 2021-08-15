use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, error};

use bytes::{Bytes, BytesMut};
use sharded_slab::Slab;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot};

use jsonrpc_core::types as jrpc_types;

pub struct LspTransport {
    response_channels: Arc<Slab<oneshot::Sender<jrpc_types::Output>>>,
    server_requests: mpsc::Receiver<jrpc_types::Call>,
    client_requests: mpsc::Sender<jrpc_types::Call>,
}

impl LspTransport {
    pub fn new<R, W>(mut stream_in: R, mut stream_out: W) -> Self
    where
        R: AsyncRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
    {
        // Notifications channel
        let (server_requests_sender, server_requests_receiver) = mpsc::channel(1024);
        let (client_requests_sender, mut client_requests_receiver) = mpsc::channel(1024);

        let response_channels = Arc::default();

        let result = Self {
            server_requests: server_requests_receiver,
            client_requests: client_requests_sender,
            response_channels,
        };

        let response_channels = result.response_channels.clone();

        // Spawn reader
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
                                server_requests_sender.send(call).await.unwrap()
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

        // Spawn writer
        tokio::spawn(async move {
            while let Some(request) = client_requests_receiver.recv().await {
                let bytes = serde_json::to_vec(&request).unwrap();
                let headers = format!("Content-Length: {}\r\n\r\n", bytes.len());
                stream_out.write_all(headers.as_bytes()).await.unwrap();
                stream_out.write_all(&bytes).await.unwrap();
            }
        });

        result
    }

    async fn write_request(&self, request: jsonrpc_core::types::Call) {
        self.client_requests.send(request).await.unwrap()
    }

    /// Read next notification
    pub async fn read_requests_from_server(&mut self) -> Option<jrpc_types::Call> {
        self.server_requests.recv().await
    }

    /// Send request returning awaitable result
    pub async fn call(&self, method: String, params: jrpc_types::Params) -> jrpc_types::Output {
        let (sender, receiver) = oneshot::channel();
        let id = self.response_channels.insert(sender).unwrap();

        let request = jrpc_types::Call::MethodCall(jrpc_types::MethodCall {
            jsonrpc: Some(jrpc_types::Version::V2),
            method,
            params,
            id: jrpc_types::Id::Num(id as u64),
        });

        self.write_request(request).await;
        receiver.await.unwrap()
    }

    /// Notify server
    pub async fn notify(&self, method: String, params: jrpc_types::Params) {
        let request = jrpc_types::Call::Notification(jrpc_types::Notification {
            jsonrpc: Some(jrpc_types::Version::V2),
            method,
            params,
        });

        self.write_request(request).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[tokio::test]
    async fn test_notifications() {
        let (client, mut server) = tokio::io::duplex(4096);
        let (client_r, client_w) = tokio::io::split(client);
        let mut lsp = LspTransport::new(client_r, client_w);

        let notification = jrpc_types::Notification {
            jsonrpc: Some(jrpc_types::Version::V2),
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

        // Client notifies server
        lsp.notify("method".to_string(), jsonrpc_core::Params::None)
            .await;

        let mut expected_buf = Vec::from(headers_str.as_bytes());
        expected_buf.extend_from_slice(&notification_bytes[..]);

        let mut buf = Vec::default();
        buf.resize(expected_buf.len(), 0);
        server.read_exact(&mut buf).await.unwrap();

        assert_eq!(buf, expected_buf);
    }

    #[tokio::test]
    async fn test_request_response() {
        let (client, mut server) = tokio::io::duplex(4096);
        let (client_r, client_w) = tokio::io::split(client);
        let lsp = LspTransport::new(client_r, client_w);

        let server_task = tokio::spawn(async move {
            // We're not gonna cheat here and not do line reading
            let length_re = Regex::new("Content-Length:\\s*([0-9]+)").unwrap();

            let mut buf = BytesMut::with_capacity(4096);

            let content_len: usize = loop {
                server.read_buf(&mut buf).await.unwrap();
                let s = dbg!(std::str::from_utf8(&buf[..]).unwrap());
                if let Some(c) = length_re.captures(s) {
                    break c.get(1).unwrap().as_str().parse().unwrap();
                }
            };
            // now find {
            let start_pos = loop {
                if let Some(p) = buf.iter().position(|b| *b == b'{') {
                    break p;
                }
                server.read_buf(&mut buf).await.unwrap();
            };

            let _ = buf.split_to(start_pos);
            while buf.len() < content_len {
                server.read_buf(&mut buf).await.unwrap();
            }
            let call: jrpc_types::MethodCall = serde_json::from_slice(&buf[..content_len]).unwrap();
            let id = match call.id {
                jrpc_types::Id::Num(n) => n,
                _ => panic!("Unexpected ID"),
            };
            let expected_call = jrpc_types::MethodCall {
                jsonrpc: Some(jrpc_types::Version::V2),
                method: "someMethod/foo".to_string(),
                params: jrpc_types::Params::None,
                id: jrpc_types::Id::Num(id),
            };
            assert_eq!(call, expected_call);

            let response = jrpc_types::Success {
                jsonrpc: Some(jrpc_types::Version::V2),
                id: jrpc_types::Id::Num(id),
                result: jrpc_types::Value::String(String::from("success")),
            };

            let bytes = serde_json::to_vec(&response).unwrap();

            let headers = format!("Content-Length: {}\r\n\r\n", bytes.len());
            server.write_all(headers.as_bytes()).await.unwrap();
            server.write_all(&bytes).await.unwrap();
        });

        let response = lsp
            .call("someMethod/foo".to_string(), jrpc_types::Params::None)
            .await;
        let id = match &response {
            jsonrpc_core::Output::Success(s) => match s.id {
                jrpc_types::Id::Num(n) => n,
                _ => panic!("Unexpected ID"),
            },
            _ => panic!("Expected success"),
        };

        let expected_response = jrpc_types::Output::Success(jrpc_types::Success {
            jsonrpc: Some(jrpc_types::Version::V2),
            id: jrpc_types::Id::Num(id),
            result: jrpc_types::Value::String(String::from("success")),
        });
        assert_eq!(response, expected_response);
        server_task.await.unwrap();
    }
}
