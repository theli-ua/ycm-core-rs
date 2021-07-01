use bytes::Bytes;
use ring::hmac;
use std::path::PathBuf;
use structopt::StructOpt;
use warp::Filter;

const HMAC_HEADER: &'static str = "x-ycm-hmac";

#[derive(Debug, StructOpt)]
#[structopt(name = "ycmd", about = "YCMD-rs")]
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
}

#[derive(serde::Deserialize)]
struct OptionsFile {
    hmac_secret: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let options: OptionsFile =
        serde_json::from_reader(std::fs::File::open(opt.options_file.clone()).unwrap()).unwrap();
    std::fs::remove_file(opt.options_file).unwrap();
    let hmac_secret = hmac::Key::new(
        hmac::HMAC_SHA256,
        &base64::decode(&options.hmac_secret).unwrap()[..],
    );

    let hmac_filter = warp::header::<String>(HMAC_HEADER)
        .and(warp::body::bytes())
        .and_then(move |hmac_value, body: bytes::Bytes| {
            let hmac_secret = hmac_secret.clone();
            async move {
                let hmac_value = base64::decode(&hmac_value).unwrap();
                if hmac::verify(&hmac_secret, body.as_ref(), hmac_value.as_ref()).is_err() {
                    Err(warp::reject::not_found())
                } else {
                    Ok(())
                }
            }
        });

    let ready = hmac_filter
        .and(warp::filters::method::get())
        .and(warp::path("ready"))
        .map(|_| warp::reply::json(&true));

    warp::serve(ready).run(([127, 0, 0, 1], 3030)).await;
}
