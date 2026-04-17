use std::{io, sync::MutexGuard};

use crate::sock::{Client, ClientInner};

#[derive(Clone)]
pub struct Data {
    client: Client,
}

impl Data {
    pub fn new() -> io::Result<Self> {
        let client = Client::new()?;
        let this = Self { client };
        Ok(this)
    }

    pub fn get(&self) -> MutexGuard<'_, ClientInner> {
        self.client.get()
    }
}
