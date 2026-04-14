#[cfg(not(target_os = "linux"))]
compile_error!("Only Linux is supported");

mod ec;
mod fw;
mod models;
mod single_instance;

pub use ec::{Ec, EcError, Method, MethodData};
pub use fw::{
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, KeyDirection, Led, ShiftMode,
    SuperBattery, Webcam, WmiVer,
};
pub use models::{Fans, MethodOp};
