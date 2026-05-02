use core::pin::Pin;
use std::{
    borrow::Cow,
    collections::HashMap,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, TryRecvError, channel},
    },
    thread,
    time::Duration,
};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString};
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
        type QList_QVariant = cxx_qt_lib::QList<QVariant>;
        type QList_QString = cxx_qt_lib::QList<QString>;
    }

    impl cxx_qt::Threading for EcchanClient {}

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, connected, READ, WRITE = set_connected, NOTIFY)]
        #[qproperty(QString, path, READ, WRITE, NOTIFY)]
        #[qproperty(u8, fan_count, READ = fan_count, NOTIFY)]
        #[qproperty(u8, fan_max, READ, NOTIFY)]
        #[qproperty(bool, has_dgpu, READ, NOTIFY)]
        #[qproperty(u8, wmi_ver, READ = wmi_ver)]
        #[qproperty(QString, fw_version, READ, NOTIFY)]
        #[qproperty(QString, fw_date, READ, NOTIFY)]
        #[qproperty(QString, fw_time, READ, NOTIFY)]
        #[qproperty(QList_QString, shift_modes, READ = shift_modes, NOTIFY)]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        // #[qproperty(, )]
        #[namespace = "ec_socket"]
        type EcchanClient = super::EcchanClientRust;

        #[qsignal]
        fn error(self: Pin<&mut Self>, message: QString);

        fn set_connected(self: Pin<&mut Self>, connected: bool);
        fn fan_count(&self) -> u8;
        fn wmi_ver(&self) -> u8;

        fn shift_modes(&self) -> QList_QString;

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
    pub fn call(
        mut self: Pin<&mut Self>,
        method: MethodCall<'static>,
    ) -> Result<RetVal<'static>, ClientError> {
        if !self.connected || self.client.is_none() {
            if !matches!(method, MethodCall::Ping) {
                self.error(format!("not connected; cannot call {method:?}").into());
            }

            return Err(ClientError::Io {
                source: io::Error::new(io::ErrorKind::NotConnected, "not connected"),
            });
        }

        let mut this = self.as_mut().rust_mut();
        let res = this.client.as_mut().unwrap().call(&method);

        match res {
            Ok(v) => {
                let v: RetVal<'static> = match v {
                    RetVal::Methods(methods) => {
                        let methods: Vec<Method<'static>> = methods
                            .into_iter()
                            .map(|m| Method {
                                name: Cow::Owned(m.name.into_owned()),
                                method: Cow::Owned(m.method.into_owned()),
                                ops: m.ops,
                            })
                            .collect::<Vec<Method<'static>>>();

                        RetVal::Methods(methods)
                    }

                    RetVal::Pong => RetVal::Pong,
                    RetVal::Unit => RetVal::Unit,
                    RetVal::Byte(b) => RetVal::Byte(b),
                    RetVal::Word(w) => RetVal::Word(w),
                    RetVal::State(s) => RetVal::State(s),
                    RetVal::Str(s) => RetVal::Str(s),
                    RetVal::Fans(fans) => RetVal::Fans(fans),
                    RetVal::WmiVer(wmi_ver) => RetVal::WmiVer(wmi_ver),
                    RetVal::ShiftModes(shift_modes) => RetVal::ShiftModes(shift_modes),
                    RetVal::ShiftMode(shift_mode) => RetVal::ShiftMode(shift_mode),
                    RetVal::BatteryChargeMode(battery_charge_mode) => {
                        RetVal::BatteryChargeMode(battery_charge_mode)
                    }
                    RetVal::SuperBattery(super_battery) => RetVal::SuperBattery(super_battery),
                    RetVal::FanModes(fan_modes) => RetVal::FanModes(fan_modes),
                    RetVal::FanMode(fan_mode) => RetVal::FanMode(fan_mode),
                    RetVal::Webcam(webcam) => RetVal::Webcam(webcam),
                    RetVal::CoolerBoost(cooler_boost) => RetVal::CoolerBoost(cooler_boost),
                    RetVal::KeyDirection(key_direction) => RetVal::KeyDirection(key_direction),
                    RetVal::Led(led) => RetVal::Led(led),
                    RetVal::Curve6(curve6) => RetVal::Curve6(curve6),
                    RetVal::Curve7(curve7) => RetVal::Curve7(curve7),
                    RetVal::EcDump(bin) => RetVal::EcDump(bin),
                    RetVal::MethodData(method_data) => RetVal::MethodData(method_data),
                };

                Ok(v)
            }

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

                self.error(e.to_string().into());

                Err(e)
            }
        }
    }

    pub fn set_connected(mut self: Pin<&mut Self>, connected: bool) {
        if connected && self.as_ref().rust().client.is_none() {
            if self.as_ref().rust().path.is_empty() {
                self.error("path property must be set".into());
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
}
