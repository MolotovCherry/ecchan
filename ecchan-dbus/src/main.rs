mod err;
mod objects;
mod sock;

use std::error::Error;

use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
use env_logger::Env;
use log::LevelFilter;

use crate::sock::Client;

const DBUS_NAME: &str = "com.cherry.ecchan";

struct Data {
    client: Client,
}

fn main() -> Result<(), Box<dyn Error>> {
    setup();

    let c = Connection::new_session()?;
    c.request_name(DBUS_NAME, false, true, false)?;
    let mut cr = Crossroads::new();

    let client = Client::new()?;

    let fw = cr.register(DBUS_NAME, objects::fw::build);

    cr.insert("/fw", &[fw], Data { client });

    cr.serve(&c)?;
    unreachable!();
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
