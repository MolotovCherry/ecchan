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

    function _call(method, data, cb, cbErr) {
        _callQueue.push(() => {
            root._cb = cb;
            root._cbErr = cbErr || null;

            let json;
            if (typeof (method) === "string" && data == null) {
                json = JSON.stringify(method);
            } else if (typeof (method) === "string" && data != null) {
                json = JSON.stringify({
                    method: data
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

    function _plainCall(method, cb, cbErr) {
        _call(method, null, cb, cbErr);
    }

    // Utils

    function ping(cb) {
        _plainCall("Ping", cb);
    }

    function fanCount(cb) {
        _plainCall("FanCount", cb);
    }

    function fanMax(cb, cbErr) {
        _plainCall("FanMax", cb, cbErr);
    }

    function hasDGpu(cb) {
        _plainCall("HasDGpu", cb);
    }

    function wmiVer(cb, cbErr) {
        _plainCall("WmiVer", cb, cbErr);
    }

    // Firmware

    function fwVersion(cb, cbErr) {
        _plainCall("FwVersion", cb);
    }

    function fwDate(cb, cbErr) {
        _plainCall("FwDate", cb, cbErr);
    }

    function fwTime(cb, cbErr) {
        _plainCall("FwTime", cb, cbErr);
    }

    // Shift Modes

    function shiftModes(cb, cbErr) {
        _plainCall("ShiftModes", cb, cbErr);
    }

    function shiftMode(mode, cb, cbErr) {
        _plainCall("ShiftMode", cb);
    }

    function setShiftMode(mode, cb, cbErr) {
        _call("SetShiftMode", mode, cb, cbErr);
    }

    function shiftModeSupported(cb) {
        _plainCall("ShiftModeSupported", cb);
    }

    // Battery

    function batteryChargeMode(cb, cbErr) {
        _plainCall("BatteryChargeMode", cb, cbErr);
    }

    function setBatteryChargeMode(mode, cb, cbErr) {
        let data;
        if (typeof (mode) == "string") {
            data = {
                "mode": mode
            };
        } else if (typeof (mode) == "number") {
            data = {
                "mode": {
                    "Custom": mode
                }
            };
        }

        _call("SetBatteryChargeMode", data, cb, cbErr);
    }

    function batteryChargeModeSupported(cb) {
        _plainCall("BatteryChargeModeSupported", cb);
    }

    function superBattery(cb, cbErr) {
        _plainCall("SuperBattery", cb);
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

        _call("SetSuperBattery", data, cb, cbErr);
    }

    function superBatterySupported(cb) {
        _plainCall("SuperBatterySupported", cb);
    }

    // Fan
    function fan1Rpm(cb, cbErr) {
        _plainCall("Fan1Rpm", cb);
    }

    function fan2Rpm(cb, cbErr) {
        _plainCall("Fan2Rpm", cb, cbErr);
    }

    function fan3Rpm(cb, cbErr) {
        _plainCall("Fan3Rpm", cb, cbErr);
    }

    function fan4Rpm(cb, cbErr) {
        _plainCall("Fan4Rpm", cb, cbErr);
    }

    function fan1Supported(cb) {
        _plainCall("Fan1Supported", cb);
    }

    function fan2Supported(cb) {
        _plainCall("Fan2Supported", cb);
    }

    function fan3Supported(cb) {
        _plainCall("Fan3Supported", cb);
    }

    function fan4Supported(cb) {
        _plainCall("Fan4Supported", cb);
    }

    function fanModes(cb, cbErr) {
        _plainCall("FanModes", cb, cbErr);
    }

    function fanMode(cb, cbErr) {
        _plainCall("FanMode", cb, cbErr);
    }

    function setFanMode(mode, cb, cbErr) {
        let data = {
            "mode": mode
        };

        _call("SetFanMode", data, cb, cbErr);
    }

    function fanModeSupported(cb) {
        _plainCall("FanModeSupported", cb);
    }

    // Webcam

    function webcam(mode, cb, cbErr) {
        _plainCall("Webcam", cb, cbErr);
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

        _call("SetWebcam", data, cb, cbErr);
    }

    function webcamBlock(mode, cb, cbErr) {
        _plainCall("WebcamBlock", cb, cbErr);
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

        _call("SetWebcamBlock", data, cb, cbErr);
    }

    function webcamSupported(cb) {
        _plainCall("WebcamSupported", cb);
    }

    function webcamBlockSupported(cb) {
        _plainCall("WebcamBlockSupported", cb);
    }

    // Cooler Boost

    function coolerBoost(cb, cbErr) {
        _plainCall("CoolerBoost", cb, cbErr);
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

        _call("SetCoolerBoost", data, cb, cbErr);
    }

    function coolerBoostSupported(cb) {
        _plainCall("CoolerBoostSupported", cb);
    }

    // Swap Keys

    function fnKey(cb, cbErr) {
        _plainCall("FnKey", cb, cbErr);
    }

    function setFnKey(dir, cb, cbErr) {
        let data = {
            "state": dir
        };

        _call("SetFnKey", data, cb, cbErr);
    }

    function winKey(cb, cbErr) {
        _plainCall("WinKey", cb, cbErr);
    }

    function setWinKey(dir, cb, cbErr) {
        let data = {
            "state": dir
        };

        _call("SetWinKey", data, cb, cbErr);
    }

    function fnWinSwapSupported(cb) {
        _plainCall("FnWinSwapSupported", cb);
    }

    // Mute LEDs

    function micMuteLed(cb, cbErr) {
        _plainCall("MicMuteLed", cb, cbErr);
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

        _call("SetMicMuteLed", data, cb, cbErr);
    }

    function muteLed(cb, cbErr) {
        _plainCall("MuteLed", cb, cbErr);
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

        _call("SetMuteLed", data, cb, cbErr);
    }

    function micMuteLedSupported(cb) {
        _plainCall("MicMuteLedSupported", cb);
    }

    function muteLedSupported(cb) {
        _plainCall("MuteLedSupported", cb);
    }

    // Realtime Stats

    function cpuRtFanSpeed(cb, cbErr) {
        _plainCall("CpuRtFanSpeed", cb, cbErr);
    }

    function cpuRtTemp(cb, cbErr) {
        _plainCall("CpuRtTemp", cb, cbErr);
    }

    function gpuRtFanSpeed(cb, cbErr) {
        _plainCall("GpuRtFanSpeed", cb, cbErr);
    }

    function gpuRtTemp(cb, cbErr) {
        _plainCall("GpuRtTemp", cb, cbErr);
    }

    // Curves

    function cpuFanCurveWmi2(cb, cbErr) {
        _plainCall("CpuFanCurveWmi2", cb, cbErr);
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

        _call("SetCpuFanCurveWmi2", data, cb, cbErr);
    }

    function cpuTempCurveWmi2(cb, cbErr) {
        _plainCall("CpuTempCurveWmi2", cb, cbErr);
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

        _call("SetCpuTempCurveWmi2", data, cb, cbErr);
    }

    function cpuHysteresisCurveWmi2(cb, cbErr) {
        _plainCall("CpuHysteresisCurveWmi2", cb, cbErr);
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

        _call("SetCpuHysteresisCurveWmi2", data, cb, cbErr);
    }

    function gpuFanCurveWmi2(curve, cb, cbErr) {
        _plainCall("GpuFanCurveWmi2", cb, cbErr);
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

        _call("SetGpuFanCurveWmi2", data, cb, cbErr);
    }

    function gpuTempCurveWmi2(cb, cbErr) {
        _plainCall("GpuTempCurveWmi2", cb, cbErr);
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

        _call("SetGpuTempCurveWmi2", data, cb, cbErr);
    }

    function gpuHysteresisCurveWmi2(cb, cbErr) {
        _plainCall("GpuHysteresisCurveWmi2", cb, cbErr);
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

        _call("SetGpuHysteresisCurveWmi2", data, cb, cbErr);
    }

    // Ec

    function ecDump(cb, cbErr) {
        _plainCall("EcDump", cb, cbErr);
    }

    function ecDumpPretty(cb, cbErr) {
        _plainCall("EcDumpPretty", cb, cbErr);
    }

    // Methods

    function methodList(cb) {
        _plainCall("MethodList", cb);
    }

    function methodRead(method, op, cb, cbErr) {
        let data = {
            "method": method,
            "op": op
        };

        _call("MethodRead", data, cb, cbErr);
    }

    function methodWrite(method, op, mdata, cb, cbErr) {
        let ty;
        if (typeof (mdata) == "boolean") {
            ty = "Bit";
        } else if (typeof (mdata) == "number") {
            ty = "Byte";
        } else if (Array.isArray(mdata)) {
            ty = "Range";
        }

        let data = {
            "method": method,
            "op": op,
            "data": {
                ty: mdata
            }
        };

        _call("MethodWrite", data, cb, cbErr);
    }
}
