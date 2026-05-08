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
        #[qproperty(u8, fan_count, READ = get_fan_count, NOTIFY)]
        #[qproperty(u8, fan_max, READ, NOTIFY)]
        #[qproperty(bool, has_dgpu, READ, NOTIFY)]
        #[qproperty(u8, wmi_ver, READ = get_wmi_ver, NOTIFY)]
        // fw
        #[qproperty(QString, fw_version, READ, NOTIFY)]
        #[qproperty(QString, fw_date, READ, NOTIFY)]
        #[qproperty(QString, fw_time, READ, NOTIFY)]
        // shift mode
        #[qproperty(QList_QString, shift_modes, READ = get_shift_modes, NOTIFY)]
        #[qproperty(QString, shift_mode, READ = get_shift_mode, WRITE = set_shift_mode, NOTIFY)]
        #[qproperty(bool, shift_mode_supported, READ, NOTIFY)]
        // battery charge mode
        #[qproperty(QVariant, battery_charge_mode, READ = get_battery_charge_mode, WRITE = set_battery_charge_mode, NOTIFY)]
        #[qproperty(bool, battery_charge_mode_supported, READ, NOTIFY)]
        // super battery
        #[qproperty(bool, super_battery, READ = get_super_battery, WRITE = set_super_battery, NOTIFY)]
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
        #[qproperty(QList_QString, fan_modes, READ = get_fan_modes, NOTIFY)]
        #[qproperty(QString, fan_mode, READ = get_fan_mode, WRITE = set_fan_mode, NOTIFY)]
        #[qproperty(bool, fan_mode_supported, READ, NOTIFY)]
        // webcam
        #[qproperty(bool, webcam, READ = get_webcam, WRITE = set_webcam, NOTIFY)]
        #[qproperty(bool, webcam_block, READ = get_webcam_block, WRITE = set_webcam_block, NOTIFY)]
        #[qproperty(bool, webcam_supported, READ, NOTIFY)]
        #[qproperty(bool, webcam_block_supported, READ, NOTIFY)]
        // cooler boost
        #[qproperty(bool, cooler_boost, READ = get_cooler_boost, WRITE = set_cooler_boost, NOTIFY)]
        #[qproperty(bool, cooler_boost_supported, READ, NOTIFY)]
        // fn/win key swap
        #[qproperty(QString, fn_key, READ = get_fn_key, WRITE = set_fn_key, NOTIFY)]
        #[qproperty(QString, win_key, READ = get_win_key, WRITE = set_win_key, NOTIFY)]
        #[qproperty(bool, fn_win_swap_supported, READ, NOTIFY)]
        // mute leds
        #[qproperty(bool, mic_mute_led, READ = get_mic_mute_led, WRITE = set_mic_mute_led, NOTIFY)]
        #[qproperty(bool, mute_led, READ = get_mute_led, WRITE = set_mute_led, NOTIFY)]
        #[qproperty(bool, mic_mute_led_supported, READ, NOTIFY)]
        #[qproperty(bool, mute_led_supported, READ, NOTIFY)]
        // rt sensors
        #[qproperty(u8, cpu_rt_fan_speed, READ, NOTIFY)]
        #[qproperty(u8, cpu_rt_temp, READ, NOTIFY)]
        #[qproperty(u8, gpu_rt_fan_speed, READ, NOTIFY)]
        #[qproperty(u8, gpu_rt_temp, READ, NOTIFY)]
        // curves
        #[qproperty(QList_u8, cpu_fan_curve_wmi2, READ = get_cpu_fan_curve_wmi2, WRITE = set_cpu_fan_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, cpu_temp_curve_wmi2, READ = get_cpu_temp_curve_wmi2, WRITE = set_cpu_temp_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, cpu_hysteresis_curve_wmi2, READ = get_cpu_hysteresis_curve_wmi2, WRITE = set_cpu_hysteresis_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, gpu_fan_curve_wmi2, READ = get_gpu_fan_curve_wmi2, WRITE = set_gpu_fan_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, gpu_temp_curve_wmi2, READ = get_gpu_temp_curve_wmi2, WRITE = set_gpu_temp_curve_wmi2, NOTIFY)]
        #[qproperty(QList_u8, gpu_hysteresis_curve_wmi2, READ = get_gpu_hysteresis_curve_wmi2, WRITE = set_gpu_hysteresis_curve_wmi2, NOTIFY)]
        // methods
        #[qproperty(QList_QVariant, method_list, READ = get_method_list, NOTIFY)]
        #[qproperty(*mut QQmlPropertyMap, methods, READ = get_methods, NOTIFY)]
        // dump
        #[qproperty(QByteArray, ec_dump, READ = get_ec_dump, NOTIFY)]
        #[qproperty(QString, ec_dump_pretty, READ, NOTIFY)]
        #[namespace = "ecchan_client"]
        type EcchanClient = super::EcchanClientRust;

        fn set_connected(self: Pin<&mut Self>, connected: bool);
        fn get_fan_count(&self) -> u8;
        fn get_wmi_ver(&self) -> u8;

        fn get_shift_modes(&self) -> QList_QString;
        fn get_shift_mode(&self) -> QString;
        fn set_shift_mode(self: Pin<&mut Self>, mode: &QString);

        fn get_battery_charge_mode(&self) -> QVariant;
        fn set_battery_charge_mode(self: Pin<&mut Self>, mode: QVariant);

        fn get_super_battery(&self) -> bool;
        fn set_super_battery(self: Pin<&mut Self>, state: bool);

        fn get_fan_modes(&self) -> QList_QString;
        fn get_fan_mode(&self) -> QString;
        fn set_fan_mode(self: Pin<&mut Self>, mode: &QString);

        fn get_webcam(&self) -> bool;
        fn get_webcam_block(&self) -> bool;
        fn set_webcam(self: Pin<&mut Self>, state: bool);
        fn set_webcam_block(self: Pin<&mut Self>, state: bool);

        fn get_cooler_boost(&self) -> bool;
        fn set_cooler_boost(self: Pin<&mut Self>, state: bool);

        fn get_fn_key(&self) -> QString;
        fn get_win_key(&self) -> QString;
        fn set_fn_key(self: Pin<&mut Self>, dir: &QString);
        fn set_win_key(self: Pin<&mut Self>, dir: &QString);

        fn get_mic_mute_led(&self) -> bool;
        fn get_mute_led(&self) -> bool;
        fn set_mic_mute_led(self: Pin<&mut Self>, state: bool);
        fn set_mute_led(self: Pin<&mut Self>, state: bool);

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

        #[qsignal]
        fn init_state_changed(self: Pin<&mut Self>, running: bool);

        #[qinvokable]
        fn init_state(self: Pin<&mut Self>);

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
            if !ctx.connected {
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
        method: MethodCall<'static>,
        cb: impl FnOnce(Pin<&mut qobject::EcchanClient>, Result<RetVal<'static>, ClientError>)
        + Send
        + 'static,
    ) {
        let mut this = self.as_mut().rust_mut();
        let Some(client) = this.client.as_mut() else {
            if !matches!(method, MethodCall::Ping) {
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

                    if matches!(method, MethodCall::Ping) {
                        q_warning!("heartbeat failed; disconnecting");
                    }

                    Err(e)
                }
            };

            cb(ctx, res);
        });
    }

    fn _update(mut self: Pin<&mut Self>, name: &str) {
        match name {
            "fanCount" => {
                self.as_mut().call(MethodCall::FanCount, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan_count = res.fans().unwrap();
                    ctx.fan_count_changed();
                });
            }

            "fanMax" => {
                self.as_mut().call(MethodCall::FanMax, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan_max = res.byte().unwrap();
                    ctx.fan_max_changed();
                });
            }

            "hasDGpu" => {
                self.as_mut().call(MethodCall::HasDGpu, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().has_dgpu = res.state().unwrap();
                    ctx.has_dgpu_changed();
                });
            }

            "wmiVer" => {
                self.as_mut().call(MethodCall::WmiVer, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };
                    ctx.as_mut().rust_mut().wmi_ver = res.wmi_ver().unwrap();
                    ctx.wmi_ver_changed();
                });
            }

            "fwVersion" => {
                self.as_mut().call(MethodCall::FwVersion, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fw_version = res.str().unwrap().into();
                    ctx.fw_version_changed();
                });
            }

            "fwDate" => {
                self.as_mut().call(MethodCall::FwDate, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fw_date = res.str().unwrap().into();
                    ctx.fw_date_changed();
                });
            }

            "fwTime" => {
                self.as_mut().call(MethodCall::FwTime, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };
                    ctx.as_mut().rust_mut().fw_time = res.str().unwrap().into();
                    ctx.fw_time_changed();
                });
            }

            "shiftModes" => {
                self.as_mut().call(MethodCall::ShiftModes, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().shift_modes = res.shift_modes().unwrap();
                    ctx.shift_modes_changed();
                });
            }

            "shiftMode" => {
                self.as_mut().call(MethodCall::ShiftMode, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().shift_mode = res.shift_mode().unwrap();
                    ctx.shift_mode_changed();
                });
            }

            "shiftModeSupported" => {
                self.as_mut()
                    .call(MethodCall::ShiftModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().shift_mode_supported = res.state().unwrap();
                        ctx.shift_mode_supported_changed();
                    });
            }

            "batteryChargeMode" => {
                self.as_mut()
                    .call(MethodCall::BatteryChargeMode, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().battery_charge_mode =
                            res.battery_charge_mode().unwrap();
                        ctx.battery_charge_mode_changed();
                    });
            }

            "batteryChargeModeSupported" => {
                self.as_mut()
                    .call(MethodCall::BatteryChargeModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().battery_charge_mode_supported =
                            res.state().unwrap();
                        ctx.battery_charge_mode_supported_changed();
                    });
            }

            "superBattery" => {
                self.as_mut()
                    .call(MethodCall::SuperBattery, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().super_battery = res.super_battery().unwrap();
                        ctx.super_battery_changed();
                    });
            }

            "superBatterySupported" => {
                self.as_mut()
                    .call(MethodCall::SuperBatterySupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().super_battery_supported = res.state().unwrap();
                        ctx.super_battery_supported_changed();
                    });
            }

            "fan1Rpm" => {
                self.as_mut().call(MethodCall::Fan1Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan1_rpm = res.word().unwrap();
                    ctx.fan1_rpm_changed();
                });
            }

            "fan2Rpm" => {
                self.as_mut().call(MethodCall::Fan2Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan2_rpm = res.word().unwrap();
                    ctx.fan2_rpm_changed();
                });
            }

            "fan3Rpm" => {
                self.as_mut().call(MethodCall::Fan3Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan3_rpm = res.word().unwrap();
                    ctx.fan3_rpm_changed();
                });
            }

            "fan4Rpm" => {
                self.as_mut().call(MethodCall::Fan4Rpm, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan4_rpm = res.word().unwrap();
                    ctx.fan4_rpm_changed();
                });
            }

            "fan1Supported" => {
                self.as_mut()
                    .call(MethodCall::Fan1Supported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().fan1_supported = res.state().unwrap();
                        ctx.fan1_supported_changed();
                    });
            }

            "fan2Supported" => {
                self.as_mut()
                    .call(MethodCall::Fan2Supported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().fan2_supported = res.state().unwrap();
                        ctx.fan2_supported_changed();
                    });
            }

            "fan3Supported" => {
                self.as_mut()
                    .call(MethodCall::Fan3Supported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().fan3_supported = res.state().unwrap();
                        ctx.fan3_supported_changed();
                    });
            }

            "fan4Supported" => {
                self.as_mut()
                    .call(MethodCall::Fan4Supported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().fan4_supported = res.state().unwrap();
                        ctx.fan4_supported_changed();
                    });
            }

            "fanModes" => {
                self.as_mut().call(MethodCall::FanModes, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan_modes = res.fan_modes().unwrap();
                    ctx.fan_modes_changed();
                });
            }

            "fanMode" => {
                self.as_mut().call(MethodCall::FanMode, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fan_mode = res.fan_mode().unwrap();
                    ctx.fan_mode_changed();
                });
            }

            "fanModeSupported" => {
                self.as_mut()
                    .call(MethodCall::FanModeSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().fan_mode_supported = res.state().unwrap();
                        ctx.fan_mode_supported_changed();
                    });
            }

            "webcam" => {
                self.as_mut().call(MethodCall::Webcam, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().webcam = res.webcam().unwrap();
                    ctx.webcam_changed();
                });
            }

            "webcamBlock" => {
                self.as_mut().call(MethodCall::WebcamBlock, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().webcam_block = res.webcam().unwrap();
                    ctx.webcam_block_changed();
                });
            }

            "webcamSupported" => {
                self.as_mut()
                    .call(MethodCall::WebcamSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().webcam_supported = res.state().unwrap();
                        ctx.webcam_supported_changed();
                    });
            }

            "webcamBlockSupported" => {
                self.as_mut()
                    .call(MethodCall::WebcamBlockSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().webcam_block_supported = res.state().unwrap();
                        ctx.webcam_block_supported_changed();
                    });
            }

            "coolerBoost" => {
                self.as_mut().call(MethodCall::CoolerBoost, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().cooler_boost = res.cooler_boost().unwrap();
                    ctx.cooler_boost_changed();
                });
            }

            "coolerBoostSupported" => {
                self.as_mut()
                    .call(MethodCall::CoolerBoostSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().cooler_boost_supported = res.state().unwrap();
                        ctx.cooler_boost_supported_changed();
                    });
            }

            "fnKey" => {
                self.as_mut().call(MethodCall::FnKey, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().fn_key = res.key_direction().unwrap();
                    ctx.fn_key_changed();
                });
            }

            "winKey" => {
                self.as_mut().call(MethodCall::WinKey, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().win_key = res.key_direction().unwrap();
                    ctx.win_key_changed();
                });
            }

            "fnWinSwapSupported" => {
                self.as_mut()
                    .call(MethodCall::FnWinSwapSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().fn_win_swap_supported = res.state().unwrap();
                        ctx.fn_win_swap_supported_changed();
                    });
            }

            "micMuteLed" => {
                self.as_mut().call(MethodCall::MicMuteLed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().mic_mute_led = res.led().unwrap();
                    ctx.mic_mute_led_changed();
                });
            }

            "muteLed" => {
                self.as_mut().call(MethodCall::MuteLed, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().mute_led = res.led().unwrap();
                    ctx.mute_led_changed();
                });
            }

            "micMuteLedSupported" => {
                self.as_mut()
                    .call(MethodCall::MicMuteLedSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().mic_mute_led_supported = res.state().unwrap();
                        ctx.mic_mute_led_supported_changed();
                    });
            }

            "muteLedSupported" => {
                self.as_mut()
                    .call(MethodCall::MuteLedSupported, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().mute_led_supported = res.state().unwrap();
                        ctx.mute_led_supported_changed();
                    });
            }

            "cpuRtFanSpeed" => {
                self.as_mut()
                    .call(MethodCall::CpuRtFanSpeed, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().cpu_rt_fan_speed = res.byte().unwrap();
                        ctx.cpu_rt_fan_speed_changed();
                    });
            }

            "cpuRtTemp" => {
                self.as_mut().call(MethodCall::CpuRtTemp, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().cpu_rt_temp = res.byte().unwrap();
                    ctx.cpu_rt_temp_changed();
                });
            }

            "gpuRtTemp" => {
                self.as_mut().call(MethodCall::GpuRtTemp, |mut ctx, res| {
                    let Ok(res) = res else {
                        return;
                    };

                    ctx.as_mut().rust_mut().gpu_rt_temp = res.byte().unwrap();
                    ctx.gpu_rt_temp_changed();
                });
            }

            "gpuRtFanSpeed" => {
                self.as_mut()
                    .call(MethodCall::GpuRtFanSpeed, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().gpu_rt_fan_speed = res.byte().unwrap();
                        ctx.gpu_rt_fan_speed_changed();
                    });
            }

            "cpuFanCurveWmi2" => {
                self.as_mut()
                    .call(MethodCall::CpuFanCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().cpu_fan_curve_wmi2 = res.curve7().unwrap();
                        ctx.cpu_fan_curve_wmi2_changed();
                    });
            }

            "cpuTempCurveWmi2" => {
                self.as_mut()
                    .call(MethodCall::CpuTempCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().cpu_temp_curve_wmi2 = res.curve7().unwrap();
                        ctx.cpu_temp_curve_wmi2_changed();
                    });
            }

            "cpuHysteresisCurveWmi2" => {
                self.as_mut()
                    .call(MethodCall::CpuHysteresisCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().cpu_hysteresis_curve_wmi2 = res.curve6().unwrap();
                        ctx.cpu_hysteresis_curve_wmi2_changed();
                    });
            }

            "gpuFanCurveWmi2" => {
                self.as_mut()
                    .call(MethodCall::GpuFanCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().gpu_fan_curve_wmi2 = res.curve7().unwrap();
                        ctx.gpu_fan_curve_wmi2_changed();
                    });
            }

            "gpuTempCurveWmi2" => {
                self.as_mut()
                    .call(MethodCall::GpuTempCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().gpu_temp_curve_wmi2 = res.curve7().unwrap();
                        ctx.gpu_temp_curve_wmi2_changed();
                    });
            }

            "gpuHysteresisCurveWmi2" => {
                self.as_mut()
                    .call(MethodCall::GpuHysteresisCurveWmi2, |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        ctx.as_mut().rust_mut().gpu_hysteresis_curve_wmi2 = res.curve6().unwrap();
                        ctx.gpu_hysteresis_curve_wmi2_changed();
                    });
            }

            "methodList" => {
                self.as_mut()
                    .call(MethodCall::MethodList, move |mut ctx, res| {
                        let Ok(res) = res else {
                            return;
                        };

                        let method_list = res.into_methods().unwrap();

                        ctx.as_mut().rust_mut().method_list = method_list;
                        ctx.as_mut().method_list_changed();
                    });
            }

            "methods" => {
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
                            ctx.as_mut().call(MethodCall::MethodRead { method: method.method, op: *read }, {
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
                                                    ctx.as_mut().call(MethodCall::MethodWrite { method: method.clone(), op, data: MethodData::Bit(state) }, move |mut ctx, res| {
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
                                                ctx.as_mut().call(MethodCall::MethodWrite { method: method.clone(), op, data: MethodData::Byte(byte) }, move |mut ctx, res| {
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
                                                ctx.as_mut().call(MethodCall::MethodWrite { method: method.clone(), op, data: MethodData::Range(bytes.clone()) }, move |mut ctx, res| {
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
                                    MethodCall::MethodRead {
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

            _ => q_warning!("{name} is not a valid update property"),
        }
    }
}

// Invokables
impl qobject::EcchanClient {
    fn init_state(mut self: Pin<&mut Self>) {
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

        self.queued_call(|ctx| {
            ctx.init_state_changed(false);
        });
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
                        ctx.call(MethodCall::Ping, move |_, res| match res {
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

    fn get_wmi_ver(&self) -> u8 {
        match self.wmi_ver {
            WmiVer::Wmi1 => 1,
            WmiVer::Wmi2 => 2,
        }
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
            .call(MethodCall::SetShiftMode { mode }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().shift_mode = mode;
                ctx.shift_mode_changed();
            });
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
                MethodCall::SetBatteryChargeMode { mode },
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
                MethodCall::SetBatteryChargeMode { mode },
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

    fn get_super_battery(&self) -> bool {
        self.super_battery.enabled()
    }

    fn set_super_battery(mut self: Pin<&mut Self>, state: bool) {
        let state = SuperBattery::from(state);

        self.as_mut().call(
            MethodCall::SetSuperBattery { state },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().super_battery = state;
                ctx.super_battery_changed();
            },
        );
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
            .call(MethodCall::SetFanMode { mode }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().fan_mode = mode;
                ctx.fan_mode_changed();
            });
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
            .call(MethodCall::SetWebcam { state }, move |mut ctx, res| {
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
            .call(MethodCall::SetWebcamBlock { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().webcam_block = state;
                ctx.webcam_block_changed();
            });
    }

    fn get_cooler_boost(&self) -> bool {
        self.cooler_boost.enabled()
    }

    fn set_cooler_boost(mut self: Pin<&mut Self>, state: bool) {
        let state = CoolerBoost::from(state);

        self.as_mut()
            .call(MethodCall::SetCoolerBoost { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cooler_boost = state;
                ctx.cooler_boost_changed();
            });
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
            .call(MethodCall::SetFnKey { state }, move |mut ctx, res| {
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
            .call(MethodCall::SetWinKey { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().win_key = state;
                ctx.win_key_changed();
            });
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
            .call(MethodCall::SetMicMuteLed { state }, move |mut ctx, res| {
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
            .call(MethodCall::SetMuteLed { state }, move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().mute_led = state;
                ctx.mute_led_changed();
            });
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

        self.as_mut().call(
            MethodCall::SetCpuFanCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().cpu_fan_curve_wmi2 = curve;
                ctx.cpu_fan_curve_wmi2_changed();
            },
        );
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
            MethodCall::SetCpuTempCurveWmi2 { curve },
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
            MethodCall::SetCpuHysteresisCurveWmi2 { curve },
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

        self.as_mut().call(
            MethodCall::SetGpuFanCurveWmi2 { curve },
            move |mut ctx, res| {
                if res.is_err() {
                    return;
                }

                ctx.as_mut().rust_mut().gpu_fan_curve_wmi2 = curve;
                ctx.gpu_fan_curve_wmi2_changed();
            },
        );
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
            MethodCall::SetGpuTempCurveWmi2 { curve },
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
            MethodCall::SetGpuHysteresisCurveWmi2 { curve },
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
}
