use dbus::MethodErr;
use ecchan_ipc::ret::RetVal;

use crate::sock::ClientError;

pub trait ToMethodErr<'a> {
    fn to_method_err(self) -> Result<RetVal<'a>, MethodErr>;
}

impl<'a> ToMethodErr<'a> for Result<RetVal<'a>, ClientError> {
    fn to_method_err(self) -> Result<RetVal<'a>, MethodErr> {
        self.map_err(|e| MethodErr::failed(&e))
    }
}
