use dbus_crossroads::IfaceBuilder;
use ecchan_ipc::method::Method;

use crate::{data::Data, err::ToMethodErr};

pub fn build(b: &mut IfaceBuilder<Data>) {
    b.method("ping", (), ("pong",), |_, data, ()| {
        let mut client = data.get();

        client
            .call(Method::Ping)
            .to_method_err()?
            .pong()
            .to_method_err()?;

        Ok(("pong",))
    });
}
