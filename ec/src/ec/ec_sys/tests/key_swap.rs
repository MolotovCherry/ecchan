use super::*;
use crate::fw::KeyDirection;

#[test]
fn test_fn_key() {
    let ec = get_ec();
    let status = ec.fn_key().unwrap();
    assert_eq!(status, KeyDirection::Right);
    assert_read(&ec, 0xE8);
    assert_unwritten(&ec);
}

#[test]
fn test_set_fn_key() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0xE8, Bit::_4).unwrap();
    assert!(!val, "fn key is right");
    assert_read(&ec, 0xE8);

    ec.set_fn_key(KeyDirection::Left).unwrap();
    assert_wrote(&ec, 0xE8, &[0x11]);

    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0xE8, Bit::_4).unwrap();
    assert!(val, "fn key is left");

    ec.set_fn_key(KeyDirection::Right).unwrap();
    assert_wrote(&ec, 0xE8, &[0x01]);
}

#[test]
fn test_win_key() {
    let ec = get_ec();
    let status = ec.win_key().unwrap();
    assert_eq!(status, KeyDirection::Left);
    assert_read(&ec, 0xE8);
    assert_unwritten(&ec);
}

#[test]
fn test_set_win_key() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0xE8, Bit::_4).unwrap();
    assert!(!val, "win key is left");
    assert_read(&ec, 0xE8);

    ec.set_win_key(KeyDirection::Right).unwrap();
    assert_wrote(&ec, 0xE8, &[0x11]);

    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0xE8, Bit::_4).unwrap();
    assert!(val, "win key is right");

    ec.set_win_key(KeyDirection::Left).unwrap();
    assert_wrote(&ec, 0xE8, &[0x01]);
}

#[test]
fn test_key_swap_supported() {
    let ec = get_ec();
    assert!(ec.fn_win_swap_supported());
}
