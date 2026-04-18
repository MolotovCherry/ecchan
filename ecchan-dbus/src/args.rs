use dbus::{
    Signature,
    arg::{Append, Arg, ArgType, IterAppend},
};
use ecchan_ipc::{
    BatteryChargeMode, CoolerBoost, FanMode, Fans, KeyDirection, Led, MethodData, MethodOp,
    ShiftMode, SuperBattery, Webcam, WmiVer,
};

/// Wrapper which impls dbus traits for Ec types
#[derive(Clone)]
pub struct DbusArg<T: Clone>(pub T);

// ShiftMode

impl Append for DbusArg<ShiftMode> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        let s: &'static str = self.0.into();
        i.append(s);
    }
}

impl Arg for DbusArg<ShiftMode> {
    const ARG_TYPE: ArgType = ArgType::String;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("s\0") }
    }
}

// Webcam

impl Append for DbusArg<Webcam> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        i.append(self.0.enabled());
    }
}

impl Arg for DbusArg<Webcam> {
    const ARG_TYPE: ArgType = ArgType::Boolean;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("b\0") }
    }
}

// SuperBattery

impl Append for DbusArg<SuperBattery> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        i.append(self.0.enabled());
    }
}

impl Arg for DbusArg<SuperBattery> {
    const ARG_TYPE: ArgType = ArgType::Boolean;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("b\0") }
    }
}

// WmiVer

impl Append for DbusArg<WmiVer> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        let s: &'static str = self.0.into();
        i.append(s);
    }
}

impl Arg for DbusArg<WmiVer> {
    const ARG_TYPE: ArgType = ArgType::String;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("s\0") }
    }
}

// CoolerBoost

impl Append for DbusArg<CoolerBoost> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        i.append(self.0.enabled());
    }
}

impl Arg for DbusArg<CoolerBoost> {
    const ARG_TYPE: ArgType = ArgType::Boolean;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("b\0") }
    }
}

// Led

impl Append for DbusArg<Led> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        i.append(self.0.enabled());
    }
}

impl Arg for DbusArg<Led> {
    const ARG_TYPE: ArgType = ArgType::Boolean;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("b\0") }
    }
}

// KeyDirection

impl Append for DbusArg<KeyDirection> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        let s: &'static str = self.0.into();
        i.append(s);
    }
}

impl Arg for DbusArg<KeyDirection> {
    const ARG_TYPE: ArgType = ArgType::String;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("s\0") }
    }
}

// Fans

impl Append for DbusArg<Fans> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        let n: u8 = match self.0 {
            Fans::One => 1,
            Fans::Two => 2,
            Fans::Three => 3,
            Fans::Four => 4,
        };

        i.append(n);
    }
}

impl Arg for DbusArg<Fans> {
    const ARG_TYPE: ArgType = ArgType::Byte;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("y\0") }
    }
}

// FanMode

impl Append for DbusArg<FanMode> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        let s: &'static str = self.0.into();
        i.append(s);
    }
}

impl Arg for DbusArg<FanMode> {
    const ARG_TYPE: ArgType = ArgType::String;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("s\0") }
    }
}

// BatteryChargeMode

impl Append for DbusArg<BatteryChargeMode> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        let s: &'static str = self.0.into();
        i.append(s);
    }
}

impl Arg for DbusArg<BatteryChargeMode> {
    const ARG_TYPE: ArgType = ArgType::String;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("s\0") }
    }
}

// MethodOp

impl Append for DbusArg<MethodOp> {
    fn append_by_ref(&self, i: &mut IterAppend) {
        let s: &'static str = self.0.into();
        i.append(s);
    }
}

impl Arg for DbusArg<MethodOp> {
    const ARG_TYPE: ArgType = ArgType::String;

    fn signature() -> Signature<'static> {
        unsafe { Signature::from_slice_unchecked("s\0") }
    }
}
