use std::{
    collections::HashMap,
    io,
    panic::Location,
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

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{
    QByteArray, QList, QMap, QMapPair as _, QMapPair_QString_QVariant, QString, QStringList,
    QVariant, QVariantValue,
};
use ecchan_ipc::{
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, Fans, KeyDirection, Led, Method,
    MethodData, ShiftMode, SuperBattery, Webcam, WmiVer,
    method::Method as MethodCall,
    ret::{Bin, RetVal},
};

use crate::client::{Client, ClientError};

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

    impl cxx_qt::Threading for EcchanClient {}

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, connected, READ, WRITE = set_connected, NOTIFY)]
        #[qproperty(QString, path, READ, WRITE, NOTIFY)]
        // utils
        #[qproperty(u8, fan_count, READ = fan_count, NOTIFY, cxx_name = "fanCount")]
        #[qproperty(u8, fan_max, READ, NOTIFY, cxx_name = "fanMax")]
        #[qproperty(bool, has_dgpu, READ, NOTIFY, cxx_name = "hasDGpu")]
        #[qproperty(u8, wmi_ver, READ = wmi_ver, NOTIFY, cxx_name = "wmiVer")]
        // fw
        #[qproperty(QString, fw_version, READ, NOTIFY, cxx_name = "fwVersion")]
        #[qproperty(QString, fw_date, READ, NOTIFY, cxx_name = "fwDate")]
        #[qproperty(QString, fw_time, READ, NOTIFY, cxx_name = "fwTime")]
        // shift mode
        #[qproperty(QList_QString, shift_modes, READ = shift_modes, NOTIFY, cxx_name = "shiftModes")]
        #[qproperty(QString, shift_mode, READ = shift_mode, WRITE = set_shift_mode, NOTIFY, cxx_name = "shiftMode")]
        #[qproperty(bool, shift_mode_supported, READ, NOTIFY, cxx_name = "shiftModeSupported")]
        // battery charge mode
        #[qproperty(QVariant, battery_charge_mode, READ = battery_charge_mode, WRITE = set_battery_charge_mode, NOTIFY, cxx_name = "batteryChargeMode")]
        #[qproperty(bool, battery_charge_mode_supported, READ, NOTIFY, cxx_name = "batteryChargeModeSupported")]
        // super battery
        #[qproperty(bool, super_battery, READ = super_battery, WRITE = set_super_battery, NOTIFY, cxx_name = "superBattery")]
        #[qproperty(bool, super_battery_supported, READ, NOTIFY, cxx_name = "superBatterySupported")]
        // fan rpm
        #[qproperty(u16, fan1_rpm, READ, NOTIFY, cxx_name = "fan1Rpm")]
        #[qproperty(u16, fan2_rpm, READ, NOTIFY, cxx_name = "fan2Rpm")]
        #[qproperty(u16, fan3_rpm, READ, NOTIFY, cxx_name = "fan3Rpm")]
        #[qproperty(u16, fan4_rpm, READ, NOTIFY, cxx_name = "fan4Rpm")]
        #[qproperty(bool, fan1_supported, READ, NOTIFY, cxx_name = "fan1Supported")]
        #[qproperty(bool, fan2_supported, READ, NOTIFY, cxx_name = "fan2Supported")]
        #[qproperty(bool, fan3_supported, READ, NOTIFY, cxx_name = "fan3Supported")]
        #[qproperty(bool, fan4_supported, READ, NOTIFY, cxx_name = "fan4Supported")]
        // fan modes
        #[qproperty(QList_QString, fan_modes, READ = fan_modes, NOTIFY, cxx_name = "fanModes")]
        #[qproperty(QString, fan_mode, READ = fan_mode, WRITE = set_fan_mode, NOTIFY, cxx_name = "fanMode")]
        #[qproperty(bool, fan_mode_supported, READ, NOTIFY, cxx_name = "fanModeSupported")]
        // webcam
        #[qproperty(bool, webcam, READ = webcam, WRITE = set_webcam, NOTIFY)]
        #[qproperty(bool, webcam_block, READ = webcam_block, WRITE = set_webcam_block, NOTIFY, cxx_name = "webcamBlock")]
        #[qproperty(bool, webcam_supported, READ, NOTIFY, cxx_name = "webcamSupported")]
        #[qproperty(bool, webcam_block_supported, READ, NOTIFY, cxx_name = "webcamBlockSupported")]
        // cooler boost
        #[qproperty(bool, cooler_boost, READ = cooler_boost, WRITE = set_cooler_boost, NOTIFY, cxx_name = "coolerBoost")]
        #[qproperty(bool, cooler_boost_supported, READ, NOTIFY, cxx_name = "coolerBoostSupported")]
        // fn/win key swap
        #[qproperty(QString, fn_key, READ = fn_key, WRITE = set_fn_key, NOTIFY, cxx_name = "fnKey")]
        #[qproperty(QString, win_key, READ = win_key, WRITE = set_win_key, NOTIFY, cxx_name = "winKey")]
        #[qproperty(bool, fn_win_swap_supported, READ, NOTIFY, cxx_name = "fnWinSwapSupported")]
        // mute leds
        #[qproperty(bool, mic_mute_led, READ = mic_mute_led, WRITE = set_mic_mute_led, NOTIFY, cxx_name = "micMuteLed")]
        #[qproperty(bool, mute_led, READ = mute_led, WRITE = set_mute_led, NOTIFY, cxx_name = "muteLed")]
        #[qproperty(bool, mic_mute_led_supported, READ, NOTIFY, cxx_name = "micMuteLedSupported")]
        #[qproperty(bool, mute_led_supported, READ, NOTIFY, cxx_name = "muteLedSupported")]
        // rt sensors
        #[qproperty(u8, cpu_rt_fan_speed, READ, NOTIFY, cxx_name = "cpuRtFanSpeed")]
        #[qproperty(u8, cpu_rt_temp, READ, NOTIFY, cxx_name = "cpuRtTemp")]
        #[qproperty(u8, gpu_rt_fan_speed, READ, NOTIFY, cxx_name = "gpuRtFanSpeed")]
        #[qproperty(u8, gpu_rt_temp, READ, NOTIFY, cxx_name = "gpuRtTemp")]
        // curves
        #[qproperty(QList_u8, cpu_fan_curve_wmi2, READ = cpu_fan_curve_wmi2, WRITE = set_cpu_fan_curve_wmi2, NOTIFY, cxx_name = "cpuFanCurveWmi2")]
        #[qproperty(QList_u8, cpu_temp_curve_wmi2, READ = cpu_temp_curve_wmi2, WRITE = set_cpu_temp_curve_wmi2, NOTIFY, cxx_name = "cpuTempCurveWmi2")]
        #[qproperty(QList_u8, cpu_hysteresis_curve_wmi2, READ = cpu_hysteresis_curve_wmi2, WRITE = set_cpu_hysteresis_curve_wmi2, NOTIFY, cxx_name = "cpuHysteresisCurveWmi2")]
        #[qproperty(QList_u8, gpu_fan_curve_wmi2, READ = gpu_fan_curve_wmi2, WRITE = set_gpu_fan_curve_wmi2, NOTIFY, cxx_name = "gpuFanCurveWmi2")]
        #[qproperty(QList_u8, gpu_temp_curve_wmi2, READ = gpu_temp_curve_wmi2, WRITE = set_gpu_temp_curve_wmi2, NOTIFY, cxx_name = "gpuTempCurveWmi2")]
        #[qproperty(QList_u8, gpu_hysteresis_curve_wmi2, READ = gpu_hysteresis_curve_wmi2, WRITE = set_gpu_hysteresis_curve_wmi2, NOTIFY, cxx_name = "gpuHysteresisCurveWmi2")]
        // methods
        #[qproperty(QList_QVariant, method_list, READ = method_list, NOTIFY, cxx_name = "methodList")]
        //#[qproperty(QQmlPropertyMap, methods, READ = methods, WRITE = set_methods, NOTIFY)]
        // dump
        #[qproperty(QByteArray, ec_dump, READ = ec_dump, NOTIFY, cxx_name = "ecDump")]
        #[qproperty(QString, ec_dump_pretty, READ, NOTIFY, cxx_name = "ecDumpPretty")]
        #[namespace = "ecchan_client"]
        type EcchanClient = super::EcchanClientRust;

        #[qsignal]
        fn error(self: Pin<&mut Self>, message: QString);

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
        //fn methods(&self) -> QMap_QString_QVariant;
        //fn set_methods(self: Pin<&mut Self>, methods: QMap_QString_QVariant);

        fn ec_dump(&self) -> QByteArray;

        // #[qinvokable]
        // #[cxx_name = "incrementNumber"]
        // fn increment_number(self: Pin<&mut Self>);

        // #[qinvokable]
        // #[cxx_name = "sayHi"]
        // fn say_hi(&self, string: &QString, number: i32);
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
    methods: HashMap<String, MethodData>,

    ec_dump: Box<Bin>,
    ec_dump_pretty: QString,
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
            methods: HashMap::new(),

            ec_dump: Box::default(),
            ec_dump_pretty: QString::default(),
        }
    }
}

impl qobject::EcchanClient {
    #[track_caller]
    pub fn call(
        mut self: Pin<&mut Self>,
        method: MethodCall<'static>,
    ) -> Result<RetVal<'static>, ClientError> {
        if !self.connected || self.client.is_none() {
            if !matches!(method, MethodCall::Ping) {
                let caller = Location::caller();
                self.error(
                    format!(
                        "<{}:{}:{}>::call: not connected; cannot call {method:?}",
                        caller.file(),
                        caller.line(),
                        caller.column()
                    )
                    .into(),
                );
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
                        self.as_mut().connected_changed();
                    }
                }

                let caller = Location::caller();
                self.error(
                    format!(
                        "<{}:{}:{}>::call: {e}",
                        caller.file(),
                        caller.line(),
                        caller.column()
                    )
                    .into(),
                );

                Err(e)
            }
        }
    }

    pub fn set_connected(mut self: Pin<&mut Self>, connected: bool) {
        if connected && self.as_ref().rust().client.is_none() {
            if self.as_ref().rust().path.is_empty() {
                self.error("connected: path property must be set".into());
                return;
            }

            let path = self.as_ref().rust().path.to_string();
            let client = match Client::new(&path) {
                Ok(c) => c,
                Err(e) => {
                    self.error(e.to_string().into());
                    return;
                }
            };

            self.as_mut().rust_mut().client = Some(client);
            self.as_mut().rust_mut().connected = true;
            self.as_mut().connected_changed();

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
            if let Some(token) = self.as_mut().rust_mut().heartbeats.take() {
                _ = token.send(());
            }

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
                self.error(format!("shift_mode: {e}").into());
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
                    self.error(format!("battery_charge_mode: {e}").into());
                    return;
                }
            };

            let res = self
                .as_mut()
                .call(MethodCall::SetBatteryChargeMode { mode });

            (mode, res)
        } else if let Some(mode) = mode.value::<u8>() {
            let Some(mode) = BatteryChargeMode::from_end(mode) else {
                self.error(
                    format!("battery_charge_mode: {mode} out of range; only accept 10..=100")
                        .into(),
                );
                return;
            };

            let res = self
                .as_mut()
                .call(MethodCall::SetBatteryChargeMode { mode });

            (mode, res)
        } else {
            self.error("battery_charge_mode: only string and number are supported".into());
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
                self.error(format!("fan_mode: {e}").into());
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
                self.error(format!("fn_key: {e}").into());
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
                self.error(format!("win_key: {e}").into());
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
            self.error(
                format!("cpu_fan_curve_wmi2: need array of len 7, instead got len {len}").into(),
            );
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
            self.error(
                format!("cpu_temp_curve_wmi2: need array of len 7, instead got len {len}").into(),
            );
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
            self.error(
                format!("cpu_hysteresis_curve_wmi2: need array of len 6, instead got len {len}")
                    .into(),
            );
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
            self.error(
                format!("gpu_fan_curve_wmi2: need array of len 7, instead got len {len}").into(),
            );
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
            self.error(
                format!("gpu_temp_curve_wmi2: need array of len 7, instead got len {len}").into(),
            );
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
            self.error(
                format!("gpu_hysteresis_curve_wmi2: need array of len 6, instead got len {len}")
                    .into(),
            );
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

    pub fn methods(&self) -> QMap<QMapPair_QString_QVariant> {
        todo!()
    }

    pub fn set_methods(self: Pin<&mut Self>, methods: QMap<QMapPair_QString_QVariant>) {
        todo!()
    }

    pub fn ec_dump(&self) -> QByteArray {
        QByteArray::from(&self.ec_dump.0)
    }
}
