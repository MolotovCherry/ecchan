mod args;
mod data;
mod dbus;
mod err;
mod objects;
mod setup;
mod sock;

use std::error::Error;

use ::dbus::blocking::Connection;
use dbus_crossroads::Crossroads;

use sayuri::convert::Never;
use setup::setup;

use crate::dbus::setup_dbus;

const DBUS_NAME: &str = "com.cherry.ecchan";

fn main() -> Result<Never, Box<dyn Error>> {
    setup();

    let c = Connection::new_session()?;
    c.request_name(DBUS_NAME, false, true, false)?;
    let mut cr = Crossroads::new();

    setup_dbus(&mut cr)?;

    #[allow(clippy::empty_loop)]
    cr.serve(&c).map(|_| loop {})?
}
