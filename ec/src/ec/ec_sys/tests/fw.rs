use super::*;

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
