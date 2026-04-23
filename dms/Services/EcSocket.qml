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

    property string _socketFile
    property var _cb: null
    property var _cbErr: null
    property var _callQueue: []

    property string _method
    property var _method_data

    property DankSocket _socket

    property Component _socketComponent: DankSocket {
        id: socket
        connected: true

        onConnectionStateChanged: {
            if (connected) {
                root.connected = true;
                pingTimer.start();
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
                        root?._cb(data);
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
        if (_cb === null && _callQueue.length > 0) {
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

    // args:
    // data: { socketFile: "/path/to/file.sock" }
    function init(socketFile) {
        _reset();

        _socketFile = socketFile;
        _socket = _socketComponent.createObject(root, {
            path: socketFile
        });
    }

    function reconnect() {
        if (_socketFile !== null) {
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

            root.ping(() => watchdogTimer.restart());
        }
    }

    function _handleReply(reply) {
        let key;
        let value;

        if (typeof (reply) == "string") {
            key = reply;
        } else {
            key = Object.keys(reply)[0];
            value = Object.values(reply)[0];
        }

        switch (key) {
            // qmlformat off

            case "Pong":
            case "Unit":
                return null;

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
                if (typeof (value) == "string") {
                    return value;
                } else {
                    const ckey = Object.keys(value)[0];
                    const cvalue = Object.values(value)[0];

                    switch (ckey) {
                        case "Custom":
                            return cvalue;
                        default:
                            console.error("Invalid Custom key", ckey, cvalue);
                    }
                }

                break;

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
                }

                break;

            // qmlformat on
        }
    }

    function _call(cb, cbErr) {
        _callQueue.push(() => {
            root._cb = cb;
            root._cbErr = cbErr || null;

            let json;
            if (typeof (root._method) != "undefined" && typeof (root._method_data) == "undefined") {
                json = JSON.stringify(root._method);
            } else if (typeof (root._method) != "undefined" && typeof (root._method_data) != "undefined") {
                json = JSON.stringify({
                    _method: _method_data
                });
            } else {
                console.error("why is _method undefined?");
                return;
            }

            // calls will be lost if not connected; this is acceptable
            _socket?.send(json);
        });

        _callQueueNext();
    }

    // Utils

    function ping(cb) {
        root._method = "Ping";
        root._call(cb);
    }

    function fanCount(cb) {
        root._method = "FanCount";
        root._call(cb);
    }

    function fanMax(cb, cbErr) {
        root._method = "FanMax";
        root._call(cb, cbErr);
    }

    function hasDGpu(cb) {
        root._method = "HasDGpu";
        root._call(cb);
    }

    function wmiVer(cb, cbErr) {
        root._method = "WmiVer";
        root._call(cb, cbErr);
    }

    // Firmware

    function fwVersion(cb, cbErr) {
        root._method = "FwVersion";
        root._call(cb);
    }

    function fwDate(cb, cbErr) {
        root._method = "FwDate";
        root._call(cb, cbErr);
    }

    function fwTime(cb, cbErr) {
        root._method = "FwTime";
        root._call(cb, cbErr);
    }

    // Shift Modes

    function shiftModes(cb, cbErr) {
        root._method = "ShiftModes";
        root._call(cb, cbErr);
    }

    function shiftMode(mode, cb, cbErr) {
        root._method = "ShiftMode";
        root._call(cb);
    }

    function setShiftMode(mode, cb, cbErr) {
        root._method = "SetShiftMode";
        root._method_data = mode;

        root._call(cb, cbErr);
    }

    function shiftModeSupported(cb) {
        root._method = "ShiftModeSupported";
        root._call(cb);
    }

    // Battery

    function batteryChargeMode(cb, cbErr) {
        root._method = "BatteryChargeMode";
        root._call(cb, cbErr);
    }

    function setBatteryChargeMode(mode, cb, cbErr) {
        root._method = "SetBatteryChargeMode";

        if (typeof (mode) == "string") {
            root._methodData = {
                "mode": mode
            };
        } else if (typeof (mode) == "number") {
            root._methodData = {
                "mode": {
                    "Custom": mode
                }
            };
        }

        root._call(cb, cbErr);
    }

    function batteryChargeModeSupported(cb) {
        root._method = "BatteryChargeModeSupported";
        root._call(cb);
    }

    function superBattery(cb, cbErr) {
        root._method = "SuperBattery";
        root._call(cb);
    }

    function setSuperBattery(mode, cb, cbErr) {
        root._method = "SetSuperBattery";

        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        root._methodData = {
            "state": state
        };

        root._call(cb, cbErr);
    }

    function superBatterySupported(cb) {
        root._method = "SuperBatterySupported";
        root._call(cb);
    }

    // Fan
    function fan1Rpm(cb, cbErr) {
        root._method = "Fan1Rpm";
        root._call(cb);
    }

    function fan2Rpm(cb, cbErr) {
        root._method = "Fan2Rpm";
        root._call(cb, cbErr);
    }

    function fan3Rpm(cb, cbErr) {
        root._method = "Fan3Rpm";
        root._call(cb, cbErr);
    }

    function fan4Rpm(cb, cbErr) {
        root._method = "Fan4Rpm";
        root._call(cb, cbErr);
    }

    function fan1Supported(cb) {
        root._method = "Fan1Supported";

        root._call(cb);
    }

    function fan2Supported(cb) {
        root._method = "Fan2Supported";

        root._call(cb);
    }

    function fan3Supported(cb) {
        root._method = "Fan3Supported";
        root._call(cb);
    }

    function fan4Supported(cb) {
        root._method = "Fan4Supported";
        root._call(cb);
    }

    function fanModes(cb, cbErr) {
        root._method = "FanModes";
        root._call(cb, cbErr);
    }

    function fanMode(cb, cbErr) {
        root._method = "FanMode";
        root._call(cb, cbErr);
    }

    function setFanMode(mode, cb, cbErr) {
        root._method = "SetFanMode";
        root._methodData = {
            "mode": mode
        };

        root._call(cb, cbErr);
    }

    function fanModeSupported(cb) {
        root._method = "FanModeSupported";
        root._call(cb);
    }

    // Webcam

    function webcam(mode, cb, cbErr) {
        root._method = "Webcam";
        root._call(cb, cbErr);
    }

    function setWebcam(mode, cb, cbErr) {
        root._method = "SetWebcam";

        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        root._methodData = {
            "state": state
        };

        root._call(cb, cbErr);
    }

    function webcamBlock(mode, cb, cbErr) {
        root._method = "WebcamBlock";
        root._call(cb, cbErr);
    }

    function setWebcamBlock(mode, cb, cbErr) {
        root._method = "SetWebcamBlock";

        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        root._methodData = {
            "state": state
        };

        root._call(cb, cbErr);
    }

    function webcamSupported(cb) {
        root._method = "WebcamSupported";
        root._call(cb);
    }

    function webcamBlockSupported(cb) {
        root._method = "WebcamBlockSupported";
        root._call(cb);
    }

    // Cooler Boost

    function coolerBoost(cb, cbErr) {
        root._method = "CoolerBoost";
        root._call(cb, cbErr);
    }

    function setCoolerBoost(mode, cb, cbErr) {
        root._method = "SetCoolerBoost";

        let state;
        if (mode) {
            state = "On";
        } else {
            state = "Off";
        }

        root._methodData = {
            "state": state
        };

        root._call(cb, cbErr);
    }

    function coolerBoostSupported(cb) {
        root._method = "CoolerBoostSupported";
        root._call(cb);
    }

    // Swap Keys

    function fnKey(cb, cbErr) {
        root._method = "FnKey";
        root._call(cb, cbErr);
    }

    function setFnKey(dir, cb, cbErr) {
        root._method = "SetFnKey";

        root._methodData = {
            "state": dir
        };

        root._call(cb, cbErr);
    }

    function winKey(cb, cbErr) {
        root._method = "WinKey";
        root._call(cb, cbErr);
    }

    function setWinKey(dir, cb, cbErr) {
        root._method = "SetWinKey";

        root._methodData = {
            "state": dir
        };

        root._call(cb, cbErr);
    }

    function fnWinSwapSupported(cb) {
        root._method = "FnWinSwapSupported";

        root._call(cb);
    }

    // Mute LEDs

    function micMuteLed(cb, cbErr) {
        root._method = "MicMuteLed";
        root._call(cb, cbErr);
    }

    function setMicMuteLed(state, cb, cbErr) {
        root._method = "SetMicMuteLed";

        let val;
        if (state) {
            val = "On";
        } else {
            val = "Off";
        }

        root._methodData = {
            "state": val
        };

        root._call(cb, cbErr);
    }

    function muteLed(cb, cbErr) {
        root._method = "MuteLed";
        root._call(cb, cbErr);
    }

    function setMuteLed(state, cb, cbErr) {
        root._method = "SetMuteLed";

        let val;
        if (state) {
            val = "On";
        } else {
            val = "Off";
        }

        root._methodData = {
            "state": val
        };

        root._call(cb, cbErr);
    }

    function micMuteLedSupported(cb) {
        root._method = "MicMuteLedSupported";
        root._call(cb);
    }

    function muteLedSupported(cb) {
        root._method = "MuteLedSupported";
        root._call(cb);
    }

    // Realtime Stats

    function cpuRtFanSpeed(cb, cbErr) {
        root._method = "CpuRtFanSpeed";
        root._call(cb, cbErr);
    }

    function cpuRtTemp(cb, cbErr) {
        root._method = "CpuRtTemp";
        root._call(cb, cbErr);
    }

    function gpuRtFanSpeed(cb, cbErr) {
        root._method = "GpuRtFanSpeed";
        root._call(cb, cbErr);
    }

    function gpuRtTemp(cb, cbErr) {
        root._method = "GpuRtTemp";
        root._call(cb, cbErr);
    }

    // Curves

    function cpuFanCurveWmi2(cb, cbErr) {
        root._method = "CpuFanCurveWmi2";
        root._call(cb, cbErr);
    }

    function setCpuFanCurveWmi2(curve, cb, cbErr) {
        root._method = "SetCpuFanCurveWmi2";
        root._methodData = {
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

        root._call(cb, cbErr);
    }

    function cpuTempCurveWmi2(cb, cbErr) {
        root._method = "CpuTempCurveWmi2";

        root._call(cb, cbErr);
    }

    function setCpuTempCurveWmi2(curve, cb, cbErr) {
        root._method = "SetCpuTempCurveWmi2";
        root._methodData = {
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

        root._call(cb, cbErr);
    }

    function cpuHysteresisCurveWmi2(cb, cbErr) {
        root._method = "CpuHysteresisCurveWmi2";
        root._call(cb, cbErr);
    }

    function setCpuHysteresisCurveWmi2(curve, cb, cbErr) {
        root._method = "SetCpuHysteresisCurveWmi2";
        root._methodData = {
            "curve": {
                "n1": curve[0],
                "n2": curve[1],
                "n3": curve[2],
                "n4": curve[3],
                "n5": curve[4],
                "n6": curve[5]
            }
        };

        root._call(cb, cbErr);
    }

    function gpuFanCurveWmi2(curve, cb, cbErr) {
        root._method = "GpuFanCurveWmi2";
        root._call(cb, cbErr);
    }

    function setGpuFanCurveWmi2(curve, cb, cbErr) {
        root._method = "SetGpuFanCurveWmi2";
        root._methodData = {
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

        root._call(cb, cbErr);
    }

    function gpuTempCurveWmi2(cb, cbErr) {
        root._method = "GpuTempCurveWmi2";
        root._call(cb, cbErr);
    }

    function setGpuTempCurveWmi2(curve, cb, cbErr) {
        root._method = "SetGpuTempCurveWmi2";
        root._methodData = {
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

        root._call(cb, cbErr);
    }

    function gpuHysteresisCurveWmi2(cb, cbErr) {
        root._method = "GpuHysteresisCurveWmi2";
        root._call(cb, cbErr);
    }

    function setGpuHysteresisCurveWmi2(curve, cb, cbErr) {
        root._method = "SetGpuHysteresisCurveWmi2";
        root._methodData = {
            "curve": {
                "n1": curve[0],
                "n2": curve[1],
                "n3": curve[2],
                "n4": curve[3],
                "n5": curve[4],
                "n6": curve[5]
            }
        };

        root._call(cb, cbErr);
    }

    // Ec

    function ecDump(cb, cbErr) {
        root._method = "EcDump";
        root._call(cb, cbErr);
    }

    function ecDumpPretty(cb, cbErr) {
        root._method = "EcDumpPretty";
        root._call(cb, cbErr);
    }

    // Methods

    function methodList(cb) {
        root._method = "MethodList";
        root._call(cb);
    }

    function methodRead(method, op, cb, cbErr) {
        root.method = "MethodRead";
        root._methodData = {
            "method": method,
            "op": op
        };

        root._call(cb, cbErr);
    }

    function methodWrite(method, op, data, cb, cbErr) {
        root.method = "MethodWrite";

        let ty;
        if (typeof (data) == "boolean") {
            ty = "Bit";
        } else if (typeof (data) == "number") {
            ty = "Byte";
        } else if (Array.isArray(data)) {
            ty = "Range";
        }

        root._methodData = {
            "method": method,
            "op": op,
            "data": {
                ty: data
            }
        };

        root._call(cb, cbErr);
    }
}
