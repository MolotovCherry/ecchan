use std::os::fd::{AsFd as _, BorrowedFd};

use ec::{Ec, EcError};
use ecchan_ipc::{
    call::Call,
    ret::{Bin, Ret, RetVal},
};
use rustix::{
    event::{PollFd, PollFlags, poll},
    io::{Errno, read, write},
};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum ClientError {
    Sock { source: Errno },
    Ec { source: EcError },
    Serde { source: serde_json::Error },
    Exit,
}

pub fn handle_client(client: BorrowedFd, ec: &mut Ec) -> Result<(), ClientError> {
    let mut buf = [0u8; 1024];
    let mut msg_buf = Vec::with_capacity(1024);

    let mut sentinel_pos = 0;
    // drain buf on next run
    let mut drain_buf = false;

    let mut events = [PollFd::from_borrowed_fd(client.as_fd(), PollFlags::IN)];

    loop {
        if drain_buf {
            msg_buf.drain(..=sentinel_pos);
            drain_buf = false;
        }

        match poll(&mut events, None) {
            Ok(_) => (),
            Err(e) => match e {
                Errno::INTR => return Err(ClientError::Exit),
                e => return Err(e).context(SockSnafu),
            },
        }

        let msg = match read(client, &mut buf) {
            Ok(n) => match n {
                0 => break,
                n => &buf[..n],
            },

            Err(e) => match e {
                Errno::WOULDBLOCK => continue,
                e => return Err(e).context(SockSnafu),
            },
        };

        // accumulate full message
        msg_buf.extend_from_slice(msg);

        // if we encountered a sentinel, we're ready to proceed
        sentinel_pos = match msg_buf.iter().position(|b| *b == 0) {
            Some(p) => p,
            // no sentinel yet, continue accumulating message
            None => continue,
        };

        drain_buf = true;

        let data = match cobs::decode_in_place(&mut msg_buf) {
            Ok(s) => &msg_buf[..s],
            Err(e) => {
                log::error!("Client COBs decode error: {e}");
                continue;
            }
        };

        let Ok(msg) = str::from_utf8(data) else {
            log::error!("Client message could not be decoded: {data:?}");
            continue;
        };

        let ret = match serde_json::from_str::<Call>(msg) {
            Ok(c) => call(c, ec),
            Err(e) => {
                log::error!("{e}");
                continue;
            }
        };

        let response = match ret {
            Ok(d) => d.map(Ret::Ok).unwrap_or(Ret::Ok(RetVal::Empty)),
            Err(e) => Ret::Err(e.to_string()),
        };

        let ser = serde_json::to_string(&response).context(SerdeSnafu)?;
        let data = cobs::encode_vec_including_sentinels(ser.as_bytes());

        let mut n = 0;

        // make sure to fully flush writes
        loop {
            let bytes = &data[n..];
            n += write(client, bytes).context(SockSnafu)?;

            if n >= msg.len() {
                break;
            }
        }
    }

    Ok(())
}

fn call(ty: Call, ec: &mut Ec) -> Result<Option<RetVal<'static>>, ClientError> {
    let val = match ty {
        Call::FwVersion => {
            let data = ec.fw_version().context(EcSnafu)?;
            RetVal::Str(data)
        }

        Call::FwDate => {
            let data = ec.fw_date().context(EcSnafu)?;
            RetVal::Str(data)
        }

        Call::FwTime => {
            let data = ec.fw_time().context(EcSnafu)?;
            RetVal::Str(data)
        }

        Call::ShiftModes => {
            let modes = ec.shift_modes().context(EcSnafu)?;
            RetVal::ShiftModes(modes)
        }

        Call::ShiftMode => {
            let mode = ec.shift_mode().context(EcSnafu)?;
            RetVal::ShiftMode(mode)
        }

        Call::SetShiftMode { mode } => {
            ec.set_shift_mode(mode).context(EcSnafu)?;
            return Ok(None);
        }

        Call::ShiftModeSupported => {
            let state = ec.shift_mode_supported();
            RetVal::State(state)
        }

        Call::BatteryChargeMode => {
            let mode = ec.battery_charge_mode().context(EcSnafu)?;
            RetVal::BatteryChargeMode(mode)
        }

        Call::SetBatteryChargeMode { mode } => {
            ec.set_battery_charge_mode(mode).context(EcSnafu)?;
            return Ok(None);
        }

        Call::BatteryChargeModeSupported => {
            let state = ec.battery_charge_mode_supported();
            RetVal::State(state)
        }

        Call::SuperBattery => {
            let state = ec.super_battery().context(EcSnafu)?;
            RetVal::SuperBattery(state)
        }

        Call::SetSuperBattery { state } => {
            ec.set_super_battery(state).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SuperBatterySupported => {
            let state = ec.super_battery_supported();
            RetVal::State(state)
        }

        Call::Fan1Rpm => {
            let rpm = ec.fan1_rpm().context(EcSnafu)?;
            RetVal::Word(rpm)
        }

        Call::Fan2Rpm => {
            let rpm = ec.fan2_rpm().context(EcSnafu)?;
            RetVal::Word(rpm)
        }

        Call::Fan3Rpm => {
            let rpm = ec.fan3_rpm().context(EcSnafu)?;
            RetVal::Word(rpm)
        }

        Call::Fan4Rpm => {
            let rpm = ec.fan4_rpm().context(EcSnafu)?;
            RetVal::Word(rpm)
        }

        Call::Fan1Supported => {
            let state = ec.fan1_supported();
            RetVal::State(state)
        }

        Call::Fan2Supported => {
            let state = ec.fan2_supported();
            RetVal::State(state)
        }

        Call::Fan3Supported => {
            let state = ec.fan3_supported();
            RetVal::State(state)
        }

        Call::Fan4Supported => {
            let state = ec.fan4_supported();
            RetVal::State(state)
        }

        Call::FanModes => {
            let modes = ec.fan_modes().context(EcSnafu)?;
            RetVal::FanModes(modes)
        }

        Call::FanMode => {
            let mode = ec.fan_mode().context(EcSnafu)?;
            RetVal::FanMode(mode)
        }

        Call::SetFanMode { mode } => {
            ec.set_fan_mode(mode).context(EcSnafu)?;
            return Ok(None);
        }

        Call::FanModeSupported => {
            let state = ec.fan_mode_supported();
            RetVal::State(state)
        }

        Call::Webcam => {
            let state = ec.webcam().context(EcSnafu)?;
            RetVal::Webcam(state)
        }

        Call::WebcamBlock => {
            let mode = ec.webcam_block().context(EcSnafu)?;
            RetVal::Webcam(mode)
        }

        Call::SetWebcam { state } => {
            ec.set_webcam(state).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SetWebcamBlock { state } => {
            ec.set_webcam_block(state).context(EcSnafu)?;
            return Ok(None);
        }

        Call::WebcamSupported => {
            let state = ec.webcam_supported();
            RetVal::State(state)
        }

        Call::WebcamBlockSupported => {
            let state = ec.webcam_block_supported();
            RetVal::State(state)
        }

        Call::CoolerBoost => {
            let state = ec.cooler_boost().context(EcSnafu)?;
            RetVal::CoolerBoost(state)
        }

        Call::SetCoolerBoost { state } => {
            ec.set_cooler_boost(state).context(EcSnafu)?;
            return Ok(None);
        }

        Call::CoolerBoostSupported => {
            let state = ec.cooler_boost_supported();
            RetVal::State(state)
        }

        Call::FnKey => {
            let state = ec.fn_key().context(EcSnafu)?;
            RetVal::KeyDirection(state)
        }

        Call::WinKey => {
            let state = ec.win_key().context(EcSnafu)?;
            RetVal::KeyDirection(state)
        }

        Call::SetFnKey { state } => {
            ec.set_fn_key(state).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SetWinkey { state } => {
            ec.set_win_key(state).context(EcSnafu)?;
            return Ok(None);
        }

        Call::FnWinSwapSupported => {
            let state = ec.fn_win_swap_supported();
            RetVal::State(state)
        }

        Call::MicMuteLed => {
            let state = ec.mic_mute_led().context(EcSnafu)?;
            RetVal::Led(state)
        }

        Call::MuteLed => {
            let state = ec.mute_led().context(EcSnafu)?;
            RetVal::Led(state)
        }

        Call::SetMicMuteLed { state } => {
            ec.set_mic_mute_led(state).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SetMuteLed { state } => {
            ec.set_mute_led(state).context(EcSnafu)?;
            return Ok(None);
        }

        Call::MicMuteLedSupported => {
            let state = ec.mic_mute_led_supported();
            RetVal::State(state)
        }

        Call::MuteLedSupported => {
            let state = ec.mute_led_supported();
            RetVal::State(state)
        }

        Call::CpuRtFanSpeed => {
            let speed = ec.cpu_rt_fan_speed().context(EcSnafu)?;
            RetVal::Byte(speed)
        }

        Call::CpuRtTemp => {
            let temp = ec.cpu_rt_temp().context(EcSnafu)?;
            RetVal::Byte(temp)
        }

        Call::GpuRtFanSpeed => {
            let speed = ec.gpu_rt_fan_speed().context(EcSnafu)?;
            RetVal::Byte(speed)
        }

        Call::GpuRtTemp => {
            let temp = ec.gpu_rt_temp().context(EcSnafu)?;
            RetVal::Byte(temp)
        }

        Call::CpuFanCurveWmi2 => {
            let curve = ec.cpu_fan_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve7(curve)
        }

        Call::CpuTempCurveWmi2 => {
            let curve = ec.cpu_temp_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve7(curve)
        }

        Call::CpuHysteresisCurveWmi2 => {
            let curve = ec.cpu_hysteresis_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve6(curve)
        }

        Call::GpuFanCurveWmi2 => {
            let curve = ec.gpu_fan_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve7(curve)
        }

        Call::GpuTempCurveWmi2 => {
            let curve = ec.gpu_temp_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve7(curve)
        }

        Call::GpuHysteresisCurveWmi2 => {
            let curve = ec.gpu_hysteresis_curve_wmi2().context(EcSnafu)?;
            RetVal::Curve6(curve)
        }

        Call::SetCpuFanCurveWmi2 { curve } => {
            ec.set_cpu_fan_curve_wmi2(curve).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SetCpuTempCurveWmi2 { curve } => {
            ec.set_cpu_temp_curve_wmi2(curve).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SetCpuHysteresisCurveWmi2 { curve } => {
            ec.set_cpu_hysteresis_curve_wmi2(curve).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SetGpuFanCurveWmi2 { curve } => {
            ec.set_gpu_fan_curve_wmi2(curve).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SetGpuTempCurveWmi2 { curve } => {
            ec.set_gpu_temp_curve_wmi2(curve).context(EcSnafu)?;
            return Ok(None);
        }

        Call::SetGpuHysteresisCurveWmi2 { curve } => {
            ec.set_gpu_hysteresis_curve_wmi2(curve).context(EcSnafu)?;
            return Ok(None);
        }

        Call::EcDumpRaw => {
            let dump = ec.ec_dump_raw().context(EcSnafu)?;
            RetVal::EcDump(Box::new(Bin(dump)))
        }

        Call::EcDumpPretty => {
            let data = ec.ec_dump_pretty().context(EcSnafu)?;
            RetVal::Str(data)
        }

        Call::MethodList => {
            let list = ec.method_list();
            RetVal::Methods(list)
        }

        Call::MethodRead { method, op } => {
            let data = ec.method_read(method, op).context(EcSnafu)?;
            RetVal::MethodData(data)
        }

        Call::MethodWrite { method, op, data } => {
            ec.method_write(method, op, data).context(EcSnafu)?;
            return Ok(None);
        }
    };

    Ok(Some(val))
}
