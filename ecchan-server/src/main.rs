mod handle_client;

use std::{error::Error, fs, os::fd::AsFd as _, path::Path};

use ec::Ec;
use env_logger::Env;
use log::LevelFilter;

use handle_client::handle_client;
use rustix::{
    event::{PollFd, PollFlags, poll},
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    io::Errno,
    net::{
        AddressFamily, SocketAddrUnix, SocketFlags, SocketType, accept, bind, listen, socket_with,
    },
};

use crate::handle_client::ClientError;

fn main() -> Result<(), Box<dyn Error>> {
    setup();

    let mut ec = Ec::new()?;

    let run_path = Path::new("/run/ecchan.sock");
    _ = fs::remove_file(run_path);

    let addr = SocketAddrUnix::new(run_path)?;

    let sock = socket_with(
        AddressFamily::UNIX,
        SocketType::STREAM,
        SocketFlags::CLOEXEC | SocketFlags::NONBLOCK,
        None,
    )?;

    bind(sock.as_fd(), &addr)?;

    listen(&sock, 1)?;

    // handle ctrl c program exit

    ctrlc::set_handler(|| ()).expect("Error setting Ctrl-C handler");

    let mut events = [PollFd::from_borrowed_fd(sock.as_fd(), PollFlags::IN)];

    loop {
        match poll(&mut events, None) {
            Ok(_) => (),
            Err(e) => match e {
                Errno::INTR => break,
                e => {
                    log::error!("poll: {e}");
                    break;
                }
            },
        }

        let client = match accept(sock.as_fd()) {
            Ok(c) => {
                let flags = fcntl_getfl(c.as_fd())? | OFlags::NONBLOCK;
                fcntl_setfl(c.as_fd(), flags)?;
                c
            }

            Err(e) => match e {
                Errno::WOULDBLOCK => continue,
                // interrupted and should exit
                Errno::INTR => break,
                e => {
                    log::error!("Client failed to connect: {e}");
                    continue;
                }
            },
        };

        match handle_client(client.as_fd(), &mut ec) {
            Ok(_) => (),
            Err(e) => match e {
                ClientError::Exit => break,
                e => log::error!("Client error: {e}"),
            },
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
