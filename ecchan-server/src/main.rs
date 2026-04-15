mod handle_client;

use std::{
    error::Error,
    fs,
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::Path,
};

use ec::Ec;
use env_logger::Env;
use log::LevelFilter;

use handle_client::handle_client;

fn main() -> Result<(), Box<dyn Error>> {
    setup();

    let mut ec = Ec::new()?;

    let sock_path = Path::new(ecchan_ipc::SOCK);
    _ = fs::remove_file(sock_path);

    let sock = UnixListener::bind(sock_path)?;
    let mut perms = fs::metadata(sock_path)?.permissions();
    perms.set_mode(0o666);
    fs::set_permissions(sock_path, perms)?;

    for stream in sock.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                log::error!("incoming client: {e}");
                continue;
            }
        };

        if let Err(e) = handle_client(&mut stream, &mut ec) {
            log::error!("Client error: {e}");
        }
    }

    Ok(())
}

fn setup() {
    let env = Env::new().filter("ECCHAN_LOG").write_style("ECCHAN_STYLE");

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(if cfg!(debug_assertions) {
            LevelFilter::Info
        } else {
            LevelFilter::Debug
        })
        .parse_env(env)
        .init();
}
