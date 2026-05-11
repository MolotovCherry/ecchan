use std::{
    borrow::Cow,
    collections::HashMap,
    io,
    pin::Pin,
    str::FromStr,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, TryRecvError, channel},
    },
    thread,
    time::Duration,
};

use cxx_qt::{Constructor, CxxQtType, QMetaObjectConnection, Threading};
use cxx_qt_lib::{QByteArray, QMetaTypeType, QQmlEngine, QString, QStringList, QVariant};
use ecchan_ipc::{
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, Fans, KeyDirection, Led, MethodData,
    MethodOp, ShiftMode, SuperBattery, Webcam, WmiVer,
    method::{Method, MethodTy},
    ret::{Bin, RetVal},
};
use sayuri::sync::{Mutex, Sendable};
use strum::IntoEnumIterator as _;

use crate::{
    client::{Client, ClientError},
    cpp::{QJSValue, QJSValueIterator, QJSValueList, qqmlengine::QQmlEngineExt as _},
    q_critical, q_warning,
    setup::setup,
};
pub use qobject::EcchanClient;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;

        include!("cxx-qt-lib/qlist.h");
        type QList_QVariant = cxx_qt_lib::QList<QVariant>;

        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
    }

    unsafe extern "C++" {
        include!("ecchan-client/qqml_property_map.h");
        type QQmlPropertyMap = crate::cpp::QQmlPropertyMap;

        include!("ecchan-client/qjsvalue.h");
        type QJSValue = crate::cpp::QJSValue;
    }

    impl cxx_qt::Threading for EcchanClient {}
    impl cxx_qt::Constructor<()> for EcchanClient {}

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, connected, READ, WRITE = set_connected, NOTIFY, FINAL)]
        #[qproperty(QString, path, READ, WRITE, NOTIFY, FINAL)]
        // utils
        #[qproperty(u8, fan_count, READ = get_fan_count, NOTIFY, FINAL)]
        #[qproperty(u8, fan_max, READ = get_fan_max, NOTIFY, FINAL)]
        #[qproperty(bool, has_dgpu, READ = get_has_dgpu, NOTIFY, FINAL)]
        #[qproperty(u8, wmi_ver, READ = get_wmi_ver, NOTIFY, FINAL)]
        // fw
        #[qproperty(QString, fw_version, READ = get_fw_version, NOTIFY, FINAL)]
        #[qproperty(QString, fw_date, READ = get_fw_date, NOTIFY, FINAL)]
        #[qproperty(QString, fw_time, READ = get_fw_time, NOTIFY, FINAL)]
        // shift mode
        #[qproperty(QStringList, shift_modes, READ = get_shift_modes, NOTIFY, FINAL)]
        #[qproperty(QString, shift_mode, READ = get_shift_mode, WRITE = set_shift_mode, NOTIFY, FINAL)]
        #[qproperty(bool, shift_mode_supported, READ = get_shift_mode_supported, NOTIFY, FINAL)]
        // battery charge mode
        #[qproperty(QVariant, battery_charge_mode, READ = get_battery_charge_mode, WRITE = set_battery_charge_mode, NOTIFY, FINAL)]
        #[qproperty(bool, battery_charge_mode_supported, READ = get_battery_charge_mode_supported, NOTIFY, FINAL)]
        // super battery
        #[qproperty(bool, super_battery, READ = get_super_battery, WRITE = set_super_battery, NOTIFY, FINAL)]
        #[qproperty(bool, super_battery_supported, READ = get_super_battery_supported, NOTIFY, FINAL)]
        // fan rpm
        #[qproperty(u16, fan1_rpm, READ = get_fan1_rpm, NOTIFY, FINAL)]
        #[qproperty(u16, fan2_rpm, READ = get_fan2_rpm, NOTIFY, FINAL)]
        #[qproperty(u16, fan3_rpm, READ = get_fan3_rpm, NOTIFY, FINAL)]
        #[qproperty(u16, fan4_rpm, READ = get_fan4_rpm, NOTIFY, FINAL)]
        #[qproperty(bool, fan1_supported, READ = get_fan1_supported, NOTIFY, FINAL)]
        #[qproperty(bool, fan2_supported, READ = get_fan2_supported, NOTIFY, FINAL)]
        #[qproperty(bool, fan3_supported, READ = get_fan3_supported, NOTIFY, FINAL)]
        #[qproperty(bool, fan4_supported, READ = get_fan4_supported, NOTIFY, FINAL)]
        // fan modes
        #[qproperty(QStringList, fan_modes, READ = get_fan_modes, NOTIFY, FINAL)]
        #[qproperty(QString, fan_mode, READ = get_fan_mode, WRITE = set_fan_mode, NOTIFY, FINAL)]
        #[qproperty(bool, fan_mode_supported, READ = get_fan_mode_supported, NOTIFY, FINAL)]
        // webcam
        #[qproperty(bool, webcam, READ = get_webcam, WRITE = set_webcam, NOTIFY, FINAL)]
        #[qproperty(bool, webcam_block, READ = get_webcam_block, WRITE = set_webcam_block, NOTIFY, FINAL)]
        #[qproperty(bool, webcam_supported, READ = get_webcam_supported, NOTIFY, FINAL)]
        #[qproperty(bool, webcam_block_supported, READ = get_webcam_block_supported, NOTIFY, FINAL)]
        // cooler boost
        #[qproperty(bool, cooler_boost, READ = get_cooler_boost, WRITE = set_cooler_boost, NOTIFY, FINAL)]
        #[qproperty(bool, cooler_boost_supported, READ = get_cooler_boost_supported, NOTIFY, FINAL)]
        // fn/win key swap
        #[qproperty(QString, fn_key, READ = get_fn_key, WRITE = set_fn_key, NOTIFY, FINAL)]
        #[qproperty(QString, win_key, READ = get_win_key, WRITE = set_win_key, NOTIFY, FINAL)]
        #[qproperty(bool, fn_win_swap_supported, READ = get_fn_win_swap_supported, NOTIFY, FINAL)]
        // mute leds
        #[qproperty(bool, mic_mute_led, READ = get_mic_mute_led, WRITE = set_mic_mute_led, NOTIFY, FINAL)]
        #[qproperty(bool, mute_led, READ = get_mute_led, WRITE = set_mute_led, NOTIFY, FINAL)]
        #[qproperty(bool, mic_mute_led_supported, READ = get_mic_mute_led_supported, NOTIFY, FINAL)]
        #[qproperty(bool, mute_led_supported, READ = get_mute_led_supported, NOTIFY, FINAL)]
        // rt sensors
        #[qproperty(u8, cpu_rt_fan_speed, READ = get_cpu_rt_fan_speed, NOTIFY, FINAL)]
        #[qproperty(u8, cpu_rt_temp, READ = get_cpu_rt_temp, NOTIFY, FINAL)]
        #[qproperty(u8, gpu_rt_fan_speed, READ = get_gpu_rt_fan_speed, NOTIFY, FINAL)]
        #[qproperty(u8, gpu_rt_temp, READ = get_gpu_rt_temp, NOTIFY, FINAL)]
        // curves
        #[qproperty(QVariant, cpu_fan_curve_wmi2, READ = get_cpu_fan_curve_wmi2, WRITE = set_cpu_fan_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QVariant, cpu_temp_curve_wmi2, READ = get_cpu_temp_curve_wmi2, WRITE = set_cpu_temp_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QVariant, cpu_hysteresis_curve_wmi2, READ = get_cpu_hysteresis_curve_wmi2, WRITE = set_cpu_hysteresis_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QVariant, gpu_fan_curve_wmi2, READ = get_gpu_fan_curve_wmi2, WRITE = set_gpu_fan_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QVariant, gpu_temp_curve_wmi2, READ = get_gpu_temp_curve_wmi2, WRITE = set_gpu_temp_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QVariant, gpu_hysteresis_curve_wmi2, READ = get_gpu_hysteresis_curve_wmi2, WRITE = set_gpu_hysteresis_curve_wmi2, NOTIFY, FINAL)]
        // methods
        #[qproperty(QVariant, methods, READ = get_methods, NOTIFY, FINAL)]
        // dump
        #[qproperty(QByteArray, ec_dump, READ = get_ec_dump, NOTIFY, FINAL)]
        #[qproperty(QString, ec_dump_pretty, READ = get_ec_dump_pretty, NOTIFY, FINAL)]
        #[namespace = "ecchan_client"]
        type EcchanClient = super::EcchanClientRust;

        fn set_connected(self: Pin<&mut Self>, connected: bool);

        fn get_has_dgpu(&self) -> bool;
        fn get_fan_max(&self) -> u8;
        fn get_fan_count(&self) -> u8;
        fn get_wmi_ver(&self) -> u8;

        fn get_fw_version(&self) -> &QString;
        fn get_fw_date(&self) -> &QString;
        fn get_fw_time(&self) -> &QString;

        fn get_shift_modes(&self) -> QStringList;
        fn get_shift_mode(&self) -> QString;
        fn set_shift_mode(self: Pin<&mut Self>, mode: &QString);
        fn get_shift_mode_supported(&self) -> bool;

        fn get_battery_charge_mode(&self) -> QVariant;
        fn set_battery_charge_mode(self: Pin<&mut Self>, mode: QVariant);
        fn get_battery_charge_mode_supported(&self) -> bool;

        fn get_super_battery(&self) -> bool;
        fn set_super_battery(self: Pin<&mut Self>, state: bool);
        fn get_super_battery_supported(&self) -> bool;

        fn get_fan1_rpm(&self) -> u16;
        fn get_fan2_rpm(&self) -> u16;
        fn get_fan3_rpm(&self) -> u16;
        fn get_fan4_rpm(&self) -> u16;
        fn get_fan1_supported(&self) -> bool;
        fn get_fan2_supported(&self) -> bool;
        fn get_fan3_supported(&self) -> bool;
        fn get_fan4_supported(&self) -> bool;

        fn get_fan_modes(&self) -> QStringList;
        fn get_fan_mode(&self) -> QString;
        fn set_fan_mode(self: Pin<&mut Self>, mode: &QString);
        fn get_fan_mode_supported(&self) -> bool;

        fn get_webcam(&self) -> bool;
        fn get_webcam_block(&self) -> bool;
        fn set_webcam(self: Pin<&mut Self>, state: bool);
        fn set_webcam_block(self: Pin<&mut Self>, state: bool);
        fn get_webcam_supported(&self) -> bool;
        fn get_webcam_block_supported(&self) -> bool;

        fn get_cooler_boost(&self) -> bool;
        fn set_cooler_boost(self: Pin<&mut Self>, state: bool);
        fn get_cooler_boost_supported(&self) -> bool;

        fn get_fn_key(&self) -> QString;
        fn get_win_key(&self) -> QString;
        fn set_fn_key(self: Pin<&mut Self>, dir: &QString);
        fn set_win_key(self: Pin<&mut Self>, dir: &QString);
        fn get_fn_win_swap_supported(&self) -> bool;

        fn get_mic_mute_led(&self) -> bool;
        fn get_mute_led(&self) -> bool;
        fn set_mic_mute_led(self: Pin<&mut Self>, state: bool);
        fn set_mute_led(self: Pin<&mut Self>, state: bool);
        fn get_mic_mute_led_supported(&self) -> bool;
        fn get_mute_led_supported(&self) -> bool;

        fn get_cpu_rt_fan_speed(&self) -> u8;
        fn get_cpu_rt_temp(&self) -> u8;
        fn get_gpu_rt_fan_speed(&self) -> u8;
        fn get_gpu_rt_temp(&self) -> u8;

        fn get_cpu_fan_curve_wmi2(&self) -> QVariant;
        fn set_cpu_fan_curve_wmi2(self: Pin<&mut Self>, curve: &QVariant);
        fn get_cpu_temp_curve_wmi2(&self) -> QVariant;
        fn set_cpu_temp_curve_wmi2(self: Pin<&mut Self>, curve: &QVariant);
        fn get_cpu_hysteresis_curve_wmi2(&self) -> QVariant;
        fn set_cpu_hysteresis_curve_wmi2(self: Pin<&mut Self>, curve: &QVariant);
        fn get_gpu_fan_curve_wmi2(&self) -> QVariant;
        fn set_gpu_fan_curve_wmi2(self: Pin<&mut Self>, curve: &QVariant);
        fn get_gpu_temp_curve_wmi2(&self) -> QVariant;
        fn set_gpu_temp_curve_wmi2(self: Pin<&mut Self>, curve: &QVariant);
        fn get_gpu_hysteresis_curve_wmi2(&self) -> QVariant;
        fn set_gpu_hysteresis_curve_wmi2(self: Pin<&mut Self>, curve: &QVariant);

        fn get_methods(&self) -> QVariant;
        #[qinvokable]
        fn method_write(self: Pin<&mut Self>, method: &QString, value: &QJSValue);

        fn get_ec_dump(&self) -> QByteArray;
        fn get_ec_dump_pretty(&self) -> &QString;

        //
        // Signals
        //

        #[qsignal]
        fn init_state_changed(self: Pin<&mut Self>, running: bool);

        #[qsignal]
        fn state_changed(self: Pin<&mut Self>, prop: QString);

        //
        // Invokables
        //

        #[qinvokable]
        fn init_state(self: Pin<&mut Self>);
        #[qinvokable]
        fn queue(self: Pin<&mut Self>, cb: &QJSValue);
        #[qinvokable]
        fn serialize(self: Pin<&mut Self>) -> QVariant;
        #[qinvokable]
        fn apply(self: Pin<&mut Self>, data: &QJSValue);

        #[qinvokable]
        fn update_fan_count(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan_max(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_has_dgpu(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_wmi_ver(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_fw_version(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fw_date(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fw_time(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_shift_modes(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_shift_mode(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_shift_mode_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_battery_charge_mode(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_battery_charge_mode_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_super_battery(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_super_battery_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_fan1_rpm(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan2_rpm(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan3_rpm(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan4_rpm(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan1_supported(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan2_supported(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan3_supported(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan4_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_fan_modes(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan_mode(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fan_mode_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_webcam(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_webcam_block(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_webcam_supported(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_webcam_block_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_cooler_boost(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_cooler_boost_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_fn_key(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_win_key(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_fn_win_swap_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_mic_mute_led(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_mute_led(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_mic_mute_led_supported(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_mute_led_supported(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_cpu_rt_fan_speed(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_cpu_rt_temp(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_gpu_rt_fan_speed(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_gpu_rt_temp(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_cpu_fan_curve_wmi2(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_cpu_temp_curve_wmi2(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_cpu_hysteresis_curve_wmi2(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_gpu_fan_curve_wmi2(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_gpu_temp_curve_wmi2(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_gpu_hysteresis_curve_wmi2(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_methods(self: Pin<&mut Self>);

        #[qinvokable]
        fn update_ec_dump(self: Pin<&mut Self>);
        #[qinvokable]
        fn update_ec_dump_pretty(self: Pin<&mut Self>);

        #[qinvokable]
        fn update(self: Pin<&mut Self>, method: Method);
    }

    #[qml_element]
    qnamespace!("Method");

    #[qenum]
    #[namespace = "Method"]
    enum Method {
        // Utils
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
        ShiftModeSupported,

        // Battery
        BatteryChargeMode,
        BatteryChargeModeSupported,

        SuperBattery,
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
        FanModeSupported,

        // Webcam
        Webcam,
        WebcamBlock,
        WebcamSupported,
        WebcamBlockSupported,

        // Cooler Boost
        CoolerBoost,
        CoolerBoostSupported,

        // Swap Keys
        FnKey,
        WinKey,
        FnWinSwapSupported,

        // Mute LEDs
        MicMuteLed,
        MuteLed,
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

        // Ec
        EcDumpRaw,
        EcDumpPretty,

        // Methods
        Methods,
    }
}

impl Constructor<()> for qobject::EcchanClient {
    type NewArguments = ();

    type BaseArguments = ();

    type InitializeArguments = ();

    fn route_arguments(
        args: (),
    ) -> (
        Self::NewArguments,
        Self::BaseArguments,
        Self::InitializeArguments,
    ) {
        (args, (), ())
    }

    fn new(_: Self::NewArguments) -> <Self as CxxQtType>::Rust {
        setup();
        <Self as CxxQtType>::Rust::default()
    }

    fn initialize(self: Pin<&mut Self>, _: Self::InitializeArguments) {
        self.on_connected_changed(|ctx| {
            if !ctx.connected {
                ctx.rust_mut().disconnected();
            }
        })
        .release();
    }
}

impl From<qobject::Method> for MethodTy {
    fn from(value: qobject::Method) -> Self {
        match value.repr {
            0 => MethodTy::FanCount,
            1 => MethodTy::FanMax,
            2 => MethodTy::HasDGpu,
            3 => MethodTy::WmiVer,
            4 => MethodTy::FwVersion,
            5 => MethodTy::FwDate,
            6 => MethodTy::FwTime,
            7 => MethodTy::ShiftModes,
            8 => MethodTy::ShiftMode,
            9 => MethodTy::ShiftModeSupported,
            10 => MethodTy::BatteryChargeMode,
            11 => MethodTy::BatteryChargeModeSupported,
            12 => MethodTy::SuperBattery,
            13 => MethodTy::SuperBatterySupported,
            14 => MethodTy::Fan1Rpm,
            15 => MethodTy::Fan2Rpm,
            16 => MethodTy::Fan3Rpm,
            17 => MethodTy::Fan4Rpm,
            18 => MethodTy::Fan1Supported,
            19 => MethodTy::Fan2Supported,
            20 => MethodTy::Fan3Supported,
            21 => MethodTy::Fan4Supported,
            22 => MethodTy::FanModes,
            23 => MethodTy::FanMode,
            24 => MethodTy::FanModeSupported,
            25 => MethodTy::Webcam,
            26 => MethodTy::WebcamBlock,
            27 => MethodTy::WebcamSupported,
            28 => MethodTy::WebcamBlockSupported,
            29 => MethodTy::CoolerBoost,
            30 => MethodTy::CoolerBoostSupported,
            31 => MethodTy::FnKey,
            32 => MethodTy::WinKey,
            33 => MethodTy::FnWinSwapSupported,
            34 => MethodTy::MicMuteLed,
            35 => MethodTy::MuteLed,
            36 => MethodTy::MicMuteLedSupported,
            37 => MethodTy::MuteLedSupported,
            38 => MethodTy::CpuRtFanSpeed,
            39 => MethodTy::CpuRtTemp,
            40 => MethodTy::GpuRtFanSpeed,
            41 => MethodTy::GpuRtTemp,
            42 => MethodTy::CpuFanCurveWmi2,
            43 => MethodTy::CpuTempCurveWmi2,
            44 => MethodTy::CpuHysteresisCurveWmi2,
            45 => MethodTy::GpuFanCurveWmi2,
            46 => MethodTy::GpuTempCurveWmi2,
            47 => MethodTy::GpuHysteresisCurveWmi2,
            48 => MethodTy::EcDumpRaw,
            49 => MethodTy::EcDumpPretty,
            50 => MethodTy::MethodRead, // there's no other useful variant, so use this as a sentinel for Methods
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq)]
struct SwapKey {
    fn_key: KeyDirection,
}

impl SwapKey {
    fn from_fn(dir: KeyDirection) -> Self {
        Self { fn_key: dir }
    }

    fn from_win(dir: KeyDirection) -> Self {
        let dir = match dir {
            KeyDirection::Left => KeyDirection::Right,
            KeyDirection::Right => KeyDirection::Left,
        };

        Self { fn_key: dir }
    }

    fn get_fn(&self) -> KeyDirection {
        self.fn_key
    }

    fn get_win(&self) -> KeyDirection {
        match self.fn_key {
            KeyDirection::Left => KeyDirection::Right,
            KeyDirection::Right => KeyDirection::Left,
        }
    }
}

pub struct EcchanClientRust {
    client: Option<Client>,
    // cancellation token
    heartbeats: Option<Sender<()>>,

    path: QString,
    connected: bool,

    fan_count: Fans,
    fan_max: u8,
    has_dgpu: bool,
    wmi_ver: WmiVer,

    fw_version: QString,
    fw_date: QString,
    fw_time: QString,

    shift_modes: Vec<ShiftMode>,
    shift_mode: ShiftMode,
    shift_mode_supported: bool,

    battery_charge_mode: BatteryChargeMode,
    battery_charge_mode_supported: bool,

    super_battery: SuperBattery,
    super_battery_supported: bool,

    fan1_rpm: u16,
    fan2_rpm: u16,
    fan3_rpm: u16,
    fan4_rpm: u16,
    fan1_supported: bool,
    fan2_supported: bool,
    fan3_supported: bool,
    fan4_supported: bool,

    fan_modes: Vec<FanMode>,
    fan_mode: FanMode,
    fan_mode_supported: bool,

    webcam: Webcam,
    webcam_block: Webcam,
    webcam_supported: bool,
    webcam_block_supported: bool,

    cooler_boost: CoolerBoost,
    cooler_boost_supported: bool,

    swap_key: SwapKey,
    fn_win_swap_supported: bool,

    mic_mute_led: Led,
    mute_led: Led,
    mic_mute_led_supported: bool,
    mute_led_supported: bool,

    cpu_rt_fan_speed: u8,
    cpu_rt_temp: u8,
    gpu_rt_fan_speed: u8,
    gpu_rt_temp: u8,

    cpu_fan_curve_wmi2: Curve7,
    cpu_temp_curve_wmi2: Curve7,
    cpu_hysteresis_curve_wmi2: Curve6,
    gpu_fan_curve_wmi2: Curve7,
    gpu_temp_curve_wmi2: Curve7,
    gpu_hysteresis_curve_wmi2: Curve6,

    methods: Methods,

    ec_dump: Box<Bin>,
    ec_dump_pretty: QString,
}

struct Methods {
    data: Vec<MethodPayload>,
}

#[derive(Debug, Clone)]
struct MethodPayload {
    name: String,
    method: String,
    data: MethodData,
    #[expect(unused)]
    read_op: MethodOp,
    write_op: MethodOp,
}

impl Default for EcchanClientRust {
    fn default() -> Self {
        Self {
            client: None,
            heartbeats: None,

            path: QString::default(),
            connected: false,

            fan_count: Fans::One,
            fan_max: 0,
            has_dgpu: false,
            wmi_ver: WmiVer::Wmi1,

            fw_version: QString::default(),
            fw_date: QString::default(),
            fw_time: QString::default(),

            shift_modes: Vec::new(),
            shift_mode: ShiftMode::Null,
            shift_mode_supported: false,

            battery_charge_mode: BatteryChargeMode::Mobility,
            battery_charge_mode_supported: false,

            super_battery: SuperBattery::Off,
            super_battery_supported: false,

            fan1_rpm: 0,
            fan2_rpm: 0,
            fan3_rpm: 0,
            fan4_rpm: 0,
            fan1_supported: true,
            fan2_supported: false,
            fan3_supported: false,
            fan4_supported: false,

            fan_modes: Vec::new(),
            fan_mode: FanMode::Null,
            fan_mode_supported: false,

            webcam: Webcam::On,
            webcam_block: Webcam::Off,
            webcam_supported: false,
            webcam_block_supported: false,

            cooler_boost: CoolerBoost::Off,
            cooler_boost_supported: false,

            swap_key: SwapKey::from_fn(KeyDirection::Left),
            fn_win_swap_supported: false,

            mic_mute_led: Led::Off,
            mute_led: Led::Off,
            mic_mute_led_supported: false,
            mute_led_supported: false,

            cpu_rt_fan_speed: 0,
            cpu_rt_temp: 0,
            gpu_rt_fan_speed: 0,
            gpu_rt_temp: 0,

            cpu_fan_curve_wmi2: Curve7::default(),
            cpu_temp_curve_wmi2: Curve7::default(),
            cpu_hysteresis_curve_wmi2: Curve6::default(),
            gpu_fan_curve_wmi2: Curve7::default(),
            gpu_temp_curve_wmi2: Curve7::default(),
            gpu_hysteresis_curve_wmi2: Curve6::default(),

            methods: Methods {
                data: Vec::new(),
            },

            ec_dump: Box::default(),
            ec_dump_pretty: "|      | _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _A _B _C _D _E _F\n|------+------------------------------------------------\n| 0x0_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x1_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x2_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x3_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x4_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x5_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x6_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x7_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x8_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x9_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xA_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xB_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xC_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xD_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xE_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xF_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n".into(),
        }
    }
}

impl EcchanClientRust {
    // common tasks to run on disconnect
    fn disconnected(&mut self) {
        if let Some(token) = self.heartbeats.take() {
            _ = token.send(());
        }

        self.client.take();
    }
}

// Internal
impl qobject::EcchanClient {
    fn disconnect(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().connected = false;
        self.connected_changed();
        // handlers connected to signal will handle cleanup
    }

    fn queued_call(mut self: Pin<&mut Self>, cb: impl FnOnce(Pin<&mut Self>) + Send + 'static) {
        let mut this = self.as_mut().rust_mut();
        let Some(client) = this.client.as_mut() else {
            q_warning!("not connected; cannot call queued cb");
            return;
        };

        client.queued_call(cb);
    }

    fn call(
        mut self: Pin<&mut Self>,
        method: Method<'static>,
        cb: impl FnOnce(Pin<&mut qobject::EcchanClient>, Result<RetVal<'static>, ClientError>)
        + Send
        + 'static,
    ) {
        let mut this = self.as_mut().rust_mut();
        let Some(client) = this.client.as_mut() else {
            if !matches!(method, Method::Ping) {
                q_warning!("not connected; cannot call {method:?}");
            }

            let err = Err(ClientError::Io {
                source: io::Error::new(io::ErrorKind::NotConnected, "not connected"),
            });

            cb(self, err);
            return;
        };

        client.call(method.clone(), move |mut ctx, res| {
            let res = match res {
                o @ Ok(_) => o,

                Err(e) => {
                    match e {
                        ClientError::Call { .. } | ClientError::Json { .. } => (),
                        ClientError::Io { .. } | ClientError::Eof => {
                            // socket error, so we now disconnect
                            ctx.as_mut().disconnect();
                        }
                    }

                    if !matches!(e, ClientError::Eof) {
                        q_critical!("{e}");
                    }

                    if matches!(method, Method::Ping) {
                        q_warning!("heartbeat failed; disconnecting");
                    }

                    Err(e)
                }
            };

            cb(ctx, res);
        });
    }

    fn _update(mut self: Pin<&mut Self>, method: MethodTy) {
        match method {
            MethodTy::FanCount => {
                self.as_mut().call(Method::FanCount, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.fans().unwrap();

                    if val == ctx.fan_count {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan_count = val;
                    ctx.as_mut().fan_count_changed();

                    ctx.state_changed("fanCount".into());
                });
            }

            MethodTy::FanMax => {
                self.as_mut().call(Method::FanMax, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.byte().unwrap();

                    if val == ctx.fan_max {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan_max = val;
                    ctx.as_mut().fan_max_changed();

                    ctx.state_changed("fanMax".into());
                });
            }

            MethodTy::HasDGpu => {
                self.as_mut().call(Method::HasDGpu, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.state().unwrap();

                    if val == ctx.has_dgpu {
                        return;
                    }

                    ctx.as_mut().rust_mut().has_dgpu = val;
                    ctx.as_mut().has_dgpu_changed();

                    ctx.state_changed("hasDgpu".into());
                });
            }

            MethodTy::WmiVer => {
                self.as_mut().call(Method::WmiVer, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.wmi_ver().unwrap();

                    if val == ctx.wmi_ver {
                        return;
                    }

                    ctx.as_mut().rust_mut().wmi_ver = val;
                    ctx.as_mut().wmi_ver_changed();

                    ctx.state_changed("wmiVer".into());
                });
            }

            MethodTy::FwVersion => {
                self.as_mut().call(Method::FwVersion, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val: QString = res.str().unwrap().into();

                    if val == ctx.fw_version {
                        return;
                    }

                    ctx.as_mut().rust_mut().fw_version = val;
                    ctx.as_mut().fw_version_changed();

                    ctx.state_changed("fwVersion".into());
                });
            }

            MethodTy::FwDate => {
                self.as_mut().call(Method::FwDate, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val: QString = res.str().unwrap().into();

                    if val == ctx.fw_date {
                        return;
                    }

                    ctx.as_mut().rust_mut().fw_date = val;
                    ctx.as_mut().fw_date_changed();

                    ctx.state_changed("fwDate".into());
                });
            }

            MethodTy::FwTime => {
                self.as_mut().call(Method::FwTime, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val: QString = res.str().unwrap().into();

                    if val == ctx.fw_time {
                        return;
                    }

                    ctx.as_mut().rust_mut().fw_time = val;
                    ctx.as_mut().fw_time_changed();

                    ctx.state_changed("fwtime".into());
                });
            }

            MethodTy::ShiftModes => {
                self.as_mut().call(Method::ShiftModes, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.shift_modes().unwrap();

                    if val == ctx.shift_modes {
                        return;
                    }

                    ctx.as_mut().rust_mut().shift_modes = val;
                    ctx.as_mut().shift_modes_changed();

                    ctx.state_changed("shiftModes".into());
                });
            }

            MethodTy::ShiftMode => {
                self.as_mut().call(Method::ShiftMode, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.shift_mode().unwrap();

                    if val == ctx.shift_mode {
                        return;
                    }

                    ctx.as_mut().rust_mut().shift_mode = val;
                    ctx.as_mut().shift_mode_changed();

                    ctx.state_changed("shiftMode".into());
                });
            }

            MethodTy::ShiftModeSupported => {
                self.as_mut()
                    .call(Method::ShiftModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.shift_mode_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().shift_mode_supported = val;
                        ctx.as_mut().shift_mode_supported_changed();

                        ctx.state_changed("shiftModeSupported".into());
                    });
            }

            MethodTy::BatteryChargeMode => {
                self.as_mut()
                    .call(Method::BatteryChargeMode, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.battery_charge_mode().unwrap();

                        if val == ctx.battery_charge_mode {
                            return;
                        }

                        ctx.as_mut().rust_mut().battery_charge_mode = val;
                        ctx.as_mut().battery_charge_mode_changed();

                        ctx.state_changed("batteryChargeMode".into());
                    });
            }

            MethodTy::BatteryChargeModeSupported => {
                self.as_mut()
                    .call(Method::BatteryChargeModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.battery_charge_mode_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().battery_charge_mode_supported = val;
                        ctx.as_mut().battery_charge_mode_supported_changed();

                        ctx.state_changed("batteryChargeModeSupported".into());
                    });
            }

            MethodTy::SuperBattery => {
                self.as_mut().call(Method::SuperBattery, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.super_battery().unwrap();

                    if val == ctx.super_battery {
                        return;
                    }

                    ctx.as_mut().rust_mut().super_battery = val;
                    ctx.as_mut().super_battery_changed();

                    ctx.state_changed("superBattery".into());
                });
            }

            MethodTy::SuperBatterySupported => {
                self.as_mut()
                    .call(Method::SuperBatterySupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.super_battery_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().super_battery_supported = val;
                        ctx.as_mut().super_battery_supported_changed();

                        ctx.state_changed("superBatterySupported".into());
                    });
            }

            MethodTy::Fan1Rpm => {
                self.as_mut().call(Method::Fan1Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.word().unwrap();

                    if val == ctx.fan1_rpm {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan1_rpm = val;
                    ctx.as_mut().fan1_rpm_changed();

                    ctx.state_changed("fan1Rpm".into());
                });
            }

            MethodTy::Fan2Rpm => {
                self.as_mut().call(Method::Fan2Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.word().unwrap();

                    if val == ctx.fan2_rpm {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan2_rpm = val;
                    ctx.as_mut().fan2_rpm_changed();

                    ctx.state_changed("fan2Rpm".into());
                });
            }

            MethodTy::Fan3Rpm => {
                self.as_mut().call(Method::Fan3Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.word().unwrap();

                    if val == ctx.fan3_rpm {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan3_rpm = val;
                    ctx.as_mut().fan3_rpm_changed();

                    ctx.state_changed("fan3Rpm".into());
                });
            }

            MethodTy::Fan4Rpm => {
                self.as_mut().call(Method::Fan4Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.word().unwrap();

                    if val == ctx.fan4_rpm {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan4_rpm = val;
                    ctx.as_mut().fan4_rpm_changed();

                    ctx.state_changed("fan4Rpm".into());
                });
            }

            MethodTy::Fan1Supported => {
                self.as_mut().call(Method::Fan1Supported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.state().unwrap();

                    if val == ctx.fan1_supported {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan1_supported = val;
                    ctx.as_mut().fan1_supported_changed();

                    ctx.state_changed("fan1Supported".into());
                });
            }

            MethodTy::Fan2Supported => {
                self.as_mut().call(Method::Fan2Supported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.state().unwrap();

                    if val == ctx.fan2_supported {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan2_supported = val;
                    ctx.as_mut().fan2_supported_changed();

                    ctx.state_changed("fan2Supported".into());
                });
            }

            MethodTy::Fan3Supported => {
                self.as_mut().call(Method::Fan3Supported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.state().unwrap();

                    if val == ctx.fan3_supported {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan3_supported = val;
                    ctx.as_mut().fan3_supported_changed();

                    ctx.state_changed("fan3Supported".into());
                });
            }

            MethodTy::Fan4Supported => {
                self.as_mut().call(Method::Fan4Supported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.state().unwrap();

                    if val == ctx.fan4_supported {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan4_supported = val;
                    ctx.as_mut().fan4_supported_changed();

                    ctx.state_changed("fan4Supported".into());
                });
            }

            MethodTy::FanModes => {
                self.as_mut().call(Method::FanModes, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.fan_modes().unwrap();

                    if val == ctx.fan_modes {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan_modes = val;
                    ctx.as_mut().fan_modes_changed();

                    ctx.state_changed("fanModes".into());
                });
            }

            MethodTy::FanMode => {
                self.as_mut().call(Method::FanMode, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.fan_mode().unwrap();

                    if val == ctx.fan_mode {
                        return;
                    }

                    ctx.as_mut().rust_mut().fan_mode = val;
                    ctx.as_mut().fan_mode_changed();

                    ctx.state_changed("fanMode".into());
                });
            }

            MethodTy::FanModeSupported => {
                self.as_mut()
                    .call(Method::FanModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.fan_mode_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().fan_mode_supported = val;
                        ctx.as_mut().fan_mode_supported_changed();

                        ctx.state_changed("fanModeSupported".into());
                    });
            }

            MethodTy::Webcam => {
                self.as_mut().call(Method::Webcam, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.webcam().unwrap();

                    if val == ctx.webcam {
                        return;
                    }

                    ctx.as_mut().rust_mut().webcam = val;
                    ctx.as_mut().webcam_changed();

                    ctx.state_changed("webcam".into());
                });
            }

            MethodTy::WebcamBlock => {
                self.as_mut().call(Method::WebcamBlock, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.webcam().unwrap();

                    if val == ctx.webcam_block {
                        return;
                    }

                    ctx.as_mut().rust_mut().webcam_block = val;
                    ctx.as_mut().webcam_block_changed();

                    ctx.state_changed("webcamBlock".into());
                });
            }

            MethodTy::WebcamSupported => {
                self.as_mut().call(Method::WebcamSupported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.state().unwrap();

                    if val == ctx.webcam_supported {
                        return;
                    }

                    ctx.as_mut().rust_mut().webcam_supported = val;
                    ctx.as_mut().webcam_supported_changed();

                    ctx.state_changed("webcamSupported".into());
                });
            }

            MethodTy::WebcamBlockSupported => {
                self.as_mut()
                    .call(Method::WebcamBlockSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.webcam_block_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().webcam_block_supported = val;
                        ctx.as_mut().webcam_block_supported_changed();

                        ctx.state_changed("webcamBlockSupported".into());
                    });
            }

            MethodTy::CoolerBoost => {
                self.as_mut().call(Method::CoolerBoost, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.cooler_boost().unwrap();

                    if val == ctx.cooler_boost {
                        return;
                    }

                    ctx.as_mut().rust_mut().cooler_boost = val;
                    ctx.as_mut().cooler_boost_changed();

                    ctx.state_changed("coolerBoost".into());
                });
            }

            MethodTy::CoolerBoostSupported => {
                self.as_mut()
                    .call(Method::CoolerBoostSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.cooler_boost_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().cooler_boost_supported = val;
                        ctx.as_mut().cooler_boost_supported_changed();

                        ctx.state_changed("coolerBoostSupported".into());
                    });
            }

            MethodTy::FnKey => {
                self.as_mut().call(Method::FnKey, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.key_direction().unwrap();
                    let val = SwapKey::from_fn(val);

                    if val == ctx.swap_key {
                        return;
                    }

                    ctx.as_mut().rust_mut().swap_key = val;
                    ctx.as_mut().fn_key_changed();
                    ctx.as_mut().win_key_changed();

                    ctx.as_mut().state_changed("fnKey".into());
                    ctx.state_changed("winKey".into());
                });
            }

            MethodTy::WinKey => {
                self.as_mut().call(Method::WinKey, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.key_direction().unwrap();
                    let val = SwapKey::from_win(val);

                    if val == ctx.swap_key {
                        return;
                    }

                    ctx.as_mut().rust_mut().swap_key = val;
                    ctx.as_mut().fn_key_changed();
                    ctx.as_mut().win_key_changed();

                    ctx.as_mut().state_changed("fnKey".into());
                    ctx.state_changed("winKey".into());
                });
            }

            MethodTy::FnWinSwapSupported => {
                self.as_mut()
                    .call(Method::FnWinSwapSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.fn_win_swap_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().fn_win_swap_supported = val;
                        ctx.as_mut().fn_win_swap_supported_changed();

                        ctx.state_changed("fnWinSwapSupported".into());
                    });
            }

            MethodTy::MicMuteLed => {
                self.as_mut().call(Method::MicMuteLed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.led().unwrap();

                    if val == ctx.mic_mute_led {
                        return;
                    }

                    ctx.as_mut().rust_mut().mic_mute_led = val;
                    ctx.as_mut().mic_mute_led_changed();

                    ctx.state_changed("micMuteLed".into());
                });
            }

            MethodTy::MuteLed => {
                self.as_mut().call(Method::MuteLed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.led().unwrap();

                    if val == ctx.mute_led {
                        return;
                    }

                    ctx.as_mut().rust_mut().mute_led = val;
                    ctx.as_mut().mute_led_changed();

                    ctx.state_changed("muteLed".into());
                });
            }

            MethodTy::MicMuteLedSupported => {
                self.as_mut()
                    .call(Method::MicMuteLedSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.mic_mute_led_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().mic_mute_led_supported = val;
                        ctx.as_mut().mic_mute_led_supported_changed();

                        ctx.state_changed("micMuteLedSupported".into());
                    });
            }

            MethodTy::MuteLedSupported => {
                self.as_mut()
                    .call(Method::MuteLedSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.state().unwrap();

                        if val == ctx.mute_led_supported {
                            return;
                        }

                        ctx.as_mut().rust_mut().mute_led_supported = val;
                        ctx.as_mut().mute_led_supported_changed();

                        ctx.state_changed("muteLedSupported".into());
                    });
            }

            MethodTy::CpuRtFanSpeed => {
                self.as_mut().call(Method::CpuRtFanSpeed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.byte().unwrap();

                    if val == ctx.cpu_rt_fan_speed {
                        return;
                    }

                    ctx.as_mut().rust_mut().cpu_rt_fan_speed = val;
                    ctx.as_mut().cpu_rt_fan_speed_changed();

                    ctx.state_changed("cpuRtFanSpeed".into());
                });
            }

            MethodTy::CpuRtTemp => {
                self.as_mut().call(Method::CpuRtTemp, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.byte().unwrap();

                    if val == ctx.cpu_rt_temp {
                        return;
                    }

                    ctx.as_mut().rust_mut().cpu_rt_temp = val;
                    ctx.as_mut().cpu_rt_temp_changed();

                    ctx.state_changed("cpuRtTemp".into());
                });
            }

            MethodTy::GpuRtTemp => {
                self.as_mut().call(Method::GpuRtTemp, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.byte().unwrap();

                    if val == ctx.gpu_rt_temp {
                        return;
                    }

                    ctx.as_mut().rust_mut().gpu_rt_temp = val;
                    ctx.as_mut().gpu_rt_temp_changed();

                    ctx.state_changed("gpuRtTemp".into());
                });
            }

            MethodTy::GpuRtFanSpeed => {
                self.as_mut().call(Method::GpuRtFanSpeed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.byte().unwrap();

                    if val == ctx.gpu_rt_fan_speed {
                        return;
                    }

                    ctx.as_mut().rust_mut().gpu_rt_fan_speed = val;
                    ctx.as_mut().gpu_rt_fan_speed_changed();

                    ctx.state_changed("gpuRtFanSpeed".into());
                });
            }

            MethodTy::CpuFanCurveWmi2 => {
                self.as_mut().call(Method::CpuFanCurveWmi2, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.curve7().unwrap();

                    if val == ctx.cpu_fan_curve_wmi2 {
                        return;
                    }

                    ctx.as_mut().rust_mut().cpu_fan_curve_wmi2 = val;
                    ctx.as_mut().cpu_fan_curve_wmi2_changed();

                    ctx.state_changed("cpuFanCurveWmi2".into());
                });
            }

            MethodTy::CpuTempCurveWmi2 => {
                self.as_mut()
                    .call(Method::CpuTempCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.curve7().unwrap();

                        if val == ctx.cpu_temp_curve_wmi2 {
                            return;
                        }

                        ctx.as_mut().rust_mut().cpu_temp_curve_wmi2 = val;
                        ctx.as_mut().cpu_temp_curve_wmi2_changed();

                        ctx.state_changed("cpuTempCurveWmi2".into());
                    });
            }

            MethodTy::CpuHysteresisCurveWmi2 => {
                self.as_mut()
                    .call(Method::CpuHysteresisCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.curve6().unwrap();

                        if val == ctx.cpu_hysteresis_curve_wmi2 {
                            return;
                        }

                        ctx.as_mut().rust_mut().cpu_hysteresis_curve_wmi2 = val;
                        ctx.as_mut().cpu_hysteresis_curve_wmi2_changed();

                        ctx.state_changed("cpuHysteresisCurveWmi2".into());
                    });
            }

            MethodTy::GpuFanCurveWmi2 => {
                self.as_mut().call(Method::GpuFanCurveWmi2, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.curve7().unwrap();

                    if val == ctx.gpu_fan_curve_wmi2 {
                        return;
                    }

                    ctx.as_mut().rust_mut().gpu_fan_curve_wmi2 = val;
                    ctx.as_mut().gpu_fan_curve_wmi2_changed();

                    ctx.state_changed("gpuFanCurveWmi2".into());
                });
            }

            MethodTy::GpuTempCurveWmi2 => {
                self.as_mut()
                    .call(Method::GpuTempCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.curve7().unwrap();

                        if val == ctx.gpu_temp_curve_wmi2 {
                            return;
                        }

                        ctx.as_mut().rust_mut().gpu_temp_curve_wmi2 = val;
                        ctx.as_mut().gpu_temp_curve_wmi2_changed();

                        ctx.state_changed("gpuTempCurveWmi2".into());
                    });
            }

            MethodTy::GpuHysteresisCurveWmi2 => {
                self.as_mut()
                    .call(Method::GpuHysteresisCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let val = res.curve6().unwrap();

                        if val == ctx.gpu_hysteresis_curve_wmi2 {
                            return;
                        }

                        ctx.as_mut().rust_mut().gpu_hysteresis_curve_wmi2 = val;
                        ctx.as_mut().gpu_hysteresis_curve_wmi2_changed();

                        ctx.state_changed("gpuHysteresisCurveWmi2".into());
                    });
            }

            // sentinel for Methods
            MethodTy::MethodRead => {
                self.as_mut().call(Method::MethodList, |mut ctx, res| {
                    let Ok(ret) = res else {
                        return;
                    };

                    // so nothing gets duplicated
                    ctx.as_mut().rust_mut().methods.data.clear();

                    let list = ret.into_methods().unwrap();

                    let accepted_groups = [
                        [MethodOp::Read, MethodOp::Write],
                        [MethodOp::ReadBit, MethodOp::WriteBit],
                        [MethodOp::ReadRange, MethodOp::WriteRange],
                    ];

                    let len = list.len();
                    for (i, method) in list.into_iter().enumerate() {
                        let mut has_read = false;
                        let mut has_write = false;
                        for group in accepted_groups {
                            has_read = method.ops.iter().any(|op| *op == group[0]);
                            has_write = method.ops.iter().any(|op| *op == group[1]);

                            if has_read && has_write {
                                break;
                            }
                        }

                        if !has_read && !has_write {
                            q_warning!("skipping method {} since it doesn't have both read and write capability", method.method);
                            continue;
                        }

                        let read_op = *method
                            .ops
                            .iter()
                            .find(|op| {
                                matches!(
                                    op,
                                    MethodOp::Read | MethodOp::ReadBit | MethodOp::ReadRange
                                )
                            })
                            .unwrap();
                        let write_op = *method
                            .ops
                            .iter()
                            .find(|op| {
                                matches!(
                                    op,
                                    MethodOp::Write | MethodOp::WriteBit | MethodOp::WriteRange
                                )
                            })
                            .unwrap();

                        let mut payload = MethodPayload {
                            name: method.name.to_string(),
                            method: method.method.to_string(),
                            data: MethodData::Bit(false), // dummy for now
                            read_op,
                            write_op,
                        };

                        let last = i == len - 1;
                        ctx.as_mut().call(
                            Method::MethodRead {
                                method: method.method.clone(),
                                op: read_op,
                            },
                            move |mut ctx, res| {
                                let Ok(ret) = res else {
                                    if last {
                                        ctx.as_mut().methods_changed();
                                        ctx.state_changed("methods".into());
                                    }

                                    return;
                                };

                                let data = ret.method_data().unwrap();
                                payload.data = data; // replace dummy with real data

                                ctx.as_mut().rust_mut().methods.data.push(payload);
                                if last {
                                    ctx.as_mut().methods_changed();
                                    ctx.state_changed("methods".into());
                                }
                            },
                        );
                    }
                });
            }

            MethodTy::EcDumpRaw => {
                self.as_mut().call(Method::EcDumpRaw, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val = res.ec_dump().unwrap();

                    if val == ctx.ec_dump {
                        return;
                    }

                    ctx.as_mut().rust_mut().ec_dump = val;
                    ctx.as_mut().ec_dump_changed();

                    ctx.state_changed("ecDump".into());
                });
            }

            MethodTy::EcDumpPretty => {
                self.as_mut().call(Method::EcDumpPretty, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let val: QString = res.str().unwrap().into();

                    if val == ctx.ec_dump_pretty {
                        return;
                    }

                    ctx.as_mut().rust_mut().ec_dump_pretty = val;
                    ctx.as_mut().ec_dump_pretty_changed();

                    ctx.state_changed("ecDumpPretty".into());
                });
            }

            x => q_warning!("Unsupported update type {x:?}"),
        }
    }
}

macro_rules! update_fns {
    ($($fn_name:ident, $update_name:ident),*) => {
        $(
            fn $fn_name(self: Pin<&mut Self>) {
                self._update(MethodTy::$update_name);
            }
        )*
    };
}

// Invokables
impl qobject::EcchanClient {
    fn init_state(mut self: Pin<&mut Self>) {
        self.as_mut().init_state_changed(true);

        for name in MethodTy::iter() {
            if matches!(
                name,
                MethodTy::EcDumpRaw | MethodTy::EcDumpPretty | MethodTy::MethodList
            ) {
                continue;
            }

            self.as_mut()._update(name);
        }

        self.queued_call(|ctx| {
            ctx.init_state_changed(false);
        });
    }

    fn queue(self: Pin<&mut Self>, cb: &QJSValue) {
        if !cb.is_callable() {
            q_warning!("queue: passed in value is not a callable");
            return;
        }

        let cb = Sendable::new(cb.clone());
        self.queued_call(move |_| {
            let cb = cb.get_inner().expect("we're on the same thread");
            let value_list = QJSValueList::new();
            cb.call(&value_list);
        });
    }

    fn serialize(self: Pin<&mut Self>) -> QVariant {
        let Some(mut engine) = QQmlEngine::js_engine(&*self) else {
            q_critical!("js engine was null");
            return QVariant::default();
        };

        let mut obj = engine.as_mut().new_object();
        let mut pin = obj.pin_mut();

        pin.as_mut().set_property(
            &"shiftMode".into(),
            &QJSValue::from_str(&self.shift_mode.to_string()),
        );

        let battery_mode = match self.battery_charge_mode {
            BatteryChargeMode::Custom(threshold) => QJSValue::from_uint(threshold.as_end() as _),
            m => QJSValue::from_str(&m.to_string()),
        };

        pin.as_mut()
            .set_property(&"batteryChargeMode".into(), &battery_mode);

        pin.as_mut().set_property(
            &"superBattery".into(),
            &QJSValue::from_bool(self.super_battery.enabled()),
        );

        pin.as_mut().set_property(
            &"fanMode".into(),
            &QJSValue::from_str(&self.fan_mode.to_string()),
        );

        pin.as_mut().set_property(
            &"webcam".into(),
            &QJSValue::from_bool(self.webcam.enabled()),
        );

        pin.as_mut().set_property(
            &"webcamBlock".into(),
            &QJSValue::from_bool(self.webcam_block.enabled()),
        );

        pin.as_mut().set_property(
            &"coolerBoost".into(),
            &QJSValue::from_bool(self.cooler_boost.enabled()),
        );

        pin.as_mut().set_property(
            &"fnKey".into(),
            &QJSValue::from_str(&self.swap_key.get_fn().to_string()),
        );

        pin.as_mut().set_property(
            &"winKey".into(),
            &QJSValue::from_str(&self.swap_key.get_win().to_string()),
        );

        pin.as_mut().set_property(
            &"micMuteLed".into(),
            &QJSValue::from_bool(self.mic_mute_led.enabled()),
        );

        pin.as_mut().set_property(
            &"muteLed".into(),
            &QJSValue::from_bool(self.mute_led.enabled()),
        );

        pin.as_mut().set_property(
            &"cpuFanCurveWmi2".into(),
            &QJSValue::from_array(
                engine.as_mut(),
                &[
                    QJSValue::from_uint(self.cpu_fan_curve_wmi2.n1 as _),
                    QJSValue::from_uint(self.cpu_fan_curve_wmi2.n2 as _),
                    QJSValue::from_uint(self.cpu_fan_curve_wmi2.n3 as _),
                    QJSValue::from_uint(self.cpu_fan_curve_wmi2.n4 as _),
                    QJSValue::from_uint(self.cpu_fan_curve_wmi2.n5 as _),
                    QJSValue::from_uint(self.cpu_fan_curve_wmi2.n6 as _),
                    QJSValue::from_uint(self.cpu_fan_curve_wmi2.n7 as _),
                ],
            ),
        );

        pin.as_mut().set_property(
            &"cpuTempCurveWmi2".into(),
            &QJSValue::from_array(
                engine.as_mut(),
                &[
                    QJSValue::from_uint(self.cpu_temp_curve_wmi2.n1 as _),
                    QJSValue::from_uint(self.cpu_temp_curve_wmi2.n2 as _),
                    QJSValue::from_uint(self.cpu_temp_curve_wmi2.n3 as _),
                    QJSValue::from_uint(self.cpu_temp_curve_wmi2.n4 as _),
                    QJSValue::from_uint(self.cpu_temp_curve_wmi2.n5 as _),
                    QJSValue::from_uint(self.cpu_temp_curve_wmi2.n6 as _),
                    QJSValue::from_uint(self.cpu_temp_curve_wmi2.n7 as _),
                ],
            ),
        );

        pin.as_mut().set_property(
            &"cpuHysteresisCurveWmi2".into(),
            &QJSValue::from_array(
                engine.as_mut(),
                &[
                    QJSValue::from_uint(self.cpu_hysteresis_curve_wmi2.n1 as _),
                    QJSValue::from_uint(self.cpu_hysteresis_curve_wmi2.n2 as _),
                    QJSValue::from_uint(self.cpu_hysteresis_curve_wmi2.n3 as _),
                    QJSValue::from_uint(self.cpu_hysteresis_curve_wmi2.n4 as _),
                    QJSValue::from_uint(self.cpu_hysteresis_curve_wmi2.n5 as _),
                    QJSValue::from_uint(self.cpu_hysteresis_curve_wmi2.n6 as _),
                ],
            ),
        );

        pin.as_mut().set_property(
            &"gpuFanCurveWmi2".into(),
            &QJSValue::from_array(
                engine.as_mut(),
                &[
                    QJSValue::from_uint(self.gpu_fan_curve_wmi2.n1 as _),
                    QJSValue::from_uint(self.gpu_fan_curve_wmi2.n2 as _),
                    QJSValue::from_uint(self.gpu_fan_curve_wmi2.n3 as _),
                    QJSValue::from_uint(self.gpu_fan_curve_wmi2.n4 as _),
                    QJSValue::from_uint(self.gpu_fan_curve_wmi2.n5 as _),
                    QJSValue::from_uint(self.gpu_fan_curve_wmi2.n6 as _),
                    QJSValue::from_uint(self.gpu_fan_curve_wmi2.n7 as _),
                ],
            ),
        );

        pin.as_mut().set_property(
            &"gpuTempCurveWmi2".into(),
            &QJSValue::from_array(
                engine.as_mut(),
                &[
                    QJSValue::from_uint(self.gpu_temp_curve_wmi2.n1 as _),
                    QJSValue::from_uint(self.gpu_temp_curve_wmi2.n2 as _),
                    QJSValue::from_uint(self.gpu_temp_curve_wmi2.n3 as _),
                    QJSValue::from_uint(self.gpu_temp_curve_wmi2.n4 as _),
                    QJSValue::from_uint(self.gpu_temp_curve_wmi2.n5 as _),
                    QJSValue::from_uint(self.gpu_temp_curve_wmi2.n6 as _),
                    QJSValue::from_uint(self.gpu_temp_curve_wmi2.n7 as _),
                ],
            ),
        );

        pin.as_mut().set_property(
            &"gpuHysteresisCurveWmi2".into(),
            &QJSValue::from_array(
                engine.as_mut(),
                &[
                    QJSValue::from_uint(self.gpu_hysteresis_curve_wmi2.n1 as _),
                    QJSValue::from_uint(self.gpu_hysteresis_curve_wmi2.n2 as _),
                    QJSValue::from_uint(self.gpu_hysteresis_curve_wmi2.n3 as _),
                    QJSValue::from_uint(self.gpu_hysteresis_curve_wmi2.n4 as _),
                    QJSValue::from_uint(self.gpu_hysteresis_curve_wmi2.n5 as _),
                    QJSValue::from_uint(self.gpu_hysteresis_curve_wmi2.n6 as _),
                ],
            ),
        );

        let mut methods = engine.as_mut().new_object();
        for data in &self.methods.data {
            let val = match &data.data {
                MethodData::Bit(b) => QJSValue::from_bool(*b),
                MethodData::Byte(b) => QJSValue::from_uint(*b as _),
                MethodData::Range(items) => {
                    let items = items
                        .iter()
                        .map(|b| QJSValue::from_uint(*b as _))
                        .collect::<Vec<_>>();

                    QJSValue::from_array(engine.as_mut(), &items)
                }
            };

            methods
                .pin_mut()
                .set_property(&data.method.clone().into(), &val);
        }

        pin.as_mut().set_property(&"methods".into(), &methods);

        obj.to_qvariant()
    }

    fn apply(mut self: Pin<&mut Self>, data: &QJSValue) {
        if !data.is_object() {
            q_warning!("apply: only accept objects");
            return;
        }

        let shift_mode = data.get_property(&"shiftMode".into());
        if shift_mode.is_string() {
            let mode = shift_mode.to_qstring();
            self.as_mut().set_shift_mode(&mode);
        }

        let battery_mode = data.get_property(&"batteryChargeMode".into());
        if battery_mode.is_string() || battery_mode.is_number() {
            self.as_mut()
                .set_battery_charge_mode(battery_mode.to_qvariant());
        }

        let super_battery = data.get_property(&"superBattery".into());
        if super_battery.is_bool() {
            self.as_mut().set_super_battery(super_battery.to_bool());
        }

        let fan_mode = data.get_property(&"fanMode".into());
        if fan_mode.is_string() {
            let mode = fan_mode.to_qstring();
            self.as_mut().set_fan_mode(&mode);
        }

        let webcam = data.get_property(&"webcam".into());
        if webcam.is_bool() {
            self.as_mut().set_webcam(webcam.to_bool());
        }

        let webcam_block = data.get_property(&"webcamBlock".into());
        if webcam_block.is_bool() {
            self.as_mut().set_webcam_block(webcam_block.to_bool());
        }

        let cooler_boost = data.get_property(&"coolerBoost".into());
        if cooler_boost.is_bool() {
            self.as_mut().set_cooler_boost(cooler_boost.to_bool());
        }

        // there's no need to set both winkey and fnkey since they internally <map to the same setting
        let fn_key = data.get_property(&"fnKey".into());
        if fn_key.is_string() {
            let mode = fn_key.to_qstring();
            self.as_mut().set_fn_key(&mode);
        }

        let mic_mute_led = data.get_property(&"micMuteLed".into());
        if mic_mute_led.is_bool() {
            self.as_mut().set_mic_mute_led(mic_mute_led.to_bool());
        }

        let mute_led = data.get_property(&"muteLed".into());
        if mute_led.is_bool() {
            self.as_mut().set_mute_led(mute_led.to_bool());
        }

        let curve = data.get_property(&"cpuFanCurveWmi2".into());
        if curve.is_array() {
            self.as_mut().set_cpu_fan_curve_wmi2(&curve.to_qvariant());
        }

        let curve = data.get_property(&"cpuTempCurveWmi2".into());
        if curve.is_array() {
            self.as_mut().set_cpu_temp_curve_wmi2(&curve.to_qvariant());
        }

        let curve = data.get_property(&"cpuHysteresisCurveWmi2".into());
        if curve.is_array() {
            self.as_mut()
                .set_cpu_hysteresis_curve_wmi2(&curve.to_qvariant());
        }

        let curve = data.get_property(&"gpuFanCurveWmi2".into());
        if curve.is_array() {
            self.as_mut().set_gpu_fan_curve_wmi2(&curve.to_qvariant());
        }

        let curve = data.get_property(&"gpuTempCurveWmi2".into());
        if curve.is_array() {
            self.as_mut().set_gpu_temp_curve_wmi2(&curve.to_qvariant());
        }

        let curve = data.get_property(&"gpuHysteresisCurveWmi2".into());
        if curve.is_array() {
            self.as_mut()
                .set_gpu_hysteresis_curve_wmi2(&curve.to_qvariant());
        }

        let methods = data.get_property(&"methods".into());
        if methods.is_object() {
            // race condition here; method list cb fires, THEN adds read cb to the end of the queue
            // but this already added write requests before that, so this is placed before the data
            // is even available; you can listen to the signal to know if/when it finished

            static GUARD: LazyLock<Mutex<Vec<QMetaObjectConnection>>> =
                LazyLock::new(Mutex::default);

            let mut data = HashMap::new();

            // for (const method in methods)
            let mut iterator = QJSValueIterator::new(&methods);
            while iterator.pin_mut().next() {
                let val = iterator.value();
                let name = iterator.name();

                let md = if val.is_bool() {
                    MethodData::Bit(val.to_bool())
                } else if val.is_number() {
                    MethodData::Byte(val.to_uint() as u8)
                } else if val.is_array() {
                    let mut data = Vec::new();
                    let len = val.get_property(&"length".into()).to_uint();
                    for n in 0..len {
                        let elem = val.get_element(n);

                        if !elem.is_number() {
                            // bad array
                            continue;
                        }

                        let n = elem.to_uint();
                        if n > u8::MAX as u32 {
                            // not a u8
                            continue;
                        }

                        data.push(n as u8);
                    }

                    MethodData::Range(data)
                } else {
                    continue;
                };

                data.insert(name.to_string(), md);
            }

            let conn = self.as_mut().on_methods_changed(move |mut ctx| {
                // disconnect them all
                GUARD.lock().drain(..).for_each(|conn| {
                    conn.disconnect();
                });

                let Some(mut engine) = QQmlEngine::js_engine(&*ctx) else {
                    q_critical!("js engine was null");
                    return;
                };

                for method in ctx.methods.data.clone() {
                    let Some(val) = data.get(&method.method) else {
                        continue;
                    };

                    let js = match val {
                        MethodData::Bit(b) => QJSValue::from_bool(*b),
                        MethodData::Byte(b) => QJSValue::from_uint(*b as u32),
                        MethodData::Range(items) => {
                            let items = items
                                .iter()
                                .map(|i| QJSValue::from_uint(*i as u32))
                                .collect::<Vec<_>>();

                            QJSValue::from_array(engine.as_mut(), &items)
                        }
                    };

                    let method = method.method.clone();
                    ctx.as_mut().method_write(&method.into(), &js);
                }
            });

            let guard = conn.release();
            GUARD.lock().push(guard);
        }
    }

    fn update(self: Pin<&mut Self>, method: qobject::Method) {
        self._update(method.into());
    }

    update_fns! {
        update_fan_count, FanCount,
        update_fan_max, FanMax,
        update_has_dgpu, HasDGpu,
        update_wmi_ver, WmiVer,

        update_fw_version, FwVersion,
        update_fw_date, FwDate,
        update_fw_time, FwTime,

        update_shift_modes, ShiftModes,
        update_shift_mode, ShiftMode,
        update_shift_mode_supported, ShiftModeSupported,

        update_battery_charge_mode, BatteryChargeMode,
        update_battery_charge_mode_supported, BatteryChargeModeSupported,

        update_super_battery, SuperBattery,
        update_super_battery_supported, SuperBatterySupported,

        update_fan1_rpm, Fan1Rpm,
        update_fan2_rpm, Fan2Rpm,
        update_fan3_rpm, Fan3Rpm,
        update_fan4_rpm, Fan4Rpm,
        update_fan1_supported, Fan1Supported,
        update_fan2_supported, Fan2Supported,
        update_fan3_supported, Fan3Supported,
        update_fan4_supported, Fan4Supported,

        update_fan_modes, FanModes,
        update_fan_mode, FanMode,
        update_fan_mode_supported, FanModeSupported,

        update_webcam, Webcam,
        update_webcam_block, WebcamBlock,
        update_webcam_supported, WebcamSupported,
        update_webcam_block_supported, WebcamBlockSupported,

        update_cooler_boost, CoolerBoost,
        update_cooler_boost_supported, CoolerBoostSupported,

        update_fn_key, FnKey,
        update_win_key, WinKey,
        update_fn_win_swap_supported, FnWinSwapSupported,

        update_mic_mute_led, MicMuteLed,
        update_mute_led, MuteLed,
        update_mic_mute_led_supported, MicMuteLedSupported,
        update_mute_led_supported, MuteLedSupported,

        update_cpu_rt_fan_speed, CpuRtFanSpeed,
        update_cpu_rt_temp, CpuRtTemp,
        update_gpu_rt_fan_speed, GpuRtFanSpeed,
        update_gpu_rt_temp, GpuRtTemp,

        update_cpu_fan_curve_wmi2, CpuFanCurveWmi2,
        update_cpu_temp_curve_wmi2, CpuTempCurveWmi2,
        update_cpu_hysteresis_curve_wmi2, CpuHysteresisCurveWmi2,
        update_gpu_fan_curve_wmi2, GpuFanCurveWmi2,
        update_gpu_temp_curve_wmi2, GpuTempCurveWmi2,
        update_gpu_hysteresis_curve_wmi2, GpuHysteresisCurveWmi2,

        update_methods, MethodRead, // sentinel for Methods

        update_ec_dump, EcDumpRaw,
        update_ec_dump_pretty, EcDumpPretty
    }
}

// Properties
impl qobject::EcchanClient {
    fn set_connected(mut self: Pin<&mut Self>, connected: bool) {
        if connected && self.client.is_none() {
            if self.path.is_empty() {
                return;
            }

            let path = self.path.to_string();
            let client = match Client::new(&path, self.qt_thread()) {
                Ok(c) => c,
                Err(e) => {
                    q_warning!("{e}");
                    return;
                }
            };

            self.as_mut().rust_mut().client = Some(client);
            self.as_mut().rust_mut().connected = true;
            self.as_mut().connected_changed();

            self.as_mut().init_state();

            let qthread = self.qt_thread();

            // start heartbeat thread
            let (tx, rx) = channel();
            self.as_mut().rust_mut().heartbeats = Some(tx);

            thread::spawn(move || {
                let should_exit = Arc::new(AtomicBool::default());

                loop {
                    if should_exit.load(Ordering::Relaxed) {
                        break;
                    }

                    match rx.try_recv() {
                        Ok(_) | Err(TryRecvError::Disconnected) => break,
                        Err(TryRecvError::Empty) => (),
                    }

                    let should_exit = should_exit.clone();
                    let res = qthread.queue(move |ctx| {
                        ctx.call(Method::Ping, move |_, res| match res {
                            Ok(_) => (),
                            Err(e) => match e {
                                ClientError::Call { .. } | ClientError::Json { .. } => (),
                                ClientError::Io { .. } | ClientError::Eof => {
                                    should_exit.store(true, Ordering::Relaxed)
                                }
                            },
                        });
                    });

                    // probably destroyed qobject
                    if res.is_err() {
                        break;
                    }

                    thread::sleep(Duration::from_millis(1500));
                }
            });
        } else {
            self.as_mut().disconnect();
        }
    }

    fn get_has_dgpu(&self) -> bool {
        self.has_dgpu
    }

    fn get_fan_max(&self) -> u8 {
        self.fan_max
    }

    fn get_wmi_ver(&self) -> u8 {
        match self.wmi_ver {
            WmiVer::Wmi1 => 1,
            WmiVer::Wmi2 => 2,
        }
    }

    fn get_fw_version(&self) -> &QString {
        &self.fw_version
    }

    fn get_fw_date(&self) -> &QString {
        &self.fw_date
    }

    fn get_fw_time(&self) -> &QString {
        &self.fw_time
    }

    fn get_fan_count(&self) -> u8 {
        match self.fan_count {
            Fans::One => 1,
            Fans::Two => 2,
            Fans::Three => 3,
            Fans::Four => 4,
        }
    }

    fn get_shift_modes(&self) -> QStringList {
        let mut qlist = QStringList::default();

        for item in &self.shift_modes {
            qlist.append(item.to_string().into());
        }

        qlist
    }

    fn get_shift_mode(&self) -> QString {
        self.shift_mode.to_string().into()
    }

    fn set_shift_mode(mut self: Pin<&mut Self>, mode: &QString) {
        let mode = match ShiftMode::from_str(&mode.to_string()) {
            Ok(m) => m,
            Err(e) => {
                q_warning!("shift_mode: {e}");
                return;
            }
        };

        if mode == self.shift_mode {
            return;
        }

        self.as_mut()
            .call(Method::SetShiftMode { mode }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().shift_mode = mode;
                ctx.as_mut().shift_mode_changed();

                ctx.state_changed("shiftMode".into());
            });
    }

    fn get_shift_mode_supported(&self) -> bool {
        self.shift_mode_supported
    }

    fn get_battery_charge_mode(&self) -> QVariant {
        match self.battery_charge_mode {
            BatteryChargeMode::Healthy
            | BatteryChargeMode::Balanced
            | BatteryChargeMode::Mobility => {
                let s: QString = self.battery_charge_mode.to_string().into();
                QVariant::from(&s)
            }
            BatteryChargeMode::Custom(threshold) => QVariant::from(&threshold.as_end()),
        }
    }

    fn set_battery_charge_mode(mut self: Pin<&mut Self>, mode: QVariant) {
        let mode = if matches!(
            mode.type_id(),
            // anything number like, but not float
            QMetaTypeType::Int
                | QMetaTypeType::UInt
                | QMetaTypeType::LongLong
                | QMetaTypeType::ULongLong
                | QMetaTypeType::Long
                | QMetaTypeType::Short
                | QMetaTypeType::Char
                | QMetaTypeType::ULong
                | QMetaTypeType::UShort
                | QMetaTypeType::UChar
                | QMetaTypeType::SChar
        ) && let Some(mode) = mode.value::<u8>()
        {
            let Some(mode) = BatteryChargeMode::from_end(mode) else {
                q_warning!("battery_charge_mode: {mode} out of range; only accept 10..=100");
                return;
            };

            mode
        } else if mode.type_id() == QMetaTypeType::QString
            && let Some(mode) = mode.value::<QString>()
        {
            match BatteryChargeMode::from_str(&mode.to_string()) {
                Ok(m) => m,
                Err(e) => {
                    q_warning!("battery_charge_mode: {e}");
                    return;
                }
            }
        } else {
            q_warning!("battery_charge_mode: only string and number are supported");
            return;
        };

        if mode == self.battery_charge_mode {
            return;
        }

        self.as_mut().call(
            Method::SetBatteryChargeMode { mode },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().battery_charge_mode = mode;
                ctx.as_mut().battery_charge_mode_changed();

                ctx.state_changed("batteryChargeMode".into());
            },
        );
    }

    fn get_battery_charge_mode_supported(&self) -> bool {
        self.battery_charge_mode_supported
    }

    fn get_super_battery(&self) -> bool {
        self.super_battery.enabled()
    }

    fn set_super_battery(mut self: Pin<&mut Self>, state: bool) {
        let state = SuperBattery::from(state);

        if state == self.super_battery {
            return;
        }

        self.as_mut()
            .call(Method::SetSuperBattery { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().super_battery = state;
                ctx.as_mut().super_battery_changed();

                ctx.state_changed("superBattery".into());
            });
    }

    fn get_super_battery_supported(&self) -> bool {
        self.super_battery_supported
    }

    fn get_fan1_rpm(&self) -> u16 {
        self.fan1_rpm
    }

    fn get_fan2_rpm(&self) -> u16 {
        self.fan2_rpm
    }

    fn get_fan3_rpm(&self) -> u16 {
        self.fan3_rpm
    }

    fn get_fan4_rpm(&self) -> u16 {
        self.fan4_rpm
    }

    fn get_fan1_supported(&self) -> bool {
        self.fan1_supported
    }

    fn get_fan2_supported(&self) -> bool {
        self.fan2_supported
    }

    fn get_fan3_supported(&self) -> bool {
        self.fan3_supported
    }

    fn get_fan4_supported(&self) -> bool {
        self.fan4_supported
    }

    fn get_fan_modes(&self) -> QStringList {
        let mut list = QStringList::default();

        for mode in &self.fan_modes {
            list.append(mode.to_string().into());
        }

        list
    }

    fn get_fan_mode(&self) -> QString {
        self.fan_mode.to_string().into()
    }

    fn set_fan_mode(mut self: Pin<&mut Self>, mode: &QString) {
        let mode = match FanMode::from_str(&mode.to_string()) {
            Ok(m) => m,
            Err(e) => {
                q_warning!("fan_mode: {e}");
                return;
            }
        };

        if mode == self.fan_mode {
            return;
        }

        self.as_mut()
            .call(Method::SetFanMode { mode }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().fan_mode = mode;
                ctx.as_mut().fan_mode_changed();

                ctx.state_changed("fanMode".into());
            });
    }

    fn get_fan_mode_supported(&self) -> bool {
        self.fan_mode_supported
    }

    fn get_webcam(&self) -> bool {
        self.webcam.enabled()
    }

    fn get_webcam_block(&self) -> bool {
        self.webcam_block.enabled()
    }

    fn set_webcam(mut self: Pin<&mut Self>, state: bool) {
        let state = Webcam::from(state);

        if state == self.webcam {
            return;
        }

        self.as_mut()
            .call(Method::SetWebcam { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().webcam = state;
                ctx.as_mut().webcam_changed();

                ctx.state_changed("webcam".into());
            });
    }

    fn set_webcam_block(mut self: Pin<&mut Self>, state: bool) {
        let state = Webcam::from(state);

        if state == self.webcam_block {
            return;
        }

        self.as_mut()
            .call(Method::SetWebcamBlock { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().webcam_block = state;
                ctx.as_mut().webcam_block_changed();

                ctx.state_changed("webcamBlock".into());
            });
    }

    fn get_webcam_supported(&self) -> bool {
        self.webcam_supported
    }

    fn get_webcam_block_supported(&self) -> bool {
        self.webcam_block_supported
    }

    fn get_cooler_boost(&self) -> bool {
        self.cooler_boost.enabled()
    }

    fn set_cooler_boost(mut self: Pin<&mut Self>, state: bool) {
        let state = CoolerBoost::from(state);

        if state == self.cooler_boost {
            return;
        }

        self.as_mut()
            .call(Method::SetCoolerBoost { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cooler_boost = state;
                ctx.as_mut().cooler_boost_changed();

                ctx.state_changed("coolerBoost".into());
            });
    }

    fn get_cooler_boost_supported(&self) -> bool {
        self.cooler_boost_supported
    }

    fn get_fn_key(&self) -> QString {
        self.swap_key.get_fn().to_string().into()
    }

    fn get_win_key(&self) -> QString {
        self.swap_key.get_win().to_string().into()
    }

    fn set_fn_key(mut self: Pin<&mut Self>, dir: &QString) {
        let state = match KeyDirection::from_str(&dir.to_string()) {
            Ok(k) => k,
            Err(e) => {
                q_warning!("fn_key: {e}");
                return;
            }
        };

        let state = SwapKey::from_fn(state);

        if state == self.swap_key {
            return;
        }

        self.as_mut().call(
            Method::SetFnKey {
                state: state.get_fn(),
            },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().swap_key = state;
                ctx.as_mut().fn_key_changed();
                ctx.as_mut().win_key_changed();

                ctx.as_mut().state_changed("fnKey".into());
                ctx.state_changed("winKey".into());
            },
        );
    }

    fn set_win_key(mut self: Pin<&mut Self>, dir: &QString) {
        let state = match KeyDirection::from_str(&dir.to_string()) {
            Ok(k) => k,
            Err(e) => {
                q_warning!("win_key: {e}");
                return;
            }
        };

        let state = SwapKey::from_win(state);

        if state == self.swap_key {
            return;
        }

        self.as_mut().call(
            Method::SetWinKey {
                state: state.get_win(),
            },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().swap_key = state;
                ctx.as_mut().fn_key_changed();
                ctx.as_mut().win_key_changed();

                ctx.as_mut().state_changed("fnKey".into());
                ctx.state_changed("winKey".into());
            },
        );
    }

    fn get_fn_win_swap_supported(&self) -> bool {
        self.fn_win_swap_supported
    }

    fn get_mic_mute_led(&self) -> bool {
        self.mic_mute_led.enabled()
    }

    fn get_mute_led(&self) -> bool {
        self.mute_led.enabled()
    }

    fn set_mic_mute_led(mut self: Pin<&mut Self>, state: bool) {
        let state = Led::from(state);

        if state == self.mic_mute_led {
            return;
        }

        self.as_mut()
            .call(Method::SetMicMuteLed { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().mic_mute_led = state;
                ctx.as_mut().mic_mute_led_changed();

                ctx.state_changed("micMuteLed".into());
            });
    }

    fn set_mute_led(mut self: Pin<&mut Self>, state: bool) {
        let state = Led::from(state);

        if state == self.mute_led {
            return;
        }

        self.as_mut()
            .call(Method::SetMuteLed { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().mute_led = state;
                ctx.as_mut().mute_led_changed();

                ctx.state_changed("muteLed".into());
            });
    }

    fn get_mic_mute_led_supported(&self) -> bool {
        self.mic_mute_led_supported
    }

    fn get_mute_led_supported(&self) -> bool {
        self.mute_led_supported
    }

    fn get_cpu_rt_fan_speed(&self) -> u8 {
        self.cpu_rt_fan_speed
    }

    fn get_cpu_rt_temp(&self) -> u8 {
        self.cpu_rt_temp
    }

    fn get_gpu_rt_fan_speed(&self) -> u8 {
        self.gpu_rt_fan_speed
    }

    fn get_gpu_rt_temp(&self) -> u8 {
        self.gpu_rt_temp
    }

    fn get_cpu_fan_curve_wmi2(&self) -> QVariant {
        let Some(engine) = QQmlEngine::js_engine(self) else {
            q_critical!("js engine was null");
            return QVariant::default();
        };

        let curve: [u8; 7] = self.cpu_fan_curve_wmi2.into();
        let mut jsarray = engine.new_array(7);
        for (i, val) in curve.iter().enumerate() {
            jsarray
                .pin_mut()
                .set_element(i as u32, &QJSValue::from_uint(*val as u32));
        }

        jsarray.to_qvariant()
    }

    fn set_cpu_fan_curve_wmi2(mut self: Pin<&mut Self>, curve: &QVariant) {
        let Some(curve) = QJSValue::from_qvariant(curve) else {
            q_warning!("cpu_fan_curve_wmi2: only supports array[u8]");
            return;
        };

        if !curve.is_array() {
            q_warning!("cpu_fan_curve_wmi2: only supports array[u8]");
            return;
        }

        let mut pcurve = [0u8; 7];
        for n in 0..7 {
            let elem = curve.get_element(n);
            if !elem.is_number() {
                q_warning!(
                    "cpu_fan_curve_wmi2: found non-number in array; only supports array[u8]"
                );
                return;
            }

            let num = elem.to_uint();
            if num > u8::MAX as u32 {
                q_warning!(
                    "cpu_fan_curve_wmi2: number {num} exceeds u8 range; only supports array[u8]"
                );
                return;
            }

            pcurve[n as usize] = num as u8;
        }

        let curve = Curve7::from(pcurve);

        if curve == self.cpu_fan_curve_wmi2 {
            return;
        }

        self.as_mut()
            .call(Method::SetCpuFanCurveWmi2 { curve }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cpu_fan_curve_wmi2 = curve;
                ctx.as_mut().cpu_fan_curve_wmi2_changed();

                ctx.state_changed("cpuFanCurveWmi2".into());
            });
    }

    fn get_cpu_temp_curve_wmi2(&self) -> QVariant {
        let Some(engine) = QQmlEngine::js_engine(self) else {
            q_critical!("js engine was null");
            return QVariant::default();
        };

        let curve: [u8; 7] = self.cpu_temp_curve_wmi2.into();
        let mut jsarray = engine.new_array(7);
        for (i, val) in curve.iter().enumerate() {
            jsarray
                .pin_mut()
                .set_element(i as u32, &QJSValue::from_uint(*val as u32));
        }

        jsarray.to_qvariant()
    }

    fn set_cpu_temp_curve_wmi2(mut self: Pin<&mut Self>, curve: &QVariant) {
        let Some(curve) = QJSValue::from_qvariant(curve) else {
            q_warning!("cpu_temp_curve_wmi2: only supports array[u8]");
            return;
        };

        if !curve.is_array() {
            q_warning!("cpu_temp_curve_wmi2: only supports array[u8]");
            return;
        }

        let mut pcurve = [0u8; 7];
        for n in 0..7 {
            let elem = curve.get_element(n);
            if !elem.is_number() {
                q_warning!(
                    "cpu_temp_curve_wmi2: found non-number in array; only supports array[u8]"
                );
                return;
            }

            let num = elem.to_uint();
            if num > u8::MAX as u32 {
                q_warning!(
                    "cpu_temp_curve_wmi2: number {num} exceeds u8 range; only supports array[u8]"
                );
                return;
            }

            pcurve[n as usize] = num as u8;
        }

        let curve = Curve7::from(pcurve);

        if curve == self.cpu_temp_curve_wmi2 {
            return;
        }

        self.as_mut().call(
            Method::SetCpuTempCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cpu_temp_curve_wmi2 = curve;
                ctx.as_mut().cpu_temp_curve_wmi2_changed();

                ctx.state_changed("cpuTempCurveWmi2".into());
            },
        );
    }

    fn get_cpu_hysteresis_curve_wmi2(&self) -> QVariant {
        let Some(engine) = QQmlEngine::js_engine(self) else {
            q_critical!("js engine was null");
            return QVariant::default();
        };

        let curve: [u8; 6] = self.cpu_hysteresis_curve_wmi2.into();
        let mut jsarray = engine.new_array(6);
        for (i, val) in curve.iter().enumerate() {
            jsarray
                .pin_mut()
                .set_element(i as u32, &QJSValue::from_uint(*val as u32));
        }

        jsarray.to_qvariant()
    }

    fn set_cpu_hysteresis_curve_wmi2(mut self: Pin<&mut Self>, curve: &QVariant) {
        let Some(curve) = QJSValue::from_qvariant(curve) else {
            q_warning!("cpu_hysteresis_curve_wmi2: only supports array[u8]");
            return;
        };

        if !curve.is_array() {
            q_warning!("cpu_hysteresis_curve_wmi2: only supports array[u8]");
            return;
        }

        let mut pcurve = [0u8; 6];
        for n in 0..6 {
            let elem = curve.get_element(n);
            if !elem.is_number() {
                q_warning!(
                    "cpu_hysteresis_curve_wmi2: found non-number in array; only supports array[u8]"
                );
                return;
            }

            let num = elem.to_uint();
            if num > u8::MAX as u32 {
                q_warning!(
                    "cpu_hysteresis_curve_wmi2: number {num} exceeds u8 range; only supports array[u8]"
                );
                return;
            }

            pcurve[n as usize] = num as u8;
        }

        let curve = Curve6::from(pcurve);

        if curve == self.cpu_hysteresis_curve_wmi2 {
            return;
        }

        self.as_mut().call(
            Method::SetCpuHysteresisCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cpu_hysteresis_curve_wmi2 = curve;
                ctx.as_mut().cpu_hysteresis_curve_wmi2_changed();

                ctx.state_changed("cpuHysteresisCurveWmi2".into());
            },
        );
    }

    fn get_gpu_fan_curve_wmi2(&self) -> QVariant {
        let Some(engine) = QQmlEngine::js_engine(self) else {
            q_critical!("js engine was null");
            return QVariant::default();
        };

        let curve: [u8; 7] = self.gpu_fan_curve_wmi2.into();
        let mut jsarray = engine.new_array(7);
        for (i, val) in curve.iter().enumerate() {
            jsarray
                .pin_mut()
                .set_element(i as u32, &QJSValue::from_uint(*val as u32));
        }

        jsarray.to_qvariant()
    }

    fn set_gpu_fan_curve_wmi2(mut self: Pin<&mut Self>, curve: &QVariant) {
        let Some(curve) = QJSValue::from_qvariant(curve) else {
            q_warning!("gpu_fan_curve_wmi2: only supports array[u8]");
            return;
        };

        if !curve.is_array() {
            q_warning!("gpu_fan_curve_wmi2: only supports array[u8]");
            return;
        }

        let mut pcurve = [0u8; 7];
        for n in 0..7 {
            let elem = curve.get_element(n);
            if !elem.is_number() {
                q_warning!(
                    "gpu_fan_curve_wmi2: found non-number in array; only supports array[u8]"
                );
                return;
            }

            let num = elem.to_uint();
            if num > u8::MAX as u32 {
                q_warning!(
                    "gpu_fan_curve_wmi2: number {num} exceeds u8 range; only supports array[u8]"
                );
                return;
            }

            pcurve[n as usize] = num as u8;
        }

        let curve = Curve7::from(pcurve);

        if curve == self.gpu_fan_curve_wmi2 {
            return;
        }

        self.as_mut()
            .call(Method::SetGpuFanCurveWmi2 { curve }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().gpu_fan_curve_wmi2 = curve;
                ctx.as_mut().gpu_fan_curve_wmi2_changed();

                ctx.state_changed("gpuFanCurveWmi2".into());
            });
    }

    fn get_gpu_temp_curve_wmi2(&self) -> QVariant {
        let Some(engine) = QQmlEngine::js_engine(self) else {
            q_critical!("js engine was null");
            return QVariant::default();
        };

        let curve: [u8; 7] = self.gpu_temp_curve_wmi2.into();
        let mut jsarray = engine.new_array(7);
        for (i, val) in curve.iter().enumerate() {
            jsarray
                .pin_mut()
                .set_element(i as u32, &QJSValue::from_uint(*val as u32));
        }

        jsarray.to_qvariant()
    }

    fn set_gpu_temp_curve_wmi2(mut self: Pin<&mut Self>, curve: &QVariant) {
        let Some(curve) = QJSValue::from_qvariant(curve) else {
            q_warning!("gpu_temp_curve_wmi2: only supports array[u8]");
            return;
        };

        if !curve.is_array() {
            q_warning!("gpu_temp_curve_wmi2: only supports array[u8]");
            return;
        }

        let mut pcurve = [0u8; 7];
        for n in 0..7 {
            let elem = curve.get_element(n);
            if !elem.is_number() {
                q_warning!(
                    "gpu_temp_curve_wmi2: found non-number in array; only supports array[u8]"
                );
                return;
            }

            let num = elem.to_uint();
            if num > u8::MAX as u32 {
                q_warning!(
                    "gpu_temp_curve_wmi2: number {num} exceeds u8 range; only supports array[u8]"
                );
                return;
            }

            pcurve[n as usize] = num as u8;
        }

        let curve = Curve7::from(pcurve);

        if curve == self.gpu_temp_curve_wmi2 {
            return;
        }

        self.as_mut().call(
            Method::SetGpuTempCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().gpu_temp_curve_wmi2 = curve;
                ctx.as_mut().gpu_temp_curve_wmi2_changed();

                ctx.state_changed("gpuTempCurveWmi2".into());
            },
        );
    }

    fn get_gpu_hysteresis_curve_wmi2(&self) -> QVariant {
        let Some(engine) = QQmlEngine::js_engine(self) else {
            q_critical!("js engine was null");
            return QVariant::default();
        };

        let curve: [u8; 6] = self.gpu_hysteresis_curve_wmi2.into();
        let mut jsarray = engine.new_array(6);
        for (i, val) in curve.iter().enumerate() {
            jsarray
                .pin_mut()
                .set_element(i as u32, &QJSValue::from_uint(*val as u32));
        }

        jsarray.to_qvariant()
    }

    fn set_gpu_hysteresis_curve_wmi2(mut self: Pin<&mut Self>, curve: &QVariant) {
        let Some(curve) = QJSValue::from_qvariant(curve) else {
            q_warning!("gpu_hysteresis_curve_wmi2: only supports array[u8]");
            return;
        };

        if !curve.is_array() {
            q_warning!("gpu_hysteresis_curve_wmi2: only supports array[u8]");
            return;
        }

        let mut pcurve = [0u8; 6];
        for n in 0..6 {
            let elem = curve.get_element(n);
            if !elem.is_number() {
                q_warning!(
                    "gpu_hysteresis_curve_wmi2: found non-number in array; only supports array[u8]"
                );
                return;
            }

            let num = elem.to_uint();
            if num > u8::MAX as u32 {
                q_warning!(
                    "gpu_hysteresis_curve_wmi2: number {num} exceeds u8 range; only supports array[u8]"
                );
                return;
            }

            pcurve[n as usize] = num as u8;
        }

        let curve = Curve6::from(pcurve);

        if curve == self.gpu_hysteresis_curve_wmi2 {
            return;
        }

        self.as_mut().call(
            Method::SetGpuHysteresisCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().gpu_hysteresis_curve_wmi2 = curve;
                ctx.as_mut().gpu_hysteresis_curve_wmi2_changed();

                ctx.state_changed("gpuHysteresisCurveWmi2".into());
            },
        );
    }

    fn get_methods(&self) -> QVariant {
        let Some(mut engine) = QQmlEngine::js_engine(self) else {
            q_critical!("js engine was null");
            return QVariant::default();
        };

        let mut arr = engine.as_mut().new_array(self.methods.data.len() as u32);
        let mut pin = arr.pin_mut();

        for (i, method) in self.methods.data.iter().enumerate() {
            let mut inner = engine.as_mut().new_object();
            let mut inner_pin = inner.pin_mut();

            let val = match &method.data {
                MethodData::Bit(b) => QJSValue::from_bool(*b),
                MethodData::Byte(b) => QJSValue::from_uint(*b as u32),
                MethodData::Range(items) => {
                    let v = items
                        .iter()
                        .map(|b| QJSValue::from_uint(*b as u32))
                        .collect::<Vec<_>>();
                    QJSValue::from_array(engine.as_mut(), &v)
                }
            };

            inner_pin
                .as_mut()
                .set_property(&QString::from("value"), &val);

            inner_pin
                .as_mut()
                .set_property(&QString::from("name"), &QJSValue::from_str(&method.name));

            inner_pin.as_mut().set_property(
                &QString::from("method"),
                &QJSValue::from_str(&method.method),
            );

            pin.as_mut().set_element(i as u32, &inner);
        }

        arr.to_qvariant()
    }

    fn method_write(self: Pin<&mut Self>, method: &QString, value: &QJSValue) {
        let method = method.to_string();

        let (data, op) = {
            let Some(payload) = self.methods.data.iter().find(|m| m.method == method) else {
                q_warning!("method_write: method {method} not found");
                return;
            };

            let data = match payload.write_op {
                MethodOp::WriteBit => {
                    if value.is_bool() {
                        MethodData::Bit(value.to_bool())
                    } else {
                        q_warning!("method_write: expected bool for method {method}");
                        return;
                    }
                }

                MethodOp::Write => {
                    if value.is_number() {
                        let n = value.to_uint();
                        if n > u8::MAX as u32 {
                            q_warning!("method_write: value {n} exceeded u8 max");
                            return;
                        }

                        MethodData::Byte(n as u8)
                    } else {
                        q_warning!("method_write: expected u8 for method {method}");
                        return;
                    }
                }

                MethodOp::WriteRange => {
                    if value.is_array() {
                        let mut data = Vec::new();
                        let length = value.get_property(&"length".into()).to_int();
                        for i in 0..length {
                            let elem = value.get_element(i as u32);
                            if !elem.is_number() {
                                q_warning!("method_write: expected array[u8] for method {method}");
                                return;
                            }

                            let elem = elem.to_uint();
                            if elem > u8::MAX as u32 {
                                q_warning!("method_write: value {elem} exceeded u8 max");
                                return;
                            }

                            data.push(elem as u8);
                        }

                        MethodData::Range(data)
                    } else {
                        q_warning!("method_write: expected array[u8] for method {method}");
                        return;
                    }
                }

                _ => unreachable!(),
            };

            // do not bother with a write if the api is already set to this
            if data == payload.data {
                return;
            }

            (data, payload.write_op)
        };

        self.call(
            Method::MethodWrite {
                method: Cow::Owned(method.clone()),
                op,
                data: data.clone(),
            },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                {
                    let mut this = ctx.as_mut().rust_mut();
                    let payload = this
                        .methods
                        .data
                        .iter_mut()
                        .find(|m| m.method == method)
                        .unwrap();

                    payload.data = data;
                }

                ctx.as_mut().methods_changed();
                ctx.state_changed("methods".into());
            },
        );
    }

    fn get_ec_dump(&self) -> QByteArray {
        QByteArray::from(&self.ec_dump.0)
    }

    fn get_ec_dump_pretty(&self) -> &QString {
        &self.ec_dump_pretty
    }
}
