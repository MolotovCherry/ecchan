use super::*;

pub const CONFIG: ModelConfig = ModelConfig {
    name: "Titan GT77 12UHS",
    has_dgpu: true,
    fans: FanConfig {
        max_speed: 150,
        count: Fans::Four,
    },
    methods: &[
        Method {
            name: "Display Overdrive",
            method: "display_overdrive",
            addr: Addr::Single(0x2E),
            ty: &[
                MethodTy::ReadBit {
                    bit: Bit::_4,
                    invert: false,
                },
                MethodTy::WriteBit {
                    bit: Bit::_4,
                    invert: false,
                },
            ],
        },
        Method {
            name: "USB Power Share",
            method: "usb_power_share",
            addr: Addr::Single(0xBF),
            ty: &[
                MethodTy::ReadBit {
                    bit: Bit::_5,
                    invert: false,
                },
                MethodTy::WriteBit {
                    bit: Bit::_5,
                    invert: false,
                },
            ],
        },
    ],
};
