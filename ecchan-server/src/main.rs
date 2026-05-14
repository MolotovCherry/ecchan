mod handle_client;
mod signal;

use std::{env, error::Error, fs, os::unix::fs::PermissionsExt, path::PathBuf};

use ec::Ec;
use env_logger::Env;

use handle_client::handle_client;
use log::LevelFilter;
use tokio::{net::UnixSocket, select};

use crate::signal::shutdown_handler;

#[derive(Default)]
struct Args {
    skip_config: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup();

    let mut args = Args::default();

    let mut _args = env::args();
    if let Some(a) = _args.nth(1)
        && a == "--skip-ping"
    {
        args.skip_config = true;
    }

    let sh1 = shutdown_handler()?;

    let mut ec = Ec::new()?;

    let sock_path = PathBuf::from(ecchan_ipc::get_socket_path());
    _ = fs::remove_file(&sock_path);

    let sock = UnixSocket::new_stream()?;
    sock.bind(&sock_path)?;

    let mut perms = fs::metadata(&sock_path)?.permissions();
    perms.set_mode(0o666);
    fs::set_permissions(&sock_path, perms)?;

    let listener = sock.listen(1)?;

    log::info!("listening @ {}", sock_path.display());

    loop {
        let client = select! {
            _ = sh1.wait() => break,
            v = listener.accept() => v,
        };

        let (mut stream, _addr) = match client {
            Ok(v) => v,
            Err(e) => {
                log::error!("incoming client: {e}");
                continue;
            }
        };

        if let Err(e) = handle_client(&mut stream, &mut ec, &sh1, &args).await {
            log::error!("Client error: {e}");
        }

        log::debug!("client disconnected");
    }

    Ok(())
}

fn setup() {
    let env = Env::new().filter("ECCHAN_LOG").write_style("ECCHAN_STYLE");

    let mut builder = env_logger::Builder::from_env(env);

    builder
        .format_timestamp(None)
        .filter_level(LevelFilter::Debug)
        .init();
}
