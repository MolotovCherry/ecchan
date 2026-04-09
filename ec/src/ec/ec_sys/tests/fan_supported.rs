use super::*;
use crate::models::Fan;

#[test]
fn test_fan1_supported() {
    let ec = get_ec();
    assert!(ec.fan1_supported());
}

#[test]
fn test_fan2_supported() {
    let ec = get_ec();
    assert!(ec.fan2_supported());
}

#[test]
fn test_fan3_supported() {
    let ec = get_ec();
    assert!(ec.fan3_supported());
}

#[test]
fn test_fan4_supported() {
    let ec = get_ec();
    assert!(ec.fan4_supported());
}

#[test]
fn test_fan2_not_supported() {
    let mut ec = get_ec();

    ec.model.as_mut().unwrap().fans = Fan::One;
    assert!(!ec.fan2_supported());
}

#[test]
fn test_fan3_not_supported() {
    let mut ec = get_ec();

    ec.model.as_mut().unwrap().fans = Fan::One;
    assert!(!ec.fan3_supported());
}

#[test]
fn test_fan4_not_supported() {
    let mut ec = get_ec();

    ec.model.as_mut().unwrap().fans = Fan::One;
    assert!(!ec.fan4_supported());
}
