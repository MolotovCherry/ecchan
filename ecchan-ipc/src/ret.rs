use ec::{
    BatteryChargeMode, CoolerBoost, Curve6, Curve7, FanMode, Fans, KeyDirection, Led, Method,
    MethodData, ShiftMode, SuperBattery, Webcam, WmiVer,
};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use snafu::Snafu;
use strum::{Display, IntoStaticStr};

macro_rules! extract_val {
    ($pat:ident, $variant:pat, $name:expr) => {
        extract_val!($pat, $variant, $name, v)
    };

    ($pat:ident, $variant:pat, $name:expr, $val:expr) => {
        match $pat {
            $variant => Ok($val),
            f => {
                return Err($crate::ret::RetValError::Unexpected {
                    expected: stringify!($variant),
                    got: f.into(),
                })
            }
        }
    };
}

#[derive(Debug, Clone, Snafu)]
pub enum RetValError {
    #[snafu(display("Expected {expected}, instead got {got}"))]
    Unexpected {
        expected: &'static str,
        got: &'static str,
    },
}

pub type Ret<'a> = Result<RetVal<'a>, String>;

/// The type to deserialize a server reply to
#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "Ret")]
enum _Ret<'a> {
    #[serde(borrow)]
    Ok(RetVal<'a>),
    Err(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Bin(#[serde(with = "BigArray")] pub [u8; 256]);

impl Default for Bin {
    fn default() -> Self {
        Self([0; _])
    }
}

/// A ipc call return value
#[derive(Debug, Serialize, Deserialize, IntoStaticStr, Display)]
pub enum RetVal<'a> {
    Pong,
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
    pub fn pong(self) -> Result<(), RetValError> {
        extract_val!(self, RetVal::Pong, "Pong", ())
    }

    pub fn byte(self) -> Result<u8, RetValError> {
        extract_val!(self, RetVal::Byte(v), "Byte", v)
    }

    pub fn word(self) -> Result<u16, RetValError> {
        extract_val!(self, RetVal::Word(v), "Word", v)
    }

    pub fn state(self) -> Result<bool, RetValError> {
        extract_val!(self, RetVal::State(v), "State", v)
    }

    pub fn str(self) -> Result<String, RetValError> {
        extract_val!(self, RetVal::Str(v), "Str", v)
    }

    pub fn fans(self) -> Result<Fans, RetValError> {
        extract_val!(self, RetVal::Fans(v), "Fans", v)
    }

    pub fn wmi_ver(self) -> Result<WmiVer, RetValError> {
        extract_val!(self, RetVal::WmiVer(v), "WmiVer", v)
    }

    pub fn shift_modes(self) -> Result<Vec<ShiftMode>, RetValError> {
        extract_val!(self, RetVal::ShiftModes(v), "ShiftModes", v)
    }

    pub fn shift_mode(self) -> Result<ShiftMode, RetValError> {
        extract_val!(self, RetVal::ShiftMode(v), "ShiftMode", v)
    }

    pub fn battery_charge_mode(self) -> Result<BatteryChargeMode, RetValError> {
        extract_val!(self, RetVal::BatteryChargeMode(v), "BatteryChargeMode", v)
    }

    pub fn super_battery(self) -> Result<SuperBattery, RetValError> {
        extract_val!(self, RetVal::SuperBattery(v), "SuperBattery", v)
    }

    pub fn fan_modes(self) -> Result<Vec<FanMode>, RetValError> {
        extract_val!(self, RetVal::FanModes(v), "FanModes", v)
    }

    pub fn fan_mode(self) -> Result<FanMode, RetValError> {
        extract_val!(self, RetVal::FanMode(v), "FanMode", v)
    }

    pub fn webcam(self) -> Result<Webcam, RetValError> {
        extract_val!(self, RetVal::Webcam(v), "Webcam", v)
    }

    pub fn cooler_boost(self) -> Result<CoolerBoost, RetValError> {
        extract_val!(self, RetVal::CoolerBoost(v), "CoolerBoost", v)
    }

    pub fn key_direction(self) -> Result<KeyDirection, RetValError> {
        extract_val!(self, RetVal::KeyDirection(v), "KeyDirection", v)
    }

    pub fn led(self) -> Result<Led, RetValError> {
        extract_val!(self, RetVal::Led(v), "Led", v)
    }

    pub fn curve6(self) -> Result<Curve6, RetValError> {
        extract_val!(self, RetVal::Curve6(v), "Curve6", v)
    }

    pub fn curve7(self) -> Result<Curve7, RetValError> {
        extract_val!(self, RetVal::Curve7(v), "Curve7", v)
    }

    pub fn ec_dump(self) -> Result<Box<Bin>, RetValError> {
        extract_val!(self, RetVal::EcDump(v), "EcDump", v)
    }

    pub fn methods(&self) -> Result<&[Method<'_>], RetValError> {
        extract_val!(self, RetVal::Methods(v), "Methods", v)
    }

    pub fn method_data(self) -> Result<MethodData, RetValError> {
        extract_val!(self, RetVal::MethodData(v), "MethodData", v)
    }
}
