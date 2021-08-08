use std::path::PathBuf;

use structopt::StructOpt;
use ycm_core::routes;

use filedescriptor::{FileDescriptor, StdioDescriptor};

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
        env_logger::Env::default().default_filter_or(format!("hyper=error,{}", opt.log.to_string())),
    )
    .init();
    let options: ycm_core::server::Options =
        serde_json::from_reader(std::fs::File::open(opt.options_file.clone()).unwrap()).unwrap();
    std::fs::remove_file(opt.options_file).unwrap();

    let _stdio_guard = opt.stdout.clone().map(|path| {
        let file = std::fs::File::create(path).unwrap();
        let fd = FileDescriptor::redirect_stdio(&file, StdioDescriptor::Stdout);
        (file, fd)
    });
    let _sterr_guard = opt.stderr.clone().map(|path| {
        let file = std::fs::File::create(path).unwrap();
        let fd = FileDescriptor::redirect_stdio(&file, StdioDescriptor::Stderr);
        (file, fd)
    });

    let addr: std::net::SocketAddr = format!("{}:{}", opt.host, opt.port).parse().unwrap();

    let (routes, mut shutdown) = routes::get_routes(options);
    warp::serve(routes)
        .bind_with_graceful_shutdown(addr, async move {
            shutdown.recv().await;
        })
        .1
        .await;

    if !opt.keep_logfiles {
        if let Some(path) = opt.stdout {
            std::fs::remove_file(path).unwrap();
        }
        if let Some(path) = opt.stderr {
            std::fs::remove_file(path).unwrap();
        }
    }
}

