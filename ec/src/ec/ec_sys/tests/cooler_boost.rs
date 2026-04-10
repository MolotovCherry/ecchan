use super::*;
use crate::fw::CoolerBoost;

#[test]
fn test_cooler_boost() {
    let ec = get_ec();
    let status = ec.cooler_boost().unwrap();
    assert_eq!(status, CoolerBoost::Off);
    assert_read(&ec, 0x98);
    assert_unwritten(&ec);
}

#[test]
fn test_set_cooler_boost() {
    let mut ec = get_ec();
    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x98, Bit::_7).unwrap();
    assert!(!val, "cooler boost is off");
    assert_read(&ec, 0x98);

    ec.set_cooler_boost(CoolerBoost::On).unwrap();
    assert_wrote(&ec, 0x98, 0x82);

    let io = get_io_mut!(ec);

    let val = io.ec_read_bit(0x98, Bit::_7).unwrap();
    assert!(val, "cooler boost is on");

    ec.set_cooler_boost(CoolerBoost::Off).unwrap();
    assert_wrote(&ec, 0x98, 0x02);
}

#[test]
fn test_cooler_boost_supported() {
    let ec = get_ec();
    assert!(ec.cooler_boost_supported());
}
