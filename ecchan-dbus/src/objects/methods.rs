use dbus::MethodErr;
use dbus_crossroads::IfaceBuilder;
use ecchan_ipc::{method::Method, ret::RetVal};

use crate::{args::DbusArg, data::Data, err::ToMethodErr};

pub fn build(b: &mut IfaceBuilder<Data>) {
    b.method("list", (), ("list",), |_, data, ()| {
        let mut client = data.get();

        let ret = client.call(Method::MethodList).to_method_err()?;
        let list = match ret {
            RetVal::Methods(methods) => methods,
            r => {
                let s = format!("expected Methods, instead got {r}");
                return Err(MethodErr::failed(&s));
            }
        };

        let list = list
            .into_iter()
            .map(|o| {
                (
                    ("name", o.name.to_owned()),
                    ("method", o.method.to_owned()),
                    ("ops", o.ops.into_iter().map(DbusArg).collect::<Vec<_>>()),
                )
            })
            .collect::<Vec<_>>();

        Ok((list,))
    });
}
