use super::*;
use crate::fw::ShiftModeKind;

#[test]
fn test_shift_modes() {
    let ec = get_ec();
    let modes = ec.shift_modes().unwrap();
    assert_eq!(
        modes,
        [
            ShiftModeKind::SuperBattery,
            ShiftModeKind::Balanced,
            ShiftModeKind::ExtremePerformance,
            ShiftModeKind::Turbo,
        ]
    );
}

#[test]
fn test_shift_mode() {
    let ec = get_ec();
    let mode = ec.shift_mode().unwrap();
    assert_eq!(mode, ShiftModeKind::ExtremePerformance);
    assert_read(&ec, 0xD2);
    assert_unwritten(&ec);
}

#[test]
fn test_set_shift_mode() {
    let mut ec = get_ec();

    ec.set_shift_mode(ShiftModeKind::Turbo).unwrap();
    assert_wrote(&ec, 0xD2, &[0xC4]);
    assert_unread(&ec);
}

#[test]
#[should_panic = "shift mode cannot be null"]
fn test_set_shift_mode_null() {
    let mut ec = get_ec();
    ec.set_shift_mode(ShiftModeKind::Null).unwrap();
}

#[test]
fn test_shift_mode_supported() {
    let ec = get_ec();

    assert!(ec.shift_mode_supported());
    assert_unwritten(&ec);
    assert_unread(&ec);
}

#[test]
#[should_panic = "ExtremePerformance mode is not supported"]
fn test_set_shift_mode_unsupported() {
    let mut ec = get_ec();

    ec.sys.as_mut().unwrap().1.shift_mode.modes = &[
        (ShiftModeKind::SuperBattery, 0xC2),
        (ShiftModeKind::Balanced, 0xC1),
        (ShiftModeKind::Turbo, 0xC4),
        (ShiftModeKind::Null, 0x00),
    ];

    let res = ec.set_shift_mode(ShiftModeKind::ExtremePerformance);

    ec.sys.as_mut().unwrap().1.shift_mode.modes = &[
        (ShiftModeKind::SuperBattery, 0xC2),
        (ShiftModeKind::Balanced, 0xC1),
        (ShiftModeKind::ExtremePerformance, 0xC0),
        (ShiftModeKind::Turbo, 0xC4),
        (ShiftModeKind::Null, 0x00),
    ];

    res.unwrap();
}
