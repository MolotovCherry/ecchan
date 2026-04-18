use dbus_crossroads::IfaceBuilder;
use ecchan_ipc::method::Method;

use crate::{args::DbusArg, data::Data, err::ToMethodErr};

pub fn build(b: &mut IfaceBuilder<Data>) {
    b.method("modes", (), ("modes",), |_, data, ()| {
        let mut client = data.get();

        let modes = client
            .call(Method::ShiftModes)
            .to_method_err()?
            .shift_modes()
            .to_method_err()?
            .into_iter()
            .map(DbusArg)
            .collect::<Vec<_>>();

        Ok((modes,))
    });

    b.method("mode", (), ("mode",), |_, data, ()| {
        let mut client = data.get();

        let mode = client
            .call(Method::ShiftMode)
            .to_method_err()?
            .shift_mode()
            .map(DbusArg)
            .to_method_err()?;

        Ok((mode,))
    });
}
