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
    Unit,
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

impl RetVal<'_> {
    pub fn byte(self) -> u8 {
        match self {
            Self::Byte(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn word(self) -> u16 {
        match self {
            Self::Word(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn state(self) -> bool {
        match self {
            Self::State(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn str(self) -> String {
        match self {
            Self::Str(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn fans(self) -> Fans {
        match self {
            Self::Fans(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn wmi_ver(self) -> WmiVer {
        match self {
            Self::WmiVer(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn shift_modes(self) -> Vec<ShiftMode> {
        match self {
            Self::ShiftModes(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn shift_mode(self) -> ShiftMode {
        match self {
            Self::ShiftMode(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn battery_charge_mode(self) -> BatteryChargeMode {
        match self {
            Self::BatteryChargeMode(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn super_battery(self) -> SuperBattery {
        match self {
            Self::SuperBattery(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn fan_modes(self) -> Vec<FanMode> {
        match self {
            Self::FanModes(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn fan_mode(self) -> FanMode {
        match self {
            Self::FanMode(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn webcam(self) -> Webcam {
        match self {
            Self::Webcam(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn cooler_boost(self) -> CoolerBoost {
        match self {
            Self::CoolerBoost(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn key_direction(self) -> KeyDirection {
        match self {
            Self::KeyDirection(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn led(self) -> Led {
        match self {
            Self::Led(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn curve6(self) -> Curve6 {
        match self {
            Self::Curve6(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn curve7(self) -> Curve7 {
        match self {
            Self::Curve7(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn ec_dump(self) -> Box<Bin> {
        match self {
            Self::EcDump(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn methods(&self) -> &[Method<'_>] {
        match self {
            Self::Methods(v) => v,
            _ => unreachable!(),
        }
    }

    pub fn method_data(self) -> MethodData {
        match self {
            Self::MethodData(v) => v,
            _ => unreachable!(),
        }
    }
}
