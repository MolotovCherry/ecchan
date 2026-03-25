mod ec_drv;
mod ec_sys;

use snafu::prelude::*;

use crate::{
    ec::ec_sys::{EcSys, EcSysError},
    fw::{FW_INFO, FwConfig, REGISTRY},
};

type Result<T> = std::result::Result<T, EcError>;

#[derive(Debug, Snafu)]
enum EcError {
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
    io: Option<EcSys>,
    fw: Option<FwConfig>,
    // TODO: ec drv
}

impl Ec {
    pub fn new() -> Result<Self> {
        let io = match EcSys::new() {
            Ok(io) => Some(io),
            Err(source) => match source {
                // It is loaded, but there was an issue
                EcSysError::RootRequired
                | EcSysError::NoWriteSupport
                | EcSysError::OtherIo { .. }
                | EcSysError::Whatever { .. }
                | EcSysError::OtherErrno { .. } => return Err(source.into()),

                // No load = Reduced functionality mode
                EcSysError::NotLoaded => None,
            },
        };

        let fw = match io.as_ref() {
            Some(io) => {
                let version = Self::_fw_version(io)?;
                match REGISTRY.get(&version) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        log::warn!(
                            "fw {version} is unsupported; certain functions will be disabled"
                        );

                        None
                    }
                }
            }

            None => None,
        };

        // TODO: ec drv

        let this = Self { io, fw };

        Ok(this)
    }

    //
    // Firmware
    //

    fn _fw_version(io: &EcSys) -> Result<String> {
        let mut buf = [0; FW_INFO.version.len];
        io.ec_read_seq(FW_INFO.version.addr, &mut buf)
            .context(EcError::EcSys { source: () })
            .whatever_context("fw_version() failed to ec_read_seq()")?;
        let s = str::from_utf8(&buf).whatever_context("fw_version() received non utf8 data")?;

        Ok(s.to_owned())
    }

    pub fn fw_version(&self) -> Result<String> {
        Self::_fw_version(&self.io)
    }

    pub fn fw_date(&self) -> Result<String> {
        let mut buf = [0; FW_INFO.date.len];
        self.io
            .ec_read_seq(FW_INFO.date.addr, &mut buf)
            .context("fw_date() failed to ec_read_seq()")?;
        let s = str::from_utf8(&buf).context("fw_date() received non utf8 data")?;

        Ok(s.to_owned())
    }

    pub fn fw_time(&self) -> Result<String> {
        let mut buf = [0; FW_INFO.time.len];
        self.io
            .ec_read_seq(FW_INFO.time.addr, &mut buf)
            .context("fw_time() failed to ec_read_seq()")?;
        let s = str::from_utf8(&buf).context("fw_time() received non utf8 data")?;

        Ok(s.to_owned())
    }

    //
    // Shift Mode
    //

    // pub fn shift_mode(&self) -> Result<ShiftMode> {
    //     let val = self
    //         .io
    //         .ec_read()
    //         .context("shift_mode() failed to ec_read()")?;
    //     let val = ShiftMode::try_from(val).context("shift_mode() failed to ShiftMode::try_from")?;
    //     Ok(val)
    // }

    // pub fn set_shift_mode(&self, mode: ShiftMode) {
    //     todo!()
    // }

    //
    // Battery
    //

    /// The battery threshold. Is set in 10% increments, with the returned mode
    /// being the high end. For examaple, this returns 60%, which means threshold
    /// is 50-60%. We return 80%, which means threashold is 70-80%. However, when
    /// we return 100%, it's 100% always.
    pub fn battery_mode(&self) -> Result<BatteryMode> {
        let val = self
            .io
            .ec_read(self.fw.charge_control_addr.get()?)
            .context("battery_mode() failed to ec_read()")?;

        let val = BatteryMode::try_from_end(val)
            .context("battery_mode() failed to BatteryMode::try_from_end")?;

        Ok(val)
    }

    pub fn super_battery(&self) -> Result<SuperBattery> {
        let val = self
            .io
            .ec_read(self.fw.super_battery.addr.get()?)
            .context("super_battery() failed to ec_read()")?;
        let val = SuperBattery::try_from(val)
            .context("super_battery() failed to BatteryMode::try_from")?;
        Ok(val)
    }

    //
    // Fan RPM
    //

    fn fan_rpm(&self, addr: u8) -> Result<u16> {
        let mut rpm = [0; 2];

        self.io
            .ec_read_seq(addr, &mut rpm)
            .context("fan_rpm() failed to ec_read()")?;

        let rpm = u16::from_be_bytes(rpm).to_rpm();

        Ok(rpm)
    }

    pub fn fan1_rpm(&self) -> Result<u16> {
        self.fan_rpm(FAN1_RPM)
    }

    pub fn fan2_rpm(&self) -> Result<u16> {
        self.fan_rpm(FAN2_RPM)
    }

    pub fn fan3_rpm(&self) -> Result<u16> {
        self.fan_rpm(FAN3_RPM)
    }

    pub fn fan4_rpm(&self) -> Result<u16> {
        self.fan_rpm(FAN4_RPM)
    }

    //
    // Temps
    //

    pub fn cpu_temp(&self) -> Result<u8> {
        let val = self
            .io
            .ec_read(CPU_TEMP)
            .context("cpu_temp() failed to ec_read()")?;

        Ok(val)
    }

    pub fn gpu_temp(&self) -> Result<u8> {
        let val = self
            .io
            .ec_read(GPU_TEMP)
            .context("gpu_temp() failed to ec_read()")?;

        Ok(val)
    }

    //
    // Fan Curves
    //

    fn fan_curve(&self, addr: u8) -> Result<u8> {
        let val = self
            .io
            .ec_read(addr)
            .context("fan_curve() failed to ec_read()")?;

        Ok(val)
    }

    fn set_fan_curve(&self, addr: u8, percentage: u8) -> Result<u8> {
        if percentage > CPU_FAN_MAX || percentage > GPU_FAN_MAX {
            bail!("fan speed cannot exceed {percentage}%");
        }

        todo!()
    }

    pub fn gpu_fan_curve(&self) -> Result<GpuFanCurve> {
        let (p1, p2, p3, p4, p5, p6) = (
            self.fan_curve(GPU_FAN_CURVE_1)?,
            self.fan_curve(GPU_FAN_CURVE_2)?,
            self.fan_curve(GPU_FAN_CURVE_3)?,
            self.fan_curve(GPU_FAN_CURVE_4)?,
            self.fan_curve(GPU_FAN_CURVE_5)?,
            self.fan_curve(GPU_FAN_CURVE_6)?,
        );

        let gpu = GpuFanCurve::new(p1, p2, p3, p4, p5, p6).context("fan point exceeded max max")?;

        Ok(gpu)
    }

    pub fn cpu_fan_curve(&self) -> Result<CpuFanCurve> {
        let (p1, p2, p3, p4, p5, p6) = (
            self.fan_curve(CPU_FAN_CURVE_1)?,
            self.fan_curve(CPU_FAN_CURVE_2)?,
            self.fan_curve(CPU_FAN_CURVE_3)?,
            self.fan_curve(CPU_FAN_CURVE_4)?,
            self.fan_curve(CPU_FAN_CURVE_5)?,
            self.fan_curve(CPU_FAN_CURVE_6)?,
        );

        let cpu = CpuFanCurve::new(p1, p2, p3, p4, p5, p6).context("fan point exceeded max max")?;

        Ok(cpu)
    }

    //
    // Utils
    //

    // Raw ec dump
    pub fn ec_dump_raw(&self) -> Result<[u8; 256]> {
        let mut dump = [0; 256];
        self.io
            .ec_read_seq(0x00, &mut dump)
            .context("ec_dump_raw() failed to ec_read_seq()")?;
        Ok(dump)
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
