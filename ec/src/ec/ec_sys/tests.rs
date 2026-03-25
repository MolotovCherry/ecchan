use std::{
    ops::{Deref, DerefMut},
    sync::LazyLock,
};

use sayuri::sync::{Mutex, MutexGuard};

use super::*;
use crate::{
    Ec,
    fw::{BatteryMode, REGISTRY, SuperBatteryKind},
};

#[rustfmt::skip]
const EC_BIN: [u8; 256] = [
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

/// EcIo Test Backend

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
        self.0.io.as_ref().unwrap().file.set_len(0).unwrap();
        self.0
            .io
            .as_ref()
            .unwrap()
            .file
            .write_all_at(&EC_BIN, 0x00)
            .unwrap();
    }
}

fn get_ec() -> TestReset {
    static EC: LazyLock<Mutex<Ec>> = LazyLock::new(|| {
        let ec_sys = EcSys::new_dummy("ec-test.bin").unwrap();

        ec_sys
            .file
            .write_all_at(&EC_BIN, 0)
            .context(OtherIoSnafu)
            .unwrap();

        let ec = Ec {
            io: Some(ec_sys),
            fw: Some(REGISTRY.get("17Q1IMS1.10C").unwrap()),
        };

        Mutex::new(ec)
    });

    TestReset(EC.lock())
}

fn patch_cmp(ec: &mut TestReset, addr: u8, vals: &[u8]) {
    let addr = addr as usize;

    assert!(
        addr.checked_add(vals.len().saturating_sub(1))
            .unwrap_or(usize::MAX)
            <= 0xFF,
        "addr + vals.len overflowed"
    );

    let dump = ec.ec_dump_raw().unwrap();
    let mut bin = EC_BIN;

    bin[addr..addr + vals.len()].copy_from_slice(vals);

    assert_eq!(dump, bin);
}

//
// Single read/write
//

#[test]
fn test_read_1() {
    let ec = get_ec();
    let val = ec.io.as_ref().unwrap().ec_read(0x2F).unwrap();
    assert_eq!(0x5B, val);
}

#[test]
fn test_write_1() {
    let mut ec = get_ec();
    unsafe {
        ec.io.as_mut().unwrap().ec_write(0x2F, 0xFB).unwrap();
    }

    patch_cmp(&mut ec, 0x2F, &[0xFB]);
}

//
// End Seq read/write
//

#[test]
fn test_end_seq_read_1() {
    let mut ec = get_ec();

    unsafe {
        ec.io.as_mut().unwrap().ec_write(0xFF, 0xFE).unwrap();
    }

    let mut buf = [0];
    ec.io.as_ref().unwrap().ec_read_seq(0xFF, &mut buf).unwrap();
    assert_eq!(0xFE, buf[0]);
}

#[test]
#[should_panic = "addr 0xFF + buf len 2 overflows"]
fn test_end_seq_read_2() {
    let ec = get_ec();
    let mut buf = [0, 0];
    ec.io.as_ref().unwrap().ec_read_seq(0xFF, &mut buf).unwrap();
}

#[test]
fn test_end_write_seq_1() {
    let mut ec = get_ec();

    unsafe {
        ec.io.as_mut().unwrap().ec_write_seq(0xFF, &[0x44]).unwrap();
    }

    patch_cmp(&mut ec, 0xFF, &[0x44]);
}

#[test]
#[should_panic = "addr 0xFF + buf len 2 overflows"]
fn test_end_seq_write_2() {
    let mut ec = get_ec();

    unsafe {
        ec.io
            .as_mut()
            .unwrap()
            .ec_write_seq(0xFF, &[0xCE, 0xDE])
            .unwrap();
    }
}

//
// General seq read/write
//

#[test]
fn test_seq_read_4() {
    let ec = get_ec();
    let mut buf = [0; 4];
    ec.io.as_ref().unwrap().ec_read_seq(0xF2, &mut buf).unwrap();
    assert_eq!([0x70, 0x00, 0x23, 0x44], buf);
}

#[test]
fn test_seq_write_4() {
    let mut ec = get_ec();

    unsafe {
        ec.io
            .as_mut()
            .unwrap()
            .ec_write_seq(0xF2, &[0x01, 0x02, 0x03, 0x04])
            .unwrap();
    }

    patch_cmp(&mut ec, 0xF2, &[0x01, 0x02, 0x03, 0x04]);
}

//
// Bit tests
//

#[test]
fn test_read_bit() {
    let ec = get_ec();

    let set = ec.io.as_ref().unwrap().ec_read_bit(0x2E, Bit::_6).unwrap();
    assert!(set, "bit 6 set");

    let set = ec.io.as_ref().unwrap().ec_read_bit(0x2E, Bit::_7).unwrap();
    assert!(!set, "bit 7 not set");
}

#[test]
fn test_write_bit() {
    let mut ec = get_ec();

    let set = ec.io.as_ref().unwrap().ec_read_bit(0x2E, Bit::_6).unwrap();
    assert!(set, "bit 6 set");

    unsafe {
        ec.io
            .as_mut()
            .unwrap()
            .ec_write_bit(0x2E, Bit::_6, false)
            .unwrap();
    }

    let set = ec.io.as_ref().unwrap().ec_read_bit(0x2E, Bit::_6).unwrap();
    assert!(!set, "bit 6 not set");

    patch_cmp(&mut ec, 0x2E, &[0x0B]);

    //

    let set = ec.io.as_ref().unwrap().ec_read_bit(0x2E, Bit::_7).unwrap();
    assert!(!set, "bit 7 not set");

    unsafe {
        ec.io
            .as_mut()
            .unwrap()
            .ec_write_bit(0x2E, Bit::_7, true)
            .unwrap();
    }

    let set = ec.io.as_ref().unwrap().ec_read_bit(0x2E, Bit::_7).unwrap();
    assert!(set, "bit 7 set");

    patch_cmp(&mut ec, 0x2E, &[0x8B]);
}

//
// Firmware
//

#[test]
fn test_firmware_version() {
    let ec = get_ec();
    let version = ec.fw_version().unwrap();
    assert_eq!(version, "17Q1IMS1.10C");
}

#[test]
fn test_firmware_date() {
    let ec = get_ec();
    let date = ec.fw_date().unwrap();
    assert_eq!(date, "06132023");
}

#[test]
fn test_firmware_time() {
    let ec = get_ec();
    let time = ec.fw_time().unwrap();
    assert_eq!(time, "13:35:33");
}

//
// Battery
//

#[test]
fn test_battery_mode() {
    let ec = get_ec();
    let mode = ec.battery_mode().unwrap();
    assert_eq!(BatteryMode::Healthy, mode);
}

#[test]
fn test_super_battery() {
    let ec = get_ec();
    let mode = ec.super_battery().unwrap();
    assert_eq!(SuperBatteryKind::On, mode);
}

//
// Fan RPMs
//

#[test]
fn test_fan1_rpm() {
    let ec = get_ec();
    let mode = ec.fan1_rpm().unwrap();
    assert_eq!(1441, mode);
}

#[test]
fn test_fan2_rpm() {
    let ec = get_ec();
    let mode = ec.fan2_rpm().unwrap();
    assert_eq!(1745, mode);
}

#[test]
fn test_fan3_rpm() {
    let ec = get_ec();
    let mode = ec.fan3_rpm().unwrap();
    assert_eq!(0, mode);
}

#[test]
fn test_fan4_rpm() {
    let ec = get_ec();
    let mode = ec.fan4_rpm().unwrap();
    assert_eq!(0, mode);
}

//
// Temps
//

#[test]
fn test_cpu_temp() {
    let ec = get_ec();
    let temp = ec.cpu_temp().unwrap();
    assert_eq!(43, temp);
}

#[test]
fn test_gpu_temp() {
    let ec = get_ec();
    let temp = ec.gpu_temp().unwrap();
    assert_eq!(46, temp);
}

//
// Curves
//

#[test]
fn test_cpu_fan_curve() {
    let ec = get_ec();
    let curve = ec.cpu_fan_curve().unwrap();
    assert_eq!(CpuFanCurve(25, 25, 35, 55, 65, 70), curve);
}

#[test]
fn test_gpu_fan_curve() {
    let ec = get_ec();
    let curve = ec.gpu_fan_curve().unwrap();
    assert_eq!(GpuFanCurve(0, 30, 40, 55, 60, 70), curve);
}

#[test]
#[should_panic = "fan point exceeded max"]
fn test_cpu_fan_curve_overflow() {
    let mut ec = get_ec();
    unsafe {
        ec.io.ec_write(CPU_FAN_CURVE_1, 0xFF).unwrap();
    }

    ec.cpu_fan_curve().unwrap();
}

#[test]
#[should_panic = "fan point exceeded max"]
fn test_gpu_fan_curve_overflow() {
    let mut ec = get_ec();
    unsafe {
        ec.io.ec_write(GPU_FAN_CURVE_1, 0xFF).unwrap();
    }

    ec.gpu_fan_curve().unwrap();
}
