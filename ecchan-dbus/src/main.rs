//mod objects;
mod sock;

use std::{error::Error, sync::Arc};

use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
use ecchan_ipc::{call::Call, ret::RetVal};
use env_logger::Env;
use log::LevelFilter;
use sayuri::sync::Mutex;

use crate::sock::Client;

const DBUS_NAME: &str = "com.cherry.ecchan";

fn main() -> Result<(), Box<dyn Error>> {
    setup();

    // let c = Connection::new_session()?;
    // c.request_name(DBUS_NAME, false, true, false)?;
    // let mut cr = Crossroads::new();

    let mut client = Client::new()?;
    let result = client.call(Call::EcDumpPretty).unwrap();
    match result {
        RetVal::Str(s) => println!("{s}"),
        _ => unreachable!(),
    }

    //let ec = Arc::new(Mutex::new(Ec::new()?));

    // let fw = cr.register(DBUS_NAME, objects::fw::build);

    // cr.insert("/fw", &[fw], EcSession { ec: ec.clone() });

    // cr.serve(&c)?;
    // unreachable!();

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
