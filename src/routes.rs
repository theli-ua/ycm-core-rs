use std::convert::Infallible;
use std::sync::Arc;

use futures::future;

use log::error;
use ring::hmac;

use warp::hyper::Method;
use warp::path::FullPath;
use warp::reply::Response;
use warp::{
    hyper::{body::Bytes, StatusCode},
    Filter, Rejection, Reply,
};

use tokio::sync::mpsc;

use super::server::{Options, ServerState};
use super::ycmd_types;
const HMAC_HEADER: &str = "x-ycm-hmac";

fn hmac_filter(
    key: Arc<hmac::Key>,
) -> impl warp::Filter<Extract = (Bytes,), Error = Rejection> + Send + Sync + 'static + Clone {
    warp::header::<String>(HMAC_HEADER)
        .and(warp::body::bytes())
        .and(warp::path::full())
        .and(warp::method())
        .and_then(
            move |hmac_value, body: Bytes, path: FullPath, method: Method| {
                let hmac_secret = key.clone();
                let hmac_value = base64::decode(&hmac_value).unwrap();
                let body_hmac = hmac::sign(&hmac_secret, &body);
                let method_hmac = hmac::sign(&hmac_secret, method.as_str().as_bytes());
                let path_hmac = hmac::sign(&hmac_secret, path.as_str().as_bytes());

                let mut ctx = hmac::Context::with_key(&hmac_secret);
                ctx.update(method_hmac.as_ref());
                ctx.update(path_hmac.as_ref());
                ctx.update(body_hmac.as_ref());
                let expected = ctx.sign();

                if !expected.as_ref().eq(&hmac_value) {
                    error!("Non matching hmac: {:?}, {:?}", hmac_value, body.as_ref());
                    future::err(warp::reject::not_found())
                } else {
                    future::ok(body)
                }
            },
        )
}

fn hmac_filter_json_body<T: Send + serde::de::DeserializeOwned>(
    key: Arc<hmac::Key>,
) -> impl warp::Filter<Extract = (T,), Error = Rejection> + Send + Sync + 'static + Clone {
    hmac_filter(key).and_then(move |body: Bytes| match serde_json::from_slice(&body) {
        Ok(v) => future::ok(v),
        Err(_) => future::err(warp::reject()),
    })
}

fn hmac_filter_discard_body(
    key: Arc<hmac::Key>,
) -> impl warp::Filter<Extract = (), Error = Rejection> + Send + Sync + 'static + Clone {
    hmac_filter(key).map(move |_: Bytes| ()).untuple_one()
}

pub fn get_routes(
    options: Options,
) -> (
    impl warp::Filter<Extract = impl Reply, Error = Infallible> + Send + Sync + 'static + Clone,
    mpsc::Receiver<()>,
) {
    let hmac_secret = Arc::from(hmac::Key::new(
        hmac::HMAC_SHA256,
        &base64::decode(&options.hmac_secret).unwrap()[..],
    ));

    let server_state = Arc::from(ServerState::new(options));
    let state_filter = warp::any().map(move || server_state.clone());

    let ready = warp::filters::method::get()
        .and(warp::path("ready"))
        .and(hmac_filter_discard_body(hmac_secret.clone()))
        .and(state_filter.clone())
        .map(|state: Arc<ServerState>| warp::reply::json(&state.is_ready()));

    let healthy = warp::filters::method::get()
        .and(warp::path("healthy"))
        .and(hmac_filter_discard_body(hmac_secret.clone()))
        .and(state_filter.clone())
        .map(|state: Arc<ServerState>| warp::reply::json(&state.is_healthy()));

    let completions = warp::filters::method::post()
        .and(warp::path("completions"))
        .and(hmac_filter_json_body(hmac_secret.clone()))
        .and(state_filter.clone())
        .map(
            |request: ycmd_types::SimpleRequest, state: Arc<ServerState>| {
                warp::reply::json(&state.completions(request))
            },
        );

    let debug_info = warp::filters::method::post()
        .and(warp::path("debug_info"))
        .and(state_filter.clone())
        .and(hmac_filter_json_body(hmac_secret.clone()))
        .map(
            |state: Arc<ServerState>, request: ycmd_types::SimpleRequest| {
                warp::reply::json(&state.debug_info(request))
            },
        );

    let defined_subcommands = warp::filters::method::post()
        .and(warp::path("debug_info"))
        .and(state_filter.clone())
        .and(hmac_filter_json_body(hmac_secret.clone()))
        .map(
            |state: Arc<ServerState>, request: ycmd_types::SimpleRequest| {
                warp::reply::json(&state.defined_subcommands(request))
            },
        );

    let semantic_completer_available = warp::filters::method::post()
        .and(warp::path("semantic_completion_available"))
        .and(state_filter.clone())
        .and(hmac_filter_json_body(hmac_secret.clone()))
        .map(
            |state: Arc<ServerState>, request: ycmd_types::SimpleRequest| {
                warp::reply::json(&state.semantic_completer_available(request))
            },
        );

    let signature_help_available = warp::filters::method::get()
        .and(state_filter.clone())
        .and(warp::path("signature_help_available"))
        .and(hmac_filter_discard_body(hmac_secret.clone()))
        .and(warp::query::query())
        .map(|state: Arc<ServerState>, request: ycmd_types::Subserver| {
            warp::reply::json(&state.signature_help_available(request))
        });

    let event_notification = warp::filters::method::post()
        .and(warp::path("event_notification"))
        .and(state_filter.clone())
        .and(hmac_filter_json_body(hmac_secret.clone()))
        .map(
            |state: Arc<ServerState>, request: ycmd_types::EventNotification| {
                warp::reply::json(&state.event_notification(request))
            },
        );

    let filter_and_sort = warp::filters::method::post()
        .and(warp::path("filter_and_sort_candidates"))
        .and(state_filter.clone())
        .and(hmac_filter_json_body(hmac_secret.clone()))
        .map(
            |state: Arc<ServerState>, request: ycmd_types::FilterAndSortRequest| {
                let max_candidates = state.options.max_num_candidates;
                let sort_property = request.sort_property.clone();
                let candidates = crate::core::query::filter_and_sort_generic_candidates(
                    request.candidates,
                    &request.query,
                    max_candidates,
                    |c| match c {
                        serde_json::Value::String(s) => s,
                        serde_json::Value::Object(o) => {
                            o.get(&sort_property).unwrap().as_str().unwrap()
                        }
                        _ => unimplemented!(),
                    },
                );
                warp::reply::json(&candidates)
            },
        );

    let receive_messages = warp::filters::method::post()
        .and(warp::path("receive_messages"))
        .and(state_filter)
        .and(hmac_filter_json_body(hmac_secret.clone()))
        .and_then(
            |state: Arc<ServerState>, request: ycmd_types::SimpleRequest| async move {
                Ok::<_, warp::Rejection>(warp::reply::json(&state.get_messages(request).await))
            },
        );

    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

    let shutdown = warp::filters::method::post()
        .and(warp::path("shutdown"))
        .and(hmac_filter_discard_body(hmac_secret.clone()))
        .and_then(move || {
            let shutdown_tx = shutdown_tx.clone();
            async move {
                shutdown_tx.send(()).await.unwrap();
                Ok::<_, warp::Rejection>(warp::reply())
            }
        });

    let ycmd_paths = ready
        .or(healthy)
        .or(receive_messages)
        .or(completions)
        .or(event_notification)
        .or(debug_info)
        .or(defined_subcommands)
        .or(semantic_completer_available)
        .or(signature_help_available)
        .or(filter_and_sort)
        .or(shutdown);

    (
        ycmd_paths
            .recover(rejection_handler)
            .and_then(move |r| {
                let hmac_secret = hmac_secret.clone();
                sign_body(r, hmac_secret)
            })
            .with(warp::log("ycmd")),
        shutdown_rx,
    )
}

/// Sign reply with hmac
async fn sign_body(
    reply: impl Reply,
    hmac_secret: Arc<hmac::Key>,
) -> Result<impl Reply, Infallible> {
    let (parts, body) = reply.into_response().into_parts();
    let (sig, body) = if let Ok(body) = warp::hyper::body::to_bytes(body).await {
        (
            base64::encode(hmac::sign(&hmac_secret, &body).as_ref()),
            warp::hyper::body::Body::from(body),
        )
    } else {
        (
            String::from(""),
            warp::hyper::body::Body::from(Bytes::default()),
        )
    };
    let response = Response::from_parts(parts, body);

    Ok(warp::reply::with_header(response, HMAC_HEADER, sig))
}

#[derive(serde::Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn rejection_handler(r: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if r.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if r
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        code = StatusCode::BAD_REQUEST;
        message = "BAD_REQUEST";
    } else if r.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.to_string(),
    });

    Ok(warp::reply::with_status(json, code))
}

