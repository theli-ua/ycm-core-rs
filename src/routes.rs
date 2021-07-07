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

use super::server::{Options, ServerState};
const HMAC_HEADER: &'static str = "x-ycm-hmac";

pub fn get_routes(
    options: Options,
) -> impl warp::Filter<Extract = impl Reply, Error = Infallible> + Send + Sync + 'static + Clone {
    let hmac_secret = Arc::from(hmac::Key::new(
        hmac::HMAC_SHA256,
        &base64::decode(&options.hmac_secret).unwrap()[..],
    ));

    let server_state = Arc::from(ServerState::new(options));
    let state_filter = warp::any().map(move || server_state.clone());

    let ready = warp::filters::method::get()
        .and(warp::path("ready"))
        .and(state_filter.clone())
        .map(|state: Arc<ServerState>| warp::reply::json(&state.is_ready()));

    let healthy = warp::filters::method::get()
        .and(warp::path("healthy"))
        .and(state_filter.clone())
        .map(|state: Arc<ServerState>| warp::reply::json(&state.is_healthy()));

    let key = hmac_secret.clone();
    warp::header::<String>(HMAC_HEADER)
        .and(warp::body::bytes())
        .and(warp::path::full())
        .and(warp::method())
        .and_then(
            move |hmac_value, body: Bytes, path: FullPath, method: Method| {
                let hmac_secret = key.clone();
                let hmac_value = base64::decode(&hmac_value).unwrap();
                let body_hmac = hmac::sign(&hmac_secret, &body);
                let method_hmac = hmac::sign(&hmac_secret, &method.as_str().as_bytes());
                let path_hmac = hmac::sign(&hmac_secret, &path.as_str().as_bytes());

                let mut ctx = hmac::Context::with_key(&hmac_secret);
                ctx.update(method_hmac.as_ref());
                ctx.update(path_hmac.as_ref());
                ctx.update(body_hmac.as_ref());
                let expected = ctx.sign();

                if !expected.as_ref().eq(&hmac_value) {
                    error!("Non matching hmac: {:?}, {:?}", hmac_value, body.as_ref());
                    future::err(warp::reject::not_found())
                } else {
                    future::ok(())
                }
            },
        )
        .untuple_one()
        .and(ready.or(healthy))
        .recover(rejection_handler)
        .and_then(move |r| {
            let hmac_secret = hmac_secret.clone();
            sign_body(r, hmac_secret)
        })
}

async fn sign_body(
    reply: impl Reply,
    hmac_secret: Arc<hmac::Key>,
) -> Result<impl Reply, std::convert::Infallible> {
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
    } else if let Some(_) = r.find::<warp::filters::body::BodyDeserializeError>() {
        code = StatusCode::BAD_REQUEST;
        message = "BAD_REQUEST";
    } else if let Some(_) = r.find::<warp::reject::MethodNotAllowed>() {
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

