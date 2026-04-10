macro_rules! get_io {
    ($test_reset:expr) => {
        &$test_reset.sys.as_ref().unwrap().0
    };
}

macro_rules! get_io_mut {
    ($test_reset:expr) => {
        &mut $test_reset.sys.as_mut().unwrap().0
    };
}

mod battery;
mod cooler_boost;
mod curves;
mod ec;
mod fan_mode;
mod fan_rpm;
mod fan_supported;
mod fw;
mod key_swap;
mod misc;
mod mute_leds;
mod rt;
mod shift_mode;
mod thermal;
mod webcam;

use std::{
    array,
    fs::File,
    io::Write as _,
    ops::{Deref, DerefMut, RangeInclusive},
    os::unix::fs::FileExt,
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
};

use nix::sys::memfd::{MFdFlags, memfd_create};
use sayuri::sync::{Mutex, MutexGuard};

use super::*;
use crate::{
    Ec,
    fw::{BatteryMode, Curve6, Curve7, FW_REGISTRY, FwConfig, SuperBattery, WmiVer},
    models::{MODEL_REGISTRY, ModelConfig},
};

#[rustfmt::skip]
static EC_BIN: [u8; 256] = [
    //           _0    _1    _2    _3    _4    _5    _6    _7    _8    _9    _A    _B    _C    _D    _E    _F
    /* 0x0_ */ 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    /* 0x1_ */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    /* 0x2_ */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x05, 0x00, 0x00, 0x00, 0x00, 0x4b, 0x5b,
    /* 0x3_ */ 0x03, 0x01, 0x00, 0x0d, 0x01, 0x00, 0x50, 0x81, 0x6a, 0x18, 0x60, 0x3b, 0x71, 0x02, 0xc0, 0x00,
    /* 0x4_ */ 0x35, 0x0c, 0x36, 0x00, 0x20, 0x17, 0x00, 0x00, 0x6b, 0x0c, 0xca, 0x3c, 0xb5, 0x0b, 0xf8, 0x43,
    /* 0x5_ */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    /* 0x6_ */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2b, 0x00, 0x37, 0x40, 0x46, 0x4c, 0x52, 0x58,
    /* 0x7_ */ 0x64, 0x19, 0x19, 0x19, 0x23, 0x37, 0x41, 0x46, 0x50, 0x00, 0x0a, 0x03, 0x03, 0x03, 0x03, 0x03,
    /* 0x8_ */ 0x2e, 0x00, 0x34, 0x3a, 0x40, 0x46, 0x4c, 0x52, 0x64, 0x1e, 0x00, 0x1e, 0x28, 0x37, 0x3c, 0x46,
    /* 0x9_ */ 0x50, 0x5f, 0x07, 0x03, 0x03, 0x03, 0x03, 0x03, 0x02, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
    /* 0xA_ */ 0x31, 0x37, 0x51, 0x31, 0x49, 0x4d, 0x53, 0x31, 0x2e, 0x31, 0x30, 0x43, 0x30, 0x36, 0x31, 0x33,
    /* 0xB_ */ 0x32, 0x30, 0x32, 0x33, 0x31, 0x33, 0x3a, 0x33, 0x35, 0x3a, 0x33, 0x33, 0x00, 0x00, 0x00, 0x08,
    /* 0xC_ */ 0x00, 0x00, 0x06, 0x30, 0x00, 0x00, 0x00, 0x00, 0x01, 0x4d, 0x01, 0x13, 0x00, 0x00, 0x00, 0x00,
    /* 0xD_ */ 0x00, 0x00, 0xc0, 0x00, 0x0d, 0x00, 0x04, 0xbc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00,
    /* 0xE_ */ 0xe2, 0x00, 0x00, 0x20, 0x17, 0x01, 0x00, 0xc1, 0x01, 0x00, 0x00, 0x0F, 0x00, 0xc1, 0x00, 0x00,
    /* 0xF_ */ 0x00, 0x00, 0x70, 0x00, 0x23, 0x44, 0x3a, 0x00, 0x44, 0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub(crate) static HAS_DGPU: AtomicBool = AtomicBool::new(true);

// EcIo Test Backend

pub struct EcTestFile {
    inner: File,
    writes: [AtomicBool; 256],
    reads: [AtomicBool; 256],
}

impl EcTestFile {
    fn new() -> Self {
        let dummy = memfd_create("ec-test.bin", MFdFlags::empty())
            .context(OtherErrnoSnafu)
            .unwrap();
        let mut file = File::from(dummy);

        file.write_all(&EC_BIN).context(OtherIoSnafu).unwrap();

        Self {
            inner: file,
            writes: array::from_fn(|_| AtomicBool::default()),
            reads: array::from_fn(|_| AtomicBool::default()),
        }
    }

    fn reset(&self) {
        self.inner.write_all_at(&EC_BIN, 0).unwrap();
        for (r, w) in self.writes.iter().zip(self.reads.iter()) {
            r.store(false, Ordering::Relaxed);
            w.store(false, Ordering::Relaxed);
        }
    }
}

impl FileExt for EcTestFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        for r in &self.reads[offset as usize..offset as usize + buf.len()] {
            r.store(true, Ordering::Relaxed);
        }

        self.inner.read_at(buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        for w in &self.writes[offset as usize..offset as usize + buf.len()] {
            w.store(true, Ordering::Relaxed);
        }

        self.inner.write_at(buf, offset)
    }
}

struct TestReset {
    inner: MutexGuard<'static, Ec>,
    _fw: FwConfig,
    _model: ModelConfig,
}

impl Deref for TestReset {
    type Target = MutexGuard<'static, Ec>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TestReset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for TestReset {
    fn drop(&mut self) {
        self.inner.sys.as_mut().unwrap().1 = self._fw;
        *self.inner.model.as_mut().unwrap() = self._model;
        self.inner.sys.as_ref().unwrap().0.file.reset();
        HAS_DGPU.store(true, Ordering::Relaxed);
    }
}

fn get_ec() -> TestReset {
    static FW: LazyLock<FwConfig> = LazyLock::new(|| FW_REGISTRY.get("17Q1IMS1.10C").unwrap());

    static MODEL: LazyLock<ModelConfig> =
        LazyLock::new(|| MODEL_REGISTRY.get_from_name("Titan GT77 12UHS").unwrap());

    static EC: LazyLock<Mutex<Ec>> = LazyLock::new(|| {
        let file = EcTestFile::new();
        let ec_sys = EcSys { file };

        let ec = Ec {
            sys: Some((ec_sys, *FW)),
            model: Some(*MODEL),
        };

        Mutex::new(ec)
    });

    TestReset {
        inner: EC.lock(),
        _fw: *FW,
        _model: *MODEL,
    }
}

#[track_caller]
fn assert_wrote(ec: &TestReset, addr: u8, val: u8) {
    assert_wrote_range(ec, addr..=addr, &[val]);
}

/// Empty vals means assert nothing was written
#[track_caller]
fn assert_wrote_range(ec: &TestReset, addrs: RangeInclusive<u8>, vals: &[u8]) {
    let start = *addrs.start() as usize;
    let end = *addrs.end() as usize;
    let range = start..=end;

    assert!(start <= end, "range start must be <= range end");

    assert_eq!(
        end - start,
        vals.len().saturating_sub(1),
        "vals must be same length as range"
    );

    // cache old reads since ec_dump_raw will taint it
    let old_read: [bool; 256] = ec
        .sys
        .as_ref()
        .unwrap()
        .0
        .file
        .reads
        .iter()
        .map(|b| b.load(Ordering::Relaxed))
        .collect::<Vec<bool>>()
        .try_into()
        .unwrap();

    let dump = ec.ec_dump_raw().unwrap();
    let mut bin = EC_BIN;

    bin[start..=end].copy_from_slice(vals);

    assert_eq!(dump, bin);

    let writes = &ec.sys.as_ref().unwrap().0.file.writes;
    for (addr, w) in writes.iter().enumerate() {
        let addr = addr as u8;
        let v = w.load(Ordering::Relaxed);
        assert_eq!(
            v,
            range.contains(&(addr as usize)),
            "illegal write at 0x{addr:>02X}"
        );
    }

    // restore old read values to untaint
    for (i, b) in ec.sys.as_ref().unwrap().0.file.reads.iter().enumerate() {
        b.store(old_read[i], Ordering::Relaxed);
    }
}

#[track_caller]
fn assert_unwritten(ec: &TestReset) {
    let writes = &ec.sys.as_ref().unwrap().0.file.writes;
    for (addr, w) in writes.iter().enumerate() {
        let addr = addr as u8;
        let v = w.load(Ordering::Relaxed);
        assert!(!v, "illegal write at 0x{addr:>02X}");
    }
}

#[track_caller]
fn assert_unread(ec: &TestReset) {
    let reads = &ec.sys.as_ref().unwrap().0.file.reads;
    for (addr, r) in reads.iter().enumerate() {
        let addr = addr as u8;
        let v = r.load(Ordering::Relaxed);
        assert!(!v, "illegal read at 0x{addr:>02X}");
    }
}

#[track_caller]
fn assert_read(ec: &Ec, addr: u8) {
    assert_read_range(ec, addr..=addr);
}

#[track_caller]
fn assert_read_range(ec: &Ec, range: RangeInclusive<u8>) {
    let reads = &ec.sys.as_ref().unwrap().0.file.reads;
    for (addr, r) in reads.iter().enumerate() {
        let addr = addr as u8;
        let v = r.load(Ordering::Relaxed);
        assert_eq!(v, range.contains(&addr), "illegal read at 0x{addr:>02X}");
    }
}
