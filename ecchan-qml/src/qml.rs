use std::{
    borrow::Cow,
    collections::HashMap,
    io,
    pin::Pin,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, TryRecvError, channel, sync_channel},
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
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, Fans, KeyDirection, Led, Method,
    MethodData, MethodOp, ShiftMode, SuperBattery, Webcam, WmiVer,
    method::Method as MethodCall,
    ret::{Bin, RetVal},
};

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
        #[qproperty(bool, connected, READ, WRITE = set_connected, NOTIFY)]
        #[qproperty(QString, path, READ, WRITE, NOTIFY)]
        // utils
        #[qproperty(u8, fan_count, READ = fan_count, NOTIFY)]
        #[qproperty(u8, fan_max, READ, NOTIFY)]
        #[qproperty(bool, has_dgpu, READ, NOTIFY)]
        #[qproperty(u8, wmi_ver, READ = wmi_ver, NOTIFY)]
        // fw
        #[qproperty(QString, fw_version, READ, NOTIFY)]
        #[qproperty(QString, fw_date, READ, NOTIFY)]
        #[qproperty(QString, fw_time, READ, NOTIFY)]
        // shift mode
        #[qproperty(QList_QString, shift_modes, READ = shift_modes, NOTIFY)]
        #[qproperty(QString, shift_mode, READ = shift_mode, WRITE = set_shift_mode, NOTIFY)]
        #[qproperty(bool, shift_mode_supported, READ, NOTIFY)]
        // battery charge mode
        #[qproperty(QVariant, battery_charge_mode, READ = battery_charge_mode, WRITE = set_battery_charge_mode, NOTIFY)]
        #[qproperty(bool, battery_charge_mode_supported, READ, NOTIFY)]
        // super battery
        #[qproperty(bool, super_battery, READ = super_battery, WRITE = set_super_battery, NOTIFY)]
        #[qproperty(bool, super_battery_supported, READ, NOTIFY)]
        // fan rpm
        #[qproperty(u16, fan1_rpm, READ, NOTIFY)]
        #[qproperty(u16, fan2_rpm, READ, NOTIFY)]
        #[qproperty(u16, fan3_rpm, READ, NOTIFY)]
        #[qproperty(u16, fan4_rpm, READ, NOTIFY)]
        #[qproperty(bool, fan1_supported, READ, NOTIFY)]
        #[qproperty(bool, fan2_supported, READ, NOTIFY)]
        #[qproperty(bool, fan3_supported, READ, NOTIFY)]
        #[qproperty(bool, fan4_supported, READ, NOTIFY)]
        // fan modes
        #[qproperty(QList_QString, fan_modes, READ = fan_modes, NOTIFY)]
        #[qproperty(QString, fan_mode, READ = fan_mode, WRITE = set_fan_mode, NOTIFY)]
        #[qproperty(bool, fan_mode_supported, READ, NOTIFY)]
        // webcam
        #[qproperty(bool, webcam, READ = webcam, WRITE = set_webcam, NOTIFY)]
        #[qproperty(bool, webcam_block, READ = webcam_block, WRITE = set_webcam_block, NOTIFY)]
        #[qproperty(bool, webcam_supported, READ, NOTIFY)]
        #[qproperty(bool, webcam_block_supported, READ, NOTIFY)]
        // cooler boost
        #[qproperty(bool, cooler_boost, READ = cooler_boost, WRITE = set_cooler_boost, NOTIFY)]
        #[qproperty(bool, cooler_boost_supported, READ, NOTIFY)]
        // fn/win key swap
        #[qproperty(QString, fn_key, READ = fn_key, WRITE = set_fn_key, NOTIFY)]
        #[qproperty(QString, win_key, READ = win_key, WRITE = set_win_key, NOTIFY)]
        #[qproperty(bool, fn_win_swap_supported, READ, NOTIFY)]
        // mute leds
        #[qproperty(bool, mic_mute_led, READ = mic_mute_led, WRITE = set_mic_mute_led, NOTIFY)]
        #[qproperty(bool, mute_led, READ = mute_led, WRITE = set_mute_led, NOTIFY)]
        #[qproperty(bool, mic_mute_led_supported, READ, NOTIFY)]
        #[qproperty(bool, mute_led_supported, READ, NOTIFY)]
        // rt sensors
        #[qproperty(u8, cpu_rt_fan_speed, READ, NOTIFY)]
        #[qproperty(u8, cpu_rt_temp, READ, NOTIFY)]
        #[qproperty(u8, gpu_rt_fan_speed, READ, NOTIFY)]
        #[qproperty(u8, gpu_rt_temp, READ, NOTIFY)]
        // curves
        #[qproperty(QList_u8, cpu_fan_curve_wmi2, READ = cpu_fan_curve_wmi2, WRITE = set_cpu_fan_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, cpu_temp_curve_wmi2, READ = cpu_temp_curve_wmi2, WRITE = set_cpu_temp_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, cpu_hysteresis_curve_wmi2, READ = cpu_hysteresis_curve_wmi2, WRITE = set_cpu_hysteresis_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, gpu_fan_curve_wmi2, READ = gpu_fan_curve_wmi2, WRITE = set_gpu_fan_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, gpu_temp_curve_wmi2, READ = gpu_temp_curve_wmi2, WRITE = set_gpu_temp_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, gpu_hysteresis_curve_wmi2, READ = gpu_hysteresis_curve_wmi2, WRITE = set_gpu_hysteresis_curve_wmi2, NOTIFY)]
        // methods
        #[qproperty(QList_QVariant, method_list, READ = method_list, NOTIFY)]
        #[qproperty(*mut QQmlPropertyMap, methods, READ = methods)]
        // dump
        #[qproperty(QByteArray, ec_dump, READ = ec_dump, NOTIFY)]
        #[qproperty(QString, ec_dump_pretty, READ, NOTIFY)]
        #[namespace = "ecchan_client"]
        type EcchanClient = super::EcchanClientRust;

        fn set_connected(self: Pin<&mut Self>, connected: bool);
        fn fan_count(&self) -> u8;
        fn wmi_ver(&self) -> u8;

        fn shift_modes(&self) -> QList_QString;
        fn shift_mode(&self) -> QString;
        fn set_shift_mode(self: Pin<&mut Self>, mode: &QString);

        fn battery_charge_mode(&self) -> QVariant;
        fn set_battery_charge_mode(self: Pin<&mut Self>, mode: QVariant);

        fn super_battery(&self) -> bool;
        fn set_super_battery(self: Pin<&mut Self>, state: bool);

        fn fan_modes(&self) -> QList_QString;
        fn fan_mode(&self) -> QString;
        fn set_fan_mode(self: Pin<&mut Self>, mode: &QString);

        fn webcam(&self) -> bool;
        fn webcam_block(&self) -> bool;
        fn set_webcam(self: Pin<&mut Self>, state: bool);
        fn set_webcam_block(self: Pin<&mut Self>, state: bool);

        fn cooler_boost(&self) -> bool;
        fn set_cooler_boost(self: Pin<&mut Self>, state: bool);

        fn fn_key(&self) -> QString;
        fn win_key(&self) -> QString;
        fn set_fn_key(self: Pin<&mut Self>, dir: &QString);
        fn set_win_key(self: Pin<&mut Self>, dir: &QString);

        fn mic_mute_led(&self) -> bool;
        fn mute_led(&self) -> bool;
        fn set_mic_mute_led(self: Pin<&mut Self>, state: bool);
        fn set_mute_led(self: Pin<&mut Self>, state: bool);

        fn cpu_fan_curve_wmi2(&self) -> QList_u8;
        fn set_cpu_fan_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn cpu_temp_curve_wmi2(&self) -> QList_u8;
        fn set_cpu_temp_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn cpu_hysteresis_curve_wmi2(&self) -> QList_u8;
        fn set_cpu_hysteresis_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn gpu_fan_curve_wmi2(&self) -> QList_u8;
        fn set_gpu_fan_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn gpu_temp_curve_wmi2(&self) -> QList_u8;
        fn set_gpu_temp_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);
        fn gpu_hysteresis_curve_wmi2(&self) -> QList_u8;
        fn set_gpu_hysteresis_curve_wmi2(self: Pin<&mut Self>, curve: QList_u8);

        fn method_list(&self) -> QList_QVariant;
        fn methods(&self) -> *mut QQmlPropertyMap;

        fn ec_dump(&self) -> QByteArray;

        #[qsignal]
        #[cxx_name = "initStateChanged"]
        fn init_state_changed(self: Pin<&mut Self>, running: bool);

        #[qinvokable]
        #[cxx_name = "initState"]
        fn init_state(self: Pin<&mut Self>);

        #[qinvokable]
        fn update(self: Pin<&mut Self>, name: &QString);

        // #[qinvokable]
        // #[cxx_name = "incrementNumber"]
        // fn increment_number(self: Pin<&mut Self>);

        // #[qinvokable]
        // #[cxx_name = "sayHi"]
        // fn say_hi(&self, string: &QString, number: i32);
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
            let state = ctx.as_ref().rust().connected;
            if !state {
                ctx.rust_mut().disconnected();
            }
        })
        .release();
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

    method_list: Vec<Method<'static>>,
    methods: Methods,

    ec_dump: Box<Bin>,
    ec_dump_pretty: QString,
}

struct Methods {
    map: UniquePtr<QQmlPropertyMap>,
    children: Vec<UniquePtr<QQmlPropertyMap>>,
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
                children: Vec::new(),
                cache: HashMap::new(),
            },

            ec_dump: Box::default(),
            ec_dump_pretty: "|      | _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _A _B _C _D _E _F\n|------+------------------------------------------------\n| 0x0_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x1_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x2_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x3_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x4_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x5_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x6_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x7_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x8_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x9_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xA_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xB_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xC_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xD_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xE_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xF_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n".into(),
        }
    }
}

impl EcchanClientRust {
    // common takss to run on disconnect
    pub fn disconnected(&mut self) {
        if let Some(token) = self.heartbeats.take() {
            _ = token.send(());
        }
    }
}

// Internal
impl qobject::EcchanClient {
    pub fn call(
        mut self: Pin<&mut Self>,
        method: MethodCall<'static>,
    ) -> Result<RetVal<'static>, ClientError> {
        if !self.connected || self.client.is_none() {
            if !matches!(method, MethodCall::Ping) {
                q_warning!("not connected; cannot call {method:?}");
            }

            return Err(ClientError::Io {
                source: io::Error::new(io::ErrorKind::NotConnected, "not connected"),
            });
        }

        let mut this = self.as_mut().rust_mut();
        let res = this.client.as_mut().unwrap().call(&method);

        match res {
            o @ Ok(_) => o,
            Err(e) => {
                match e {
                    ClientError::Call { .. } | ClientError::Json { .. } => (),
                    ClientError::Io { .. } | ClientError::Eof => {
                        // socket error, so we now disconnect
                        this.connected = false;
                        this.client.take();
                        self.connected_changed();
                    }
                }

                if !matches!(e, ClientError::Eof) {
                    q_critical!("{e}");
                }

                if matches!(method, MethodCall::Ping) {
                    q_warning!("heartbeat failed; disconnecting");
                }

                Err(e)
            }
        }
    }

    pub fn _update(mut self: Pin<&mut Self>, name: &str) {
        match name {
            "fanCount" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FanCount) {
                    self.as_mut().rust_mut().fan_count = val.fans().unwrap();
                    self.fan_count_changed();
                }
            }

            "fanMax" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FanMax) {
                    self.as_mut().rust_mut().fan_max = val.byte().unwrap();
                    self.fan_max_changed();
                }
            }

            "hasDGpu" => {
                if let Ok(val) = self.as_mut().call(MethodCall::HasDGpu) {
                    self.as_mut().rust_mut().has_dgpu = val.state().unwrap();
                    self.has_dgpu_changed();
                }
            }

            "wmiVer" => {
                if let Ok(val) = self.as_mut().call(MethodCall::WmiVer) {
                    self.as_mut().rust_mut().wmi_ver = val.wmi_ver().unwrap();
                    self.wmi_ver_changed();
                }
            }

            "fwVersion" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FwVersion) {
                    self.as_mut().rust_mut().fw_version = val.str().unwrap().into();
                    self.fw_version_changed();
                }
            }

            "fwDate" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FwDate) {
                    self.as_mut().rust_mut().fw_date = val.str().unwrap().into();
                    self.fw_date_changed();
                }
            }

            "fwTime" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FwTime) {
                    self.as_mut().rust_mut().fw_time = val.str().unwrap().into();
                    self.fw_time_changed();
                }
            }

            "shiftModes" => {
                if let Ok(val) = self.as_mut().call(MethodCall::ShiftModes) {
                    self.as_mut().rust_mut().shift_modes = val.shift_modes().unwrap();
                    self.shift_modes_changed();
                }
            }

            "shiftMode" => {
                if let Ok(val) = self.as_mut().call(MethodCall::ShiftMode) {
                    self.as_mut().rust_mut().shift_mode = val.shift_mode().unwrap();
                    self.shift_mode_changed();
                }
            }

            "shiftModeSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::ShiftModeSupported) {
                    self.as_mut().rust_mut().shift_mode_supported = val.state().unwrap();
                    self.shift_mode_supported_changed();
                }
            }

            "batteryChargeMode" => {
                if let Ok(val) = self.as_mut().call(MethodCall::BatteryChargeMode) {
                    self.as_mut().rust_mut().battery_charge_mode =
                        val.battery_charge_mode().unwrap();
                    self.battery_charge_mode_changed();
                }
            }

            "batteryChargeModeSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::BatteryChargeModeSupported) {
                    self.as_mut().rust_mut().battery_charge_mode_supported = val.state().unwrap();
                    self.battery_charge_mode_supported_changed();
                }
            }

            "superBattery" => {
                if let Ok(val) = self.as_mut().call(MethodCall::SuperBattery) {
                    self.as_mut().rust_mut().super_battery = val.super_battery().unwrap();
                    self.super_battery_changed();
                }
            }

            "superBatterySupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::SuperBatterySupported) {
                    self.as_mut().rust_mut().super_battery_supported = val.state().unwrap();
                    self.super_battery_supported_changed();
                }
            }

            "fan1Rpm" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Fan1Rpm) {
                    self.as_mut().rust_mut().fan1_rpm = val.word().unwrap();
                    self.fan1_rpm_changed();
                }
            }

            "fan2Rpm" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Fan2Rpm) {
                    self.as_mut().rust_mut().fan2_rpm = val.word().unwrap();
                    self.fan2_rpm_changed();
                }
            }

            "fan3Rpm" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Fan3Rpm) {
                    self.as_mut().rust_mut().fan3_rpm = val.word().unwrap();
                    self.fan3_rpm_changed();
                }
            }

            "fan4Rpm" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Fan4Rpm) {
                    self.as_mut().rust_mut().fan4_rpm = val.word().unwrap();
                    self.fan4_rpm_changed();
                }
            }

            "fan1Supported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Fan1Supported) {
                    self.as_mut().rust_mut().fan1_supported = val.state().unwrap();
                    self.fan1_supported_changed();
                }
            }

            "fan2Supported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Fan2Supported) {
                    self.as_mut().rust_mut().fan2_supported = val.state().unwrap();
                    self.fan2_supported_changed();
                }
            }

            "fan3Supported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Fan3Supported) {
                    self.as_mut().rust_mut().fan3_supported = val.state().unwrap();
                    self.fan3_supported_changed();
                }
            }

            "fan4Supported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Fan4Supported) {
                    self.as_mut().rust_mut().fan4_supported = val.state().unwrap();
                    self.fan4_supported_changed();
                }
            }

            "fanModes" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FanModes) {
                    self.as_mut().rust_mut().fan_modes = val.fan_modes().unwrap();
                    self.fan_modes_changed();
                }
            }

            "fanMode" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FanMode) {
                    self.as_mut().rust_mut().fan_mode = val.fan_mode().unwrap();
                    self.fan_mode_changed();
                }
            }

            "fanModeSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FanModeSupported) {
                    self.as_mut().rust_mut().fan_mode_supported = val.state().unwrap();
                    self.fan_mode_supported_changed();
                }
            }

            "webcam" => {
                if let Ok(val) = self.as_mut().call(MethodCall::Webcam) {
                    self.as_mut().rust_mut().webcam = val.webcam().unwrap();
                    self.webcam_changed();
                }
            }

            "webcamBlock" => {
                if let Ok(val) = self.as_mut().call(MethodCall::WebcamBlock) {
                    self.as_mut().rust_mut().webcam_block = val.webcam().unwrap();
                    self.webcam_block_changed();
                }
            }

            "webcamSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::WebcamSupported) {
                    self.as_mut().rust_mut().webcam_supported = val.state().unwrap();
                    self.webcam_supported_changed();
                }
            }

            "webcamBlockSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::WebcamBlockSupported) {
                    self.as_mut().rust_mut().webcam_block_supported = val.state().unwrap();
                    self.webcam_block_supported_changed();
                }
            }

            "coolerBoost" => {
                if let Ok(val) = self.as_mut().call(MethodCall::CoolerBoost) {
                    self.as_mut().rust_mut().cooler_boost = val.cooler_boost().unwrap();
                    self.cooler_boost_changed();
                }
            }

            "coolerBoostSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::CoolerBoostSupported) {
                    self.as_mut().rust_mut().cooler_boost_supported = val.state().unwrap();
                    self.cooler_boost_supported_changed();
                }
            }

            "fnKey" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FnKey) {
                    self.as_mut().rust_mut().fn_key = val.key_direction().unwrap();
                    self.fn_key_changed();
                }
            }

            "winKey" => {
                if let Ok(val) = self.as_mut().call(MethodCall::WinKey) {
                    self.as_mut().rust_mut().win_key = val.key_direction().unwrap();
                    self.win_key_changed();
                }
            }

            "fnWinSwapSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::FnWinSwapSupported) {
                    self.as_mut().rust_mut().fn_win_swap_supported = val.state().unwrap();
                    self.fn_win_swap_supported_changed();
                }
            }

            "micMuteLed" => {
                if let Ok(val) = self.as_mut().call(MethodCall::MicMuteLed) {
                    self.as_mut().rust_mut().mic_mute_led = val.led().unwrap();
                    self.mic_mute_led_changed();
                }
            }

            "muteLed" => {
                if let Ok(val) = self.as_mut().call(MethodCall::MuteLed) {
                    self.as_mut().rust_mut().mute_led = val.led().unwrap();
                    self.mute_led_changed();
                }
            }

            "micMuteLedSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::MicMuteLedSupported) {
                    self.as_mut().rust_mut().mic_mute_led_supported = val.state().unwrap();
                    self.mic_mute_led_supported_changed();
                }
            }

            "muteLedSupported" => {
                if let Ok(val) = self.as_mut().call(MethodCall::MuteLedSupported) {
                    self.as_mut().rust_mut().mute_led_supported = val.state().unwrap();
                    self.mute_led_supported_changed();
                }
            }

            "cpuRtFanSpeed" => {
                if let Ok(val) = self.as_mut().call(MethodCall::CpuRtFanSpeed) {
                    self.as_mut().rust_mut().cpu_rt_fan_speed = val.byte().unwrap();
                    self.cpu_rt_fan_speed_changed();
                }
            }

            "cpuRtTemp" => {
                if let Ok(val) = self.as_mut().call(MethodCall::CpuRtTemp) {
                    self.as_mut().rust_mut().cpu_rt_temp = val.byte().unwrap();
                    self.cpu_rt_temp_changed();
                }
            }

            "gpuRtTemp" => {
                if let Ok(val) = self.as_mut().call(MethodCall::GpuRtTemp) {
                    self.as_mut().rust_mut().gpu_rt_temp = val.byte().unwrap();
                    self.gpu_rt_temp_changed();
                }
            }

            "gpuRtFanSpeed" => {
                if let Ok(val) = self.as_mut().call(MethodCall::GpuRtFanSpeed) {
                    self.as_mut().rust_mut().gpu_rt_fan_speed = val.byte().unwrap();
                    self.gpu_rt_fan_speed_changed();
                }
            }

            "cpuFanCurveWmi2" => {
                if let Ok(val) = self.as_mut().call(MethodCall::CpuFanCurveWmi2) {
                    self.as_mut().rust_mut().cpu_fan_curve_wmi2 = val.curve7().unwrap();
                    self.cpu_fan_curve_wmi2_changed();
                }
            }

            "cpuTempCurveWmi2" => {
                if let Ok(val) = self.as_mut().call(MethodCall::CpuTempCurveWmi2) {
                    self.as_mut().rust_mut().cpu_temp_curve_wmi2 = val.curve7().unwrap();
                    self.cpu_temp_curve_wmi2_changed();
                }
            }

            "cpuHysteresisCurveWmi2" => {
                if let Ok(val) = self.as_mut().call(MethodCall::CpuHysteresisCurveWmi2) {
                    self.as_mut().rust_mut().cpu_hysteresis_curve_wmi2 = val.curve6().unwrap();
                    self.cpu_hysteresis_curve_wmi2_changed();
                }
            }

            "gpuFanCurveWmi2" => {
                if let Ok(val) = self.as_mut().call(MethodCall::GpuFanCurveWmi2) {
                    self.as_mut().rust_mut().gpu_fan_curve_wmi2 = val.curve7().unwrap();
                    self.gpu_fan_curve_wmi2_changed();
                }
            }

            "gpuTempCurveWmi2" => {
                if let Ok(val) = self.as_mut().call(MethodCall::GpuTempCurveWmi2) {
                    self.as_mut().rust_mut().gpu_temp_curve_wmi2 = val.curve7().unwrap();
                    self.gpu_temp_curve_wmi2_changed();
                }
            }

            "gpuHysteresisCurveWmi2" => {
                if let Ok(val) = self.as_mut().call(MethodCall::GpuHysteresisCurveWmi2) {
                    self.as_mut().rust_mut().gpu_hysteresis_curve_wmi2 = val.curve6().unwrap();
                    self.gpu_hysteresis_curve_wmi2_changed();
                }
            }

            "methodList" => {
                if let Ok(val) = self.as_mut().call(MethodCall::MethodList) {
                    let method_list = val.into_methods().unwrap();

                    self.as_mut().rust_mut().method_list = method_list;
                    self.as_mut().method_list_changed();
                }
            }

            "methods" => 'b: {
                // > 1 because objectName property is added by default
                if self.method_list.is_empty() || self.methods.map.size() > 1 {
                    break 'b;
                }

                let method_list = self.method_list.clone();
                let mut list = Vec::with_capacity(self.method_list.len());

                list.resize_with(list.capacity(), || {
                    let mut map = QQmlPropertyMap::new();
                    map.pin_mut()
                        .set_parent(self.as_mut().rust_mut().methods.map.pin_mut());
                    map
                });

                for (method, mut map) in method_list.into_iter().zip(list) {
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
                            let qthread = self.qt_thread();
                            let op = method.ops.iter().find(|op| matches!(op, MethodOp::Write | MethodOp::WriteBit | MethodOp::WriteRange)).copied();
                            let method = method.method.clone().into_owned();

                            move |ctx, key, value| {
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

                                        let (tx, rx) = sync_channel(1);
                                        let tx = Arc::new(tx);

                                        let res = qthread.queue({
                                            let tx = tx.clone();
                                            let method: Cow<str> = Cow::Owned(method.clone());
                                            move |mut ctx| {
                                                let res = ctx.as_mut().call(MethodCall::MethodWrite { method: method.clone(), op, data: MethodData::Bit(state) }).ok();

                                                let method_data = if res.is_none() {
                                                    // get previous value; don't update the cache since it failed
                                                    ctx.as_ref().rust().methods.cache.get(&method.into_owned()).cloned()
                                                } else {
                                                    // update the previous cache to new value
                                                    ctx.as_mut().rust_mut().methods.cache.insert(method.into(), MethodData::Bit(state));
                                                    None
                                                };

                                                _ = tx.send(method_data);

                                            }
                                        });

                                        // so we don't deadlock if call fails
                                        if res.is_err() {
                                            _ = tx.try_send(None);
                                        }

                                        let data = rx.recv().unwrap();
                                        if let Some(data) = data {
                                            ctx.insert(&"value".into(), &QVariant::from(&data.as_bit()));
                                        }
                                    }

                                    MethodOp::Write => {
                                        let Some(byte) = QVariant::value::<u8>(value) else {
                                            q_warning!("custom method {method} received an unsupported type; please use `number` (u8) for setting");
                                            return;
                                        };

                                        let (tx, rx) = sync_channel(1);
                                        let tx = Arc::new(tx);

                                        let res = qthread.queue({
                                            let tx = tx.clone();
                                            let method: Cow<str> = Cow::Owned(method.clone());
                                            move |mut ctx| {
                                                let res = ctx.as_mut().call(MethodCall::MethodWrite { method: method.clone(), op, data: MethodData::Byte(byte) }).ok();

                                                let method_data = if res.is_none() {
                                                    // get previous value; don't update the cache since it failed
                                                    ctx.as_ref().rust().methods.cache.get(&method.into_owned()).cloned()
                                                } else {
                                                    // update the previous cache to new value
                                                    ctx.as_mut().rust_mut().methods.cache.insert(method.into(), MethodData::Byte(byte));
                                                    None
                                                };

                                                _ = tx.send(method_data);

                                            }
                                        });

                                        // so we don't deadlock if call fails
                                        if res.is_err() {
                                            _ = tx.try_send(None);
                                        }

                                        let data = rx.recv().unwrap();
                                        if let Some(data) = data {
                                            ctx.insert(&"value".into(), &QVariant::from(&data.as_byte()));
                                        }
                                    }

                                    MethodOp::WriteRange => {
                                        let Some(bytes) = QVariant::value::<QByteArray>(value) else {
                                            q_warning!("custom method {method} received an unsupported type; please use a byte array for setting");
                                            return;
                                        };

                                        let bytes = bytes.as_slice().to_vec();

                                        let (tx, rx) = sync_channel(1);
                                        let tx = Arc::new(tx);

                                        let res = qthread.queue({
                                            let tx = tx.clone();
                                            let method: Cow<str> = Cow::Owned(method.clone());
                                            move |mut ctx| {
                                                let res = ctx.as_mut().call(MethodCall::MethodWrite { method: method.clone(), op, data: MethodData::Range(bytes.clone()) }).ok();

                                                let method_data = if res.is_none() {
                                                    // get previous value; don't update the cache since it failed
                                                    ctx.as_ref().rust().methods.cache.get(&method.into_owned()).cloned()
                                                } else {
                                                    // update the previous cache to new value
                                                    ctx.as_mut().rust_mut().methods.cache.insert(method.into(), MethodData::Range(bytes));
                                                    None
                                                };

                                                _ = tx.send(method_data);

                                            }
                                        });

                                        // so we don't deadlock if call fails
                                        if res.is_err() {
                                            _ = tx.try_send(None);
                                        }

                                        let data = rx.recv().unwrap();
                                        if let Some(data) = data {
                                            let data = data.into_range();
                                            ctx.insert(&"value".into(), &QVariant::from(&QByteArray::from(&*data)));
                                        }
                                    }

                                    _ => {
                                        q_warning!("entered unreachable code");
                                    }
                                }
                            }
                        })
                        .release();

                    let op = method
                        .ops
                        .iter()
                        .find(|o| {
                            matches!(o, MethodOp::Read | MethodOp::ReadBit | MethodOp::ReadRange)
                        })
                        .copied();

                    let value = match op {
                        Some(op) => {
                            let res = self.as_mut().call(MethodCall::MethodRead {
                                method: method.method.clone(),
                                op,
                            });

                            if let Ok(res) = res {
                                let data = res.method_data().unwrap();

                                self.as_mut()
                                    .rust_mut()
                                    .methods
                                    .cache
                                    .insert(method.method.into_owned(), data.clone());

                                let val = match data {
                                    MethodData::Bit(b) => QVariant::from(&b),
                                    MethodData::Byte(b) => QVariant::from(&b),
                                    MethodData::Range(items) => {
                                        let arr = QByteArray::from(&*items);
                                        QVariant::from(&arr)
                                    }
                                };

                                Some(val)
                            } else {
                                None
                            }
                        }

                        None => None,
                    };

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

                    if let Some(variant) = &value {
                        pin_map.as_mut().insert(&"value".into(), variant);
                    }

                    pin_map.as_mut().freeze();

                    let variant = unsafe { map.as_qvariant() };
                    self.as_mut()
                        .rust_mut()
                        .methods
                        .map
                        .pin_mut()
                        .insert(&name, &variant);

                    self.as_mut().rust_mut().methods.children.push(map);
                }

                // no more changes thx
                self.as_mut().rust_mut().methods.map.pin_mut().freeze();
            }

            _ => q_warning!("{name} is not a valid update property"),
        }
    }
}

// Invokables
impl qobject::EcchanClient {
    pub fn update(self: Pin<&mut Self>, name: &QString) {
        self._update(&name.to_string());
    }

    pub fn init_state(mut self: Pin<&mut Self>) {
        self.as_mut().init_state_changed(true);

        let names = [
            "fanCount",
            "fanMax",
            "hasDGpu",
            "wmiVer",
            "fwVersion",
            "fwDate",
            "fwTime",
            "shiftModes",
            "shiftMode",
            "shiftModeSupported",
            "batteryChargeMode",
            "batteryChargeModeSupported",
            "superBattery",
            "superBatterySupported",
            "fan1Rpm",
            "fan2Rpm",
            "fan3Rpm",
            "fan4Rpm",
            "fan1Supported",
            "fan2Supported",
            "fan3Supported",
            "fan4Supported",
            "fanModes",
            "fanMode",
            "fanModeSupported",
            "webcam",
            "webcamBlock",
            "webcamSupported",
            "webcamBlockSupported",
            "coolerBoost",
            "coolerBoostSupported",
            "fnKey",
            "winKey",
            "fnWinSwapSupported",
            "micMuteLed",
            "muteLed",
            "micMuteLedSupported",
            "muteLedSupported",
            "cpuRtFanSpeed",
            "cpuRtTemp",
            "gpuRtFanSpeed",
            "gpuRtTemp",
            "cpuFanCurveWmi2",
            "cpuTempCurveWmi2",
            "cpuHysteresisCurveWmi2",
            "gpuFanCurveWmi2",
            "gpuTempCurveWmi2",
            "gpuHysteresisCurveWmi2",
            "methodList",
            "methods",
        ];

        for name in names {
            self.as_mut()._update(name);
        }

        self.init_state_changed(false);
    }
}

// Properties
impl qobject::EcchanClient {
    pub fn set_connected(mut self: Pin<&mut Self>, connected: bool) {
        if connected && self.as_ref().rust().client.is_none() {
            if self.as_ref().rust().path.is_empty() {
                return;
            }

            let path = self.as_ref().rust().path.to_string();
            let client = match Client::new(&path) {
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

            let qt_thread = self.qt_thread();

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

                    thread::sleep(Duration::from_millis(1500));

                    let event_loop_should_exit = should_exit.clone();
                    let res =
                        qt_thread.queue(move |mut ctx| match ctx.as_mut().call(MethodCall::Ping) {
                            Ok(_) => (),
                            Err(e) => match e {
                                ClientError::Call { .. } | ClientError::Json { .. } => (),
                                ClientError::Io { .. } | ClientError::Eof => {
                                    event_loop_should_exit.store(true, Ordering::Relaxed)
                                }
                            },
                        });

                    // probably destroyed qobject
                    if res.is_err() {
                        break;
                    }
                }
            });
        } else {
            self.as_mut().rust_mut().disconnected();

            if self.as_mut().rust_mut().client.take().is_some() {
                // take client and drop it, causing a disconnection
                self.as_mut().rust_mut().connected = false;
                self.as_mut().connected_changed();
            }
        }
    }

    pub fn wmi_ver(&self) -> u8 {
        match self.wmi_ver {
            WmiVer::Wmi1 => 1,
            WmiVer::Wmi2 => 2,
        }
    }

    pub fn fan_count(&self) -> u8 {
        match self.fan_count {
            Fans::One => 1,
            Fans::Two => 2,
            Fans::Three => 3,
            Fans::Four => 4,
        }
    }

    pub fn shift_modes(&self) -> QList<QString> {
        let mut qlist = QList::default();

        for item in &self.shift_modes {
            qlist.append(item.to_string().into());
        }

        qlist
    }

    pub fn shift_mode(&self) -> QString {
        self.shift_mode.to_string().into()
    }

    pub fn set_shift_mode(mut self: Pin<&mut Self>, mode: &QString) {
        let mode = match ShiftMode::from_str(&mode.to_string()) {
            Ok(m) => m,
            Err(e) => {
                q_warning!("shift_mode: {e}");
                return;
            }
        };

        let res = self.as_mut().call(MethodCall::SetShiftMode { mode });

        if res.is_ok() {
            self.as_mut().rust_mut().shift_mode = mode;
            self.shift_mode_changed();
        }
    }

    pub fn battery_charge_mode(&self) -> QVariant {
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

    pub fn set_battery_charge_mode(mut self: Pin<&mut Self>, mode: QVariant) {
        let (mode, res) = if let Some(mode) = mode.value::<QString>() {
            let mode = match BatteryChargeMode::from_str(&mode.to_string()) {
                Ok(m) => m,
                Err(e) => {
                    q_warning!("battery_charge_mode: {e}");
                    return;
                }
            };

            let res = self
                .as_mut()
                .call(MethodCall::SetBatteryChargeMode { mode });

            (mode, res)
        } else if let Some(mode) = mode.value::<u8>() {
            let Some(mode) = BatteryChargeMode::from_end(mode) else {
                q_warning!("battery_charge_mode: {mode} out of range; only accept 10..=100");
                return;
            };

            let res = self
                .as_mut()
                .call(MethodCall::SetBatteryChargeMode { mode });

            (mode, res)
        } else {
            q_warning!("battery_charge_mode: only string and number are supported");
            return;
        };

        if res.is_ok() {
            self.as_mut().rust_mut().battery_charge_mode = mode;
            self.battery_charge_mode_changed();
        }
    }

    pub fn super_battery(&self) -> bool {
        self.super_battery.enabled()
    }

    pub fn set_super_battery(mut self: Pin<&mut Self>, state: bool) {
        let state = SuperBattery::from(state);

        let res = self.as_mut().call(MethodCall::SetSuperBattery { state });

        if res.is_ok() {
            self.as_mut().rust_mut().super_battery = state;
            self.super_battery_changed();
        }
    }

    pub fn fan_modes(&self) -> QList<QString> {
        let mut list = QList::default();

        for mode in &self.fan_modes {
            list.append(mode.to_string().into());
        }

        list
    }

    pub fn fan_mode(&self) -> QString {
        self.fan_mode.to_string().into()
    }

    pub fn set_fan_mode(mut self: Pin<&mut Self>, mode: &QString) {
        let mode = match FanMode::from_str(&mode.to_string()) {
            Ok(m) => m,
            Err(e) => {
                q_warning!("fan_mode: {e}");
                return;
            }
        };

        let res = self.as_mut().call(MethodCall::SetFanMode { mode });

        if res.is_ok() {
            self.as_mut().rust_mut().fan_mode = mode;
            self.fan_mode_changed();
        }
    }

    pub fn webcam(&self) -> bool {
        self.webcam.enabled()
    }

    pub fn webcam_block(&self) -> bool {
        self.webcam_block.enabled()
    }

    pub fn set_webcam(mut self: Pin<&mut Self>, state: bool) {
        let state = Webcam::from(state);

        let res = self.as_mut().call(MethodCall::SetWebcam { state });

        if res.is_ok() {
            self.as_mut().rust_mut().webcam = state;
            self.webcam_changed();
        }
    }

    pub fn set_webcam_block(mut self: Pin<&mut Self>, state: bool) {
        let state = Webcam::from(state);

        let res = self.as_mut().call(MethodCall::SetWebcamBlock { state });

        if res.is_ok() {
            self.as_mut().rust_mut().webcam_block = state;
            self.webcam_block_changed();
        }
    }

    fn cooler_boost(&self) -> bool {
        self.cooler_boost.enabled()
    }

    fn set_cooler_boost(mut self: Pin<&mut Self>, state: bool) {
        let state = CoolerBoost::from(state);

        let res = self.as_mut().call(MethodCall::SetCoolerBoost { state });

        if res.is_ok() {
            self.as_mut().rust_mut().cooler_boost = state;
            self.cooler_boost_changed();
        }
    }

    fn fn_key(&self) -> QString {
        self.fn_key.to_string().into()
    }

    fn win_key(&self) -> QString {
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

        let res = self.as_mut().call(MethodCall::SetFnKey { state });

        if res.is_ok() {
            self.as_mut().rust_mut().fn_key = state;
            self.fn_key_changed();
        }
    }

    fn set_win_key(mut self: Pin<&mut Self>, dir: &QString) {
        let state = match KeyDirection::from_str(&dir.to_string()) {
            Ok(k) => k,
            Err(e) => {
                q_warning!("win_key: {e}");
                return;
            }
        };

        let res = self.as_mut().call(MethodCall::SetWinKey { state });

        if res.is_ok() {
            self.as_mut().rust_mut().win_key = state;
            self.win_key_changed();
        }
    }

    fn mic_mute_led(&self) -> bool {
        self.mic_mute_led.enabled()
    }

    fn mute_led(&self) -> bool {
        self.mute_led.enabled()
    }

    fn set_mic_mute_led(mut self: Pin<&mut Self>, state: bool) {
        let state = Led::from(state);

        let res = self.as_mut().call(MethodCall::SetMicMuteLed { state });

        if res.is_ok() {
            self.as_mut().rust_mut().mic_mute_led = state;
            self.mic_mute_led_changed();
        }
    }

    fn set_mute_led(mut self: Pin<&mut Self>, state: bool) {
        let state = Led::from(state);

        let res = self.as_mut().call(MethodCall::SetMuteLed { state });

        if res.is_ok() {
            self.as_mut().rust_mut().mute_led = state;
            self.mute_led_changed();
        }
    }

    fn cpu_fan_curve_wmi2(&self) -> QList<u8> {
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

        let res = self.as_mut().call(MethodCall::SetCpuFanCurveWmi2 { curve });

        if res.is_ok() {
            self.as_mut().rust_mut().cpu_fan_curve_wmi2 = curve;
            self.cpu_fan_curve_wmi2_changed();
        }
    }

    fn cpu_temp_curve_wmi2(&self) -> QList<u8> {
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

        let res = self
            .as_mut()
            .call(MethodCall::SetCpuTempCurveWmi2 { curve });

        if res.is_ok() {
            self.as_mut().rust_mut().cpu_temp_curve_wmi2 = curve;
            self.cpu_temp_curve_wmi2_changed();
        }
    }

    fn cpu_hysteresis_curve_wmi2(&self) -> QList<u8> {
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

        let res = self
            .as_mut()
            .call(MethodCall::SetCpuHysteresisCurveWmi2 { curve });

        if res.is_ok() {
            self.as_mut().rust_mut().cpu_hysteresis_curve_wmi2 = curve;
            self.cpu_hysteresis_curve_wmi2_changed();
        }
    }

    fn gpu_fan_curve_wmi2(&self) -> QList<u8> {
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

        let res = self.as_mut().call(MethodCall::SetGpuFanCurveWmi2 { curve });

        if res.is_ok() {
            self.as_mut().rust_mut().gpu_fan_curve_wmi2 = curve;
            self.gpu_fan_curve_wmi2_changed();
        }
    }

    fn gpu_temp_curve_wmi2(&self) -> QList<u8> {
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

        let res = self
            .as_mut()
            .call(MethodCall::SetGpuTempCurveWmi2 { curve });

        if res.is_ok() {
            self.as_mut().rust_mut().gpu_temp_curve_wmi2 = curve;
            self.gpu_temp_curve_wmi2_changed();
        }
    }

    fn gpu_hysteresis_curve_wmi2(&self) -> QList<u8> {
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

        let res = self
            .as_mut()
            .call(MethodCall::SetGpuHysteresisCurveWmi2 { curve });

        if res.is_ok() {
            self.as_mut().rust_mut().gpu_hysteresis_curve_wmi2 = curve;
            self.gpu_hysteresis_curve_wmi2_changed();
        }
    }

    pub fn method_list(&self) -> QList<QVariant> {
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

    pub fn methods(&self) -> *mut QQmlPropertyMap {
        self.methods.map.as_mut_ptr()
    }

    pub fn ec_dump(&self) -> QByteArray {
        QByteArray::from(&self.ec_dump.0)
    }
}
