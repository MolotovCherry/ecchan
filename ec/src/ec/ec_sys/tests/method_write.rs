use super::*;
use crate::{
    MethodData,
    fw::Addr,
    models::{Method, MethodOp, MethodTy},
};

fn inject(ec: &mut Ec, methods: &'static [Method]) {
    ec.model.as_mut().unwrap().methods = methods;
}

#[test]
#[should_panic = "Unsupported { name: \"method_write\" }"]
fn test_method_write_unsupported() {
    let mut ec = get_ec();

    ec.sys = None;
    ec.method_write("-", MethodOp::Write, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "Read is not a kind of write"]
fn test_method_write_unread() {
    let mut ec = get_ec();

    ec.method_write("-", MethodOp::Read, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "ReadBit is not a kind of write"]
fn test_method_write_unread2() {
    let mut ec = get_ec();

    ec.method_write("-", MethodOp::ReadBit, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "ReadRange is not a kind of write"]
fn test_method_write_unread3() {
    let mut ec = get_ec();

    ec.method_write("-", MethodOp::ReadRange, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "model config does not exist"]
fn test_method_write_no_model() {
    let mut ec = get_ec();

    ec.model = None;
    ec.method_write("-", MethodOp::Write, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "address should be supported"]
fn test_method_write_addr_unsupported() {
    let mut ec = get_ec();

    inject(
        &mut ec,
        &[Method {
            name: "-",
            method: "-",
            addr: Addr::Unsupported,
            ty: &[],
        }],
    );

    ec.method_write("-", MethodOp::Write, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "-() does not expose Write capability"]
fn test_method_write_write_unsupported() {
    let mut ec = get_ec();

    inject(
        &mut ec,
        &[Method {
            name: "-",
            method: "-",
            addr: Addr::Range(0x00..=0xFF),
            ty: &[],
        }],
    );

    ec.method_write("-", MethodOp::Write, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "-() does not expose WriteRange capability"]
fn test_method_write_write_range_unsupported() {
    let mut ec = get_ec();

    inject(
        &mut ec,
        &[Method {
            name: "-",
            method: "-",
            addr: Addr::Single(0xFF),
            ty: &[],
        }],
    );

    ec.method_write("-", MethodOp::WriteRange, MethodData::Range(Vec::new()))
        .unwrap();
}

#[test]
#[should_panic = "-() does not expose WriteBit capability"]
fn test_method_write_write_bit_unsupported() {
    let mut ec = get_ec();

    inject(
        &mut ec,
        &[Method {
            name: "-",
            method: "-",
            addr: Addr::Range(0x00..=0xFF),
            ty: &[],
        }],
    );

    ec.method_write("-", MethodOp::WriteBit, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "method abc() not found"]
fn test_method_write_method_not_found() {
    let mut ec = get_ec();

    inject(&mut ec, &[]);
    ec.method_write("abc", MethodOp::WriteRange, MethodData::Bit(false))
        .unwrap();
}

#[test]
#[should_panic = "assertion failed: end >= start"]
fn test_method_write_bad_range() {
    let mut ec = get_ec();

    inject(
        &mut ec,
        &[Method {
            name: "-",
            method: "-",
            #[allow(clippy::reversed_empty_ranges)]
            addr: Addr::Range(0xFF..=0x00),
            ty: &[MethodTy::WriteRange],
        }],
    );

    _ = ec.method_write("-", MethodOp::WriteRange, MethodData::Range(Vec::new()));
}

//
// Test actual model functions for safety
//

#[test]
fn test_method_write_display_overdrive() {
    let mut ec = get_ec();
    ec.method_write(
        "display_overdrive",
        MethodOp::WriteBit,
        MethodData::Bit(true),
    )
    .unwrap();

    assert_write(&ec, 0x2E, 0x5B);
    assert_read(&ec, 0x2E);

    ec.method_write(
        "display_overdrive",
        MethodOp::WriteBit,
        MethodData::Bit(false),
    )
    .unwrap();

    assert_write(&ec, 0x2E, 0x4B);
    assert_read(&ec, 0x2E);
}

#[test]
fn test_method_write_usb_power_share() {
    let mut ec = get_ec();
    ec.method_write("usb_power_share", MethodOp::WriteBit, MethodData::Bit(true))
        .unwrap();

    assert_write(&ec, 0xBF, 0x28);
    assert_read(&ec, 0xBF);

    ec.method_write(
        "usb_power_share",
        MethodOp::WriteBit,
        MethodData::Bit(false),
    )
    .unwrap();

    assert_write(&ec, 0xBF, 0x08);
    assert_read(&ec, 0xBF);
}
