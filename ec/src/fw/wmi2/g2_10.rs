use crate::fw::{
    Addr, Bit, CoolerBoost, Curve, CurveKind, FanMode, FanModeKind, FanRpm, FnWinSwap, FwConfig,
    KbdBl, Leds, ShiftMode, ShiftModeKind, SuperBattery, Thermal, Webcam,
};

pub const G2_10: FwConfig = FwConfig {
    allowed_fw: &[
        "1562EMS1.117", // Stealth 15M A11SEK
        "1563EMS1.106", // Stealth 15M A11UEK
        "1563EMS1.111",
        "1563EMS1.115",
        "1571EMS1.106", // Creator Z16 A11UE
        "1572EMS1.106", // Creator Z16 A12U
        "1572EMS1.107",
        "1587EMS1.102", // Katana 15 HX B14WEK
        "15F2EMS1.109", // Stealth 16 Studio A13VG
        "15F4EMS1.105", // Stealth 16 AI Studio A1VFG
        "15F4EMS1.106",
        "15FKIMS1.106", // Stealth A16 AI+ A3XVFG / A3XVGG
        "15FKIMS1.109",
        "15FKIMS1.110", // Stealth A16 AI+ A3XVGG
        "15FLIMS1.107", // Stealth A16 AI+ A3XWHG
        "15K2EMS1.106", // Cyborg 15 AI A1VFK
        "15M1IMS1.109", // Vector GP68 HX 13V
        "15M1IMS1.110",
        "15M1IMS1.113", // Vector GP68 HX 12V
        "15M1IMS2.104", // Raider GE68 HX 14VIG
        "15M1IMS2.105", // Vector 16 HX A13V* / A14V*
        "15M1IMS2.111",
        "15M1IMS2.112",
        "15M2IMS2.112", // Raider GE68 HX 14VGG
        "15M2IMS1.110", // Raider GE68HX 13V(F/G)
        "15M2IMS1.112", // Vector GP68HX 13VF
        "15M2IMS1.113",
        "15M2IMS1.114",
        "15M3EMS1.105", // Vector 16 HX AI A2XWHG / A2XWIG
        "15M3EMS1.106",
        "15M3EMS1.107",
        "15M3EMS1.109",
        "15M3EMS1.110",
        "15M3EMS1.112",
        "15M3EMS1.113",
        "15P2EMS1.108", // Sword 16 HX B13V / B14V
        "15P2EMS1.110",
        "15P3EMS1.103", // Pulse 16 AI C1VGKG/C1VFKG
        "15P3EMS1.106",
        "15P3EMS1.107",
        "15P4EMS1.105", // Crosshair 16 HX AI D2XW(GKG)
        "15P4EMS1.107",
        "17L5EMS1.111", // Pulse/Katana 17 B13V/GK
        "17L5EMS1.113",
        "17L5EMS1.115",
        "17L5EMS2.115", // Katana 17 B12VEK
        "17L7EMS1.102", // Katana 17 HX B14WGK
        "17N1EMS1.109", // Creator Z17 A12UGST
        "17P1EMS1.104", // Stealth GS77 12U(E/GS)
        "17P1EMS1.106",
        "17P2EMS1.111", // Stealth 17 Studio A13VI
        "17Q1IMS1.10C", // Titan GT77 12UHS
        "17Q2IMS1.107", // Titan GT77HX 13VH
        "17Q2IMS1.10D",
        "17S1IMS1.105", // Raider GE78HX 13VI
        "17S1IMS1.113",
        "17S1IMS1.114",
        "17S1IMS2.104", // Raider GE78 HX 14VHG
        "17S1IMS2.107", // Vector 17 HX A14V
        "17S1IMS2.111", // Vector 17 HX A13VHG
        "17S1IMS2.112",
        "17S2IMS1.113", // Raider GE78 HX Smart Touchpad 13V
        "17S3EMS1.104", // Vector 17 HX AI A2XWHG
        "17T2EMS1.110", // Sword 17 HX B14VGKG
        "1822EMS1.105", // Titan 18 HX A14V
        "1822EMS1.109", // WMI 2.8
        "1822EMS1.111",
        "1822EMS1.112",
        "1822EMS1.114",
        "1822EMS1.115",
        "1824EMS1.107", // Titan 18 HX Dragon Edition
        "182LIMS1.108", // Vector A18 HX A9WHG
        "182LIMS1.111", // New ec version for Vector A18 HX A9WHG
        "182KIMS1.113", // Raider A18 HX A7VIG
    ],
    charge_control_addr: Addr::Addr(0xD7),
    webcam: Webcam {
        addr: Addr::Addr(0x2E),
        block_addr: Addr::Addr(0x2F),
        bit: Bit::_1,
    },
    fn_win_swap: FnWinSwap {
        addr: Addr::Addr(0xE8),
        bit: Bit::_4,
        invert: true,
    },
    cooler_boost: CoolerBoost {
        addr: Addr::Addr(0x98),
        bit: Bit::_7,
    },
    shift_mode: ShiftMode {
        addr: Addr::Addr(0xD2),
        modes: &[
            (ShiftModeKind::SuperBattery, 0xC2),
            (ShiftModeKind::Balanced, 0xC1),
            (ShiftModeKind::ExtremePerformance, 0xC0),
            (ShiftModeKind::Turbo, 0xC4),
            (ShiftModeKind::Null, 0x00),
        ],
    },
    super_battery: SuperBattery {
        addr: Addr::Addr(0xEB),
        mask: 0x0F,
    },
    fan_mode: FanMode {
        addr: Addr::Addr(0xD4),
        modes: &[
            (FanModeKind::Auto, 0x0D),
            (FanModeKind::Silent, 0x1D),
            (FanModeKind::Advanced, 0x8D),
            (FanModeKind::Null, 0x00),
        ],
    },
    cpu: Thermal {
        rt_temp_addr: Addr::Addr(0x68),
        rt_fan_speed_addr: Addr::Addr(0x71),
    },
    gpu: Thermal {
        rt_temp_addr: Addr::Addr(0x80),
        rt_fan_speed_addr: Addr::Addr(0x89),
    },
    leds: Leds {
        mic_mute_led_addr: Addr::Addr(0x2C),
        mute_led_addr: Addr::Addr(0x2D),
        bit: Bit::_1,
    },
    kbd_bl: KbdBl {
        bl_mode_addr: Addr::Unsupported,
        bl_modes: &[0x00, 0x08],
        max_mode: 1,
        bl_state_addr: Addr::Unsupported,
        state_base_value: 0x80,
        max_state: 3,
    },
    fan_rpm: FanRpm {
        fan1_addr: 0xC8,
        fan2_addr: 0xCA,
        fan3_addr: 0xCC,
        fan4_addr: 0xCE,
    },
    // 0x72-0x78
    cpu_fan_curve: Curve {
        addr: Addr::Addr(0x72),
        kind: CurveKind::Curve7,
    },
    // 0x69-0x6F
    cpu_temp_curve: Curve {
        addr: Addr::Addr(0x69),
        kind: CurveKind::Curve7,
    },
    // 0x7A-0x7F
    cpu_hysteresis_curve: Curve {
        addr: Addr::Addr(0x7A),
        kind: CurveKind::Curve6,
    },
    // 0x8A-0x8F
    gpu_fan_curve: Curve {
        addr: Addr::Addr(0x8A),
        kind: CurveKind::Curve6,
    },
    // 0x81-0x87
    gpu_temp_curve: Curve {
        addr: Addr::Addr(0x81),
        kind: CurveKind::Curve7,
    },
    // 0x92-0x97
    gpu_hysteresis_curve: Curve {
        addr: Addr::Addr(0x92),
        kind: CurveKind::Curve6,
    },
};
