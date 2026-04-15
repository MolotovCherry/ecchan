use ec::{
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, Fans, KeyDirection, Led, Method,
    MethodData, ShiftMode, SuperBattery, Webcam, WmiVer,
};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

#[derive(Debug, Serialize, Deserialize)]
pub enum Ret<'a> {
    #[serde(borrow)]
    Ok(RetVal<'a>),
    Err(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Bin(#[serde(with = "BigArray")] pub [u8; 256]);

/// A ipc call return value. Make a value of this and json serialize it to call.
/// Look at original functions for reply type to deserialize from
#[derive(Debug, Serialize, Deserialize)]
pub enum RetVal<'a> {
    /// Call returned no data, but was successful
    Empty,
    Byte(u8),
    Word(u16),
    State(bool),
    Str(String),
    Fans(Fans),
    WmiVer(WmiVer),
    ShiftModes(Vec<ShiftMode>),
    ShiftMode(ShiftMode),
    BatteryChargeMode(BatteryChargeMode),
    SuperBattery(SuperBattery),
    FanModes(Vec<FanMode>),
    FanMode(FanMode),
    Webcam(Webcam),
    CoolerBoost(CoolerBoost),
    KeyDirection(KeyDirection),
    Led(Led),
    Curve6(Curve6),
    Curve7(Curve7),
    EcDump(Box<Bin>),
    #[serde(borrow)]
    Methods(Vec<Method<'a>>),
    MethodData(MethodData),
}
