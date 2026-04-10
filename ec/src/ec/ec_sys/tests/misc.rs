use crate::models::Fan;

use super::*;

#[test]
fn test_has_dgpu() {
    let ec = get_ec();
    assert!(ec.has_dgpu());
}

#[test]
fn test_fan_count() {
    let ec = get_ec();
    assert_eq!(ec.fan_count(), Fan::Four);
}

#[test]
fn test_wmi_ver() {
    let ec = get_ec();
    assert_eq!(ec.wmi_ver(), Some(WmiVer::Wmi2));
}
