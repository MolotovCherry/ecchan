use ec::{
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, KeyDirection, Led, MethodData,
    MethodOp, ShiftMode, SuperBattery, Webcam,
};
use serde::{Deserialize, Serialize};

/// A ipc call. Make a value of this and json serialize it to call.
/// Returns an equivalent return type in Ret (see original fn for type)
#[derive(Clone, Serialize, Deserialize)]
pub enum Method<'a> {
    // Utils
    Ping,
    FanCount,
    FanMax,
    HasDGpu,
    WmiVer,

    // Firmware
    FwVersion,
    FwDate,
    FwTime,

    // Shift Modes
    ShiftModes,
    ShiftMode,
    SetShiftMode {
        mode: ShiftMode,
    },
    ShiftModeSupported,

    // Battery
    BatteryChargeMode,
    SetBatteryChargeMode {
        mode: BatteryChargeMode,
    },
    BatteryChargeModeSupported,

    SuperBattery,
    SetSuperBattery {
        state: SuperBattery,
    },
    SuperBatterySupported,

    // Fan
    Fan1Rpm,
    Fan2Rpm,
    Fan3Rpm,
    Fan4Rpm,

    Fan1Supported,
    Fan2Supported,
    Fan3Supported,
    Fan4Supported,

    FanModes,
    FanMode,
    SetFanMode {
        mode: FanMode,
    },
    FanModeSupported,

    // Webcam
    Webcam,
    WebcamBlock,
    SetWebcam {
        state: Webcam,
    },
    SetWebcamBlock {
        state: Webcam,
    },

    WebcamSupported,
    WebcamBlockSupported,

    // Cooler Boost
    CoolerBoost,
    SetCoolerBoost {
        state: CoolerBoost,
    },
    CoolerBoostSupported,

    // Swap Keys
    FnKey,
    WinKey,
    SetFnKey {
        state: KeyDirection,
    },
    SetWinKey {
        state: KeyDirection,
    },

    FnWinSwapSupported,

    // Mute LEDs
    MicMuteLed,
    MuteLed,
    SetMicMuteLed {
        state: Led,
    },
    SetMuteLed {
        state: Led,
    },

    MicMuteLedSupported,
    MuteLedSupported,

    // Realtime Stats
    CpuRtFanSpeed,
    CpuRtTemp,
    GpuRtFanSpeed,
    GpuRtTemp,

    // Curves
    CpuFanCurveWmi2,
    CpuTempCurveWmi2,
    CpuHysteresisCurveWmi2,
    GpuFanCurveWmi2,
    GpuTempCurveWmi2,
    GpuHysteresisCurveWmi2,

    SetCpuFanCurveWmi2 {
        curve: Curve7,
    },
    SetCpuTempCurveWmi2 {
        curve: Curve7,
    },
    SetCpuHysteresisCurveWmi2 {
        curve: Curve6,
    },
    SetGpuFanCurveWmi2 {
        curve: Curve7,
    },
    SetGpuTempCurveWmi2 {
        curve: Curve7,
    },
    SetGpuHysteresisCurveWmi2 {
        curve: Curve6,
    },

    // Ec
    EcDumpRaw,
    EcDumpPretty,

    // Methods
    MethodList,
    MethodRead {
        method: &'a str,
        op: MethodOp,
    },
    MethodWrite {
        method: &'a str,
        op: MethodOp,
        data: MethodData,
    },
}
