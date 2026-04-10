use super::*;
use crate::fw::ShiftMode;

#[test]
fn test_shift_modes() {
    let ec = get_ec();
    let modes = ec.shift_modes().unwrap();
    assert_eq!(
        modes,
        [
            ShiftMode::SuperBattery,
            ShiftMode::Balanced,
            ShiftMode::ExtremePerformance,
            ShiftMode::Turbo,
        ]
    );
}

#[test]
fn test_shift_mode() {
    let ec = get_ec();
    let mode = ec.shift_mode().unwrap();
    assert_eq!(mode, ShiftMode::ExtremePerformance);
    assert_read(&ec, 0xD2);
    assert_unwritten(&ec);
}

#[test]
fn test_set_shift_mode() {
    let mut ec = get_ec();

    ec.set_shift_mode(ShiftMode::Turbo).unwrap();
    assert_write(&ec, 0xD2, 0xC4);
    assert_unread(&ec);
}

#[test]
#[should_panic = "shift mode cannot be null"]
fn test_set_shift_mode_null() {
    let mut ec = get_ec();
    ec.set_shift_mode(ShiftMode::Null).unwrap();
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

    ec.sys.as_mut().unwrap().1.shift_mode.modes = &[];

    ec.set_shift_mode(ShiftMode::ExtremePerformance).unwrap();
}
