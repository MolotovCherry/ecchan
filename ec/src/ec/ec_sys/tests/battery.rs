use super::*;

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
fn test_set_battery_mode() {
    let mut ec = get_ec();
    ec.set_battery_mode(BatteryMode::Mobility).unwrap();
    assert_unread(&ec);
    assert_wrote(&ec, 0xD7, &[0xE4]);
}

#[test]
fn test_battery_mode_supported() {
    let ec = get_ec();
    assert!(ec.battery_mode_supported());
    assert_unread(&ec);
    assert_unwritten(&ec);
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
    assert_wrote(&ec, 0xEB, &[0x00]);
    assert_read(&ec, 0xEB);
}

#[test]
fn test_super_battery_supported() {
    let ec = get_ec();
    assert!(ec.super_battery_supported());
    assert_unread(&ec);
    assert_unwritten(&ec);
}
