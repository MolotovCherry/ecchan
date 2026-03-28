//! Please see <https://github.com/BeardOverflow/msi-ec>
//! for more hardware configurations if they aren't supported here.
//!
//! If your hardware/version isn't supported there, please check the readme,
//! follow the instructions, and make an issue or PR for your device.
//!
//! I intentionally maintain similar api compatibility with
//! <https://github.com/BeardOverflow/msi-ec/blob/main/msi-ec.c>
//! such that devices which are supported there may easily be added here.
//!
//! After there's an update to the supported ec list on msi-ec,
//! feel free to make a PR for inclusion here with corresponding links.

mod wmi2;

/// A registry of supported fw configs.
///
/// Once a config is made (following module docs),
/// add it here to support it.
#[rustfmt::skip]
pub static FW_REGISTRY: Registry<'static> = Registry(&[
    wmi2::g2_10::G2_10
]);

pub struct Registry<'a>(&'a [FwConfig]);

impl Registry<'_> {
    pub fn get(&self, ec_version: &str) -> Option<FwConfig> {
        self.0.iter().find(|fw| fw.supports_fw(ec_version)).copied()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Addr {
    Unsupported,
    Addr(u8),
}

impl Addr {
    pub fn is_supported(&self) -> bool {
        matches!(self, Addr::Unsupported)
    }

    pub fn get(&self) -> Option<u8> {
        match self {
            Self::Unsupported => None,
            Self::Addr(addr) => Some(*addr),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Webcam {
    pub addr: Addr,
    pub block_addr: Addr,
    pub bit: Bit,
}

#[derive(Debug, Copy, Clone)]
pub struct FnWinSwap {
    pub addr: Addr,
    pub bit: Bit,
    pub invert: bool,
}

impl FnWinSwap {
    pub fn is_supported(&self) -> bool {
        self.addr.is_supported()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CoolerBoost {
    pub addr: Addr,
    pub bit: Bit,
}

#[derive(Debug, Copy, Clone)]
pub struct ShiftMode {
    pub addr: Addr,
    pub modes: &'static [(ShiftModeKind, u8)],
}

impl ShiftMode {
    pub fn get_modes(&self) -> Vec<ShiftModeKind> {
        self.modes
            .iter()
            .map(|(k, _)| *k)
            .filter(|m| !matches!(m, ShiftModeKind::Null))
            .collect()
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShiftModeKind {
    /// User High / Extreme Performance (old: Sport Mode)
    ExtremePerformance,
    /// User Medium / Balance / Silent (old: Comfort Mode)
    Balanced,
    /// User_Low / Super Battery (old: ECO Mode)
    SuperBattery,
    /// Turbo Mode
    Turbo,
    /// Unspecified; This mode cannot be set
    Null,
}

#[derive(Debug, Copy, Clone)]
pub struct SuperBattery {
    pub addr: Addr,
    pub mask: u8,
}

impl SuperBattery {
    pub fn is_supported(&self) -> bool {
        self.addr.is_supported()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FanMode {
    pub addr: Addr,
    pub modes: &'static [(FanModeKind, u8)],
}

impl FanMode {
    pub fn get_modes(&self) -> Vec<FanModeKind> {
        self.modes.iter().map(|(k, _)| *k).collect()
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum FanModeKind {
    Auto,
    Silent,
    Advanced,
    Null,
}

#[derive(Debug, Copy, Clone)]
pub struct Thermal {
    pub rt_temp_addr: Addr,
    pub rt_fan_speed_addr: Addr,
}

#[derive(Debug, Copy, Clone)]
pub struct Leds {
    pub micmute_led_addr: Addr,
    pub mute_led_addr: Addr,
    pub bit: Bit,
}

#[derive(Debug, Copy, Clone)]
pub struct KbdBl {
    pub bl_mode_addr: Addr,
    pub bl_modes: &'static [u8],
    pub max_mode: u8,
    pub bl_state_addr: Addr,
    pub state_base_value: u8,
    pub max_state: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct FwConfig {
    pub allowed_fw: &'static [&'static str],
    pub charge_control_addr: Addr,
    pub webcam: Webcam,
    pub fn_win_swap: FnWinSwap,
    pub cooler_boost: CoolerBoost,
    pub shift_mode: ShiftMode,
    pub super_battery: SuperBattery,
    pub fan_mode: FanMode,
    pub cpu: Thermal,
    pub gpu: Thermal,
    pub leds: Leds,
    pub kbd_bl: KbdBl,
    pub fan_rpm: FanRpm,
    pub cpu_fan_curve: Curve7,
    pub gpu_fan_curve: Curve6,
    pub cpu_temp_curve: Curve7,
    pub gpu_temp_curve: Curve7,
    pub cpu_hysteresis_curve: Curve6,
    pub gpu_hysteresis_curve: Curve6,
}

impl FwConfig {
    pub fn supports_fw(&self, ec_version: &str) -> bool {
        self.allowed_fw.contains(&ec_version)
    }
}

//
// Firmware info addresses are universal
//

#[derive(Debug, Copy, Clone)]
pub struct FwStr {
    pub addr: u8,
    pub len: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct FwInfo {
    pub version: FwStr,
    pub date: FwStr,
    pub time: FwStr,
}

pub const FW_INFO: FwInfo = FwInfo {
    version: FwStr {
        addr: 0xA0,
        len: 12,
    },
    date: FwStr { addr: 0xAC, len: 8 },
    time: FwStr { addr: 0xB4, len: 8 },
};

//
// Fan addresses are universal
//

/// Start address span 2 bytes, read as
/// big endian
///
/// Formula: (take care for division by 0)
/// 480000 / val = rpm
#[derive(Debug, Copy, Clone)]
pub struct FanRpm {
    pub fan1_addr: u8,
    pub fan2_addr: u8,
    pub fan3_addr: u8,
    pub fan4_addr: u8,
}

//
// Curves
//

#[derive(Debug, Copy, Clone)]
pub struct Curve6 {
    pub node1_addr: u8,
    pub node2_addr: u8,
    pub node3_addr: u8,
    pub node4_addr: u8,
    pub node5_addr: u8,
    pub node6_addr: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Curve7 {
    pub node1_addr: u8,
    pub node2_addr: u8,
    pub node3_addr: u8,
    pub node4_addr: u8,
    pub node5_addr: u8,
    pub node6_addr: u8,
    pub node7_addr: u8,
}

//
// Misc
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bit {
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
}

pub trait BitSet {
    fn set_bit(&mut self, bit: Bit, state: bool);
    fn is_bit_set(self, bit: Bit) -> bool;
}

impl BitSet for u8 {
    fn set_bit(&mut self, bit: Bit, state: bool) {
        if state {
            *self |= 1 << bit as u8;
        } else {
            *self &= !(1 << bit as u8);
        }
    }

    fn is_bit_set(self, bit: Bit) -> bool {
        (self & 1 << bit as u8) != 0
    }
}

//
// Battery
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SuperBatteryKind {
    Off,
    On,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BatteryMode {
    /// 50-60
    Healthy,
    /// 70-80
    Balanced,
    /// 100
    Mobility,
    Custom(u8),
}
