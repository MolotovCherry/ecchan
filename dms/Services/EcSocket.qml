pragma Singleton
pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Io

import qs.Common
import qs.Services

import "../Common"

Singleton {
    id: root

    property bool connected: false

    signal initStarted
    signal initFinished
    signal applyStarted
    signal applyFinished
    signal dataReady(int id, var payload, bool isErr)

    property int _counter: 0
    property var _currentCounterId: 0
    property var _callBlocked: false
    property var _callQueue: []
    property SocketCbManager _cbManager: SocketCbManager {}

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
                const id = root._currentCounterId;

                try {
                    // { "Ok": { .. } } / { "Err": "" }
                    const reply = JSON.parse(line);

                    if (reply.hasOwnProperty("Err")) {
                        console.error("Call returned error:", reply.Err);
                        ToastService.showError("Ecchan ipc call failed", reply.Err);

                        root.dataReady(id, reply.Err, true);

                        return;
                    } else if (!reply.hasOwnProperty("Ok")) {
                        console.error("Failed to parse reply:", line);
                        ToastService.showError("Ecchan failed to parse server reply", line);

                        root.dataReady(id, line, true);

                        return;
                    }

                    const data = root._handleReply(reply.Ok);

                    root.dataReady(id, data, false);
                } catch (e) {
                    console.error("Failed to parse reply:", line, e);
                    ToastService.showError("Ecchan failed to parse server reply", `${e}\n\n${line}`);

                    root.dataReady(id, reply.Err, true);
                }

                const cb = root._callQueue.shift();
                if (cb != null) {
                    root._currentCounterId = cb();
                } else {
                    root._callBlocked = false;
                }
            }
        }
    }

    function _reset() {
        watchdogTimer.stop();
        pingTimer.stop();

        _socket?.destroy();
        _socket = null;

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
        interval: 2500
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

            const pId = root.ping();
            _cbManager.registerCb(pId, () => watchdogTimer.restart());
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
        applyStarted();

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
                        const method = newState.methods[m.method];
                        if (method != null) {
                            methodWrite(m.method, op, newState.methods[m.method]);
                        }
                    }
                }
            }
        }

        const pId = ping();
        _cbManager.registerCb(pId, () => {
            applyFinished();
        });
    }

    function _initState() {
        initStarted();

        fanCount();
        fanMax();
        hasDGpu();
        const wmiId = wmiVer();
        _cbManager.registerCb(wmiId, ver => {
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

        const mId = methodList();
        _cbManager.registerCb(mId, list => {
            for (const m of list) {
                for (const op of m.ops) {
                    if (op.startsWith("Read")) {
                        methodRead(m.method, op);
                    }
                }
            }

            // dummy ping to schedule event after all the others
            const pId = ping();
            _cbManager.registerCb(pId, () => {
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
        const id = _counter += 1;

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
            _cbManager.registerCb(id, data => {
                if (stateKey === "methods") {
                    root.state[stateKey][callData.raw.method] = callData.raw.data;
                } else {
                    root.state[stateKey] = callData.raw;
                }

                root.stateChanged();
            });
        } else {
            _cbManager.registerCb(id, data => {
                if (stateKey === "methods") {
                    root.state[stateKey][callData.raw.method] = data;
                } else {
                    root.state[stateKey] = data;
                }

                root.stateChanged();
            });
        }

        const call = () => {
            let json;
            if (typeof (callData.method) === "string" && callData.payload == null) {
                json = JSON.stringify(callData.method);
            } else if (typeof (callData.method) === "string" && callData.payload != null) {
                json = JSON.stringify({
                    [callData.method]: callData.payload
                });
            }

            // calls will be lost if not connected; this is acceptable
            _socket?.send(json);
            return id;
        };

        _callQueue.push(call);

        if (!_callBlocked) {
            _callBlocked = true;
            _currentCounterId = _callQueue.shift()();
        }

        return id;
    }

    // Utils

    function ping() {
        return _call({
            "method": "Ping"
        });
    }

    function fanCount() {
        return _call({
            "method": "FanCount"
        });
    }

    function fanMax() {
        return _call({
            "method": "FanMax"
        });
    }

    function hasDGpu() {
        return _call({
            "method": "HasDGpu"
        });
    }

    function wmiVer() {
        return _call({
            "method": "WmiVer"
        });
    }

    // Firmware

    function fwVersion() {
        return _call({
            "method": "FwVersion"
        });
    }

    function fwDate() {
        return _call({
            "method": "FwDate"
        });
    }

    function fwTime() {
        return _call({
            "method": "FwTime"
        });
    }

    // Shift Modes

    function shiftModes() {
        return _call({
            "method": "ShiftModes"
        });
    }

    function shiftMode(mode) {
        return _call({
            "method": "ShiftMode"
        });
    }

    function setShiftMode(mode) {
        return _call({
            "method": "SetShiftMode",
            "raw": mode,
            "payload": {
                "mode": mode
            }
        });
    }

    function shiftModeSupported() {
        return _call({
            "method": "ShiftModeSupported"
        });
    }

    // Battery

    function batteryChargeMode() {
        return _call({
            "method": "BatteryChargeMode"
        });
    }

    function setBatteryChargeMode(mode) {
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

        return _call({
            "method": "SetBatteryChargeMode",
            "raw": mode,
            "payload": data
        });
    }

    function batteryChargeModeSupported() {
        return _call({
            "method": "BatteryChargeModeSupported"
        });
    }

    function superBattery() {
        return _call({
            "method": "SuperBattery"
        });
    }

    function setSuperBattery(mode) {
        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        let data = {
            "state": state
        };

        return _call({
            "method": "SetSuperBattery",
            "raw": mode,
            "payload": data
        });
    }

    function superBatterySupported() {
        return _call({
            "method": "SuperBatterySupported"
        });
    }

    // Fan
    function fan1Rpm() {
        return _call({
            "method": "Fan1Rpm"
        });
    }

    function fan2Rpm() {
        return _call({
            "method": "Fan2Rpm"
        });
    }

    function fan3Rpm() {
        return _call({
            "method": "Fan3Rpm"
        });
    }

    function fan4Rpm() {
        return _call({
            "method": "Fan4Rpm"
        });
    }

    function fan1Supported() {
        return _call({
            "method": "Fan1Supported"
        });
    }

    function fan2Supported() {
        return _call({
            "method": "Fan2Supported"
        });
    }

    function fan3Supported() {
        return _call({
            "method": "Fan3Supported"
        });
    }

    function fan4Supported() {
        return _call({
            "method": "Fan4Supported"
        });
    }

    function fanModes() {
        return _call({
            "method": "FanModes"
        });
    }

    function fanMode() {
        return _call({
            "method": "FanMode"
        });
    }

    function setFanMode(mode) {
        let data = {
            "mode": mode
        };

        return _call({
            "method": "SetFanMode",
            "raw": mode,
            "payload": data
        });
    }

    function fanModeSupported() {
        return _call({
            "method": "FanModeSupported"
        });
    }

    // Webcam

    function webcam(mode) {
        return _call({
            "method": "Webcam"
        });
    }

    function setWebcam(mode) {
        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        let data = {
            "state": state
        };

        return _call({
            "method": "SetWebcam",
            "raw": mode,
            "payload": data
        });
    }

    function webcamBlock(mode) {
        return _call({
            "method": "WebcamBlock"
        });
    }

    function setWebcamBlock(mode) {
        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        let data = {
            "state": state
        };

        return _call({
            "method": "SetWebcamBlock",
            "raw": mode,
            "payload": data
        });
    }

    function webcamSupported() {
        return _call({
            "method": "WebcamSupported"
        });
    }

    function webcamBlockSupported() {
        return _call({
            "method": "WebcamBlockSupported"
        });
    }

    // Cooler Boost

    function coolerBoost() {
        return _call({
            "method": "CoolerBoost"
        });
    }

    function setCoolerBoost(mode) {
        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        let data = {
            "state": state
        };

        return _call({
            "method": "SetCoolerBoost",
            "raw": mode,
            "payload": data
        });
    }

    function coolerBoostSupported() {
        return _call({
            "method": "CoolerBoostSupported"
        });
    }

    // Swap Keys

    function fnKey() {
        return _call({
            "method": "FnKey"
        });
    }

    function setFnKey(dir) {
        let data = {
            "state": dir
        };

        return _call({
            "method": "SetFnKey",
            "raw": dir,
            "payload": data
        });
    }

    function winKey() {
        return _call({
            "method": "WinKey"
        });
    }

    function setWinKey(dir) {
        let data = {
            "state": dir
        };

        return _call({
            "method": "SetWinKey",
            "raw": dir,
            "payload": data
        });
    }

    function fnWinSwapSupported() {
        return _call({
            "method": "FnWinSwapSupported"
        });
    }

    // Mute LEDs

    function micMuteLed() {
        return _call({
            "method": "MicMuteLed"
        });
    }

    function setMicMuteLed(state) {
        let val;
        if (state) {
            val = "On";
        } else {
            val = "Off";
        }

        let data = {
            "state": val
        };

        return _call({
            "method": "SetMicMuteLed",
            "raw": state,
            "payload": data
        });
    }

    function muteLed() {
        return _call({
            "method": "MuteLed"
        });
    }

    function setMuteLed(state) {
        let val;
        if (state) {
            val = "On";
        } else {
            val = "Off";
        }

        let data = {
            "state": val
        };

        return _call({
            "method": "SetMuteLed",
            "raw": state,
            "payload": data
        });
    }

    function micMuteLedSupported() {
        return _call({
            "method": "MicMuteLedSupported"
        });
    }

    function muteLedSupported() {
        return _call({
            "method": "MuteLedSupported"
        });
    }

    // Realtime Stats

    function cpuRtFanSpeed() {
        return _call({
            "method": "CpuRtFanSpeed"
        });
    }

    function cpuRtTemp() {
        return _call({
            "method": "CpuRtTemp"
        });
    }

    function gpuRtFanSpeed() {
        return _call({
            "method": "GpuRtFanSpeed"
        });
    }

    function gpuRtTemp() {
        return _call({
            "method": "GpuRtTemp"
        });
    }

    // Curves

    function cpuFanCurveWmi2() {
        return _call({
            "method": "CpuFanCurveWmi2"
        });
    }

    function setCpuFanCurveWmi2(curve) {
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

        return _call({
            "method": "SetCpuFanCurveWmi2",
            "raw": curve,
            "payload": data
        });
    }

    function cpuTempCurveWmi2() {
        return _call({
            "method": "CpuTempCurveWmi2"
        });
    }

    function setCpuTempCurveWmi2(curve) {
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

        return _call({
            "method": "SetCpuTempCurveWmi2",
            "raw": curve,
            "payload": data
        });
    }

    function cpuHysteresisCurveWmi2() {
        return _call({
            "method": "CpuHysteresisCurveWmi2"
        });
    }

    function setCpuHysteresisCurveWmi2(curve) {
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

        return _call({
            "method": "SetCpuHysteresisCurveWmi2",
            "raw": curve,
            "payload": data
        });
    }

    function gpuFanCurveWmi2(curve) {
        return _call({
            "method": "GpuFanCurveWmi2"
        });
    }

    function setGpuFanCurveWmi2(curve) {
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

        return _call({
            "method": "SetGpuFanCurveWmi2",
            "raw": curve,
            "payload": data
        });
    }

    function gpuTempCurveWmi2() {
        return _call({
            "method": "GpuTempCurveWmi2"
        });
    }

    function setGpuTempCurveWmi2(curve) {
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

        return _call({
            "method": "SetGpuTempCurveWmi2",
            "raw": curve,
            "payload": data
        });
    }

    function gpuHysteresisCurveWmi2() {
        return _call({
            "method": "GpuHysteresisCurveWmi2"
        });
    }

    function setGpuHysteresisCurveWmi2(curve) {
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

        return _call({
            "method": "SetGpuHysteresisCurveWmi2",
            "raw": curve,
            "payload": data
        });
    }

    // Ec

    function ecDump() {
        return _call({
            "method": "EcDump"
        });
    }

    function ecDumpPretty() {
        return _call({
            "method": "EcDumpPretty"
        });
    }

    // Methods

    function methodList() {
        return _call({
            "method": "MethodList"
        });
    }

    function methodRead(method, op) {
        let data = {
            "method": method,
            "op": op
        };

        return _call({
            "method": "MethodRead",
            "raw": {
                "method": method
            },
            "payload": data
        });
    }

    function methodWrite(method, op, mdata) {
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

        return _call({
            "method": "MethodWrite",
            "raw": {
                "data": mdata,
                "method": method
            },
            "payload": data
        });
    }
}
