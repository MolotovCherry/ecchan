use dbus_crossroads::IfaceBuilder;
use ecchan_ipc::method::Method;

use crate::{data::Data, err::ToMethodErr};

pub fn build(b: &mut IfaceBuilder<Data>) {
    b.method("version", (), ("version",), |_, data, ()| {
        let mut client = data.get();

        let version = client
            .call(Method::FwVersion)
            .to_method_err()?
            .str()
            .to_method_err()?;

        Ok((version,))
    });

    b.method("date", (), ("date",), |_, data, ()| {
        let mut client = data.get();

        let date = client
            .call(Method::FwDate)
            .to_method_err()?
            .str()
            .to_method_err()?;

        Ok((date,))
    });

    b.method("time", (), ("time",), |_, data, ()| {
        let mut client = data.get();

        let time = client
            .call(Method::FwTime)
            .to_method_err()?
            .str()
            .to_method_err()?;

        Ok((time,))
    });
}
