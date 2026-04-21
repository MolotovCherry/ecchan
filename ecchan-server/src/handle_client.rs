use std::io::{self, ErrorKind};

use ec::{Ec, EcError};
use ecchan_ipc::{
    method::Method,
    ret::{Bin, Ret, RetVal},
};
use snafu::{OptionExt, ResultExt, Snafu};
use tokio::{net::UnixStream, select};

use crate::signal::ShutdownSignal;

#[derive(Debug, Snafu)]
pub enum ClientError {
    Io {
        source: io::Error,
    },
    Ec {
        source: EcError,
    },
    Serde {
        source: serde_json::Error,
    },
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error>, Some)))]
        source: Option<Box<dyn std::error::Error>>,
    },
}

pub async fn handle_client(
    client: &mut UnixStream,
    ec: &mut Ec,
    shutdown: &ShutdownSignal,
) -> Result<(), ClientError> {
    let mut buf = [0u8; 1024];
    let mut msg_buf = Vec::with_capacity(1024);

    log::debug!("client connected");

    let mut sentinel_pos = 0;
    let mut drain = false;

    loop {
        if drain {
            msg_buf.drain(..=sentinel_pos);
            drain = false;
        }

        select! {
            _ = shutdown.wait() => break,
            v = client.readable() => v.context(IoSnafu)?,
        }

        let data = match client.try_read(&mut buf) {
            Ok(n) => match n {
                0 => break,
                n => {
                    let msg = &buf[..n];

                    // accumulate full message
                    msg_buf.extend_from_slice(msg);

                    // if we finally reached our sentinel (newline)
                    match msg_buf.iter().position(|n| *n == b'\n') {
                        Some(pos) => sentinel_pos = pos,
                        None => continue,
                    }

                    drain = true;

                    &msg_buf[..sentinel_pos]
                }
            },

            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => continue,
                _ => return Err(e).context(IoSnafu),
            },
        };

        let Ok(msg) = str::from_utf8(data) else {
            log::error!("Client message could not be decoded: {data:?}");
            continue;
        };

        log::debug!("got client req: {msg}");

        let ret = match serde_json::from_str::<Method>(msg) {
            Ok(c) => call(c, ec),
            Err(e) => {
                log::error!("{e}");
                continue;
            }
        };

        let response = match ret {
            Ok(d) => Ret::Ok(d),
            Err(e) => Ret::Err(e.to_string()),
        };

        let mut ser = serde_json::to_string(&response).context(SerdeSnafu)?;

        log::debug!("sending reply: {ser}");

        // push sentinel back on
        ser.push('\n');

        let data = ser.as_bytes();

        let mut n = 0;
        loop {
            client.writable().await.context(IoSnafu)?;

            let data_slice = &data[n..];

            match client.try_write(data_slice) {
                Ok(c) => {
                    n += c;

                    if n >= data.len() {
                        break;
                    }
                }

                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => continue,
                    _ => return Err(e).context(IoSnafu),
                },
            }
        }
    }

    Ok(())
}

fn call(ty: Method, ec: &mut Ec) -> Result<RetVal<'static>, ClientError> {
    let val = match ty {
        Method::Ping => RetVal::Pong,

        Method::FanCount => {
            let data = ec.fan_count();
            RetVal::Fans(data)
        }

        Method::FanMax => {
            let data = ec.fan_max().whatever_context("model not supported")?;
            RetVal::Byte(data)
        }

        Method::HasDGpu => {
            let data = ec.has_dgpu();
            RetVal::State(data)
        }

        Method::WmiVer => {
            let data = ec.wmi_ver().whatever_context("model not supported")?;
            RetVal::WmiVer(data)
        }

        Method::FwVersion => {
            let data = ec.fw_version().context(EcSnafu)?;
            RetVal::Str(data)
        }

        Method::FwDate => {
            let data = ec.fw_date().context(EcSnafu)?;
            RetVal::Str(data)
        }

        Method::FwTime => {
            let data = ec.fw_time().context(EcSnafu)?;
            RetVal::Str(data)
        }

        Method::ShiftModes => {
            let modes = ec.shift_modes().context(EcSnafu)?;
            RetVal::ShiftModes(modes)
        }

        Method::ShiftMode => {
            let mode = ec.shift_mode().context(EcSnafu)?;
            RetVal::ShiftMode(mode)
        }

        Method::SetShiftMode { mode } => {
            ec.set_shift_mode(mode).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::ShiftModeSupported => {
            let state = ec.shift_mode_supported();
            RetVal::State(state)
        }

        Method::BatteryChargeMode => {
            let mode = ec.battery_charge_mode().context(EcSnafu)?;
            RetVal::BatteryChargeMode(mode)
        }

        Method::SetBatteryChargeMode { mode } => {
            ec.set_battery_charge_mode(mode).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::BatteryChargeModeSupported => {
            let state = ec.battery_charge_mode_supported();
            RetVal::State(state)
        }

        Method::SuperBattery => {
            let state = ec.super_battery().context(EcSnafu)?;
            RetVal::SuperBattery(state)
        }

        Method::SetSuperBattery { state } => {
            ec.set_super_battery(state).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SuperBatterySupported => {
            let state = ec.super_battery_supported();
            RetVal::State(state)
        }

        Method::Fan1Rpm => {
            let rpm = ec.fan1_rpm().context(EcSnafu)?;
            RetVal::Word(rpm)
        }

        Method::Fan2Rpm => {
            let rpm = ec.fan2_rpm().context(EcSnafu)?;
            RetVal::Word(rpm)
        }

        Method::Fan3Rpm => {
            let rpm = ec.fan3_rpm().context(EcSnafu)?;
            RetVal::Word(rpm)
        }

        Method::Fan4Rpm => {
            let rpm = ec.fan4_rpm().context(EcSnafu)?;
            RetVal::Word(rpm)
        }

        Method::Fan1Supported => {
            let state = ec.fan1_supported();
            RetVal::State(state)
        }

        Method::Fan2Supported => {
            let state = ec.fan2_supported();
            RetVal::State(state)
        }

        Method::Fan3Supported => {
            let state = ec.fan3_supported();
            RetVal::State(state)
        }

        Method::Fan4Supported => {
            let state = ec.fan4_supported();
            RetVal::State(state)
        }

        Method::FanModes => {
            let modes = ec.fan_modes().context(EcSnafu)?;
            RetVal::FanModes(modes)
        }

        Method::FanMode => {
            let mode = ec.fan_mode().context(EcSnafu)?;
            RetVal::FanMode(mode)
        }

        Method::SetFanMode { mode } => {
            ec.set_fan_mode(mode).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::FanModeSupported => {
            let state = ec.fan_mode_supported();
            RetVal::State(state)
        }

        Method::Webcam => {
            let state = ec.webcam().context(EcSnafu)?;
            RetVal::Webcam(state)
        }

        Method::WebcamBlock => {
            let mode = ec.webcam_block().context(EcSnafu)?;
            RetVal::Webcam(mode)
        }

        Method::SetWebcam { state } => {
            ec.set_webcam(state).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SetWebcamBlock { state } => {
            ec.set_webcam_block(state).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::WebcamSupported => {
            let state = ec.webcam_supported();
            RetVal::State(state)
        }

        Method::WebcamBlockSupported => {
            let state = ec.webcam_block_supported();
            RetVal::State(state)
        }

        Method::CoolerBoost => {
            let state = ec.cooler_boost().context(EcSnafu)?;
            RetVal::CoolerBoost(state)
        }

        Method::SetCoolerBoost { state } => {
            ec.set_cooler_boost(state).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::CoolerBoostSupported => {
            let state = ec.cooler_boost_supported();
            RetVal::State(state)
        }

        Method::FnKey => {
            let state = ec.fn_key().context(EcSnafu)?;
            RetVal::KeyDirection(state)
        }

        Method::WinKey => {
            let state = ec.win_key().context(EcSnafu)?;
            RetVal::KeyDirection(state)
        }

        Method::SetFnKey { state } => {
            ec.set_fn_key(state).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SetWinkey { state } => {
            ec.set_win_key(state).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::FnWinSwapSupported => {
            let state = ec.fn_win_swap_supported();
            RetVal::State(state)
        }

        Method::MicMuteLed => {
            let state = ec.mic_mute_led().context(EcSnafu)?;
            RetVal::Led(state)
        }

        Method::MuteLed => {
            let state = ec.mute_led().context(EcSnafu)?;
            RetVal::Led(state)
        }

        Method::SetMicMuteLed { state } => {
            ec.set_mic_mute_led(state).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SetMuteLed { state } => {
            ec.set_mute_led(state).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::MicMuteLedSupported => {
            let state = ec.mic_mute_led_supported();
            RetVal::State(state)
        }

        Method::MuteLedSupported => {
            let state = ec.mute_led_supported();
            RetVal::State(state)
        }

        Method::CpuRtFanSpeed => {
            let speed = ec.cpu_rt_fan_speed().context(EcSnafu)?;
            RetVal::Byte(speed)
        }

        Method::CpuRtTemp => {
            let temp = ec.cpu_rt_temp().context(EcSnafu)?;
            RetVal::Byte(temp)
        }

        Method::GpuRtFanSpeed => {
            let speed = ec.gpu_rt_fan_speed().context(EcSnafu)?;
            RetVal::Byte(speed)
        }

        Method::GpuRtTemp => {
            let temp = ec.gpu_rt_temp().context(EcSnafu)?;
            RetVal::Byte(temp)
        }

        Method::CpuFanCurveWmi2 => {
            let curve = ec.cpu_fan_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve7(curve)
        }

        Method::CpuTempCurveWmi2 => {
            let curve = ec.cpu_temp_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve7(curve)
        }

        Method::CpuHysteresisCurveWmi2 => {
            let curve = ec.cpu_hysteresis_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve6(curve)
        }

        Method::GpuFanCurveWmi2 => {
            let curve = ec.gpu_fan_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve7(curve)
        }

        Method::GpuTempCurveWmi2 => {
            let curve = ec.gpu_temp_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve7(curve)
        }

        Method::GpuHysteresisCurveWmi2 => {
            let curve = ec.gpu_hysteresis_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve6(curve)
        }

        Method::SetCpuFanCurveWmi2 { curve } => {
            ec.set_cpu_fan_curve_wmi2(curve).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SetCpuTempCurveWmi2 { curve } => {
            ec.set_cpu_temp_curve_wmi2(curve).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SetCpuHysteresisCurveWmi2 { curve } => {
            ec.set_cpu_hysteresis_curve_wmi2(curve).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SetGpuFanCurveWmi2 { curve } => {
            ec.set_gpu_fan_curve_wmi2(curve).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SetGpuTempCurveWmi2 { curve } => {
            ec.set_gpu_temp_curve_wmi2(curve).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::SetGpuHysteresisCurveWmi2 { curve } => {
            ec.set_gpu_hysteresis_curve_wmi2(curve).context(EcSnafu)?;
            RetVal::Unit
        }

        Method::EcDumpRaw => {
            let dump = ec.ec_dump_raw().context(EcSnafu)?;
            RetVal::EcDump(Box::new(Bin(dump)))
        }

        Method::EcDumpPretty => {
            let data = ec.ec_dump_pretty().context(EcSnafu)?;
            RetVal::Str(data)
        }

        Method::MethodList => {
            let list = ec.method_list();
            RetVal::Methods(list)
        }

        Method::MethodRead { method, op } => {
            let data = ec.method_read(method, op).context(EcSnafu)?;
            RetVal::MethodData(data)
        }

        Method::MethodWrite { method, op, data } => {
            ec.method_write(method, op, data).context(EcSnafu)?;
            RetVal::Unit
        }
    };

    Ok(val)
}
