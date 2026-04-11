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

use std::ops::{BitAnd, BitOrAssign, BitXor, Not};

mod wmi2;

pub struct FwRegistry;

impl FwRegistry {
    pub fn from_name(ec_version: &str) -> Option<FwConfig> {
        /// A registry of supported fw configs.
        ///
        /// Once a config is made (following module docs),
        /// add it here to support it.
        #[rustfmt::skip]
        static FW_REGISTRY: &[FwConfig] = &[
            wmi2::g2_10::G2_10
        ];

        FW_REGISTRY
            .iter()
            .find(|fw| fw.allowed_fw.contains(&ec_version))
            .copied()
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Addr {
    Unsupported,
    Addr(u8),
}

impl Addr {
    pub fn is_supported(&self) -> bool {
        matches!(self, Addr::Addr(_))
    }

    pub fn get(&self) -> Option<u8> {
        match self {
            Self::Unsupported => None,
            Self::Addr(addr) => Some(*addr),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WebcamConfig {
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

#[derive(Debug, Copy, Clone)]
pub struct CoolerBoostConfig {
    pub addr: Addr,
    pub bit: Bit,
}

#[derive(Debug, Copy, Clone)]
pub struct ShiftModeConfig {
    pub addr: Addr,
    pub modes: &'static [(ShiftMode, u8)],
}

impl ShiftModeConfig {
    pub fn get_modes(&self) -> Vec<ShiftMode> {
        self.modes
            .iter()
            .map(|(k, _)| *k)
            .filter(|m| !matches!(m, ShiftMode::Null))
            .collect()
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShiftMode {
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
pub struct SuperBatteryConfig {
    pub addr: Addr,
    pub mask: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct FanModeConfig {
    pub addr: Addr,
    pub modes: &'static [(FanMode, u8)],
}

impl FanModeConfig {
    pub fn get_modes(&self) -> Vec<FanMode> {
        self.modes
            .iter()
            .map(|(k, _)| *k)
            .filter(|m| !matches!(m, FanMode::Null))
            .collect()
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FanMode {
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
    pub mic_mute_led_addr: Addr,
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
    pub ver: WmiVer,
    pub charge_control_addr: Addr,
    pub webcam: WebcamConfig,
    pub fn_win_swap: FnWinSwap,
    pub cooler_boost: CoolerBoostConfig,
    pub shift_mode: ShiftModeConfig,
    pub super_battery: SuperBatteryConfig,
    pub fan_mode: FanModeConfig,
    pub cpu: Thermal,
    pub gpu: Thermal,
    pub leds: Leds,
    pub kbd_bl: KbdBl,
    pub fan_rpm: FanRpm,
    pub cpu_fan_curve: Curve,
    pub cpu_temp_curve: Curve,
    pub cpu_hysteresis_curve: Curve,
    pub gpu_fan_curve: Curve,
    pub gpu_temp_curve: Curve,
    pub gpu_hysteresis_curve: Curve,
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
pub struct Curve {
    pub addr: Addr,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Curve6 {
    pub n1: u8,
    pub n2: u8,
    pub n3: u8,
    pub n4: u8,
    pub n5: u8,
    pub n6: u8,
}

impl From<[u8; 6]> for Curve6 {
    fn from(value: [u8; 6]) -> Self {
        let [n1, n2, n3, n4, n5, n6] = value;

        Curve6 {
            n1,
            n2,
            n3,
            n4,
            n5,
            n6,
        }
    }
}

impl From<Curve6> for [u8; 6] {
    #[rustfmt::skip]
    fn from(value: Curve6) -> Self {
        [
            value.n1,
            value.n2,
            value.n3,
            value.n4,
            value.n5,
            value.n6,
        ]
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Curve7 {
    pub n1: u8,
    pub n2: u8,
    pub n3: u8,
    pub n4: u8,
    pub n5: u8,
    pub n6: u8,
    pub n7: u8,
}

impl From<[u8; 7]> for Curve7 {
    fn from(value: [u8; 7]) -> Self {
        let [n1, n2, n3, n4, n5, n6, n7] = value;

        Curve7 {
            n1,
            n2,
            n3,
            n4,
            n5,
            n6,
            n7,
        }
    }
}

impl From<Curve7> for [u8; 7] {
    #[rustfmt::skip]
    fn from(value: Curve7) -> Self {
        [
            value.n1,
            value.n2,
            value.n3,
            value.n4,
            value.n5,
            value.n6,
            value.n7,
        ]
    }
}

//
// Misc
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bit {
    _0 = 0b00000001, // 1
    _1 = 0b00000010, // 2
    _2 = 0b00000100, // 4
    _3 = 0b00001000, // 8
    _4 = 0b00010000, // 16
    _5 = 0b00100000, // 32
    _6 = 0b01000000, // 64
    _7 = 0b10000000, // 128
}

pub trait BitSet {
    fn set_bit_state(&mut self, bit: Bit, state: bool);
    fn set_bit(&mut self, bit: Bit);
    fn unset_bit(&mut self, bit: Bit);
    fn bit_set(self, bit: Bit) -> bool;
}

impl BitXor<Bit> for u8 {
    type Output = Self;

    fn bitxor(self, rhs: Bit) -> Self::Output {
        self ^ rhs as u8
    }
}

impl BitOrAssign<Bit> for u8 {
    fn bitor_assign(&mut self, rhs: Bit) {
        *self |= rhs as u8;
    }
}

impl Not for Bit {
    type Output = u8;

    fn not(self) -> Self::Output {
        !(self as u8)
    }
}

impl BitAnd<Bit> for u8 {
    type Output = Self;

    fn bitand(self, rhs: Bit) -> Self::Output {
        self & rhs as u8
    }
}

impl BitSet for u8 {
    fn set_bit_state(&mut self, bit: Bit, state: bool) {
        if state {
            *self |= bit; // 1 << n
        } else {
            *self &= !bit; // 1 << n
        }
    }

    fn set_bit(&mut self, bit: Bit) {
        self.set_bit_state(bit, true);
    }

    fn unset_bit(&mut self, bit: Bit) {
        self.set_bit_state(bit, false);
    }

    fn bit_set(self, bit: Bit) -> bool {
        (self & bit) != 0
    }
}

//
// Battery
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SuperBattery {
    Off,
    On,
}

impl SuperBattery {
    pub fn enabled(&self) -> bool {
        matches!(self, Self::On)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Threshold(u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BatteryMode {
    /// 50-60
    Healthy,
    /// 70-80
    Balanced,
    /// 100
    Mobility,
    /// Custom end threshold. Charges between N-10..=N
    /// Valid values: 10..=100.
    ///
    /// This variant is non-constructable for safety reasons.
    /// Use from_start and from_end to safely create a value.
    ///
    /// Note: 100 means 100..=100
    Custom(Threshold),
}

impl BatteryMode {
    /// The start threshold. Charges between N..=N+10
    /// Valid values: 0..=90
    ///
    /// Note: 90 means total charge of 100..=100
    pub fn from_start(val: u8) -> Option<Self> {
        let this = match val {
            50 => Self::Healthy,
            70 => Self::Balanced,
            90 => Self::Mobility,
            0..=90 => Self::Custom(Threshold(val + 10)),
            _ => return None,
        };

        Some(this)
    }

    /// The end threshold. Charges between N-10..=N
    /// Valid values: 10..=100
    ///
    /// Note: 100 means 100..=100
    pub fn from_end(val: u8) -> Option<Self> {
        let this = match val {
            60 => Self::Healthy,
            80 => Self::Balanced,
            100 => Self::Mobility,
            10..=100 => Self::Custom(Threshold(val)),
            _ => return None,
        };

        Some(this)
    }

    /// Note: 90 means total charge of 100..=100
    pub fn as_start(&self) -> u8 {
        match *self {
            Self::Healthy => 50,
            Self::Balanced => 70,
            Self::Mobility => 90,
            Self::Custom(Threshold(t)) => t - 10,
        }
    }

    /// Note: 100 means 100..=100
    pub fn as_end(&self) -> u8 {
        match *self {
            Self::Healthy => 60,
            Self::Balanced => 80,
            Self::Mobility => 100,
            Self::Custom(Threshold(t)) => t,
        }
    }
}

//
// Webcam
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Webcam {
    On,
    Off,
}

impl Webcam {
    pub fn enabled(&self) -> bool {
        matches!(self, Self::On)
    }
}

//
// Cooler Boost
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CoolerBoost {
    On,
    Off,
}

impl CoolerBoost {
    pub fn enabled(&self) -> bool {
        matches!(self, Self::On)
    }
}

//
// Fn Win Swap
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyDirection {
    Left,
    Right,
}

//
// LEDs
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Led {
    On,
    Off,
}

//
// Format
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WmiVer {
    Wmi1,
    Wmi2,
}
