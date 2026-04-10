use super::*;
use crate::fw::WebcamKind;

#[test]
fn test_webcam() {
    let ec = get_ec();
    let status = ec.webcam().unwrap();
    assert_eq!(status, WebcamKind::On);
    assert_read(&ec, 0x2E);
    assert_unwritten(&ec);
}

#[test]
fn test_set_webcam() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x2E, Bit::_1).unwrap();
    assert!(val, "webcam is on");
    assert_read(&ec, 0x2E);

    ec.set_webcam(WebcamKind::Off).unwrap();
    assert_wrote(&ec, 0x2E, &[0x49]);

    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x2E, Bit::_1).unwrap();
    assert!(!val, "webcam is off");

    ec.set_webcam(WebcamKind::On).unwrap();
    assert_wrote(&ec, 0x2E, &[0x4B]);
}

#[test]
fn test_webcam_block() {
    let ec = get_ec();
    let status = ec.webcam_block().unwrap();
    assert_eq!(status, WebcamKind::Off);
    assert_read(&ec, 0x2F);
    assert_unwritten(&ec);
}

#[test]
fn test_set_webcam_block() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x2F, Bit::_1).unwrap() ^ true;
    assert!(!val, "webcam block is off");
    assert_read(&ec, 0x2F);

    ec.set_webcam_block(WebcamKind::On).unwrap();
    assert_wrote(&ec, 0x2F, &[0x59]);

    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x2F, Bit::_1).unwrap() ^ true;
    assert!(val, "webcam block is on");

    ec.set_webcam_block(WebcamKind::Off).unwrap();
    assert_wrote(&ec, 0x2F, &[0x5B]);
}

#[test]
fn test_webcam_supported() {
    let ec = get_ec();
    assert!(ec.webcam_supported());
}

#[test]
fn test_webcam_block_supported() {
    let ec = get_ec();
    assert!(ec.webcam_block_supported());
}
