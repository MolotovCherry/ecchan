use super::*;
use crate::fw::Led;

//
// Mute LED
//

#[test]
fn test_mute_led() {
    let ec = get_ec();
    let status = ec.mute_led().unwrap();
    assert_eq!(status, Led::Off);
    assert_read(&ec, 0x2D);
    assert_unwritten(&ec);
}

#[test]
fn test_set_mute_led() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x2D, Bit::_1).unwrap();
    assert!(!val, "mute led is off");
    assert_read(&ec, 0x2D);

    ec.set_mute_led(Led::On).unwrap();
    assert_write(&ec, 0x2D, 0x02);

    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x2D, Bit::_1).unwrap();
    assert!(val, "mute led is on");

    ec.set_mute_led(Led::Off).unwrap();
    assert_write(&ec, 0x2D, 0x00);
}

#[test]
fn test_mute_led_supported() {
    let ec = get_ec();
    assert!(ec.mute_led_supported());
}

//
// Mic Mute LED
//

#[test]
fn test_mic_mute_led() {
    let ec = get_ec();
    let status = ec.mic_mute_led().unwrap();
    assert_eq!(status, Led::Off);
    assert_read(&ec, 0x2C);
    assert_unwritten(&ec);
}

#[test]
fn test_set_mic_mute_led() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x2C, Bit::_1).unwrap();
    assert!(!val, "mic mute led is off");
    assert_read(&ec, 0x2C);

    ec.set_mic_mute_led(Led::On).unwrap();
    assert_write(&ec, 0x2C, 0x02);

    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x2C, Bit::_1).unwrap();
    assert!(val, "mic mute led is on");

    ec.set_mic_mute_led(Led::Off).unwrap();
    assert_write(&ec, 0x2C, 0x00);
}

#[test]
fn test_mic_mute_led_supported() {
    let ec = get_ec();
    assert!(ec.mic_mute_led_supported());
}
