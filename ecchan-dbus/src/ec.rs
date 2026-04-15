use std::sync::Arc;

use dbus::MethodErr;
use ec::{Ec, EcError};
use sayuri::sync::Mutex;

pub struct EcSession {
    pub ec: Arc<Mutex<Ec>>,
}

pub struct DbusEcError(EcError);

impl From<DbusEcError> for MethodErr {
    fn from(value: DbusEcError) -> Self {
        Self::failed(&value.0)
    }
}

impl From<EcError> for DbusEcError {
    fn from(value: EcError) -> Self {
        Self(value)
    }
}
