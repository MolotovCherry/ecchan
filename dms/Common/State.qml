import QtQml

QtObject {
    property var ping: null

    property int fanCount: 1
    property int fanMax: 0
    property bool hasDGpu: false
    property int wmiVer: 1

    property string fwVersion: ""
    property string fwDate: ""
    property string fwTime: ""

    // array[string]
    property var shiftModes: []
    property string shiftMode: "Null"
    property bool shiftModeSupported: false

    // string | int
    property var batteryChargeMode: "Mobility"
    property bool batteryChargeModeSupported: false
    property bool superBattery: false
    property bool superBatterySupported: false

    property int fan1Rpm: 0
    property int fan2Rpm: 0
    property int fan3Rpm: 0
    property int fan4Rpm: 0
    property bool fan1Supported: true
    property bool fan2Supported: false
    property bool fan3Supported: false
    property bool fan4Supported: false

    // array[string]
    property var fanModes: []
    property string fanMode: "Null"
    property bool fanModeSupported: false

    property bool webcam: false
    property bool webcamBlock: false
    property bool webcamSupported: false
    property bool webcamBlockSupported: false

    property bool coolerBoost: false
    property bool coolerBoostSupported: false

    property string fnKey: "Right"
    property string winKey: "Left"
    property bool fnWinSwapSupported: false

    property bool micMuteLed: false
    property bool muteLed: false
    property bool micMuteLedSupported: false
    property bool muteLedSupported: false

    property int cpuRtFanSpeed: 0
    property int cpuRtTemp: 0
    property int gpuRtFanSpeed: 0
    property int gpuRtTemp: 0

    // array[int] (all of the below)
    property var cpuFanCurveWmi2: [0, 0, 0, 0, 0, 0, 0]
    property var cpuTempCurveWmi2: [0, 0, 0, 0, 0, 0, 0]
    property var cpuHysteresisCurveWmi2: [0, 0, 0, 0, 0, 0]
    property var gpuFanCurveWmi2: [0, 0, 0, 0, 0, 0, 0]
    property var gpuTempCurveWmi2: [0, 0, 0, 0, 0, 0, 0]
    property var gpuHysteresisCurveWmi2: [0, 0, 0, 0, 0, 0]

    // array[
    //   {
    //     "name": string,
    //     "method": string,
    //     "ops": array[
    //       "ReadBit" | "WriteBit"
    //       "Read" | "Write
    //       "ReadRange" | "WriteRange"
    //     ]
    //   }
    // ]
    property var methodList: []
    // (object is keyed by method key above)
    // {
    //   [method]: bool | int | array[int]
    // }
    property var methods: ({})

    property var ecDump: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    property string ecDumpPretty: "|      | _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _A _B _C _D _E _F\n|------+------------------------------------------------\n| 0x0_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x1_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x2_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x3_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x4_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x5_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x6_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x7_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x8_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x9_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xA_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xB_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xC_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xD_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xE_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xF_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n"

    function serialize() {
        return {
            "shiftMode": shiftMode,
            "batteryChargeMode": batteryChargeMode,
            "superBattery": superBattery,
            "fanMode": fanMode,
            "webcam": webcam,
            "webcamBlock": webcamBlock,
            "coolerBoost": coolerBoost,
            "fnKey": fnKey,
            "winKey": winKey,
            "micMuteLed": micMuteLed,
            "muteLed": muteLed,
            "cpuFanCurveWmi2": cpuFanCurveWmi2,
            "cpuTempCurveWmi2": cpuTempCurveWmi2,
            "cpuHysteresisCurveWmi2": cpuHysteresisCurveWmi2,
            "gpuFanCurveWmi2": gpuFanCurveWmi2,
            "gpuTempCurveWmi2": gpuTempCurveWmi2,
            "gpuHysteresisCurveWmi2": gpuHysteresisCurveWmi2,
            "methods": methods
        };
    }

    // arg: string of json data or object
    // returns object of (validated) data
    //
    // properties in object are validated and ignored otherwise; so ensure there
    // is a valid state
    //
    // this returned object can be supplied to EcSocket.applyState
    function deserialize(json) {
        let state;
        if (typeof (json) === "string") {
            state = JSON.parse(json);
        } else {
            state = json;
        }

        const outState = {};

        // Shift Modes

        if (typeof (state.shiftMode) === "string" && shiftModes.includes(state.shiftMode)) {
            outState.shiftMode = state.shiftMode;
        }

        // Battery Charge Mode
        // qmlformat off
        if (
            (
                typeof (state.batteryChargeMode) === "string" &&
                ["Healthy", "Balanced", "Mobility"].includes(state.batteryChargeMode)
            ) || (
                typeof (state.batteryChargeMode) === "number" &&
                state.batteryChargeMode >= 10 && state.batteryChargeMode <= 100
            )
        ) {
            outState.batteryChargeMode = state.batteryChargeMode;
        }
        // qmlformat on

        // SuperBattery

        if (typeof (state.superBattery) === "boolean") {
            outState.superBattery = state.superBattery;
        }

        // Fan

        if (typeof (state.fanMode) === "string" && fanModes.includes(state.fanMode)) {
            outState.fanMode = state.fanMode;
        }

        // Webcam

        if (typeof (state.webcam) === "boolean") {
            outState.webcam = state.webcam;
        }
        if (typeof (state.webcamBlock) === "boolean") {
            outState.webcamBlock = state.webcamBlock;
        }

        // Coolerboost

        if (typeof (state.coolerBoost) === "boolean") {
            outState.coolerBoost = state.coolerBoost;
        }

        // Switch keys

        if (typeof (state.fnKey) === "string" && ["Left", "Right"].includes(state.fnKey)) {
            outState.fnKey = state.fnKey;
        }
        if (typeof (state.winKey) === "string" && ["Left", "Right"].includes(state.winKey)) {
            outState.winKey = state.winKey;
        }

        // Leds

        if (typeof (state.micMuteLed) === "boolean") {
            outState.micMuteLed = state.micMuteLed;
        }
        if (typeof (state.muteLed) === "boolean") {
            outState.muteLed = state.muteLed;
        }

        // Curves (cpu)

        if (Array.isArray(state.cpuFanCurveWmi2) && state.cpuFanCurveWmi2.every(item => typeof (item) === "number" && item <= fanMax)) {
            outState.cpuFanCurveWmi2 = state.cpuFanCurveWmi2;
        }

        if (Array.isArray(state.cpuTempCurveWmi2) && state.cpuTempCurveWmi2.every(item => typeof (item) === "number")) {
            outState.cpuTempCurveWmi2 = state.cpuTempCurveWmi2;
        }

        if (Array.isArray(state.cpuHysteresisCurveWmi2) && state.cpuHysteresisCurveWmi2.every(item => typeof (item) === "number")) {
            outState.cpuHysteresisCurveWmi2 = state.cpuHysteresisCurveWmi2;
        }

        // Curves (gpu)

        if (Array.isArray(state.gpuFanCurveWmi2) && state.gpuFanCurveWmi2.every(item => typeof (item) === "number" && item <= fanMax)) {
            outState.gpuFanCurveWmi2 = state.gpuFanCurveWmi2;
        }

        if (Array.isArray(state.gpuTempCurveWmi2) && state.gpuTempCurveWmi2.every(item => typeof (item) === "number")) {
            outState.gpuTempCurveWmi2 = state.gpuTempCurveWmi2;
        }

        if (Array.isArray(state.gpuHysteresisCurveWmi2) && state.gpuHysteresisCurveWmi2.every(item => typeof (item) === "number")) {
            outState.gpuHysteresisCurveWmi2 = state.gpuHysteresisCurveWmi2;
        }

        // Methods
        outState.methods = {};
        // https://stackoverflow.com/a/51458052/9423933
        if (state.methods != null && state.methods.constructor.name === "Object") {
            for (const m of Object.keys(state.methods)) {
                if (typeof (m) === "string") {
                    const data = state.methods[m];
                    if (["number", "boolean"].includes(typeof (data)) || (Array.isArray(data) && data.every(item => typeof (item) === "number" && item >= 0 && item <= 255))) {
                        outState.methods[m] = data;
                    }
                }
            }
        }

        return outState;
    }
}
