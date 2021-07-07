use std::path::PathBuf;

use structopt::StructOpt;
use ycm_core::routes;

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

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(opt.log.to_string()),
    )
    .init();
    let options: ycm_core::server::Options =
        serde_json::from_reader(std::fs::File::open(opt.options_file.clone()).unwrap()).unwrap();
    std::fs::remove_file(opt.options_file).unwrap();

    let addr: std::net::SocketAddr = format!("{}:{}", opt.host, opt.port).parse().unwrap();

    let routes = routes::get_routes(options);
    warp::serve(routes).run(addr).await;
}

