use super::*;
use crate::FanModeKind;

#[test]
fn test_fan_modes() {
    let ec = get_ec();
    let modes = ec.fan_modes().unwrap();
    assert_eq!(
        modes,
        [
            FanModeKind::Auto,
            FanModeKind::Silent,
            FanModeKind::Advanced
        ]
    );
}

#[test]
fn test_fan_mode() {
    let ec = get_ec();
    let mode = ec.fan_mode().unwrap();
    assert_eq!(mode, FanModeKind::Auto);
    assert_read(&ec, 0xD4);
    assert_unwritten(&ec);
}

#[test]
fn test_set_fan_mode() {
    let mut ec = get_ec();
    ec.set_fan_mode(FanModeKind::Advanced).unwrap();

    assert_wrote(&ec, 0xD4, &[0x8D]);
    assert_unread(&ec);
}

#[test]
#[should_panic = "fan mode cannot be null"]
fn test_set_fan_mode_null() {
    let mut ec = get_ec();
    ec.set_fan_mode(FanModeKind::Null).unwrap();
}

#[test]
#[should_panic = "Advanced mode is not supported"]
fn test_set_fan_mode_unsupported() {
    let mut ec = get_ec();

    ec.sys.as_mut().unwrap().1.fan_mode.modes = &[
        (FanModeKind::Auto, 0x0D),
        (FanModeKind::Silent, 0x1D),
        (FanModeKind::Null, 0x00),
    ];

    ec.set_fan_mode(FanModeKind::Advanced).unwrap();
}
