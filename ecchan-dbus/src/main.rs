mod ec;
mod objects;

use std::{error::Error, sync::Arc};

use ::ec::Ec;
use dbus::blocking::Connection;
use dbus_crossroads::Crossroads;
use sayuri::sync::Mutex;

use ec::EcSession;

const DBUS_NAME: &str = "com.cherry.ecchan";

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "unix:path=/run/user/1000/bus");
    }

    let c = Connection::new_session()?;
    c.request_name(DBUS_NAME, false, true, false)?;
    let mut cr = Crossroads::new();

    // let ec = Arc::new(Mutex::new(Ec::new()?));

    // let fw = cr.register(DBUS_NAME, objects::fw::build);

    // cr.insert("/fw", &[fw], EcSession { ec: ec.clone() });

    cr.serve(&c)?;
    unreachable!();
}
