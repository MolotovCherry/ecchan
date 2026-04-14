use super::*;
use crate::{
    ec::MethodData,
    fw::Addr,
    models::{Method, MethodOp, MethodTy},
};

fn inject(ec: &mut Ec, methods: &'static [Method]) {
    ec.model.as_mut().unwrap().methods = methods;
}

#[test]
#[should_panic = "Unsupported { name: \"method_read\" }"]
fn test_method_read_unsupported() {
    let mut ec = get_ec();

    ec.sys = None;
    ec.method_read("-", MethodOp::Read).unwrap();
}

#[test]
#[should_panic = "Write is not a kind of read"]
fn test_method_read_unread() {
    let ec = get_ec();

    ec.method_read("-", MethodOp::Write).unwrap();
}

#[test]
#[should_panic = "WriteBit is not a kind of read"]
fn test_method_read_unread2() {
    let ec = get_ec();

    ec.method_read("-", MethodOp::WriteBit).unwrap();
}

#[test]
#[should_panic = "WriteRange is not a kind of read"]
fn test_method_read_unread3() {
    let ec = get_ec();

    ec.method_read("-", MethodOp::WriteRange).unwrap();
}

#[test]
#[should_panic = "model config does not exist"]
fn test_method_read_no_model() {
    let mut ec = get_ec();

    ec.model = None;
    ec.method_read("-", MethodOp::Read).unwrap();
}

#[test]
#[should_panic = "address should be supported"]
fn test_method_read_addr_unsupported() {
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

    ec.method_read("-", MethodOp::Read).unwrap();
}

#[test]
#[should_panic = "-() does not expose Read capability"]
fn test_method_read_read_unsupported() {
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

    ec.method_read("-", MethodOp::Read).unwrap();
}

#[test]
#[should_panic = "-() does not expose ReadRange capability"]
fn test_method_read_read_range_unsupported() {
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

    ec.method_read("-", MethodOp::ReadRange).unwrap();
}

#[test]
#[should_panic = "-() does not expose ReadBit capability"]
fn test_method_read_read_bit_unsupported() {
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

    ec.method_read("-", MethodOp::ReadBit).unwrap();
}

#[test]
#[should_panic = "method abc() not found"]
fn test_method_read_method_not_found() {
    let mut ec = get_ec();

    inject(&mut ec, &[]);
    ec.method_read("abc", MethodOp::ReadRange).unwrap();
}

#[test]
#[should_panic = "assertion failed: end >= start"]
fn test_method_read_bad_range() {
    let mut ec = get_ec();

    inject(
        &mut ec,
        &[Method {
            name: "-",
            method: "-",
            #[allow(clippy::reversed_empty_ranges)]
            addr: Addr::Range(0xFF..=0x00),
            ty: &[MethodTy::ReadRange],
        }],
    );

    _ = ec.method_read("-", MethodOp::ReadRange);
}

//
// Test actual model functions for safety
//

#[test]
fn test_method_read_display_overdrive() {
    let ec = get_ec();
    let r = ec
        .method_read("display_overdrive", MethodOp::ReadBit)
        .unwrap();

    assert_eq!(r, MethodData::Bit(false));
    assert_read(&ec, 0x2E);
    assert_unwritten(&ec);
}

#[test]
fn test_method_read_usb_power_share() {
    let ec = get_ec();
    let r = ec
        .method_read("usb_power_share", MethodOp::ReadBit)
        .unwrap();

    assert_eq!(r, MethodData::Bit(false));
    assert_read(&ec, 0xBF);
    assert_unwritten(&ec);
}
