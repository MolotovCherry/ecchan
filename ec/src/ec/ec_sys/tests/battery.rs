use super::*;

//
// Battery
//

#[test]
fn test_battery_mode() {
    let ec = get_ec();
    let mode = ec.battery_charge_mode().unwrap();
    assert_eq!(BatteryChargeMode::Healthy, mode);

    assert_read(&ec, 0xD7);
    assert_unwritten(&ec);
}

#[test]
fn test_set_battery_charge_mode() {
    let mut ec = get_ec();
    ec.set_battery_charge_mode(BatteryChargeMode::Mobility).unwrap();
    assert_unread(&ec);
    assert_write(&ec, 0xD7, 0xE4);

    ec.set_battery_charge_mode(BatteryChargeMode::from_start(0).unwrap())
        .unwrap();
    assert_unread(&ec);
    assert_write(&ec, 0xD7, 0x8A);

    ec.set_battery_charge_mode(BatteryChargeMode::from_start(23).unwrap())
        .unwrap();
    assert_unread(&ec);
    assert_write(&ec, 0xD7, 0xA1);
}

#[test]
fn test_battery_charge_mode_supported() {
    let ec = get_ec();
    assert!(ec.battery_charge_mode_supported());
    assert_unread(&ec);
    assert_unwritten(&ec);
}

#[test]
#[should_panic]
fn test_set_battery_mode_too_low_end() {
    BatteryChargeMode::from_end(9).unwrap();
}

#[test]
fn test_set_battery_mode_lowest() {
    BatteryChargeMode::from_end(10).unwrap();
}

#[test]
#[should_panic]
fn test_set_battery_mode_too_high_start() {
    BatteryChargeMode::from_start(91).unwrap();
}

#[test]
fn test_set_battery_mode_high_start() {
    BatteryChargeMode::from_start(90).unwrap();
}

#[test]
#[should_panic]
fn test_set_battery_mode_too_high_end() {
    BatteryChargeMode::from_end(101).unwrap();
}

#[test]
fn test_set_battery_mode_high_end() {
    BatteryChargeMode::from_end(100).unwrap();
}

//
// Super Battery
//

#[test]
fn test_super_battery() {
    let ec = get_ec();
    let mode = ec.super_battery().unwrap();
    assert_eq!(SuperBattery::On, mode);

    assert_read(&ec, 0xEB);
    assert_unwritten(&ec);
}

#[test]
fn test_set_super_battery() {
    let mut ec = get_ec();
    ec.set_super_battery(SuperBattery::Off).unwrap();
    assert_write(&ec, 0xEB, 0x00);
    assert_read(&ec, 0xEB);
}

#[test]
fn test_super_battery_supported() {
    let ec = get_ec();
    assert!(ec.super_battery_supported());
    assert_unread(&ec);
    assert_unwritten(&ec);
}
