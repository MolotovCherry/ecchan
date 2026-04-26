pragma Singleton
pragma ComponentBehavior: Bound

import QtQuick

import Quickshell
import Quickshell.Io
import qs.Common
import qs.Services

Singleton {
    id: root

    property bool connected: false

    signal initFinished
    signal applyFinished

    // the state of our api at any given point in time
    // can also be used for saving/loading prefs.
    //
    // Only keys which can be set are written when imported,
    // so someone can't for example change "hasDGPU" to true
    //
    // qmlformat off
    property var state: {
        "fanCount": 1,                       // int
        "fanMax": null,                      // null | int
        "hasDGpu": false,                    // bool
        "wmiVer": 1,                         // int

        "shiftModes": [],                    // array[string]
        "shiftMode": null,                   // null | string
        "shiftModeSupported": false,         // bool

        "batteryChargeMode": null,           // null | string | int
        "batteryChargeModeSupported": false, // bool
        "superBattery": false,               // bool
        "superBatterySupported": false,      // bool

        "fan1Rpm": 0,                        // int
        "fan2Rpm": 0,                        // int
        "fan3Rpm": 0,                        // int
        "fan4Rpm": 0,                        // int
        "fan1Supported": true,               // bool
        "fan2Supported": true,               // bool
        "fan3Supported": true,               // bool
        "fan4Supported": true,               // bool

        "fanModes": [],                      // array[string]
        "fanMode": null,                     // null | string
        "fanModeSupported": false,           // bool

        "webcam": false,                     // bool
        "webcamBlock": false,                // bool
        "webcamSupported": false,            // bool
        "webcamBlockSupported": false,       // bool

        "coolerBoost": false,                // bool
        "coolerBoostSupported": false,       // bool

        "fnKey": "Right",                    // string
        "winKey": "Left",                    // string
        "fnWinSwapSupported": false,         // bool

        "micMuteLed": false,                 // bool
        "muteLed": false,                    // bool
        "micMuteLedSupported": false,        // bool
        "muteLedSupported": false,           // bool

        "cpuRtFanSpeed": 0,                  // int
        "cpuRtTemp": 0,                      // int
        "gpuRtFanSpeed": 0,                  // int
        "gpuRtTemp": 0,                      // int

        "cpuFanCurveWmi2":  [0, 0, 0, 0, 0, 0, 0],    // array[int]
        "cpuTempCurveWmi2": [0, 0, 0, 0, 0, 0, 0],    // array[int]
        "cpuHysteresisCurveWmi2": [0, 0, 0, 0, 0, 0], // array[int]
        "gpuFanCurveWmi2":  [0, 0, 0, 0, 0, 0, 0],    // array[int]
        "gpuTempCurveWmi2": [0, 0, 0, 0, 0, 0, 0],    // array[int]
        "gpuHysteresisCurveWmi2": [0, 0, 0, 0, 0, 0], // array[int]

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
        "methodList": [],
        // (object is keyed by method key above)
        // {
        //   [method]: bool | int | array[int]
        // }
        "methods": {}
    }
    // qmlformat on

    property string _socketFile
    property var _cb: null
    property var _cbErr: null
    property var _callQueue: []

    property DankSocket _socket

    property Component _socketComponent: DankSocket {
        id: socket
        connected: true

        onConnectionStateChanged: {
            if (connected) {
                root.connected = true;
                pingTimer.start();

                Qt.callLater(root._initState);
            }

            if (!connected) {
                root.connected = false;
                _reset();
            }
        }

        parser: SplitParser {
            onRead: line => {
                try {
                    // { "Ok": { .. } } / { "Err": "" }
                    const reply = JSON.parse(line);

                    if (reply.hasOwnProperty("Err")) {
                        console.error("Call returned error:", reply.Err);
                        ToastService.showError("Ecchan ipc call failed", reply.Err);

                        try {
                            _cbErr?.(reply.Err);
                        } catch (e) {}

                        return;
                    } else if (!reply.hasOwnProperty("Ok")) {
                        console.error("Failed to parse reply:", line);
                        ToastService.showError("Ecchan failed to parse server reply", line);

                        try {
                            _cbErr?.(line);
                        } catch (e) {}

                        return;
                    }

                    const data = root._handleReply(reply.Ok);

                    try {
                        _cb?.(data);
                    } catch (e) {
                        console.error("Cb failed:", e);
                        ToastService.showError("Ecchan ipc cb failed", e);
                    }
                } catch (e) {
                    console.error("Failed to parse reply:", line, e);
                    ToastService.showError("Ecchan failed to parse server reply", `${e}\n\n${line}`);

                    try {
                        _cbErr?.(reply.Err);
                    } catch (e) {}
                }

                root._cb = null;
                root._cbErr = null;
                _callQueueNext();
            }
        }
    }

    function _callQueueNext() {
        if (_cb == null && _callQueue.length > 0) {
            const call = _callQueue.shift();
            call();
        }
    }

    function _reset() {
        watchdogTimer.stop();
        pingTimer.stop();

        _socket?.destroy();
        _socket = null;

        _callQueue = [];
        _cb = null;
        _cbErr = null;

        if (connected) {
            connected = false;
        }
    }

    function getSanitizedState() {
        // qmlformat off
        const onlyKeep = [
            "shiftMode",
            "batteryChargeMode",
            "superBattery",
            "fanMode",
            "webcam",
            "webcamBlock",
            "coolerBoost",
            "fnKey",
            "winKey",
            "micMuteLed",
            "muteLed",
            "cpuFanCurveWmi2",
            "cpuTempCurveWmi2",
            "cpuHysteresisCurveWmi2",
            "gpuFanCurveWmi2",
            "gpuTempCurveWmi2",
            "gpuHysteresisCurveWmi2",
            "methods"
        ];
        // qmlformat on

        const out = {};
        for (let i = 0; i < onlyKeep.length; i++) {
            const k = onlyKeep[i];
            if (k in state) {
                out[k] = state[k];
            }
        }
        return out;
    }

    // args:
    // data: { socketFile: "/path/to/file.sock" }
    function init(socketFile) {
        _reset();

        _socketFile = socketFile;
        _socket = _socketComponent.createObject(null, {
            path: socketFile
        });
    }

    function reconnect() {
        if (_socketFile != null) {
            init(_socketFile);
        }
    }

    function shutdown() {
        _reset();
    }

    Timer {
        id: watchdogTimer
        interval: 2000
        repeat: false
        onTriggered: _reset()
    }

    Timer {
        id: pingTimer
        interval: 1500
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            if (!watchdogTimer.running) {
                watchdogTimer.start();
            }

            root.ping(watchdogTimer.restart);
        }
    }

    function _handleReply(reply) {
        let key;
        let value;

        if (typeof (reply) === "string") {
            key = reply;
        } else {
            key = Object.keys(reply)[0];
            value = Object.values(reply)[0];
        }

        switch (key) {
            // qmlformat off

            case "Pong":
            case "Unit":
                return;

            case "Byte":
            case "Word":
            case "State":
            case "Str":
            case "ShiftModes":
            case "ShiftMode":
            case "FanModes":
            case "FanMode":
            case "KeyDirection":
            case "EcDump":
            case "Methods":
                return value;

            case "Fans":
                switch (value) {
                    case "One":
                    default:
                        return 1;
                    case "Two":
                        return 2;
                    case "Three":
                        return 3;
                    case "Four":
                        return 4;
                }

            case "WmiVer":
                switch (value) {
                    case "Wmi1":
                        return 1;
                    case "Wmi2":
                        return 2;
                    default:
                        return 1;
                }

            case "BatteryChargeMode":
                if (typeof (value) === "string") {
                    return value;
                } else {
                    const ckey = Object.keys(value)[0];
                    const cvalue = Object.values(value)[0];

                    switch (ckey) {
                        case "Custom":
                            return cvalue;
                        default:
                            console.error("Invalid Custom key", ckey, cvalue);
                            break;
                    }
                }

            case "SuperBattery":
            case "Webcam":
            case "WebcamBlock":
            case "CoolerBoost":
            case "Led":
                switch (value) {
                    case "On":
                        return true;
                    case "Off":
                    default:
                        return false;
                }

            case "Curve6":
                return [value.n1, value.n2, value.n3, value.n4, value.n5, value.n6];

            case "Curve7":
                return [value.n1, value.n2, value.n3, value.n4, value.n5, value.n6, value.n7];

            case "MethodData":
                const mkey = Object.keys(value)[0];
                const mvalue = Object.values(value)[0];

                switch (mkey) {
                    case "Bit":
                    case "Byte":
                    case "Range":
                        return mvalue;
                    default:
                        console.error("Invalid MethodData key", mkey, mvalue);
                        break;
                }

            // qmlformat on
        }
    }

    function applyState(newState) {
        if (state.shiftModeSupported && (typeof (newState.shiftMode) === "string")) {
            setShiftMode(newState.shiftMode);
        }

        if (state.batteryChargeModeSupported && ((typeof (newState.batteryChargeMode) === "string") || (typeof (newState.batteryChargeMode) === "number"))) {
            setBatteryChargeMode(newState.batteryChargeMode);
        }
        if (state.superBatterySupported && (typeof (newState.superBattery) === "boolean")) {
            setSuperBattery(newState.superBattery);
        }

        if (state.fanModeSupported && (typeof (newState.fanMode) === "string")) {
            setFanMode(newState.fanMode);
        }

        if (state.webcamSupported && (typeof (newState.webcam) === "boolean")) {
            setWebcam(newState.webcam);
        }
        if (state.webcamBlockSupported && (typeof (newState.webcamBlock) === "boolean")) {
            setWebcamBlock(newState.webcamBlock);
        }

        if (state.coolerBoostSupported && (typeof (newState.coolerBoost) === "boolean")) {
            setCoolerBoost(newState.coolerBoost);
        }

        // we only need to set one of these because it swaps the
        // win key at the same time; so setting Win key is redundant
        if (state.fnWinSwapSupported && (typeof (newState.fnKey) === "string")) {
            setFnKey(newState.fnKey);
        }

        if (state.micMuteLedSupported && (typeof (newState.micMuteLed) === "boolean")) {
            setMicMuteLed(newState.micMuteLed);
        }
        if (state.muteLedSupported && (typeof (newState.muteLed) === "boolean")) {
            setMuteLed(newState.muteLed);
        }

        if (state.wmiVer === 2) {
            if (Array.isArray(newState.cpuFanCurveWmi2)) {
                setCpuFanCurveWmi2(newState.cpuFanCurveWmi2);
            }
            if (Array.isArray(newState.cpuTempCurveWmi2)) {
                setCpuTempCurveWmi2(newState.cpuTempCurveWmi2);
            }
            if (Array.isArray(newState.cpuHysteresisCurveWmi2)) {
                setCpuHysteresisCurveWmi2(newState.cpuHysteresisCurveWmi2);
            }

            if (state.hasDGpu) {
                if (Array.isArray(newState.gpuFanCurveWmi2)) {
                    setGpuFanCurveWmi2(newState.gpuFanCurveWmi2);
                }
                if (Array.isArray(newState.gpuTempCurveWmi2)) {
                    setGpuTempCurveWmi2(newState.gpuTempCurveWmi2);
                }
                if (Array.isArray(newState.gpuHysteresisCurveWmi2)) {
                    setGpuHysteresisCurveWmi2(newState.gpuHysteresisCurveWmi2);
                }
            }
        }

        // https://stackoverflow.com/a/51458052/9423933
        if (newState.methods != null && newState.methods.constructor.name === "Object") {
            for (const m of state.methodList) {
                for (const op of m.ops) {
                    if (typeof (op) === "string" && op.startsWith("Write")) {
                        methodWrite(m.method, op, newState.methods[m.method]);
                    }
                }
            }
        }

        ping(() => {
            applyFinished();
        });
    }

    function _initState() {
        fanCount();
        fanMax();
        hasDGpu();
        wmiVer(ver => {
            if (ver === 2) {
                cpuFanCurveWmi2();
                cpuTempCurveWmi2();
                cpuHysteresisCurveWmi2();

                if (state.hasDGpu) {
                    gpuFanCurveWmi2();
                    gpuTempCurveWmi2();
                    gpuHysteresisCurveWmi2();
                }
            }
        });

        fwVersion();
        fwDate();
        fwTime();

        shiftModes();
        shiftMode();
        shiftModeSupported();

        batteryChargeMode();
        batteryChargeModeSupported();
        superBattery();
        superBatterySupported();

        fan1Rpm();
        fan2Rpm();
        fan3Rpm();
        fan4Rpm();
        fan1Supported();
        fan2Supported();
        fan3Supported();
        fan4Supported();

        fanModes();
        fanMode();
        fanModeSupported();

        webcam();
        webcamBlock();
        webcamSupported();
        webcamBlockSupported();

        coolerBoost();
        coolerBoostSupported();

        fnKey();
        winKey();
        fnWinSwapSupported();

        micMuteLed();
        muteLed();
        micMuteLedSupported();
        muteLedSupported();

        cpuRtFanSpeed();
        cpuRtTemp();
        gpuRtTemp();
        gpuRtFanSpeed();

        methodList(list => {
            for (const m of list) {
                for (const op of m.ops) {
                    if (op.startsWith("Read")) {
                        methodRead(m.method, op);
                    }
                }
            }

            // dummy ping to schedule event after all the others
            ping(() => {
                initFinished();
            });
        });
    }

    // Take in "MethodName" and convert to our state key,
    // which is "methodName". If "SetMethodName" was given in,
    // remove "Set", and lowercase to "methodName"
    function _getStateKey(method) {
        if (method.startsWith('Set')) {
            method = method.slice(3);
        }

        return method[0].toLowerCase() + method.slice(1);
    }

    //method, data, cb, cbErr
    function _call(callData) {
        _callQueue.push(() => {
            const isSet = callData.method.startsWith('Set') || callData.method === "MethodWrite";

            let stateKey;
            // qmlformat off
            switch (callData.method) {
                case "MethodRead":
                case "MethodWrite":
                    stateKey = "methods";
                    break;

                default:
                    stateKey = _getStateKey(callData.method);
                    break;
            }
            // qmlformat on

            if (isSet) {
                root._cb = data => {
                    if (stateKey === "methods") {
                        root.state[stateKey][callData.raw.method] = callData.raw.data;
                    } else {
                        root.state[stateKey] = callData.raw;
                    }

                    root.stateChanged();
                    callData.cb?.(data);
                };
            } else {
                root._cb = data => {
                    if (stateKey === "methods") {
                        root.state[stateKey][callData.raw.method] = data;
                    } else {
                        root.state[stateKey] = data;
                    }

                    root.stateChanged();
                    callData.cb?.(data);
                };
            }

            root._cbErr = callData.cbErr || null;

            let json;
            if (typeof (callData.method) === "string" && callData.payload === undefined) {
                json = JSON.stringify(callData.method);
            } else if (typeof (callData.method) === "string" && callData.payload !== undefined) {
                json = JSON.stringify({
                    [callData.method]: callData.payload
                });
            } else {
                console.error("why is method undefined?");
                return;
            }

            // calls will be lost if not connected; this is acceptable
            _socket?.send(json);
        });

        _callQueueNext();
    }

    // Utils

    function ping(cb) {
        _call({
            "method": "Ping",
            "cb": cb
        });
    }

    function fanCount(cb) {
        _call({
            "method": "FanCount",
            "cb": cb
        });
    }

    function fanMax(cb, cbErr) {
        _call({
            "method": "FanMax",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function hasDGpu(cb) {
        _call({
            "method": "HasDGpu",
            "cb": cb
        });
    }

    function wmiVer(cb, cbErr) {
        _call({
            "method": "WmiVer",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    // Firmware

    function fwVersion(cb, cbErr) {
        _call({
            "method": "FwVersion",
            "cb": cb
        });
    }

    function fwDate(cb, cbErr) {
        _call({
            "method": "FwDate",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function fwTime(cb, cbErr) {
        _call({
            "method": "FwTime",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    // Shift Modes

    function shiftModes(cb, cbErr) {
        _call({
            "method": "ShiftModes",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function shiftMode(mode, cb, cbErr) {
        _call({
            "method": "ShiftMode",
            "cb": cb
        });
    }

    function setShiftMode(mode, cb, cbErr) {
        _call({
            "method": "SetShiftMode",
            "raw": mode,
            "payload": {
                "mode": mode
            },
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function shiftModeSupported(cb) {
        _call({
            "method": "ShiftModeSupported",
            "cb": cb
        });
    }

    // Battery

    function batteryChargeMode(cb, cbErr) {
        _call({
            "method": "BatteryChargeMode",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setBatteryChargeMode(mode, cb, cbErr) {
        let data;
        if (typeof (mode) === "string") {
            data = {
                "mode": mode
            };
        } else if (typeof (mode) === "number") {
            data = {
                "mode": {
                    "Custom": mode
                }
            };
        }

        _call({
            "method": "SetBatteryChargeMode",
            "raw": mode,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function batteryChargeModeSupported(cb) {
        _call({
            "method": "BatteryChargeModeSupported",
            "cb": cb
        });
    }

    function superBattery(cb, cbErr) {
        _call({
            "method": "SuperBattery",
            "cb": cb
        });
    }

    function setSuperBattery(mode, cb, cbErr) {
        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        let data = {
            "state": state
        };

        _call({
            "method": "SetSuperBattery",
            "raw": mode,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function superBatterySupported(cb) {
        _call({
            "method": "SuperBatterySupported",
            "cb": cb
        });
    }

    // Fan
    function fan1Rpm(cb, cbErr) {
        _call({
            "method": "Fan1Rpm",
            "cb": cb
        });
    }

    function fan2Rpm(cb, cbErr) {
        _call({
            "method": "Fan2Rpm",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function fan3Rpm(cb, cbErr) {
        _call({
            "method": "Fan3Rpm",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function fan4Rpm(cb, cbErr) {
        _call({
            "method": "Fan4Rpm",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function fan1Supported(cb) {
        _call({
            "method": "Fan1Supported",
            "cb": cb
        });
    }

    function fan2Supported(cb) {
        _call({
            "method": "Fan2Supported",
            "cb": cb
        });
    }

    function fan3Supported(cb) {
        _call({
            "method": "Fan3Supported",
            "cb": cb
        });
    }

    function fan4Supported(cb) {
        _call({
            "method": "Fan4Supported",
            "cb": cb
        });
    }

    function fanModes(cb, cbErr) {
        _call({
            "method": "FanModes",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function fanMode(cb, cbErr) {
        _call({
            "method": "FanMode",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setFanMode(mode, cb, cbErr) {
        let data = {
            "mode": mode
        };

        _call({
            "method": "SetFanMode",
            "raw": mode,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function fanModeSupported(cb) {
        _call({
            "method": "FanModeSupported",
            "cb": cb
        });
    }

    // Webcam

    function webcam(mode, cb, cbErr) {
        _call({
            "method": "Webcam",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setWebcam(mode, cb, cbErr) {
        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        let data = {
            "state": state
        };

        _call({
            "method": "SetWebcam",
            "raw": mode,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function webcamBlock(mode, cb, cbErr) {
        _call({
            "method": "WebcamBlock",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setWebcamBlock(mode, cb, cbErr) {
        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        let data = {
            "state": state
        };

        _call({
            "method": "SetWebcamBlock",
            "raw": mode,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function webcamSupported(cb) {
        _call({
            "method": "WebcamSupported",
            "cb": cb
        });
    }

    function webcamBlockSupported(cb) {
        _call({
            "method": "WebcamBlockSupported",
            "cb": cb
        });
    }

    // Cooler Boost

    function coolerBoost(cb, cbErr) {
        _call({
            "method": "CoolerBoost",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setCoolerBoost(mode, cb, cbErr) {
        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        let data = {
            "state": state
        };

        _call({
            "method": "SetCoolerBoost",
            "raw": mode,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function coolerBoostSupported(cb) {
        _call({
            "method": "CoolerBoostSupported",
            "cb": cb
        });
    }

    // Swap Keys

    function fnKey(cb, cbErr) {
        _call({
            "method": "FnKey",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setFnKey(dir, cb, cbErr) {
        let data = {
            "state": dir
        };

        _call({
            "method": "SetFnKey",
            "raw": dir,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function winKey(cb, cbErr) {
        _call({
            "method": "WinKey",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setWinKey(dir, cb, cbErr) {
        let data = {
            "state": dir
        };

        _call({
            "method": "SetWinKey",
            "raw": dir,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function fnWinSwapSupported(cb) {
        _call({
            "method": "FnWinSwapSupported",
            "cb": cb
        });
    }

    // Mute LEDs

    function micMuteLed(cb, cbErr) {
        _call({
            "method": "MicMuteLed",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setMicMuteLed(state, cb, cbErr) {
        let val;
        if (state) {
            val = "On";
        } else {
            val = "Off";
        }

        let data = {
            "state": val
        };

        _call({
            "method": "SetMicMuteLed",
            "raw": state,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function muteLed(cb, cbErr) {
        _call({
            "method": "MuteLed",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setMuteLed(state, cb, cbErr) {
        let val;
        if (state) {
            val = "On";
        } else {
            val = "Off";
        }

        let data = {
            "state": val
        };

        _call({
            "method": "SetMuteLed",
            "raw": state,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function micMuteLedSupported(cb) {
        _call({
            "method": "MicMuteLedSupported",
            "cb": cb
        });
    }

    function muteLedSupported(cb) {
        _call({
            "method": "MuteLedSupported",
            "cb": cb
        });
    }

    // Realtime Stats

    function cpuRtFanSpeed(cb, cbErr) {
        _call({
            "method": "CpuRtFanSpeed",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function cpuRtTemp(cb, cbErr) {
        _call({
            "method": "CpuRtTemp",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function gpuRtFanSpeed(cb, cbErr) {
        _call({
            "method": "GpuRtFanSpeed",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function gpuRtTemp(cb, cbErr) {
        _call({
            "method": "GpuRtTemp",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    // Curves

    function cpuFanCurveWmi2(cb, cbErr) {
        _call({
            "method": "CpuFanCurveWmi2",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setCpuFanCurveWmi2(curve, cb, cbErr) {
        let data = {
            "curve": {
                "n1": curve[0],
                "n2": curve[1],
                "n3": curve[2],
                "n4": curve[3],
                "n5": curve[4],
                "n6": curve[5],
                "n7": curve[6]
            }
        };

        _call({
            "method": "SetCpuFanCurveWmi2",
            "raw": curve,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function cpuTempCurveWmi2(cb, cbErr) {
        _call({
            "method": "CpuTempCurveWmi2",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setCpuTempCurveWmi2(curve, cb, cbErr) {
        let data = {
            "curve": {
                "n1": curve[0],
                "n2": curve[1],
                "n3": curve[2],
                "n4": curve[3],
                "n5": curve[4],
                "n6": curve[5],
                "n7": curve[6]
            }
        };

        _call({
            "method": "SetCpuTempCurveWmi2",
            "raw": curve,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function cpuHysteresisCurveWmi2(cb, cbErr) {
        _call({
            "method": "CpuHysteresisCurveWmi2",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setCpuHysteresisCurveWmi2(curve, cb, cbErr) {
        let data = {
            "curve": {
                "n1": curve[0],
                "n2": curve[1],
                "n3": curve[2],
                "n4": curve[3],
                "n5": curve[4],
                "n6": curve[5]
            }
        };

        _call({
            "method": "SetCpuHysteresisCurveWmi2",
            "raw": curve,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function gpuFanCurveWmi2(curve, cb, cbErr) {
        _call({
            "method": "GpuFanCurveWmi2",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setGpuFanCurveWmi2(curve, cb, cbErr) {
        let data = {
            "curve": {
                "n1": curve[0],
                "n2": curve[1],
                "n3": curve[2],
                "n4": curve[3],
                "n5": curve[4],
                "n6": curve[5],
                "n7": curve[6]
            }
        };

        _call({
            "method": "SetGpuFanCurveWmi2",
            "raw": curve,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function gpuTempCurveWmi2(cb, cbErr) {
        _call({
            "method": "GpuTempCurveWmi2",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setGpuTempCurveWmi2(curve, cb, cbErr) {
        let data = {
            "curve": {
                "n1": curve[0],
                "n2": curve[1],
                "n3": curve[2],
                "n4": curve[3],
                "n5": curve[4],
                "n6": curve[5],
                "n7": curve[6]
            }
        };

        _call({
            "method": "SetGpuTempCurveWmi2",
            "raw": curve,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function gpuHysteresisCurveWmi2(cb, cbErr) {
        _call({
            "method": "GpuHysteresisCurveWmi2",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function setGpuHysteresisCurveWmi2(curve, cb, cbErr) {
        let data = {
            "curve": {
                "n1": curve[0],
                "n2": curve[1],
                "n3": curve[2],
                "n4": curve[3],
                "n5": curve[4],
                "n6": curve[5]
            }
        };

        _call({
            "method": "SetGpuHysteresisCurveWmi2",
            "raw": curve,
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    // Ec

    function ecDump(cb, cbErr) {
        _call({
            "method": "EcDump",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function ecDumpPretty(cb, cbErr) {
        _call({
            "method": "EcDumpPretty",
            "cb": cb,
            "cbErr": cbErr
        });
    }

    // Methods

    function methodList(cb) {
        _call({
            "method": "MethodList",
            "cb": cb
        });
    }

    function methodRead(method, op, cb, cbErr) {
        let data = {
            "method": method,
            "op": op
        };

        _call({
            "method": "MethodRead",
            "raw": {
                "method": method
            },
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }

    function methodWrite(method, op, mdata, cb, cbErr) {
        let ty;
        if (typeof (mdata) === "boolean") {
            ty = "Bit";
        } else if (typeof (mdata) === "number") {
            ty = "Byte";
        } else if (Array.isArray(mdata)) {
            ty = "Range";
        }

        let data = {
            "method": method,
            "op": op,
            "data": {
                [ty]: mdata
            }
        };

        _call({
            "method": "MethodWrite",
            "raw": {
                "data": mdata,
                "method": method
            },
            "payload": data,
            "cb": cb,
            "cbErr": cbErr
        });
    }
}
