use dbus_crossroads::IfaceBuilder;
use ecchan_ipc::method::Method;

use crate::{Data, err::ToMethodErr};

pub fn build(b: &mut IfaceBuilder<Data>) {
    b.method("version", (), ("version",), |_, data, ()| {
        let client = &mut data.client;

        let version = client.call(Method::FwVersion).to_method_err()?.str();

        Ok((version,))
    });

    b.method("date", (), ("date",), |_, data, ()| {
        let client = &mut data.client;

        let date = client.call(Method::FwDate).to_method_err()?.str();

        Ok((date,))
    });

    b.method("time", (), ("time",), |_, data, ()| {
        let client = &mut data.client;

        let time = client.call(Method::FwTime).to_method_err()?.str();

        Ok((time,))
    });
}
