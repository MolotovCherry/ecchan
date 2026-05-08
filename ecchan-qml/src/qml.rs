use std::{
    borrow::Cow,
    collections::HashMap,
    io,
    pin::Pin,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, TryRecvError, channel},
    },
    thread,
    time::Duration,
};

use cxx::UniquePtr;
use cxx_qt::{Constructor, CxxQtType, Threading};
use cxx_qt_lib::{
    QByteArray, QList, QMap, QMapPair as _, QMapPair_QString_QVariant, QObjectExt, QString,
    QStringList, QVariant, QVariantValue,
};
use ecchan_ipc::{
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, Fans, KeyDirection, Led,
    Method as CustomMethod, MethodData, MethodOp, ShiftMode, SuperBattery, Webcam, WmiVer,
    method::{Method, MethodTy},
    ret::{Bin, RetVal},
};
use strum::IntoEnumIterator as _;

use crate::{
    client::{Client, ClientError},
    q_critical, q_warning,
    qqml_property_map::{QQmlPropertyMap, QVariantConvertQQmlPropertyMap},
    setup::setup,
};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qlist.h");
        type QList_QString = cxx_qt_lib::QList<QString>;
        type QList_u8 = cxx_qt_lib::QList<u8>;
        type QList_QVariant = cxx_qt_lib::QList<QVariant>;

        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
    }

    unsafe extern "C++" {
        include!("ecchan-client/qqml_property_map.h");
        type QQmlPropertyMap = crate::qqml_property_map::QQmlPropertyMap;
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
        #[qproperty(QList_QString, shift_modes, READ = get_shift_modes, NOTIFY, FINAL)]
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
        #[qproperty(QList_QString, fan_modes, READ = get_fan_modes, NOTIFY, FINAL)]
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
        #[qproperty(QList_u8, cpu_fan_curve_wmi2, READ = get_cpu_fan_curve_wmi2, WRITE = set_cpu_fan_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QList_u8, cpu_temp_curve_wmi2, READ = get_cpu_temp_curve_wmi2, WRITE = set_cpu_temp_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QList_u8, cpu_hysteresis_curve_wmi2, READ = get_cpu_hysteresis_curve_wmi2, WRITE = set_cpu_hysteresis_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QList_u8, gpu_fan_curve_wmi2, READ = get_gpu_fan_curve_wmi2, WRITE = set_gpu_fan_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QList_u8, gpu_temp_curve_wmi2, READ = get_gpu_temp_curve_wmi2, WRITE = set_gpu_temp_curve_wmi2, NOTIFY, FINAL)]
        #[qproperty(QList_u8, gpu_hysteresis_curve_wmi2, READ = get_gpu_hysteresis_curve_wmi2, WRITE = set_gpu_hysteresis_curve_wmi2, NOTIFY, FINAL)]
        // methods
        #[qproperty(QList_QVariant, method_list, READ = get_method_list, NOTIFY, FINAL)]
        #[qproperty(*mut QQmlPropertyMap, methods, READ = get_methods, NOTIFY, FINAL)]
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

        fn get_shift_modes(&self) -> QList_QString;
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

        fn get_fan_modes(&self) -> QList_QString;
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

        fn get_cpu_fan_curve_wmi2(&self) -> QList_u8;
        fn set_cpu_fan_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn get_cpu_temp_curve_wmi2(&self) -> QList_u8;
        fn set_cpu_temp_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn get_cpu_hysteresis_curve_wmi2(&self) -> QList_u8;
        fn set_cpu_hysteresis_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn get_gpu_fan_curve_wmi2(&self) -> QList_u8;
        fn set_gpu_fan_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn get_gpu_temp_curve_wmi2(&self) -> QList_u8;
        fn set_gpu_temp_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn get_gpu_hysteresis_curve_wmi2(&self) -> QList_u8;
        fn set_gpu_hysteresis_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);

        fn get_method_list(&self) -> QList_QVariant;
        fn get_methods(&self) -> *mut QQmlPropertyMap;

        fn get_ec_dump(&self) -> QByteArray;
        fn get_ec_dump_pretty(&self) -> &QString;

        //
        // Signals
        //

        #[qsignal]
        fn init_state_changed(self: Pin<&mut Self>, running: bool);

        //
        // Invokables
        //

        #[qinvokable]
        fn init_state(self: Pin<&mut Self>);

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
        fn update_method_list(self: Pin<&mut Self>);
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
        MethodList,
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
            50 => MethodTy::MethodList,
            51 => MethodTy::Methods,
            _ => unreachable!(),
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

    fn_key: KeyDirection,
    win_key: KeyDirection,
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

    method_list: Vec<CustomMethod<'static>>,
    methods: Methods,

    ec_dump: Box<Bin>,
    ec_dump_pretty: QString,
}

struct Methods {
    map: UniquePtr<QQmlPropertyMap>,
    children: HashMap<String, UniquePtr<QQmlPropertyMap>>,
    cache: HashMap<String, MethodData>,
}

impl Default for EcchanClientRust {
    fn default() -> Self {
        Self {
            heartbeats: None,

            client: None,
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

            fn_key: KeyDirection::Left,
            win_key: KeyDirection::Right,
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

            method_list: Vec::new(),
            methods: Methods {
                map: QQmlPropertyMap::new(),
                children: HashMap::new(),
                cache: HashMap::new(),
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

                    ctx.as_mut().rust_mut().fan_count = res.fans().unwrap();
                    ctx.fan_count_changed();
                });
            }

            MethodTy::FanMax => {
                self.as_mut().call(Method::FanMax, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan_max = res.byte().unwrap();
                    ctx.fan_max_changed();
                });
            }

            MethodTy::HasDGpu => {
                self.as_mut().call(Method::HasDGpu, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().has_dgpu = res.state().unwrap();
                    ctx.has_dgpu_changed();
                });
            }

            MethodTy::WmiVer => {
                self.as_mut().call(Method::WmiVer, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };
                    ctx.as_mut().rust_mut().wmi_ver = res.wmi_ver().unwrap();
                    ctx.wmi_ver_changed();
                });
            }

            MethodTy::FwVersion => {
                self.as_mut().call(Method::FwVersion, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fw_version = res.str().unwrap().into();
                    ctx.fw_version_changed();
                });
            }

            MethodTy::FwDate => {
                self.as_mut().call(Method::FwDate, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fw_date = res.str().unwrap().into();
                    ctx.fw_date_changed();
                });
            }

            MethodTy::FwTime => {
                self.as_mut().call(Method::FwTime, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };
                    ctx.as_mut().rust_mut().fw_time = res.str().unwrap().into();
                    ctx.fw_time_changed();
                });
            }

            MethodTy::ShiftModes => {
                self.as_mut().call(Method::ShiftModes, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().shift_modes = res.shift_modes().unwrap();
                    ctx.shift_modes_changed();
                });
            }

            MethodTy::ShiftMode => {
                self.as_mut().call(Method::ShiftMode, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().shift_mode = res.shift_mode().unwrap();
                    ctx.shift_mode_changed();
                });
            }

            MethodTy::ShiftModeSupported => {
                self.as_mut()
                    .call(Method::ShiftModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().shift_mode_supported = res.state().unwrap();
                        ctx.shift_mode_supported_changed();
                    });
            }

            MethodTy::BatteryChargeMode => {
                self.as_mut()
                    .call(Method::BatteryChargeMode, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().battery_charge_mode =
                            res.battery_charge_mode().unwrap();
                        ctx.battery_charge_mode_changed();
                    });
            }

            MethodTy::BatteryChargeModeSupported => {
                self.as_mut()
                    .call(Method::BatteryChargeModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().battery_charge_mode_supported =
                            res.state().unwrap();
                        ctx.battery_charge_mode_supported_changed();
                    });
            }

            MethodTy::SuperBattery => {
                self.as_mut().call(Method::SuperBattery, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().super_battery = res.super_battery().unwrap();
                    ctx.super_battery_changed();
                });
            }

            MethodTy::SuperBatterySupported => {
                self.as_mut()
                    .call(Method::SuperBatterySupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().super_battery_supported = res.state().unwrap();
                        ctx.super_battery_supported_changed();
                    });
            }

            MethodTy::Fan1Rpm => {
                self.as_mut().call(Method::Fan1Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan1_rpm = res.word().unwrap();
                    ctx.fan1_rpm_changed();
                });
            }

            MethodTy::Fan2Rpm => {
                self.as_mut().call(Method::Fan2Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan2_rpm = res.word().unwrap();
                    ctx.fan2_rpm_changed();
                });
            }

            MethodTy::Fan3Rpm => {
                self.as_mut().call(Method::Fan3Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan3_rpm = res.word().unwrap();
                    ctx.fan3_rpm_changed();
                });
            }

            MethodTy::Fan4Rpm => {
                self.as_mut().call(Method::Fan4Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan4_rpm = res.word().unwrap();
                    ctx.fan4_rpm_changed();
                });
            }

            MethodTy::Fan1Supported => {
                self.as_mut().call(Method::Fan1Supported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan1_supported = res.state().unwrap();
                    ctx.fan1_supported_changed();
                });
            }

            MethodTy::Fan2Supported => {
                self.as_mut().call(Method::Fan2Supported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan2_supported = res.state().unwrap();
                    ctx.fan2_supported_changed();
                });
            }

            MethodTy::Fan3Supported => {
                self.as_mut().call(Method::Fan3Supported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan3_supported = res.state().unwrap();
                    ctx.fan3_supported_changed();
                });
            }

            MethodTy::Fan4Supported => {
                self.as_mut().call(Method::Fan4Supported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan4_supported = res.state().unwrap();
                    ctx.fan4_supported_changed();
                });
            }

            MethodTy::FanModes => {
                self.as_mut().call(Method::FanModes, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan_modes = res.fan_modes().unwrap();
                    ctx.fan_modes_changed();
                });
            }

            MethodTy::FanMode => {
                self.as_mut().call(Method::FanMode, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan_mode = res.fan_mode().unwrap();
                    ctx.fan_mode_changed();
                });
            }

            MethodTy::FanModeSupported => {
                self.as_mut()
                    .call(Method::FanModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().fan_mode_supported = res.state().unwrap();
                        ctx.fan_mode_supported_changed();
                    });
            }

            MethodTy::Webcam => {
                self.as_mut().call(Method::Webcam, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().webcam = res.webcam().unwrap();
                    ctx.webcam_changed();
                });
            }

            MethodTy::WebcamBlock => {
                self.as_mut().call(Method::WebcamBlock, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().webcam_block = res.webcam().unwrap();
                    ctx.webcam_block_changed();
                });
            }

            MethodTy::WebcamSupported => {
                self.as_mut().call(Method::WebcamSupported, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().webcam_supported = res.state().unwrap();
                    ctx.webcam_supported_changed();
                });
            }

            MethodTy::WebcamBlockSupported => {
                self.as_mut()
                    .call(Method::WebcamBlockSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().webcam_block_supported = res.state().unwrap();
                        ctx.webcam_block_supported_changed();
                    });
            }

            MethodTy::CoolerBoost => {
                self.as_mut().call(Method::CoolerBoost, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().cooler_boost = res.cooler_boost().unwrap();
                    ctx.cooler_boost_changed();
                });
            }

            MethodTy::CoolerBoostSupported => {
                self.as_mut()
                    .call(Method::CoolerBoostSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().cooler_boost_supported = res.state().unwrap();
                        ctx.cooler_boost_supported_changed();
                    });
            }

            MethodTy::FnKey => {
                self.as_mut().call(Method::FnKey, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fn_key = res.key_direction().unwrap();
                    ctx.fn_key_changed();
                });
            }

            MethodTy::WinKey => {
                self.as_mut().call(Method::WinKey, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().win_key = res.key_direction().unwrap();
                    ctx.win_key_changed();
                });
            }

            MethodTy::FnWinSwapSupported => {
                self.as_mut()
                    .call(Method::FnWinSwapSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().fn_win_swap_supported = res.state().unwrap();
                        ctx.fn_win_swap_supported_changed();
                    });
            }

            MethodTy::MicMuteLed => {
                self.as_mut().call(Method::MicMuteLed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().mic_mute_led = res.led().unwrap();
                    ctx.mic_mute_led_changed();
                });
            }

            MethodTy::MuteLed => {
                self.as_mut().call(Method::MuteLed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().mute_led = res.led().unwrap();
                    ctx.mute_led_changed();
                });
            }

            MethodTy::MicMuteLedSupported => {
                self.as_mut()
                    .call(Method::MicMuteLedSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().mic_mute_led_supported = res.state().unwrap();
                        ctx.mic_mute_led_supported_changed();
                    });
            }

            MethodTy::MuteLedSupported => {
                self.as_mut()
                    .call(Method::MuteLedSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().mute_led_supported = res.state().unwrap();
                        ctx.mute_led_supported_changed();
                    });
            }

            MethodTy::CpuRtFanSpeed => {
                self.as_mut().call(Method::CpuRtFanSpeed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().cpu_rt_fan_speed = res.byte().unwrap();
                    ctx.cpu_rt_fan_speed_changed();
                });
            }

            MethodTy::CpuRtTemp => {
                self.as_mut().call(Method::CpuRtTemp, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().cpu_rt_temp = res.byte().unwrap();
                    ctx.cpu_rt_temp_changed();
                });
            }

            MethodTy::GpuRtTemp => {
                self.as_mut().call(Method::GpuRtTemp, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().gpu_rt_temp = res.byte().unwrap();
                    ctx.gpu_rt_temp_changed();
                });
            }

            MethodTy::GpuRtFanSpeed => {
                self.as_mut().call(Method::GpuRtFanSpeed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().gpu_rt_fan_speed = res.byte().unwrap();
                    ctx.gpu_rt_fan_speed_changed();
                });
            }

            MethodTy::CpuFanCurveWmi2 => {
                self.as_mut().call(Method::CpuFanCurveWmi2, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().cpu_fan_curve_wmi2 = res.curve7().unwrap();
                    ctx.cpu_fan_curve_wmi2_changed();
                });
            }

            MethodTy::CpuTempCurveWmi2 => {
                self.as_mut()
                    .call(Method::CpuTempCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().cpu_temp_curve_wmi2 = res.curve7().unwrap();
                        ctx.cpu_temp_curve_wmi2_changed();
                    });
            }

            MethodTy::CpuHysteresisCurveWmi2 => {
                self.as_mut()
                    .call(Method::CpuHysteresisCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().cpu_hysteresis_curve_wmi2 = res.curve6().unwrap();
                        ctx.cpu_hysteresis_curve_wmi2_changed();
                    });
            }

            MethodTy::GpuFanCurveWmi2 => {
                self.as_mut().call(Method::GpuFanCurveWmi2, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().gpu_fan_curve_wmi2 = res.curve7().unwrap();
                    ctx.gpu_fan_curve_wmi2_changed();
                });
            }

            MethodTy::GpuTempCurveWmi2 => {
                self.as_mut()
                    .call(Method::GpuTempCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().gpu_temp_curve_wmi2 = res.curve7().unwrap();
                        ctx.gpu_temp_curve_wmi2_changed();
                    });
            }

            MethodTy::GpuHysteresisCurveWmi2 => {
                self.as_mut()
                    .call(Method::GpuHysteresisCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().gpu_hysteresis_curve_wmi2 = res.curve6().unwrap();
                        ctx.gpu_hysteresis_curve_wmi2_changed();
                    });
            }

            MethodTy::MethodList => {
                self.as_mut().call(Method::MethodList, move |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    let method_list = res.into_methods().unwrap();

                    ctx.as_mut().rust_mut().method_list = method_list;
                    ctx.as_mut().method_list_changed();
                });
            }

            MethodTy::Methods => {
                // this is queued because it is called after methodList, and is expected to work in order
                self.as_mut().queued_call(|mut ctx| {
                    // > 1 because objectName property is added by default
                    if ctx.method_list.is_empty() {
                        return;
                    }

                    // fast path

                    // we already initialized, so just query values and update the relevant maps
                    if ctx.method_list.len() > 1 {
                        let method_list = ctx.method_list.clone();

                        let updated: Arc<AtomicBool> = Arc::default();
                        for method in method_list.iter().cloned() {
                            let read = method.ops.iter().find(|o| {
                                matches!(o, MethodOp::Read | MethodOp::ReadBit | MethodOp::ReadRange)
                            });

                            let Some(read) = read else {
                                continue;
                            };

                            let _method = method.method.clone().into_owned();
                            ctx.as_mut().call(Method::MethodRead { method: method.method, op: *read }, {
                                let updated = updated.clone();
                                move |mut ctx, res| {
                                let Ok(ret) = res else {
                                    return;
                                };

                                let data = ret.method_data().unwrap();
                                ctx.as_mut().rust_mut().methods.cache.entry(_method.clone()).and_modify({
                                    let data = data.clone();
                                    |v| {
                                    if data != *v {
                                        *v = data;
                                        updated.store(true, Ordering::Relaxed);
                                    }
                                }}).or_insert_with(|| data.clone());

                                // data changed; we should update the actual map
                                if updated.load(Ordering::Relaxed) {
                                    let mut this = ctx.as_mut().rust_mut();
                                    let m = this.methods.children.get_mut(&_method).unwrap();

                                    let variant = match data {
                                        MethodData::Bit(b) => QVariant::from(&b),
                                        MethodData::Byte(b) => QVariant::from(&b),
                                        MethodData::Range(items) => {
                                            let arr = QByteArray::from(&*items);
                                            QVariant::from(&arr)
                                        }
                                    };

                                    m.pin_mut().insert(&"value".into(), &variant);
                                }
                            }});
                        }

                        ctx.queued_call(move |ctx| {
                            if updated.load(Ordering::Relaxed) {
                                ctx.methods_changed();
                            }
                        });

                        return;
                    }

                    // cold path

                    let method_list = ctx.method_list.clone();
                    let mut list = Vec::with_capacity(ctx.method_list.len());

                    list.resize_with(list.capacity(), || {
                        let mut map = QQmlPropertyMap::new();
                        map.pin_mut()
                            .set_parent(ctx.as_mut().rust_mut().methods.map.pin_mut());
                        map
                    });

                    let last = method_list.len().saturating_sub(1);
                    for (i, (method, mut map)) in method_list.into_iter().zip(list).enumerate() {
                        let is_last_iter = i == last;

                        let name = QString::from(&*method.method);

                        let is_read = method.ops.iter().any(|o| {
                            matches!(o, MethodOp::Read | MethodOp::ReadBit | MethodOp::ReadRange)
                        });

                        let is_write = method.ops.iter().any(|o| {
                            matches!(
                                o,
                                MethodOp::Write | MethodOp::WriteBit | MethodOp::WriteRange
                            )
                        });

                        let string_op = if let Some(op) = method.ops.first() {
                            let op = match op {
                                MethodOp::ReadBit | MethodOp::WriteBit => "Bit",
                                MethodOp::Read | MethodOp::Write => "Byte",
                                MethodOp::ReadRange | MethodOp::WriteRange => "Range",
                            };

                            Some(op)
                        } else {
                            None
                        };

                        // set custom slot to react to value setting
                        map.pin_mut()
                            .on_value_changed({
                                let qthread = ctx.qt_thread();
                                let op = method.ops.iter().find(|op| matches!(op, MethodOp::Write | MethodOp::WriteBit | MethodOp::WriteRange)).copied();
                                let method = method.method.clone().into_owned();

                                move |_, key, value| {
                                    let key = key.to_string();
                                    if key !=  "value" {
                                        q_warning!("custom method {method}'s key {key} should not be set by user; setting anything else may cause unexpected failures; please only set `value`");
                                        return;
                                    }

                                    if !is_write {
                                        q_warning!("custom method {method} does not support writes");
                                        return;
                                    }

                                    let Some(op) = op else {
                                        q_warning!("no write ops were found for custom method {method}");
                                        return;
                                    };

                                    match op {
                                        MethodOp::WriteBit => {
                                            let Some(state) = QVariant::value::<bool>(value) else {
                                                q_warning!("custom method {method} received an unsupported type; please use `bool` for setting");
                                                return;
                                            };

                                            _ = qthread.queue({
                                                let method: Cow<str> = Cow::Owned(method.clone());
                                                move |mut ctx| {
                                                    ctx.as_mut().call(Method::MethodWrite { method: method.clone(), op, data: MethodData::Bit(state) }, move |mut ctx, res| {
                                                        if res.is_err() {
                                                            // get previous value; don't update the cache since it failed
                                                            let prev = ctx.as_ref().rust().methods.cache.get(&*method).cloned();
                                                            if let Some(prev) = prev && let Some(child) = ctx.as_mut().rust_mut().methods.children.get_mut(&*method) {
                                                                child.pin_mut().insert(&"value".into(), &QVariant::from(&prev.as_bit()));
                                                            }
                                                        } else {
                                                            // update the previous cache to new value
                                                            ctx.as_mut().rust_mut().methods.cache.insert(method.into(), MethodData::Bit(state));
                                                        }
                                                    });

                                                }
                                            });
                                        }

                                        MethodOp::Write => {
                                            let Some(byte) = QVariant::value::<u8>(value) else {
                                                q_warning!("custom method {method} received an unsupported type; please use `number` (u8) for setting");
                                                return;
                                            };

                                            _ = qthread.queue({
                                                let method: Cow<str> = Cow::Owned(method.clone());
                                                move |mut ctx| {
                                                ctx.as_mut().call(Method::MethodWrite { method: method.clone(), op, data: MethodData::Byte(byte) }, move |mut ctx, res| {
                                                        if res.is_err() {
                                                            // get previous value; don't update the cache since it failed
                                                            let prev = ctx.as_ref().rust().methods.cache.get(&*method).cloned();
                                                            if let Some(prev) = prev && let Some(child) = ctx.as_mut().rust_mut().methods.children.get_mut(&*method) {
                                                                child.pin_mut().insert(&"value".into(), &QVariant::from(&prev.as_byte()));
                                                            }
                                                        } else {
                                                            // update the previous cache to new value
                                                            ctx.as_mut().rust_mut().methods.cache.insert(method.into(), MethodData::Byte(byte));
                                                        }
                                                    });
                                                }
                                            });
                                        }

                                        MethodOp::WriteRange => {
                                            let Some(bytes) = QVariant::value::<QByteArray>(value) else {
                                                q_warning!("custom method {method} received an unsupported type; please use a byte array for setting");
                                                return;
                                            };

                                            let bytes = bytes.as_slice().to_vec();

                                            _ = qthread.queue({
                                                let method: Cow<str> = Cow::Owned(method.clone());
                                                move |mut ctx| {
                                                ctx.as_mut().call(Method::MethodWrite { method: method.clone(), op, data: MethodData::Range(bytes.clone()) }, move |mut ctx, res| {
                                                        if res.is_err() {
                                                            // get previous value; don't update the cache since it failed
                                                            let prev = ctx.as_ref().rust().methods.cache.get(&*method).cloned();
                                                            if let Some(prev) = prev && let Some(child) = ctx.as_mut().rust_mut().methods.children.get_mut(&*method) {
                                                                child.pin_mut().insert(&"value".into(), &QVariant::from(&QByteArray::from(prev.as_range())));
                                                            }
                                                        } else {
                                                            // update the previous cache to new value
                                                            ctx.as_mut().rust_mut().methods.cache.insert(method.into(), MethodData::Range(bytes));
                                                        }
                                                    });
                                                }
                                            });
                                        }

                                        _ => {
                                            q_warning!("entered unreachable code");
                                        }
                                    }
                                }
                            })
                            .release();

                        let variant = unsafe { map.as_qvariant() };
                        ctx.as_mut()
                            .rust_mut()
                            .methods
                            .map
                            .pin_mut()
                            .insert(&name, &variant);

                        // insert map as child to keep it alive for qvariant above
                        ctx.as_mut()
                            .rust_mut()
                            .methods
                            .children
                            .insert(method.method.clone().into_owned(), map);

                        let op = method.ops.into_iter().find(|o| {
                            matches!(o, MethodOp::Read | MethodOp::ReadBit | MethodOp::ReadRange)
                        });

                        let _method = method.method.clone().into_owned();
                        let finish = move |mut ctx: Pin<&mut qobject::EcchanClient>| {
                            let mut this = ctx.as_mut().rust_mut();
                            let map = this.methods.children.get_mut(&*_method).unwrap();

                            let mut pin_map = map.pin_mut();

                            if let Some(op) = string_op {
                                pin_map
                                    .as_mut()
                                    .insert(&"type".into(), &QVariant::from(&QString::from(op)));
                            }

                            pin_map
                                .as_mut()
                                .insert(&"read".into(), &QVariant::from(&is_read));

                            pin_map
                                .as_mut()
                                .insert(&"write".into(), &QVariant::from(&is_write));

                            pin_map.as_mut().freeze();
                        };

                        match op {
                            Some(op) => {
                                let _method = method.method.clone().into_owned();

                                ctx.as_mut().call(
                                    Method::MethodRead {
                                        method: method.method.clone(),
                                        op,
                                    },
                                    move |mut ctx, res| {
                                        if let Ok(res) = res {
                                            let data = res.method_data().unwrap();

                                            ctx.as_mut()
                                                .rust_mut()
                                                .methods
                                                .cache
                                                .insert(_method.clone(), data.clone());

                                            let variant = match data {
                                                MethodData::Bit(b) => QVariant::from(&b),
                                                MethodData::Byte(b) => QVariant::from(&b),
                                                MethodData::Range(items) => {
                                                    let arr = QByteArray::from(&*items);
                                                    QVariant::from(&arr)
                                                }
                                            };

                                            let mut this = ctx.as_mut().rust_mut();
                                            let map = this.methods.children.get_mut(&*_method).unwrap();
                                            map.pin_mut().insert(&"value".into(), &variant);

                                            finish(ctx.as_mut());
                                            if is_last_iter {
                                                ctx.as_mut().methods_changed();
                                            }
                                        } else {
                                            finish(ctx.as_mut());
                                            if is_last_iter {
                                                ctx.as_mut().methods_changed();
                                            }
                                        }
                                    },
                                );
                            }

                            None => {
                                finish(ctx.as_mut());

                                if is_last_iter {
                                    ctx.as_mut().methods_changed();
                                }
                            }
                        }
                    }

                    // no more changes thx
                    ctx.as_mut().rust_mut().methods.map.pin_mut().freeze();
                });
            }

            MethodTy::EcDumpRaw => {
                self.as_mut().call(Method::EcDumpRaw, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().ec_dump = res.ec_dump().unwrap();
                    ctx.ec_dump_changed();
                });
            }

            MethodTy::EcDumpPretty => {
                self.as_mut().call(Method::EcDumpPretty, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().ec_dump_pretty = res.str().unwrap().into();
                    ctx.ec_dump_pretty_changed();
                });
            }
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
            self.as_mut()._update(name);
        }

        self.queued_call(|ctx| {
            ctx.init_state_changed(false);
        });
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

        update_method_list, MethodList,
        update_methods, Methods,

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

    fn get_shift_modes(&self) -> QList<QString> {
        let mut qlist = QList::default();

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

        self.as_mut()
            .call(Method::SetShiftMode { mode }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().shift_mode = mode;
                ctx.shift_mode_changed();
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
                <QString as QVariantValue>::construct(&self.battery_charge_mode.to_string().into())
            }
            BatteryChargeMode::Custom(threshold) => {
                <u8 as QVariantValue>::construct(&threshold.as_end())
            }
        }
    }

    fn set_battery_charge_mode(mut self: Pin<&mut Self>, mode: QVariant) {
        if let Some(mode) = mode.value::<u8>() {
            let Some(mode) = BatteryChargeMode::from_end(mode) else {
                q_warning!("battery_charge_mode: {mode} out of range; only accept 10..=100");
                return;
            };

            self.as_mut().call(
                Method::SetBatteryChargeMode { mode },
                move |mut ctx, res| {
                    if res.is_err() {
                        return;
                    }

                    ctx.as_mut().rust_mut().battery_charge_mode = mode;
                    ctx.battery_charge_mode_changed();
                },
            );
        } else if let Some(mode) = mode.value::<QString>() {
            let mode = match BatteryChargeMode::from_str(&mode.to_string()) {
                Ok(m) => m,
                Err(e) => {
                    q_warning!("battery_charge_mode: {e}");
                    return;
                }
            };

            self.as_mut().call(
                Method::SetBatteryChargeMode { mode },
                move |mut ctx, res| {
                    if res.is_err() {
                        return;
                    }

                    ctx.as_mut().rust_mut().battery_charge_mode = mode;
                    ctx.battery_charge_mode_changed();
                },
            );
        } else {
            q_warning!("battery_charge_mode: only string and number are supported");
        }
    }

    fn get_battery_charge_mode_supported(&self) -> bool {
        self.battery_charge_mode_supported
    }

    fn get_super_battery(&self) -> bool {
        self.super_battery.enabled()
    }

    fn set_super_battery(mut self: Pin<&mut Self>, state: bool) {
        let state = SuperBattery::from(state);

        self.as_mut()
            .call(Method::SetSuperBattery { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().super_battery = state;
                ctx.super_battery_changed();
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

    fn get_fan_modes(&self) -> QList<QString> {
        let mut list = QList::default();

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

        self.as_mut()
            .call(Method::SetFanMode { mode }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().fan_mode = mode;
                ctx.fan_mode_changed();
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

        self.as_mut()
            .call(Method::SetWebcam { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().webcam = state;
                ctx.webcam_changed();
            });
    }

    fn set_webcam_block(mut self: Pin<&mut Self>, state: bool) {
        let state = Webcam::from(state);

        self.as_mut()
            .call(Method::SetWebcamBlock { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().webcam_block = state;
                ctx.webcam_block_changed();
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

        self.as_mut()
            .call(Method::SetCoolerBoost { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cooler_boost = state;
                ctx.cooler_boost_changed();
            });
    }

    fn get_cooler_boost_supported(&self) -> bool {
        self.cooler_boost_supported
    }

    fn get_fn_key(&self) -> QString {
        self.fn_key.to_string().into()
    }

    fn get_win_key(&self) -> QString {
        self.win_key.to_string().into()
    }

    fn set_fn_key(mut self: Pin<&mut Self>, dir: &QString) {
        let state = match KeyDirection::from_str(&dir.to_string()) {
            Ok(k) => k,
            Err(e) => {
                q_warning!("fn_key: {e}");
                return;
            }
        };

        self.as_mut()
            .call(Method::SetFnKey { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().fn_key = state;
                ctx.fn_key_changed();
            });
    }

    fn set_win_key(mut self: Pin<&mut Self>, dir: &QString) {
        let state = match KeyDirection::from_str(&dir.to_string()) {
            Ok(k) => k,
            Err(e) => {
                q_warning!("win_key: {e}");
                return;
            }
        };

        self.as_mut()
            .call(Method::SetWinKey { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().win_key = state;
                ctx.win_key_changed();
            });
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

        self.as_mut()
            .call(Method::SetMicMuteLed { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().mic_mute_led = state;
                ctx.mic_mute_led_changed();
            });
    }

    fn set_mute_led(mut self: Pin<&mut Self>, state: bool) {
        let state = Led::from(state);

        self.as_mut()
            .call(Method::SetMuteLed { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().mute_led = state;
                ctx.mute_led_changed();
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

    fn get_cpu_fan_curve_wmi2(&self) -> QList<u8> {
        let mut list = QList::default();

        let curve = self.cpu_fan_curve_wmi2;
        list.extend([
            curve.n1, curve.n2, curve.n3, curve.n4, curve.n5, curve.n6, curve.n7,
        ]);

        list
    }

    fn set_cpu_fan_curve_wmi2(mut self: Pin<&mut Self>, curve: QList<u8>) {
        let len = curve.len();
        if len != 7 {
            q_warning!("cpu_fan_curve_wmi2: need array of len 7, instead got len {len}");
            return;
        }

        let curve = Curve7 {
            n1: curve.get(0).copied().unwrap(),
            n2: curve.get(1).copied().unwrap(),
            n3: curve.get(2).copied().unwrap(),
            n4: curve.get(3).copied().unwrap(),
            n5: curve.get(4).copied().unwrap(),
            n6: curve.get(5).copied().unwrap(),
            n7: curve.get(6).copied().unwrap(),
        };

        self.as_mut()
            .call(Method::SetCpuFanCurveWmi2 { curve }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cpu_fan_curve_wmi2 = curve;
                ctx.cpu_fan_curve_wmi2_changed();
            });
    }

    fn get_cpu_temp_curve_wmi2(&self) -> QList<u8> {
        let mut list = QList::default();

        let curve = self.cpu_temp_curve_wmi2;
        list.extend([
            curve.n1, curve.n2, curve.n3, curve.n4, curve.n5, curve.n6, curve.n7,
        ]);

        list
    }

    fn set_cpu_temp_curve_wmi2(mut self: Pin<&mut Self>, curve: QList<u8>) {
        let len = curve.len();
        if len != 7 {
            q_warning!("cpu_temp_curve_wmi2: need array of len 7, instead got len {len}");
            return;
        }

        let curve = Curve7 {
            n1: curve.get(0).copied().unwrap(),
            n2: curve.get(1).copied().unwrap(),
            n3: curve.get(2).copied().unwrap(),
            n4: curve.get(3).copied().unwrap(),
            n5: curve.get(4).copied().unwrap(),
            n6: curve.get(5).copied().unwrap(),
            n7: curve.get(6).copied().unwrap(),
        };

        self.as_mut().call(
            Method::SetCpuTempCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cpu_temp_curve_wmi2 = curve;
                ctx.cpu_temp_curve_wmi2_changed();
            },
        );
    }

    fn get_cpu_hysteresis_curve_wmi2(&self) -> QList<u8> {
        let mut list = QList::default();

        let curve = self.cpu_hysteresis_curve_wmi2;
        list.extend([curve.n1, curve.n2, curve.n3, curve.n4, curve.n5, curve.n6]);

        list
    }

    fn set_cpu_hysteresis_curve_wmi2(mut self: Pin<&mut Self>, curve: QList<u8>) {
        let len = curve.len();
        if len != 6 {
            q_warning!("cpu_hysteresis_curve_wmi2: need array of len 6, instead got len {len}");
            return;
        }

        let curve = Curve6 {
            n1: curve.get(0).copied().unwrap(),
            n2: curve.get(1).copied().unwrap(),
            n3: curve.get(2).copied().unwrap(),
            n4: curve.get(3).copied().unwrap(),
            n5: curve.get(4).copied().unwrap(),
            n6: curve.get(5).copied().unwrap(),
        };

        self.as_mut().call(
            Method::SetCpuHysteresisCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cpu_hysteresis_curve_wmi2 = curve;
                ctx.cpu_hysteresis_curve_wmi2_changed();
            },
        );
    }

    fn get_gpu_fan_curve_wmi2(&self) -> QList<u8> {
        let mut list = QList::default();

        let curve = self.gpu_fan_curve_wmi2;
        list.extend([
            curve.n1, curve.n2, curve.n3, curve.n4, curve.n5, curve.n6, curve.n7,
        ]);

        list
    }

    fn set_gpu_fan_curve_wmi2(mut self: Pin<&mut Self>, curve: QList<u8>) {
        let len = curve.len();
        if len != 7 {
            q_warning!("gpu_fan_curve_wmi2: need array of len 7, instead got len {len}");
            return;
        }

        let curve = Curve7 {
            n1: curve.get(0).copied().unwrap(),
            n2: curve.get(1).copied().unwrap(),
            n3: curve.get(2).copied().unwrap(),
            n4: curve.get(3).copied().unwrap(),
            n5: curve.get(4).copied().unwrap(),
            n6: curve.get(5).copied().unwrap(),
            n7: curve.get(6).copied().unwrap(),
        };

        self.as_mut()
            .call(Method::SetGpuFanCurveWmi2 { curve }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().gpu_fan_curve_wmi2 = curve;
                ctx.gpu_fan_curve_wmi2_changed();
            });
    }

    fn get_gpu_temp_curve_wmi2(&self) -> QList<u8> {
        let mut list = QList::default();

        let curve = self.gpu_temp_curve_wmi2;
        list.extend([
            curve.n1, curve.n2, curve.n3, curve.n4, curve.n5, curve.n6, curve.n7,
        ]);

        list
    }

    fn set_gpu_temp_curve_wmi2(mut self: Pin<&mut Self>, curve: QList<u8>) {
        let len = curve.len();
        if len != 7 {
            q_warning!("gpu_temp_curve_wmi2: need array of len 7, instead got len {len}");
            return;
        }

        let curve = Curve7 {
            n1: curve.get(0).copied().unwrap(),
            n2: curve.get(1).copied().unwrap(),
            n3: curve.get(2).copied().unwrap(),
            n4: curve.get(3).copied().unwrap(),
            n5: curve.get(4).copied().unwrap(),
            n6: curve.get(5).copied().unwrap(),
            n7: curve.get(6).copied().unwrap(),
        };

        self.as_mut().call(
            Method::SetGpuTempCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().gpu_temp_curve_wmi2 = curve;
                ctx.gpu_temp_curve_wmi2_changed();
            },
        );
    }

    fn get_gpu_hysteresis_curve_wmi2(&self) -> QList<u8> {
        let mut list = QList::default();

        let curve = self.gpu_hysteresis_curve_wmi2;
        list.extend([curve.n1, curve.n2, curve.n3, curve.n4, curve.n5, curve.n6]);

        list
    }

    fn set_gpu_hysteresis_curve_wmi2(mut self: Pin<&mut Self>, curve: QList<u8>) {
        let len = curve.len();
        if len != 6 {
            q_warning!("gpu_hysteresis_curve_wmi2: need array of len 6, instead got len {len}");
            return;
        }

        let curve = Curve6 {
            n1: curve.get(0).copied().unwrap(),
            n2: curve.get(1).copied().unwrap(),
            n3: curve.get(2).copied().unwrap(),
            n4: curve.get(3).copied().unwrap(),
            n5: curve.get(4).copied().unwrap(),
            n6: curve.get(5).copied().unwrap(),
        };

        self.as_mut().call(
            Method::SetGpuHysteresisCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().gpu_hysteresis_curve_wmi2 = curve;
                ctx.gpu_hysteresis_curve_wmi2_changed();
            },
        );
    }

    fn get_method_list(&self) -> QList<QVariant> {
        let mut list = QList::default();

        for m in &self.method_list {
            let mut map = QMapPair_QString_QVariant::default();

            let name = QString::construct(&(&*m.name).into());
            let method = QString::construct(&(&*m.method).into());

            map.insert("name".into(), name);
            map.insert("method".into(), method);

            let mut ops = QStringList::default();
            for op in &m.ops {
                let qs = QString::from(op.to_string());
                ops.append(qs);
            }

            let ops = QStringList::construct(&ops);
            map.insert("ops".into(), ops);

            let variant = <QMap<QMapPair_QString_QVariant> as QVariantValue>::construct(&map);
            list.append(variant);
        }

        list
    }

    fn get_methods(&self) -> *mut QQmlPropertyMap {
        self.methods.map.as_mut_ptr()
    }

    fn get_ec_dump(&self) -> QByteArray {
        QByteArray::from(&self.ec_dump.0)
    }

    fn get_ec_dump_pretty(&self) -> &QString {
        &self.ec_dump_pretty
    }
}
