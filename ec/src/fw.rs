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
//!
//! This app only plans to support WMI2 devices.

mod g2_10;

use snafu::{Whatever, prelude::*};

/// A registry of supported fw configs.
///
/// Once a config is made (following module docs),
/// add it here to support it.
#[rustfmt::skip]
pub static REGISTRY: Registry<'static> = Registry(&[
    g2_10::G2_10
]);

pub struct Registry<'a>(&'a [FwConfig]);

impl Registry<'_> {
    pub fn get(&self, ec_version: &str) -> Result<FwConfig, Whatever> {
        self.0
            .iter()
            .find(|fw| fw.supports_fw(ec_version))
            .copied()
            .with_whatever_context(|| format!("fw {ec_version} is unsupported"))
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

    pub fn get(&self) -> Result<u8, Whatever> {
        match self {
            Self::Unsupported => whatever!("feature is not supported"),
            Self::Addr(addr) => Ok(*addr),
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
        self.modes.iter().map(|(k, _)| *k).collect()
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum ShiftModeKind {
    ExtremePerformance,
    Balanced,
    SuperBattery,
    Turbo,
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
}

impl FwConfig {
    pub fn supports_fw(&self, ec_version: &str) -> bool {
        self.allowed_fw.contains(&ec_version)
    }

    pub fn supports_charge_control(&self) -> Result<(), Whatever> {
        if !self.charge_control_addr.is_supported() {
            whatever!("charge control is not supported");
        }

        Ok(())
    }

    pub fn supports_webcam(&self) -> Result<(), Whatever> {
        if !self.webcam.addr.is_supported() {
            whatever!("webcam is not supported");
        }

        Ok(())
    }

    pub fn supports_webcam_block(&self) -> Result<(), Whatever> {
        if !self.webcam.block_addr.is_supported() {
            whatever!("webcam block is not supported");
        }

        Ok(())
    }

    pub fn supports_fn_win_swap(&self) -> Result<(), Whatever> {
        if !self.fn_win_swap.addr.is_supported() {
            whatever!("fn win swap is not supported");
        }

        Ok(())
    }

    pub fn supports_cooler_boost(&self) -> Result<(), Whatever> {
        if !self.cooler_boost.addr.is_supported() {
            whatever!("cooler boost is not supported");
        }

        Ok(())
    }

    pub fn supports_shift_mode(&self) -> Result<(), Whatever> {
        if !self.shift_mode.addr.is_supported() {
            whatever!("shift mode is not supported");
        }

        Ok(())
    }

    pub fn supports_super_battery(&self) -> Result<(), Whatever> {
        if !self.super_battery.addr.is_supported() {
            whatever!("super battery is not supported");
        }

        Ok(())
    }

    pub fn supports_fan_mode(&self) -> Result<(), Whatever> {
        if !self.fan_mode.addr.is_supported() {
            whatever!("fan mode is not supported");
        }

        Ok(())
    }

    pub fn supports_cpu_rt_fan_speed(&self) -> Result<(), Whatever> {
        if !self.cpu.rt_fan_speed_addr.is_supported() {
            whatever!("cpu rt fan speed is not supported");
        }

        Ok(())
    }

    pub fn supports_cpu_rt_temp(&self) -> Result<(), Whatever> {
        if !self.cpu.rt_temp_addr.is_supported() {
            whatever!("cpu rt temp is not supported");
        }

        Ok(())
    }

    pub fn supports_gpu_rt_fan_speed(&self) -> Result<(), Whatever> {
        if !self.gpu.rt_fan_speed_addr.is_supported() {
            whatever!("gpu rt fan speed is not supported");
        }

        Ok(())
    }

    pub fn supports_gpu_rt_temp(&self) -> Result<(), Whatever> {
        if !self.gpu.rt_temp_addr.is_supported() {
            whatever!("gpu rt temp is not supported");
        }

        Ok(())
    }

    pub fn supports_micmute_led(&self) -> Result<(), Whatever> {
        if !self.leds.micmute_led_addr.is_supported() {
            whatever!("mic mute led is not supported");
        }

        Ok(())
    }

    pub fn supports_mute_led(&self) -> Result<(), Whatever> {
        if !self.leds.mute_led_addr.is_supported() {
            whatever!("mute led is not supported");
        }

        Ok(())
    }

    pub fn supports_kbd_bl(&self) -> Result<(), Whatever> {
        if !self.kbd_bl.bl_mode_addr.is_supported() {
            whatever!("kbd bl mode is not supported");
        }

        Ok(())
    }

    pub fn supports_kbd_bl_state(&self) -> Result<(), Whatever> {
        if !self.kbd_bl.bl_state_addr.is_supported() {
            whatever!("kbd bl state is not supported");
        }

        Ok(())
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
// Misc
//

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bit {
    _0 = 0b00000001,
    _1 = 0b00000010,
    _2 = 0b00000100,
    _3 = 0b00001000,
    _4 = 0b00010000,
    _5 = 0b00100000,
    _6 = 0b01000000,
    _7 = 0b10000000,
}

pub trait BitSet {
    fn set_bit(&mut self, bit: Bit, state: bool);
    fn is_bit_set(self, bit: Bit) -> bool;
}

impl BitSet for u8 {
    fn set_bit(&mut self, bit: Bit, state: bool) {
        if state {
            *self |= bit as u8;
        } else {
            *self &= !(bit as u8);
        }
    }

    fn is_bit_set(self, bit: Bit) -> bool {
        (self & bit as u8) != 0
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
