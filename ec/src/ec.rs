mod ec_drv;
mod ec_sys;

use nix::libc::geteuid;
use snafu::prelude::*;

use crate::{
    ec::ec_sys::{EcSys, EcSysError},
    fw::{
        BatteryMode, FW_INFO, FW_REGISTRY, FwConfig, ShiftModeKind, SuperBattery, SuperBatteryKind,
    },
    models::{FanCount, MODEL_REGISTRY, ModelConfig},
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

        let model = MODEL_REGISTRY.get();

        // TODO: ec drv

        let this = Self { sys, model };

        Ok(this)
    }

    fn fan_count(&self) -> FanCount {
        self.model.map(|m| m.fan_count).unwrap_or(FanCount::One)
    }

    fn has_dgpu(&self) -> bool {
        self.model.map(|m| m.has_dgpu).unwrap_or(false)
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
        let Some((io, _)) = self.sys.as_ref() else {
            return Err(EcError::Unsupported {
                name: "fw_version".to_owned(),
            });
        };

        Self::_fw_version(&io)
    }

    pub fn fw_date(&self) -> Result<String> {
        let Some((io, _)) = self.sys.as_ref() else {
            return Err(EcError::Unsupported {
                name: "fw_date".to_owned(),
            });
        };

        let mut buf = [0; FW_INFO.date.len];
        io.ec_read_seq(FW_INFO.date.addr, &mut buf)
            .whatever_context::<_, EcError>("fw_date() failed to ec_read_seq()")?;
        let s = str::from_utf8(&buf)
            .whatever_context::<_, EcError>("fw_date() received non utf8 data")?;

        Ok(s.to_owned())
    }

    pub fn fw_time(&self) -> Result<String> {
        let Some((io, _)) = self.sys.as_ref() else {
            return Err(EcError::Unsupported {
                name: "fw_time".to_owned(),
            });
        };

        let mut buf = [0; FW_INFO.time.len];
        io.ec_read_seq(FW_INFO.time.addr, &mut buf)
            .whatever_context::<_, EcError>("fw_time() failed to ec_read_seq()")?;
        let s = str::from_utf8(&buf)
            .whatever_context::<_, EcError>("fw_time() received non utf8 data")?;

        Ok(s.to_owned())
    }

    //
    // Shift Mode
    //

    /// Supported shift modes
    pub fn shift_modes(&self) -> Result<Vec<ShiftModeKind>> {
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
    pub fn shift_mode(&self) -> Result<ShiftModeKind> {
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
    pub fn set_shift_mode(&mut self, mode: ShiftModeKind) -> Result<()> {
        if matches!(mode, ShiftModeKind::Null) {
            whatever!("shift mode cannot be null");
        }

        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_shift_mode", fw.shift_mode.addr);

            let val = fw
                .shift_mode
                .modes
                .into_iter()
                .find(|(m, _)| *m == mode)
                .map(|(_, v)| *v);

            let Some(val) = val else {
                whatever!("{mode:?} is not supported");
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

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("battery_mode() failed to ec_read()")?;

            // pre-validate before doing bit ops
            if !matches!(val, 0x8A..=0xE4) {
                whatever!("got 0x{val:0>2X} ({val}), but it does not represent valid battery mode");
            }

            let mode = match 0x7F & val {
                60 => BatteryMode::Healthy,
                80 => BatteryMode::Balanced,
                100 => BatteryMode::Mobility,
                c @ 10..=100 => BatteryMode::Custom(c),
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

            let val = match mode {
                BatteryMode::Healthy => 60,
                BatteryMode::Balanced => 80,
                BatteryMode::Mobility => 100,
                BatteryMode::Custom(c) => c,
            };

            let val = val | 1 << 7;

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

    pub fn super_battery(&self) -> Result<SuperBatteryKind> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_ref() {
            let addr = addr!("super_battery", fw.super_battery.addr);

            let val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("super_battery() failed to ec_read()")?;

            let val = val & fw.super_battery.mask == fw.super_battery.mask;

            let kind = match val {
                true => SuperBatteryKind::On,
                false => SuperBatteryKind::Off,
            };

            Ok(kind)
        } else {
            Err(EcError::Unsupported {
                name: "super_battery".to_owned(),
            })
        }
    }

    pub fn set_super_battery(&mut self, kind: SuperBatteryKind) -> Result<()> {
        if false {
            todo!("ec drv");
        } else if let Some((io, fw)) = self.sys.as_mut() {
            let addr = addr!("set_super_battery", fw.super_battery.addr);

            let mut val = io
                .ec_read(addr)
                .whatever_context::<_, EcError>("set_super_battery() failed to ec_read()")?;

            match kind {
                SuperBatteryKind::Off => val &= !fw.super_battery.mask,
                SuperBatteryKind::On => val |= fw.super_battery.mask,
            }

            // SAFETY: assert guarantees only valid values are written
            //         also uses charge control address given from config
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

    // //
    // // Fan RPM
    // //

    // fn fan_rpm(&self, addr: u8) -> Result<u16> {
    //     let mut rpm = [0; 2];

    //     self.io
    //         .ec_read_seq(addr, &mut rpm)
    //         .context("fan_rpm() failed to ec_read()")?;

    //     let rpm = u16::from_be_bytes(rpm).to_rpm();

    //     Ok(rpm)
    // }

    // pub fn fan1_rpm(&self) -> Result<u16> {
    //     self.fan_rpm(FAN1_RPM)
    // }

    // pub fn fan2_rpm(&self) -> Result<u16> {
    //     self.fan_rpm(FAN2_RPM)
    // }

    // pub fn fan3_rpm(&self) -> Result<u16> {
    //     self.fan_rpm(FAN3_RPM)
    // }

    // pub fn fan4_rpm(&self) -> Result<u16> {
    //     self.fan_rpm(FAN4_RPM)
    // }

    // //
    // // Temps
    // //

    // pub fn cpu_temp(&self) -> Result<u8> {
    //     let val = self
    //         .io
    //         .ec_read(CPU_TEMP)
    //         .context("cpu_temp() failed to ec_read()")?;

    //     Ok(val)
    // }

    // pub fn gpu_temp(&self) -> Result<u8> {
    //     let val = self
    //         .io
    //         .ec_read(GPU_TEMP)
    //         .context("gpu_temp() failed to ec_read()")?;

    //     Ok(val)
    // }

    // //
    // // Fan Curves
    // //

    // fn fan_curve(&self, addr: u8) -> Result<u8> {
    //     let val = self
    //         .io
    //         .ec_read(addr)
    //         .context("fan_curve() failed to ec_read()")?;

    //     Ok(val)
    // }

    // fn set_fan_curve(&self, addr: u8, percentage: u8) -> Result<u8> {
    //     if percentage > CPU_FAN_MAX || percentage > GPU_FAN_MAX {
    //         whatever!("fan speed cannot exceed {percentage}%");
    //     }

    //     todo!()
    // }

    // pub fn gpu_fan_curve(&self) -> Result<GpuFanCurve> {
    //     let (p1, p2, p3, p4, p5, p6) = (
    //         self.fan_curve(GPU_FAN_CURVE_1)?,
    //         self.fan_curve(GPU_FAN_CURVE_2)?,
    //         self.fan_curve(GPU_FAN_CURVE_3)?,
    //         self.fan_curve(GPU_FAN_CURVE_4)?,
    //         self.fan_curve(GPU_FAN_CURVE_5)?,
    //         self.fan_curve(GPU_FAN_CURVE_6)?,
    //     );

    //     let gpu = GpuFanCurve::new(p1, p2, p3, p4, p5, p6).context("fan point exceeded max max")?;

    //     Ok(gpu)
    // }

    // pub fn cpu_fan_curve(&self) -> Result<CpuFanCurve> {
    //     let (p1, p2, p3, p4, p5, p6) = (
    //         self.fan_curve(CPU_FAN_CURVE_1)?,
    //         self.fan_curve(CPU_FAN_CURVE_2)?,
    //         self.fan_curve(CPU_FAN_CURVE_3)?,
    //         self.fan_curve(CPU_FAN_CURVE_4)?,
    //         self.fan_curve(CPU_FAN_CURVE_5)?,
    //         self.fan_curve(CPU_FAN_CURVE_6)?,
    //     );

    //     let cpu = CpuFanCurve::new(p1, p2, p3, p4, p5, p6).context("fan point exceeded max max")?;

    //     Ok(cpu)
    // }

    // //
    // // Utils
    // //

    // // Raw ec dump
    // pub fn ec_dump_raw(&self) -> Result<[u8; 256]> {
    //     let mut dump = [0; 256];
    //     self.io
    //         .ec_read_seq(0x00, &mut dump)
    //         .context("ec_dump_raw() failed to ec_read_seq()")?;
    //     Ok(dump)
    // }

    // Pretty formatted ec dump
    // pub fn ec_dump_pretty(&self) -> Result<String> {
    //     const HEADER: &str = "|      | _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _A _B _C _D _E _F\n\
    //         |------+------------------------------------------------\n";

    //     let mut buf = HEADER.to_string();

    //     let dump = self.ec_dump_raw()?;
    //     let mut ascii_buf = ['\0'; 16];

    //     let mut dump_idx = 0;
    //     for h in 0..0x10 {
    //         buf.push_str(&format!("| 0x{h:X}_ |"));

    //         for (i, byte) in dump.into_iter().skip(dump_idx).take(0x10).enumerate() {
    //             buf.push_str(&format!(" {byte:0>2X}"));
    //             ascii_buf[i] = match byte {
    //                 0x20..=0x7E => byte as char,
    //                 _ => '.',
    //             };

    //             dump_idx += 1;
    //         }

    //         buf.push_str(" |");
    //         buf.extend(ascii_buf);
    //         buf.push_str("|\n");
    //     }

    //     Ok(buf)
    // }
}
