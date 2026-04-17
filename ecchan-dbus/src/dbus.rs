use std::error::Error;

use dbus_crossroads::Crossroads;

use crate::{DBUS_NAME, data::Data, objects};

pub fn setup_dbus(cr: &mut Crossroads) -> Result<(), Box<dyn Error>> {
    let client = Data::new()?;

    let utils = cr.register(DBUS_NAME, objects::utils::build);
    let fw = cr.register(DBUS_NAME, objects::fw::build);

    cr.insert("/utils", &[utils], client.clone());
    cr.insert("/fw", &[fw], client);

    Ok(())
}
