use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;

use log::error;
use ring::hmac;

use structopt::StructOpt;

use warp::hyper::Method;
use warp::path::FullPath;
use warp::reply::Response;
use warp::{
    hyper::{body::Bytes, StatusCode},
    Filter, Rejection, Reply,
};

mod routes;

const HMAC_HEADER: &'static str = "x-ycm-hmac";

#[derive(Debug, StructOpt)]
#[structopt(name = "ycmd", about = "YCMD-rs", rename_all = "snake-case")]
struct Opt {
    #[structopt(long, parse(from_os_str))]
    options_file: PathBuf,
    #[structopt(long, default_value = "127.0.0.1")]
    host: String,

    #[structopt(long, default_value = "3030")]
    port: u32,

    #[structopt(long, default_value = "error")]
    log: log::Level,

    #[structopt(long)]
    idle_suicide_seconds: Option<usize>,

    #[structopt(long, default_value = "600")]
    check_interval_seconds: usize,

    #[structopt(long)]
    stdout: Option<PathBuf>,

    #[structopt(long)]
    stderr: Option<PathBuf>,

    #[structopt(long)]
    keep_logfiles: bool,

    // positional to capture stuff
    #[structopt(name = "FOO")]
    _foo: String,
}

#[derive(serde::Deserialize)]
struct OptionsFile {
    hmac_secret: String,
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
#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(opt.log.to_string()),
    )
    .init();
    let options: OptionsFile =
        serde_json::from_reader(std::fs::File::open(opt.options_file.clone()).unwrap()).unwrap();
    std::fs::remove_file(opt.options_file).unwrap();
    let hmac_secret = Arc::from(hmac::Key::new(
        hmac::HMAC_SHA256,
        &base64::decode(&options.hmac_secret).unwrap()[..],
    ));

    let hmac_secret_clone = hmac_secret.clone();
    let hmac_filter = warp::header::<String>(HMAC_HEADER)
        .and(warp::body::bytes())
        .and(warp::path::full())
        .and(warp::method())
        .and_then(
            move |hmac_value, body: Bytes, path: FullPath, method: Method| {
                let hmac_secret = hmac_secret_clone.clone();
                async move {
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
                        Err(warp::reject::not_found())
                    } else {
                        Ok(())
                    }
                }
            },
        );

    let ready = hmac_filter
        .and(warp::filters::method::get())
        .and(warp::path("ready"))
        .or(warp::path("healthy"))
        .map(|_| warp::reply::json(&true))
        .recover(rejection_handler)
        .and_then(move |r| {
            let hmac_secret = hmac_secret.clone();
            sign_body(r, hmac_secret)
        })
        .with(warp::log("ycmd"));

    let addr: std::net::SocketAddr = format!("{}:{}", opt.host, opt.port).parse().unwrap();

    warp::serve(ready).run(addr).await;
}

