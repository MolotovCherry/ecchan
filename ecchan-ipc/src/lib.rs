pub mod method;
pub mod ret;
pub use ec::{
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, Fans, KeyDirection, Led, Method,
    MethodData, MethodOp, ShiftMode, SuperBattery, Webcam, WmiVer,
};

pub const SOCK: &str = "/run/ecchan.sock";

/// Get the socket path from env ECCHAN_SOCK or fallback to default /run/ecchan.sock
pub fn get_socket_path() -> String {
    std::env::var("ECCHAN_SOCK").unwrap_or_else(|_| SOCK.to_owned())
}
