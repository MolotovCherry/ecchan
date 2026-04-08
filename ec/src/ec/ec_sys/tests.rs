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
    fw::{BatteryMode, Curve6, Curve7, FW_REGISTRY, SuperBatteryKind},
    models::MODEL_REGISTRY,
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

// EcIo Test Backend

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

struct TestReset(MutexGuard<'static, Ec>);

impl Deref for TestReset {
    type Target = MutexGuard<'static, Ec>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TestReset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for TestReset {
    fn drop(&mut self) {
        self.0.sys.as_ref().unwrap().0.file.reset();
    }
}

fn get_ec() -> TestReset {
    static EC: LazyLock<Mutex<Ec>> = LazyLock::new(|| {
        let file = EcTestFile::new();

        let ec_sys = EcSys { file };
        let fw = FW_REGISTRY.get("17Q1IMS1.10C").unwrap();
        let model = MODEL_REGISTRY.get_from_name("Titan GT77 12UHS").unwrap();

        let ec = Ec {
            sys: Some((ec_sys, fw)),
            model: Some(model),
        };

        Mutex::new(ec)
    });

    TestReset(EC.lock())
}

/// Empty vals means assert nothing was written
#[track_caller]
fn assert_wrote(ec: &TestReset, addr: u8, vals: &[u8]) {
    let addr_u = addr as usize;

    assert!(
        addr_u.saturating_add(vals.len().saturating_sub(1)) <= 0xFF,
        "addr + vals.len overflowed"
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

    bin[addr_u..addr_u + vals.len()].copy_from_slice(vals);

    let range = addr..=addr + vals.len().saturating_sub(1) as u8;

    assert_eq!(dump, bin);

    let writes = &ec.sys.as_ref().unwrap().0.file.writes;
    for (addr, w) in writes.iter().enumerate() {
        let addr = addr as u8;
        let v = w.load(Ordering::Relaxed);

        if vals.is_empty() {
            assert!(!v, "illegal write at 0x{addr:>02X}");
        } else {
            assert_eq!(v, range.contains(&addr), "illegal write at 0x{addr:>02X}");
        }
    }

    // restore old read values to untaint
    for (i, b) in ec.sys.as_ref().unwrap().0.file.reads.iter().enumerate() {
        b.store(old_read[i], Ordering::Relaxed);
    }
}

#[track_caller]
fn assert_unwritten(ec: &TestReset) {
    assert_wrote(ec, 0x00, &[]);
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

//
// Single read/write
//

#[test]
fn test_read_1() {
    let ec = get_ec();
    let io = get_io!(ec);
    let val = io.ec_read(0x2F).unwrap();
    assert_read(&ec, 0x2F);
    assert_eq!(0x5B, val);
    assert_unwritten(&ec);
}

#[test]
fn test_write_1() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);
    unsafe {
        io.ec_write(0x2F, 0xFB).unwrap();
    }

    assert_wrote(&ec, 0x2F, &[0xFB]);
    assert_unread(&ec);
}

//
// End Seq read/write
//

#[test]
fn test_end_seq_read_1() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write(0xFF, 0xFE).unwrap();
    }

    let mut buf = [0];
    io.ec_read_seq(0xFF, &mut buf).unwrap();
    assert_read(&ec, 0xFF);
    assert_eq!(0xFE, buf[0]);
    assert_wrote(&ec, 0xFF, &[0xFE]);
}

#[test]
#[should_panic = "addr 0xFF + buf len 2 overflows"]
fn test_end_seq_read_2() {
    let ec = get_ec();
    let io = get_io!(ec);

    let mut buf = [0, 0];
    io.ec_read_seq(0xFF, &mut buf).unwrap();
    assert_unwritten(&ec);
}

#[test]
fn test_end_write_seq_1() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write_seq(0xFF, &[0x44]).unwrap();
    }

    assert_wrote(&ec, 0xFF, &[0x44]);
    assert_unread(&ec);
}

#[test]
#[should_panic = "addr 0xFF + buf len 2 overflows"]
fn test_end_seq_write_2() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write_seq(0xFF, &[0xCE, 0xDE]).unwrap();
    }
}

//
// General seq read/write
//

#[test]
fn test_seq_read_4() {
    let ec = get_ec();
    let io = get_io!(ec);

    let mut buf = [0; 4];
    io.ec_read_seq(0xF2, &mut buf).unwrap();
    assert_read_range(&ec, 0xF2..=0xF5);
    assert_eq!([0x70, 0x00, 0x23, 0x44], buf);
    assert_unwritten(&ec);
}

#[test]
fn test_seq_write_4() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let vals = &[0x01, 0x02, 0x03, 0x04];

    unsafe {
        io.ec_write_seq(0xF2, vals).unwrap();
    }

    assert_wrote(&ec, 0xF2, vals);
    assert_unread(&ec);
}

//
// Bit tests
//

#[test]
fn test_read_bit() {
    let ec = get_ec();
    let io = get_io!(ec);

    let set = io.ec_read_bit(0x2E, Bit::_6).unwrap();
    assert_read(&ec, 0x2E);
    assert!(set, "bit 6 set");

    let set = io.ec_read_bit(0x2E, Bit::_7).unwrap();
    assert_read(&ec, 0x2E);
    assert!(!set, "bit 7 not set");

    assert_unwritten(&ec);
}

#[test]
fn test_write_bit() {
    let mut ec = get_ec();
    let io = get_io!(ec);

    let set = io.ec_read_bit(0x2E, Bit::_6).unwrap();
    assert_read(&ec, 0x2E);
    assert!(set, "bit 6 set");

    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write_bit(0x2E, Bit::_6, false).unwrap();
    }

    let set = io.ec_read_bit(0x2E, Bit::_6).unwrap();
    assert_read(&ec, 0x2E);
    assert!(!set, "bit 6 not set");

    assert_wrote(&ec, 0x2E, &[0x0B]);

    drop(ec);

    //

    let mut ec = get_ec();
    let io = get_io!(ec);

    let set = io.ec_read_bit(0x2E, Bit::_7).unwrap();
    assert_read(&ec, 0x2E);
    assert!(!set, "bit 7 not set");

    let io = get_io_mut!(ec);

    unsafe {
        io.ec_write_bit(0x2E, Bit::_7, true).unwrap();
    }

    let set = io.ec_read_bit(0x2E, Bit::_7).unwrap();
    assert_read(&ec, 0x2E);
    assert!(set, "bit 7 set");

    assert_wrote(&ec, 0x2E, &[0xCB]);
}

//
// Firmware
//

#[test]
fn test_firmware_version() {
    let ec = get_ec();
    let version = ec.fw_version().unwrap();
    assert_eq!(version, "17Q1IMS1.10C");

    assert_read_range(&ec, 0xA0..=0xAB);
    assert_unwritten(&ec);
}

#[test]
fn test_firmware_date() {
    let ec = get_ec();
    let date = ec.fw_date().unwrap();
    assert_eq!(date, "06132023");

    assert_read_range(&ec, 0xAC..=0xB3);
    assert_unwritten(&ec);
}

#[test]
fn test_firmware_time() {
    let ec = get_ec();
    let time = ec.fw_time().unwrap();
    assert_eq!(time, "13:35:33");

    assert_read_range(&ec, 0xB4..=0xBB);
    assert_unwritten(&ec);
}

//
// Battery
//

#[test]
fn test_battery_mode() {
    let ec = get_ec();
    let mode = ec.battery_mode().unwrap();
    assert_eq!(BatteryMode::Healthy, mode);

    assert_read(&ec, 0xD7);
    assert_unwritten(&ec);
}

#[test]
fn test_super_battery() {
    let ec = get_ec();
    let mode = ec.super_battery().unwrap();
    assert_eq!(SuperBatteryKind::On, mode);

    assert_read(&ec, 0xEB);
    assert_unwritten(&ec);
}

//
// Fan RPMs
//

#[test]
fn test_fan1_rpm() {
    let ec = get_ec();
    let mode = ec.fan1_rpm().unwrap();
    assert_eq!(1441, mode);

    assert_read_range(&ec, 0xC8..=0xC9);
    assert_unwritten(&ec);
}

#[test]
fn test_fan2_rpm() {
    let ec = get_ec();
    let mode = ec.fan2_rpm().unwrap();
    assert_eq!(1745, mode);

    assert_read_range(&ec, 0xCA..=0xCB);
    assert_unwritten(&ec);
}

#[test]
fn test_fan3_rpm() {
    let ec = get_ec();
    let mode = ec.fan3_rpm().unwrap();
    assert_eq!(0, mode);

    assert_read_range(&ec, 0xCC..=0xCD);
    assert_unwritten(&ec);
}

#[test]
fn test_fan4_rpm() {
    let ec = get_ec();
    let mode = ec.fan4_rpm().unwrap();
    assert_eq!(0, mode);

    assert_read_range(&ec, 0xCE..=0xCF);
    assert_unwritten(&ec);
}

//
// Thermal
//

#[test]
fn test_cpu_rt_temp() {
    let ec = get_ec();
    let temp = ec.cpu_rt_temp().unwrap();
    assert_eq!(43, temp);

    assert_read(&ec, 0x68);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_rt_temp() {
    let ec = get_ec();
    let temp = ec.gpu_rt_temp().unwrap();
    assert_eq!(46, temp);

    assert_read(&ec, 0x80);
    assert_unwritten(&ec);
}

#[test]
fn test_cpu_rt_fan_speed() {
    let ec = get_ec();
    let temp = ec.cpu_rt_fan_speed().unwrap();
    assert_eq!(25, temp);

    assert_read(&ec, 0x71);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_rt_fan_speed() {
    let ec = get_ec();
    let temp = ec.gpu_rt_fan_speed().unwrap();
    assert_eq!(30, temp);

    assert_read(&ec, 0x89);
    assert_unwritten(&ec);
}

//
// Curves
//

#[test]
fn test_cpu_fan_curve() {
    let ec = get_ec();
    let curve = ec.cpu_fan_curve().unwrap();
    assert_eq!(
        Curve7 {
            n1: 25,
            n2: 25,
            n3: 35,
            n4: 55,
            n5: 65,
            n6: 70,
            n7: 80
        },
        curve
    );

    assert_read_range(&ec, 0x72..=0x78);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_fan_curve() {
    let ec = get_ec();
    let curve = ec.gpu_fan_curve().unwrap();

    assert_eq!(
        Curve6 {
            n1: 0,
            n2: 30,
            n3: 40,
            n4: 55,
            n5: 60,
            n6: 70,
        },
        curve
    );

    assert_read_range(&ec, 0x8A..=0x8F);
    assert_unwritten(&ec);
}

#[test]
fn test_cpu_temp_curve() {
    let ec = get_ec();
    let curve = ec.cpu_temp_curve().unwrap();
    assert_eq!(
        Curve7 {
            n1: 0,
            n2: 55,
            n3: 64,
            n4: 70,
            n5: 76,
            n6: 82,
            n7: 88
        },
        curve
    );

    assert_read_range(&ec, 0x69..=0x6F);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_temp_curve() {
    let ec = get_ec();
    let curve = ec.gpu_temp_curve().unwrap();

    assert_eq!(
        Curve7 {
            n1: 0,
            n2: 52,
            n3: 58,
            n4: 64,
            n5: 70,
            n6: 76,
            n7: 82
        },
        curve
    );

    assert_read_range(&ec, 0x81..=0x87);
    assert_unwritten(&ec);
}

#[test]
fn test_cpu_hysteresis_curve() {
    let ec = get_ec();
    let curve = ec.cpu_hysteresis_curve().unwrap();
    assert_eq!(
        Curve6 {
            n1: 10,
            n2: 3,
            n3: 3,
            n4: 3,
            n5: 3,
            n6: 3,
        },
        curve
    );

    assert_read_range(&ec, 0x7A..=0x7F);
    assert_unwritten(&ec);
}

#[test]
fn test_gpu_hysteresis_curve() {
    let ec = get_ec();
    let curve = ec.gpu_hysteresis_curve().unwrap();

    assert_eq!(
        Curve6 {
            n1: 7,
            n2: 3,
            n3: 3,
            n4: 3,
            n5: 3,
            n6: 3,
        },
        curve
    );

    assert_read_range(&ec, 0x92..=0x97);
    assert_unwritten(&ec);
}

#[test]
#[should_panic = "no dgpu available"]
fn test_gpu_fan_curve_unsupported() {
    let mut ec = get_ec();
    ec.model.as_mut().unwrap().has_dgpu = false;
    let res = ec.gpu_fan_curve();
    ec.model.as_mut().unwrap().has_dgpu = true;
    res.unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_gpu_temp_curve_unsupported() {
    let mut ec = get_ec();
    ec.model.as_mut().unwrap().has_dgpu = false;
    let res = ec.gpu_temp_curve();
    ec.model.as_mut().unwrap().has_dgpu = true;
    res.unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_gpu_hysteresis_curve_unsupported() {
    let mut ec = get_ec();
    ec.model.as_mut().unwrap().has_dgpu = false;
    let res = ec.gpu_hysteresis_curve();
    ec.model.as_mut().unwrap().has_dgpu = true;
    res.unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_set_gpu_fan_curve_unsupported() {
    let mut ec = get_ec();
    ec.model.as_mut().unwrap().has_dgpu = false;
    let res = ec.set_gpu_fan_curve(Default::default());
    ec.model.as_mut().unwrap().has_dgpu = true;
    res.unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_set_gpu_temp_curve_unsupported() {
    let mut ec = get_ec();
    ec.model.as_mut().unwrap().has_dgpu = false;
    let res = ec.set_gpu_temp_curve(Default::default());
    ec.model.as_mut().unwrap().has_dgpu = true;
    res.unwrap();
}

#[test]
#[should_panic = "no dgpu available"]
fn test_set_gpu_hysteresis_curve_unsupported() {
    let mut ec = get_ec();
    ec.model.as_mut().unwrap().has_dgpu = false;
    let res = ec.set_gpu_hysteresis_curve(Default::default());
    ec.model.as_mut().unwrap().has_dgpu = true;
    res.unwrap();
}
