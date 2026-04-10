mod ec_drv;
mod ec_sys;

use nix::libc::geteuid;
use snafu::prelude::*;

use crate::{
    ec::ec_sys::{EcSys, EcSysError},
    fw::{
        BatteryMode, Bit, BitSet, CoolerBoost, Curve6, Curve7, FW_INFO, FW_REGISTRY, FanMode,
        FwConfig, KeyDirection, Led, ShiftMode, SuperBattery, Threshold, Webcam, WmiVer,
    },
    models::{Fan, MODEL_REGISTRY, ModelConfig},
};

macro_rules! addr {
    ($name:literal, $addr:expr) => {{
        if let Some(addr) = $addr.get() {
            addr
        } else {
            return Err(EcError::Unsupported {
                name: $name.to_owned(),
            });
        }
    }};
}

type Result<T> = std::result::Result<T, EcError>;

#[derive(Debug, Snafu)]
pub enum EcError {
    #[snafu(display("ec requires root privileges"))]
    RootRequired,
    #[snafu(display("{name}() is unsupported"))]
    Unsupported { name: String },
    #[snafu(transparent)]
    EcSys { source: EcSysError },
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

pub struct Ec {
    sys: Option<(EcSys, FwConfig)>,
    model: Option<ModelConfig>,
    // TODO: ec drv
}

impl Ec {
    pub fn new() -> Result<Self> {
        if unsafe { geteuid() } != 0 {
            log::error!("ec requires root privileges; please run program as root");
            return Err(EcError::RootRequired);
        }

        let sys = match EcSys::new() {
            Ok(io) => 'val: {
                let Some(version) = Self::_fw_version(&io).ok() else {
                    log::warn!("fw not found");
                    break 'val None;
                };

                if let Some(fw) = FW_REGISTRY.get(&version) {
                    Some((io, fw))
                } else {
                    log::warn!("fw {version} is unsupported by ec_sys");
                    None
                }
            }

            Err(source) => match source {
                // It is loaded, but there was an issue
                EcSysError::NoWriteSupport
                | EcSysError::OtherIo { .. }
                | EcSysError::Whatever { .. }
                | EcSysError::OtherErrno { .. } => return Err(source.into()),

                // No load = Reduced functionality mode
                EcSysError::NotLoaded => None,
            },
        };

        let model = MODEL_REGISTRY.find();

        // TODO: ec drv

        let this = Self { sys, model };

        Ok(this)
    }

    pub fn fan_count(&self) -> Fan {
        self.model.map(|m| m.fans).unwrap_or(Fan::One)
    }

    pub fn has_dgpu(&self) -> bool {
        self.model.map(|m| m.has_dgpu).unwrap_or(false)
    }

    pub fn wmi_ver(&self) -> Option<WmiVer> {
        self.sys.as_ref().map(|(_, fw)| fw.ver)
    }

    //
    // Firmware
    //

    fn _fw_version(io: &EcSys) -> Result<String> {
        let mut buf = [0; FW_INFO.version.len];
        io.ec_read_seq(FW_INFO.version.addr, &mut buf)?;
        str::from_utf8(&buf)
            .whatever_context("fw_version() received non utf8 data")
            .map(ToOwned::to_owned)
    }

    pub fn fw_version(&self) -> Result<String> {
        if let Some((io, _)) = self.sys.as_ref() {
            Self::_fw_version(io)
        } else {
            Err(EcError::Unsupported {
                name: "fw_version".to_owned(),
            })
        }
    }

    pub fn fw_date(&self) -> Result<String> {
        if let Some((io, _)) = self.sys.as_ref() {
            let mut buf = [0; FW_INFO.date.len];
            io.ec_read_seq(FW_INFO.date.addr, &mut buf)
                .whatever_context::<_, EcError>("fw_date() failed to ec_read_seq()")?;
            let s = str::from_utf8(&buf)
                .whatever_context::<_, EcError>("fw_date() received non utf8 data")?;

            Ok(s.to_owned())
        } else {
            Err(EcError::Unsupported {
                name: "fw_date".to_owned(),
            })
        }
    }

    pub fn fw_time(&self) -> Result<String> {
        if let Some((io, _)) = self.sys.as_ref() {
            let mut buf = [0; FW_INFO.time.len];
            io.ec_read_seq(FW_INFO.time.addr, &mut buf)
                .whatever_context::<_, EcError>("fw_time() failed to ec_read_seq()")?;
            let s = str::from_utf8(&buf)
                .whatever_context::<_, EcError>("fw_time() received non utf8 data")?;

            Ok(s.to_owned())
        } else {
            Err(EcError::Unsupported {
                name: "fw_time".to_owned(),
            })
        }
    }

    //
    // Shift Mode
    //

    /// Supported shift modes
    pub fn shift_modes(&self) -> Result<Vec<ShiftMode>> {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            Ok(fw.shift_mode.get_modes())
        } else {
            Err(EcError::Unsupported {
                name: "shift_modes".to_owned(),
            })
        }
    }

    /// The current shift mode
    pub fn shift_mode(&self) -> Result<ShiftMode> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("shift_mode", fw.shift_mode.addr);

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("shift_mode() failed to ec_read()")?;

            for (mode, cur_val) in fw.shift_mode.modes {
                if val == *cur_val {
                    return Ok(*mode);
                }
            }

            whatever!("got 0x{val:0>2X}, but it does not represent valid shift mode");
        } else {
            Err(EcError::Unsupported {
                name: "shift_mode".to_owned(),
            })
        }
    }

    /// Set shift mode
    ///
    /// Make sure you are setting a supported shift mode according to shift_modes()
    /// Null is not a valid mode
    pub fn set_shift_mode(&mut self, mode: ShiftMode) -> Result<()> {
        if matches!(mode, ShiftMode::Null) {
            whatever!("shift mode cannot be null");
        }

        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_shift_mode", fw.shift_mode.addr);

            let val = fw
                .shift_mode
                .modes
                .iter()
                .find(|(m, _)| *m == mode)
                .map(|(_, v)| *v);

            let Some(val) = val else {
                whatever!("{mode:?} mode is not supported");
            };

            unsafe {
                io.ec_write(addr, val)
                    .whatever_context::<_, EcError>("set_shift_mode() failed to ec_write()")?
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_shift_mode".to_owned(),
            })
        }
    }

    pub fn shift_mode_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.shift_mode.addr.is_supported()
        } else {
            false
        }
    }

    //
    // Battery
    //

    /// The battery end threshold. Is set in 10% increments, with the returned mode
    /// being the high end. For examaple, if this returns 60%, the threshold
    /// is 50-60%. We return 80%, which means threashold is 70-80%. 10% means
    /// 0-10% charge. However, when when 100%, it's 100% always.
    pub fn battery_mode(&self) -> Result<BatteryMode> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("battery_mode", fw.charge_control_addr);

            let mut val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("battery_mode() failed to ec_read()")?;

            // pre-validate before doing bit ops
            if !matches!(val, 0x8A..=0xE4) {
                whatever!("got invalid value: 0x{val:0>2X} ({val})");
            }

            val.unset_bit(Bit::_7);

            let mode = match val {
                60 => BatteryMode::Healthy,
                80 => BatteryMode::Balanced,
                100 => BatteryMode::Mobility,
                c @ 10..=100 => BatteryMode::Custom(Threshold::from_end(c).unwrap()),
                _ => unreachable!(),
            };

            Ok(mode)
        } else {
            Err(EcError::Unsupported {
                name: "battery_mode".to_owned(),
            })
        }
    }

    pub fn set_battery_mode(&mut self, mode: BatteryMode) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_battery_mode", fw.charge_control_addr);

            let mut val = mode.as_end();

            // just for a sanity check
            assert!(
                matches!(val, 10..=100),
                "{val} is not within the allowed end threshold limit 10..=100"
            );

            val.set_bit(Bit::_7);

            // another sanity check
            assert!(matches!(val, 0x8A..=0xE4));

            // SAFETY: assert guarantees only valid values are written
            //         also uses charge control address given from config
            unsafe {
                io.ec_write(addr, val)
                    .whatever_context::<_, EcError>("set_battery_mode() failed to ec_write()")?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_battery_mode".to_owned(),
            })
        }
    }

    pub fn battery_mode_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.charge_control_addr.is_supported()
        } else {
            false
        }
    }

    pub fn super_battery(&self) -> Result<SuperBattery> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("super_battery", fw.super_battery.addr);

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("super_battery() failed to ec_read()")?;

            let val = (val & fw.super_battery.mask) == fw.super_battery.mask;

            let kind = match val {
                true => SuperBattery::On,
                false => SuperBattery::Off,
            };

            Ok(kind)
        } else {
            Err(EcError::Unsupported {
                name: "super_battery".to_owned(),
            })
        }
    }

    pub fn set_super_battery(&mut self, kind: SuperBattery) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_super_battery", fw.super_battery.addr);

            let raw = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("set_super_battery() failed to ec_read()")?;

            let val = match kind {
                SuperBattery::Off => raw & !fw.super_battery.mask,
                SuperBattery::On => raw | fw.super_battery.mask,
            };

            unsafe {
                io.ec_write(addr, val)
                    .whatever_context::<_, EcError>("set_super_battery() failed to ec_write()")?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_super_battery".to_owned(),
            })
        }
    }

    pub fn super_battery_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.super_battery.addr.is_supported()
        } else {
            false
        }
    }

    //
    // Fan RPM
    //

    fn fan_rpm(&self, fan: Fan) -> Result<u16> {
        let supported = match fan {
            Fan::One => self.fan1_supported(),
            Fan::Two => self.fan2_supported(),
            Fan::Three => self.fan3_supported(),
            Fan::Four => self.fan4_supported(),
        };

        if !supported {
            return Err(EcError::Unsupported {
                name: format!("{fan:?}"),
            });
        }

        let Some((io, fw)) = self.sys.as_ref() else {
            return Err(EcError::Unsupported {
                name: format!("{fan:?}"),
            });
        };

        let addr = match fan {
            Fan::One => fw.fan_rpm.fan1_addr,
            Fan::Two => fw.fan_rpm.fan2_addr,
            Fan::Three => fw.fan_rpm.fan3_addr,
            Fan::Four => fw.fan_rpm.fan4_addr,
        };

        let mut rpm = [0; 2];

        io.ec_read_seq(addr, &mut rpm)
            .whatever_context::<_, EcError>("fan_rpm() failed to ec_read()")?;

        let raw = u16::from_be_bytes(rpm);
        let rpm = 480000u32.checked_div(raw as u32).unwrap_or(0);

        Ok(rpm as u16)
    }

    pub fn fan1_rpm(&self) -> Result<u16> {
        self.fan_rpm(Fan::One)
    }

    pub fn fan2_rpm(&self) -> Result<u16> {
        self.fan_rpm(Fan::Two)
    }

    pub fn fan3_rpm(&self) -> Result<u16> {
        self.fan_rpm(Fan::Three)
    }

    pub fn fan4_rpm(&self) -> Result<u16> {
        self.fan_rpm(Fan::Four)
    }

    fn fan_supported(&self, fan: Fan) -> bool {
        self.fan_count() >= fan
    }

    pub fn fan1_supported(&self) -> bool {
        self.fan_supported(Fan::One)
    }

    pub fn fan2_supported(&self) -> bool {
        self.fan_supported(Fan::Two)
    }

    pub fn fan3_supported(&self) -> bool {
        self.fan_supported(Fan::Three)
    }

    pub fn fan4_supported(&self) -> bool {
        self.fan_supported(Fan::Four)
    }

    //
    // Fan Modes
    //

    /// Supported fan modes
    pub fn fan_modes(&self) -> Result<Vec<FanMode>> {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            Ok(fw.fan_mode.get_modes())
        } else {
            Err(EcError::Unsupported {
                name: "fan_modes".to_owned(),
            })
        }
    }

    pub fn fan_mode(&self) -> Result<FanMode> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("fan_mode", fw.fan_mode.addr);

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("fan_mode() failed to ec_read()")?;

            let mode = fw
                .fan_mode
                .modes
                .iter()
                .find(|(_, v)| val == *v)
                .map(|&(mode, _)| mode);

            let Some(mode) = mode else {
                whatever!("got invalid value: 0x{val:0>2X} ({val})");
            };

            Ok(mode)
        } else {
            Err(EcError::Unsupported {
                name: "fan_mode".to_owned(),
            })
        }
    }

    pub fn set_fan_mode(&mut self, mode: FanMode) -> Result<()> {
        if matches!(mode, FanMode::Null) {
            whatever!("fan mode cannot be null");
        }

        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_fan_mode", fw.fan_mode.addr);

            let val = fw
                .fan_mode
                .modes
                .iter()
                .find(|(m, _)| mode == *m)
                .map(|&(_, v)| v);

            let Some(val) = val else {
                whatever!("{mode:?} mode is not supported");
            };

            unsafe {
                io.ec_write(addr, val)
                    .whatever_context::<_, EcError>("set_fan_mode() failed to ec_write()")?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_fan_mode".to_owned(),
            })
        }
    }

    pub fn fan_mode_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.fan_mode.addr.is_supported()
        } else {
            false
        }
    }

    //
    // Webcam
    //

    pub fn webcam(&self) -> Result<Webcam> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("webcam", fw.webcam.addr);

            let set = {
                let v = io
                    .ec_read_bit(addr, fw.webcam.bit)
                    .whatever_context::<_, EcError>("webcam() failed to ec_read_bit()")?;

                v ^ false
            };

            let webcam = match set {
                true => Webcam::On,
                false => Webcam::Off,
            };

            Ok(webcam)
        } else {
            Err(EcError::Unsupported {
                name: "webcam".to_owned(),
            })
        }
    }

    pub fn webcam_block(&self) -> Result<Webcam> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("webcam_block", fw.webcam.block_addr);

            let set = {
                let v = io
                    .ec_read_bit(addr, fw.webcam.bit)
                    .whatever_context::<_, EcError>("webcam_block() failed to ec_read_bit()")?;

                v ^ true
            };

            let webcam = match set {
                true => Webcam::On,
                false => Webcam::Off,
            };

            Ok(webcam)
        } else {
            Err(EcError::Unsupported {
                name: "webcam_block".to_owned(),
            })
        }
    }

    pub fn set_webcam(&mut self, state: Webcam) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_webcam", fw.webcam.addr);

            let val = state.enabled() ^ false;

            unsafe {
                io.ec_write_bit(addr, fw.webcam.bit, val)
                    .whatever_context::<_, EcError>("set_webcam() failed to ec_write_bit()")?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_webcam".to_owned(),
            })
        }
    }

    pub fn set_webcam_block(&mut self, state: Webcam) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_webcam_block", fw.webcam.block_addr);

            let val = state.enabled() ^ true;

            unsafe {
                io.ec_write_bit(addr, fw.webcam.bit, val)
                    .whatever_context::<_, EcError>(
                        "set_webcam_block() failed to ec_write_bit()",
                    )?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_webcam_block".to_owned(),
            })
        }
    }

    pub fn webcam_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.webcam.addr.is_supported()
        } else {
            false
        }
    }

    pub fn webcam_block_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.webcam.block_addr.is_supported()
        } else {
            false
        }
    }

    //
    // Cooler Boost
    //

    pub fn cooler_boost(&self) -> Result<CoolerBoost> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("cooler_boost", fw.cooler_boost.addr);

            let set = io
                .ec_read_bit(addr, fw.cooler_boost.bit)
                .whatever_context::<_, EcError>("cooler_boost() failed to ec_read_bit()")?;

            let cooler_boost = match set {
                true => CoolerBoost::On,
                false => CoolerBoost::Off,
            };

            Ok(cooler_boost)
        } else {
            Err(EcError::Unsupported {
                name: "cooler_boost".to_owned(),
            })
        }
    }

    pub fn set_cooler_boost(&mut self, state: CoolerBoost) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_cooler_boost", fw.cooler_boost.addr);

            let set = state.enabled();

            unsafe {
                io.ec_write_bit(addr, fw.cooler_boost.bit, set)
                    .whatever_context::<_, EcError>(
                        "set_cooler_boost() failed to ec_write_bit()",
                    )?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_cooler_boost".to_owned(),
            })
        }
    }

    pub fn cooler_boost_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.cooler_boost.addr.is_supported()
        } else {
            false
        }
    }

    //
    // Fn Win Key Swap
    //

    pub fn fn_key(&self) -> Result<KeyDirection> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("fn_key", fw.fn_win_swap.addr);

            let mut set = io
                .ec_read_bit(addr, fw.fn_win_swap.bit)
                .whatever_context::<_, EcError>("fn_key() failed to ec_read_bit()")?;

            set ^= fw.fn_win_swap.invert; // invert the direction for some laptops
            set = !set; // fn key position is the opposite of win key

            let fn_win_swap = match set {
                true => KeyDirection::Left,
                false => KeyDirection::Right,
            };

            Ok(fn_win_swap)
        } else {
            Err(EcError::Unsupported {
                name: "fn_key".to_owned(),
            })
        }
    }

    pub fn set_fn_key(&mut self, dir: KeyDirection) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_fn_key", fw.fn_win_swap.addr);

            let mut val = matches!(dir, KeyDirection::Left);
            val ^= fw.fn_win_swap.invert; // invert the direction for some laptops
            val = !val; // fn key position is the opposite of win key

            unsafe {
                io.ec_write_bit(addr, fw.fn_win_swap.bit, val)
                    .whatever_context::<_, EcError>("set_fn_key() failed to ec_write_bit()")?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_fn_key".to_owned(),
            })
        }
    }

    pub fn fn_win_swap_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.fn_win_swap.addr.is_supported()
        } else {
            false
        }
    }

    pub fn win_key(&self) -> Result<KeyDirection> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("win_key", fw.fn_win_swap.addr);

            let mut val = io
                .ec_read_bit(addr, fw.fn_win_swap.bit)
                .whatever_context::<_, EcError>("win_key() failed to ec_read_bit()")?;

            val ^= fw.fn_win_swap.invert; // invert the direction for some laptops

            let fn_win_swap = match val {
                true => KeyDirection::Left,
                false => KeyDirection::Right,
            };

            Ok(fn_win_swap)
        } else {
            Err(EcError::Unsupported {
                name: "win_key".to_owned(),
            })
        }
    }

    pub fn set_win_key(&mut self, dir: KeyDirection) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_win_key", fw.fn_win_swap.addr);

            let mut val = matches!(dir, KeyDirection::Left);
            val ^= fw.fn_win_swap.invert; // invert the direction for some laptops

            unsafe {
                io.ec_write_bit(addr, fw.fn_win_swap.bit, val)
                    .whatever_context::<_, EcError>("set_win_key() failed to ec_write_bit()")?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_win_key".to_owned(),
            })
        }
    }

    //
    // LEDs
    //

    pub fn mic_mute_led(&self) -> Result<Led> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("mic_mute_led", fw.leds.mic_mute_led_addr);

            let val = io
                .ec_read_bit(addr, fw.leds.bit)
                .whatever_context::<_, EcError>("mic_mute_led() failed to ec_read_bit()")?;

            let led = match val {
                true => Led::On,
                false => Led::Off,
            };

            Ok(led)
        } else {
            Err(EcError::Unsupported {
                name: "mic_mute_led".to_owned(),
            })
        }
    }

    pub fn mute_led(&self) -> Result<Led> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("mute_led", fw.leds.mute_led_addr);

            let val = io
                .ec_read_bit(addr, fw.leds.bit)
                .whatever_context::<_, EcError>("mute_led() failed to ec_read_bit()")?;

            let led = match val {
                true => Led::On,
                false => Led::Off,
            };

            Ok(led)
        } else {
            Err(EcError::Unsupported {
                name: "mute_led".to_owned(),
            })
        }
    }

    pub fn set_mic_mute_led(&mut self, state: Led) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_mic_mute_led", fw.leds.mic_mute_led_addr);

            let state = match state {
                Led::On => true,
                Led::Off => false,
            };

            unsafe {
                io.ec_write_bit(addr, fw.leds.bit, state)
                    .whatever_context::<_, EcError>(
                        "set_mic_mute_led() failed to ec_write_bit()",
                    )?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_mic_mute_led".to_owned(),
            })
        }
    }

    pub fn set_mute_led(&mut self, state: Led) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_mute_led", fw.leds.mute_led_addr);

            let state = match state {
                Led::On => true,
                Led::Off => false,
            };

            unsafe {
                io.ec_write_bit(addr, fw.leds.bit, state)
                    .whatever_context::<_, EcError>("set_mute_led() failed to ec_write_bit()")?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "set_mute_led".to_owned(),
            })
        }
    }

    pub fn mic_mute_led_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.leds.mic_mute_led_addr.is_supported()
        } else {
            false
        }
    }

    pub fn mute_led_supported(&self) -> bool {
        if false {
            todo!("ec drv");
        } else if let Some((_, fw)) = self.sys.as_ref() {
            fw.leds.mute_led_addr.is_supported()
        } else {
            false
        }
    }

    //
    // Thermal
    //

    pub fn cpu_rt_fan_speed(&self) -> Result<u8> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("cpu_rt_fan_speed", fw.cpu.rt_fan_speed_addr);

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("cpu_rt_fan_speed() failed to ec_read()")?;

            Ok(val)
        } else {
            Err(EcError::Unsupported {
                name: "cpu_rt_fan_speed".to_owned(),
            })
        }
    }

    pub fn cpu_rt_temp(&self) -> Result<u8> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("cpu_rt_temp", fw.cpu.rt_temp_addr);

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("cpu_rt_temp() failed to ec_read()")?;

            Ok(val)
        } else {
            Err(EcError::Unsupported {
                name: "cpu_rt_temp".to_owned(),
            })
        }
    }

    pub fn gpu_rt_fan_speed(&self) -> Result<u8> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            // no dgpu check, because it's hardcoded into fw

            let addr = addr!("gpu_rt_fan_speed", fw.gpu.rt_fan_speed_addr);

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("gpu_rt_fan_speed() failed to ec_read()")?;

            Ok(val)
        } else {
            Err(EcError::Unsupported {
                name: "gpu_rt_fan_speed".to_owned(),
            })
        }
    }

    pub fn gpu_rt_temp(&self) -> Result<u8> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            // no dgpu check, because it's hardcoded into fw

            let addr = addr!("gpu_rt_temp", fw.gpu.rt_temp_addr);

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("gpu_rt_temp() failed to ec_read()")?;

            Ok(val)
        } else {
            Err(EcError::Unsupported {
                name: "gpu_rt_temp".to_owned(),
            })
        }
    }

    //
    // Fan curves ; only for WMI2
    //

    fn read_curve<const N: usize, T: From<[u8; N]>>(addr: u8, io: &EcSys) -> Result<T> {
        let mut buf = [0; N];
        io.ec_read_seq(addr, &mut buf)
            .whatever_context::<_, EcError>("read_curve() failed to ec_read_seq()")?;

        Ok(T::from(buf))
    }

    unsafe fn set_curve<const N: usize, T: Into<[u8; N]>>(
        io: &mut EcSys,
        addr: u8,
        curve: T,
    ) -> Result<()> {
        let buf = curve.into();
        unsafe {
            io.ec_write_seq(addr, &buf)
                .whatever_context::<_, EcError>("set_curve() failed to ec_write_seq()")?;
        }

        Ok(())
    }

    pub fn cpu_fan_curve(&self) -> Result<Curve7> {
        if let Some((io, fw)) = self.sys.as_ref() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("cpu_fan_curve", fw.cpu_fan_curve.addr);

            let curve = Self::read_curve(addr, io)?;
            Ok(curve)
        } else {
            Err(EcError::Unsupported {
                name: "cpu_fan_curve".to_owned(),
            })
        }
    }

    pub fn cpu_temp_curve(&self) -> Result<Curve7> {
        if let Some((io, fw)) = self.sys.as_ref() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("cpu_temp_curve", fw.cpu_temp_curve.addr);

            let curve = Self::read_curve(addr, io)?;
            Ok(curve)
        } else {
            Err(EcError::Unsupported {
                name: "cpu_temp_curve".to_owned(),
            })
        }
    }

    pub fn cpu_hysteresis_curve(&self) -> Result<Curve6> {
        if let Some((io, fw)) = self.sys.as_ref() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("cpu_hysteresis_curve", fw.cpu_hysteresis_curve.addr);

            let curve = Self::read_curve(addr, io)?;
            Ok(curve)
        } else {
            Err(EcError::Unsupported {
                name: "cpu_hysteresis_curve".to_owned(),
            })
        }
    }

    pub fn gpu_fan_curve(&self) -> Result<Curve6> {
        if !self.has_dgpu() {
            whatever!("no dgpu available");
        }

        if let Some((io, fw)) = self.sys.as_ref() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("gpu_fan_curve", fw.gpu_fan_curve.addr);

            let curve = Self::read_curve(addr, io)?;
            Ok(curve)
        } else {
            Err(EcError::Unsupported {
                name: "gpu_fan_curve".to_owned(),
            })
        }
    }

    pub fn gpu_temp_curve(&self) -> Result<Curve7> {
        if !self.has_dgpu() {
            whatever!("no dgpu available");
        }

        if let Some((io, fw)) = self.sys.as_ref() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("gpu_temp_curve", fw.gpu_temp_curve.addr);

            let curve = Self::read_curve(addr, io)?;
            Ok(curve)
        } else {
            Err(EcError::Unsupported {
                name: "gpu_temp_curve".to_owned(),
            })
        }
    }

    pub fn gpu_hysteresis_curve(&self) -> Result<Curve6> {
        if !self.has_dgpu() {
            whatever!("no dgpu available");
        }

        if let Some((io, fw)) = self.sys.as_ref() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("gpu_hysteresis_curve", fw.gpu_hysteresis_curve.addr);

            let curve = Self::read_curve(addr, io)?;
            Ok(curve)
        } else {
            Err(EcError::Unsupported {
                name: "gpu_hysteresis_curve".to_owned(),
            })
        }
    }

    pub fn set_cpu_fan_curve(&mut self, curve: Curve7) -> Result<()> {
        if let Some((io, fw)) = self.sys.as_mut() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("cpu_fan_curve", fw.cpu_fan_curve.addr);

            unsafe {
                Self::set_curve(io, addr, curve)?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "cpu_fan_curve".to_owned(),
            })
        }
    }

    pub fn set_cpu_temp_curve(&mut self, curve: Curve7) -> Result<()> {
        if let Some((io, fw)) = self.sys.as_mut() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("cpu_temp_curve", fw.cpu_temp_curve.addr);

            unsafe {
                Self::set_curve(io, addr, curve)?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "cpu_temp_curve".to_owned(),
            })
        }
    }

    pub fn set_cpu_hysteresis_curve(&mut self, curve: Curve6) -> Result<()> {
        if let Some((io, fw)) = self.sys.as_mut() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("cpu_hysteresis_curve", fw.cpu_hysteresis_curve.addr);

            unsafe {
                Self::set_curve(io, addr, curve)?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "cpu_hysteresis_curve".to_owned(),
            })
        }
    }

    pub fn set_gpu_fan_curve(&mut self, curve: Curve6) -> Result<()> {
        if !self.has_dgpu() {
            whatever!("no dgpu available");
        }

        if let Some((io, fw)) = self.sys.as_mut() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("gpu_fan_curve", fw.gpu_fan_curve.addr);

            unsafe {
                Self::set_curve(io, addr, curve)?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "gpu_fan_curve".to_owned(),
            })
        }
    }

    pub fn set_gpu_temp_curve(&mut self, curve: Curve7) -> Result<()> {
        if !self.has_dgpu() {
            whatever!("no dgpu available");
        }

        if let Some((io, fw)) = self.sys.as_mut() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("gpu_temp_curve", fw.gpu_temp_curve.addr);

            unsafe {
                Self::set_curve(io, addr, curve)?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "gpu_temp_curve".to_owned(),
            })
        }
    }

    pub fn set_gpu_hysteresis_curve(&mut self, curve: Curve6) -> Result<()> {
        if !self.has_dgpu() {
            whatever!("no dgpu available");
        }

        if let Some((io, fw)) = self.sys.as_mut() {
            if !matches!(fw.ver, WmiVer::Wmi2) {
                whatever!("only wmi2 is supported");
            }

            let addr = addr!("gpu_hysteresis_curve", fw.gpu_hysteresis_curve.addr);

            unsafe {
                Self::set_curve(io, addr, curve)?;
            }

            Ok(())
        } else {
            Err(EcError::Unsupported {
                name: "gpu_hysteresis_curve".to_owned(),
            })
        }
    }

    //
    // Utils
    //

    // Raw ec dump
    pub fn ec_dump_raw(&self) -> Result<[u8; 256]> {
        if let Some((io, _)) = self.sys.as_ref() {
            let mut dump = [0; 256];
            io.ec_read_seq(0x00, &mut dump)
                .whatever_context::<_, EcError>("ec_dump_raw() failed to ec_read_seq()")?;
            Ok(dump)
        } else {
            Err(EcError::Unsupported {
                name: "ec_dump_raw".to_owned(),
            })
        }
    }

    // Pretty formatted ec dump
    pub fn ec_dump_pretty(&self) -> Result<String> {
        const HEADER: &str = "|      | _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _A _B _C _D _E _F\n\
            |------+------------------------------------------------\n";

        let mut buf = HEADER.to_string();

        let dump = self.ec_dump_raw()?;
        let mut ascii_buf = ['\0'; 16];

        let mut dump_idx = 0;
        for h in 0..0x10 {
            buf.push_str(&format!("| 0x{h:X}_ |"));

            for (i, byte) in dump.into_iter().skip(dump_idx).take(0x10).enumerate() {
                buf.push_str(&format!(" {byte:0>2X}"));
                ascii_buf[i] = match byte {
                    0x20..=0x7E => byte as char,
                    _ => '.',
                };

                dump_idx += 1;
            }

            buf.push_str(" |");
            buf.extend(ascii_buf);
            buf.push_str("|\n");
        }

        Ok(buf)
    }
}
