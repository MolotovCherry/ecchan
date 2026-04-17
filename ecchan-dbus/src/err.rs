use std::fmt::Display;

use dbus::MethodErr;

pub trait ToMethodErr<'a, T> {
    fn to_method_err(self) -> Result<T, MethodErr>;
}

impl<'a, T, E: Display> ToMethodErr<'a, T> for Result<T, E> {
    fn to_method_err(self) -> Result<T, MethodErr> {
        self.map_err(|e| MethodErr::failed(&e))
    }
}
