#[cfg(test)]
mod tests;

#[cfg(not(test))]
use std::fs::File;
#[cfg(test)]
use tests::EcTestFile as File;
#[cfg(test)]
pub(crate) use tests::HAS_DGPU;

use std::{io, os::unix::fs::FileExt as _};

use nix::errno::Errno;
use snafu::prelude::*;

use crate::fw::{Bit, BitSet as _};

pub(super) type Result<T> = std::result::Result<T, EcSysError>;

#[derive(Debug, Snafu)]
pub enum EcSysError {
    #[snafu(display("ec_sys write_support is not enabled"))]
    NoWriteSupport,
    #[snafu(display("ec_sys module not loaded"))]
    NotLoaded,
    #[snafu(display("ec_sys io error"))]
    OtherIo { source: io::Error },
    #[snafu(display("ec_sys io error"))]
    OtherErrno { source: Errno },
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

#[allow(clippy::assertions_on_constants)]
#[cfg(not(test))]
fn create_ec_io() -> Result<File> {
    use io::ErrorKind;

    assert!(cfg!(not(test)), "this cannot be called in test mode");

    use std::fs::OpenOptions;

    const EC_IO: &str = "/sys/kernel/debug/ec/ec0/io";
    const EC_WRITE_SUPPORT: &str = "/sys/module/ec_sys/parameters/write_support";

    let res = OpenOptions::new()
        .read(true)
        .write(false)
        .append(false)
        .truncate(false)
        .create(false)
        .create_new(false)
        .open(EC_WRITE_SUPPORT);

    let file = match res {
        Ok(f) => f,
        Err(e) => {
            let err = match e.kind() {
                ErrorKind::NotFound => {
                    log::warn!("ec_sys is not loaded; certain functions will be disabled");
                    EcSysError::NotLoaded
                }

                _ => EcSysError::OtherIo { source: e },
            };

            return Err(err);
        }
    };

    let mut buf = [0; 1];
    file.read_exact_at(&mut buf, 0)
        .map_err(|e| EcSysError::OtherIo { source: e })?;

    let write_enabled = &buf == b"Y";
    if !write_enabled {
        log::error!("ec_sys write_support is disabled; please enable it");
        return Err(EcSysError::NoWriteSupport);
    }

    let res = OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .truncate(false)
        .create(false)
        .create_new(false)
        .open(EC_IO);

    let file = match res {
        Ok(f) => f,
        Err(e) => {
            let err = match e.kind() {
                ErrorKind::NotFound => {
                    log::warn!("ec_sys is not loaded; certain functions will be disabled");
                    EcSysError::NotLoaded
                }

                _ => EcSysError::OtherIo { source: e },
            };

            return Err(err);
        }
    };

    Ok(file)
}

pub struct EcSys {
    file: File,
}

impl EcSys {
    #[allow(clippy::assertions_on_constants)]
    #[cfg(not(test))]
    pub(super) fn new() -> Result<Self> {
        assert!(cfg!(not(test)), "this cannot be called in test mode");

        let this = Self {
            file: create_ec_io()?,
        };

        log::info!("using ec_sys io");

        Ok(this)
    }

    #[cfg(test)]
    pub(super) fn new() -> Result<Self> {
        unreachable!("this cannot be called in test mode");
    }

    pub fn ec_read(&self, addr: u8) -> Result<u8> {
        let mut val = [0];

        match self
            .file
            .read_exact_at(&mut val, addr as _)
            .context(OtherIoSnafu)
        {
            Ok(_) => Ok(val[0]),
            Err(e) => {
                log::error!("ec_read(): {e}");
                Err(e)
            }
        }
    }

    pub fn ec_read_seq(&self, addr: u8, buf: &mut [u8]) -> Result<()> {
        let len = buf.len().saturating_sub(1);
        if (addr as usize).saturating_add(len) > 0xFF {
            whatever!("addr 0x{addr:X} + buf len {} overflows", buf.len());
        }

        match self
            .file
            .read_exact_at(buf, addr as _)
            .context(OtherIoSnafu)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("ec_read_seq(): {e}");
                Err(e)
            }
        }
    }

    pub fn ec_read_bit(&self, addr: u8, bit: Bit) -> Result<bool> {
        let val = match self.ec_read(addr) {
            Ok(v) => v,
            Err(e) => {
                log::error!("ec_read_bit(): {e}");
                return Err(e);
            }
        };

        Ok(val.bit_set(bit))
    }

    //
    // The following write functions do not require mutable access,
    // but mutable access guarantees we get semaphore synchronization
    // for free thanks to Rust's typesystem.
    //

    /// # SAFETY
    /// Improper usage of this function will result in hardware damage or bricked computer
    pub unsafe fn ec_write(&mut self, addr: u8, val: u8) -> Result<()> {
        // we are protected against writing past bounds since addr's type guarantees this,
        // and we're writing a single value

        match self.file.write_at(&[val], addr as _).context(OtherIoSnafu) {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("ec_write(): {e}");
                Err(e)
            }
        }
    }

    /// # SAFETY
    /// Improper usage of this function will result in permanent hardware damage or a bricked computer
    pub unsafe fn ec_write_seq(&mut self, addr: u8, vals: &[u8]) -> Result<()> {
        let len = vals.len().saturating_sub(1);
        if (addr as usize).saturating_add(len) > 0xFF {
            whatever!("addr 0x{addr:X} + buf len {} overflows", vals.len());
        }

        match self
            .file
            .write_all_at(vals, addr as _)
            .context(OtherIoSnafu)
        {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("ec_write_seq(): {e}");
                Err(e)
            }
        }
    }

    /// # SAFETY
    /// Improper usage of this function will result in permanent hardware damage or a bricked computer
    pub unsafe fn ec_write_bit(&mut self, addr: u8, bit: Bit, state: bool) -> Result<()> {
        let mut val = match self.ec_read(addr) {
            Ok(v) => v,
            Err(e) => {
                log::error!("ec_read_bit(): {e}");
                return Err(e);
            }
        };

        val.set_bit_state(bit, state);

        match unsafe { self.ec_write(addr, val) } {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("ec_write_bit(): {e}");
                Err(e)
            }
        }
    }
}
