use super::*;
use crate::models::Fan;

#[test]
fn test_fan1_rpm() {
    let ec = get_ec();
    let mode = ec.fan1_rpm().unwrap();
    assert_eq!(1441, mode);

    assert_read_range(&ec, 0xC8..=0xC9);
    assert_unwritten(&ec);
}

#[test]
fn test_fan2_rpm() {
    let ec = get_ec();
    let mode = ec.fan2_rpm().unwrap();
    assert_eq!(1745, mode);

    assert_read_range(&ec, 0xCA..=0xCB);
    assert_unwritten(&ec);
}

#[test]
fn test_fan3_rpm() {
    let ec = get_ec();
    let mode = ec.fan3_rpm().unwrap();
    assert_eq!(0, mode);

    assert_read_range(&ec, 0xCC..=0xCD);
    assert_unwritten(&ec);
}

#[test]
fn test_fan4_rpm() {
    let ec = get_ec();
    let mode = ec.fan4_rpm().unwrap();
    assert_eq!(0, mode);

    assert_read_range(&ec, 0xCE..=0xCF);
    assert_unwritten(&ec);
}

#[test]
#[should_panic = "Unsupported { name: \"Two\" }"]
fn test_fan2_rpm_missing() {
    let mut ec = get_ec();

    ec.model.as_mut().unwrap().fans = Fan::One;
    let res = ec.fan2_rpm();

    res.unwrap();
}

#[test]
#[should_panic = "Unsupported { name: \"Three\" }"]
fn test_fan3_rpm_missing() {
    let mut ec = get_ec();

    ec.model.as_mut().unwrap().fans = Fan::Two;
    let res = ec.fan3_rpm();

    res.unwrap();
}

#[test]
#[should_panic = "Unsupported { name: \"Four\" }"]
fn test_fan4_rpm_missing() {
    let mut ec = get_ec();

    ec.model.as_mut().unwrap().fans = Fan::Three;
    let res = ec.fan4_rpm();

    res.unwrap();
}
