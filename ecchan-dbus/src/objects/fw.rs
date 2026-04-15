use dbus_crossroads::IfaceBuilder;

use crate::{EcSession, ec::DbusEcError};

pub fn build(b: &mut IfaceBuilder<EcSession>) {
    b.method("info", (), ("version", "date", "time"), |_, session, ()| {
        let ec = session.ec.lock();

        let version = ec.fw_version().map_err(Into::<DbusEcError>::into)?;
        let date = ec.fw_date().map_err(Into::<DbusEcError>::into)?;
        let time = ec.fw_time().map_err(Into::<DbusEcError>::into)?;

        Ok((version, date, time))
    });
}
