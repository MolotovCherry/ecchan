mod handle_client;
mod signal;

use std::{error::Error, fs, os::unix::fs::PermissionsExt, path::PathBuf};

use ec::Ec;
use env_logger::Env;
use log::LevelFilter;

use handle_client::handle_client;
use tokio::{net::UnixSocket, select};

use crate::signal::shutdown_handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup();

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

        if let Err(e) = handle_client(&mut stream, &mut ec, &sh1).await {
            log::error!("Client error: {e}");
        }

        log::debug!("client disconnected");
    }

    Ok(())
}

fn setup() {
    let env = Env::new().filter("ECCHAN_LOG").write_style("ECCHAN_STYLE");

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .parse_env(env)
        .init();
}
