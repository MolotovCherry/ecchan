pragma Singleton

import QtQuick

import Quickshell
import Quickshell.Io
import qs.Common

Singleton {
    id: root

    property alias connected: socket.connected
    property string socketPath: Quickshell.env("ECCHAN_SOCK") || "/run/ecchan.sock"

    property var _cb

    property string _method
    property var _method_data

    DankSocket {
        id: socket
        path: root.socketPath
        connected: true

        parser: SplitParser {
            onRead: line => {
                try {
                    // { "Ok": { .. } } / { "Err": { .. } }
                    const reply = JSON.parse(line);

                    if (reply.hasOwnProperty("Err")) {
                        console.error("EcSocket: Call returned error:", reply.Err);
                        return;
                    } else if (!reply.hasOwnProperty("Ok")) {
                        console.error("EcSocket: Failed to parse reply:", line);
                        return;
                    }

                    let data = root.handleReply(reply.Ok);

                    root._cb(data);
                } catch (e) {
                    console.error("EcSocket: Failed to parse reply:", line, e);
                }
            }
        }
    }

    function handleReply(reply) {
        console.error("handling", JSON.stringify(reply));
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
                    let ckey = Object.keys(value)[0];
                    let cvalue = Object.values(value)[0];

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
                let mkey = Object.keys(value)[0];
                let mvalue = Object.values(value)[0];

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

    function _call(cb) {
        root._cb = cb;

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

        console.info("Calling", json);

        socket.send(json);
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

    function fanMax(cb) {
        root._method = "FanMax";

        root._call(cb);
    }

    function hasDGpu(cb) {
        root._method = "HasDGpu";

        root._call(cb);
    }

    function wmiVer(cb) {
        root._method = "WmiVer";

        root._call(cb);
    }

    // Firmware

    function fwVersion(cb) {
        root._method = "FwVersion";

        root._call(cb);
    }

    function fwDate(cb) {
        root._method = "FwDate";

        root._call(cb);
    }

    function fwTime(cb) {
        root._method = "FwTime";

        root._call(cb);
    }

    // Shift Modes

    function shiftModes(cb) {
        root._method = "ShiftModes";

        root._call(cb);
    }

    function shiftMode(mode, cb) {
        root._method = "ShiftMode";

        root._call(cb);
    }

    function setShiftMode(mode, cb) {
        root._method = "SetShiftMode";
        root._method_data = mode;

        root._call(cb);
    }

    function shiftModeSupported(cb) {
        root._method = "ShiftModeSupported";

        root._call(cb);
    }

    // Battery

    function batteryChargeMode(cb) {
        root._method = "BatteryChargeMode";
        root._call(cb);
    }

    function setBatteryChargeMode(mode, cb) {
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

        root._call(cb);
    }

    function batteryChargeModeSupported(cb) {
        root._method = "BatteryChargeModeSupported";
    }

    function superBattery(cb) {
        root._method = "SuperBattery";

        root._call(cb);
    }

    function setSuperBattery(mode, cb) {
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

        root._call(cb);
    }

    function superBatterySupported(cb) {
        root._method = "SuperBatterySupported";

        root._call(cb);
    }

    // Fan
    function fan1Rpm(cb) {
        root._method = "Fan1Rpm";

        root._call(cb);
    }

    function fan2Rpm(cb) {
        root._method = "Fan2Rpm";

        root._call(cb);
    }

    function fan3Rpm(cb) {
        root._method = "Fan3Rpm";

        root._call(cb);
    }

    function fan4Rpm(cb) {
        root._method = "Fan4Rpm";

        root._call(cb);
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

    function fanModes(cb) {
        root._method = "FanModes";

        root._call(cb);
    }

    function fanMode(cb) {
        root._method = "FanMode";

        root._call(cb);
    }

    function setFanMode(mode, cb) {
        root._method = "SetFanMode";
        root._methodData = {
            "mode": mode
        };

        root._call(cb);
    }

    function fanModeSupported(cb) {
        root._method = "FanModeSupported";

        root._call(cb);
    }

    // Webcam

    function webcam(mode, cb) {
        root._method = "Webcam";

        root._call(cb);
    }

    function setWebcam(mode, cb) {
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

        root._call(cb);
    }

    function webcamBlock(mode, cb) {
        root._method = "WebcamBlock";

        root._call(cb);
    }

    function setWebcamBlock(mode, cb) {
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

        root._call(cb);
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

    function coolerBoost(cb) {
        root._method = "CoolerBoost";

        root._call(cb);
    }

    function setCoolerBoost(mode, cb) {
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

        root._call(cb);
    }

    function coolerBoostSupported(cb) {
        root._method = "CoolerBoostSupported";

        root._call(cb);
    }

    // Swap Keys

    function fnKey(cb) {
        root._method = "FnKey";

        root._call(cb);
    }

    function setFnKey(dir, cb) {
        root._method = "SetFnKey";

        root._methodData = {
            "state": dir
        };

        root._call(cb);
    }

    function winKey(cb) {
        root._method = "WinKey";

        root._call(cb);
    }

    function setWinKey(dir, cb) {
        root._method = "SetWinKey";

        root._methodData = {
            "state": dir
        };

        root._call(cb);
    }

    function fnWinSwapSupported(cb) {
        root._method = "FnWinSwapSupported";

        root._call(cb);
    }

    // Mute LEDs

    function micMuteLed(cb) {
        root._method = "MicMuteLed";

        root._call(cb);
    }

    function setMicMuteLed(state, cb) {
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

        root._call(cb);
    }

    function muteLed(cb) {
        root._method = "MuteLed";

        root._call(cb);
    }

    function setMuteLed(state, cb) {
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

        root._call(cb);
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

    function cpuRtFanSpeed(cb) {
        root._method = "CpuRtFanSpeed";

        root._call(cb);
    }

    function cpuRtTemp(cb) {
        root._method = "CpuRtTemp";

        root._call(cb);
    }

    function gpuRtFanSpeed(cb) {
        root._method = "GpuRtFanSpeed";

        root._call(cb);
    }

    function gpuRtTemp(cb) {
        root._method = "GpuRtTemp";

        root._call(cb);
    }

    // Curves

    function cpuFanCurveWmi2(cb) {
        root._method = "CpuFanCurveWmi2";

        root._call(cb);
    }

    function setCpuFanCurveWmi2(curve, cb) {
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

        root._call(cb);
    }

    function cpuTempCurveWmi2(cb) {
        root._method = "CpuTempCurveWmi2";

        root._call(cb);
    }

    function setCpuTempCurveWmi2(curve, cb) {
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

        root._call(cb);
    }

    function cpuHysteresisCurveWmi2(cb) {
        root._method = "CpuHysteresisCurveWmi2";

        root._call(cb);
    }

    function setCpuHysteresisCurveWmi2(curve, cb) {
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

        root._call(cb);
    }

    function gpuFanCurveWmi2(curve, cb) {
        root._method = "GpuFanCurveWmi2";

        root._call(cb);
    }

    function setGpuFanCurveWmi2(curve, cb) {
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

        root._call(cb);
    }

    function gpuTempCurveWmi2(cb) {
        root._method = "GpuTempCurveWmi2";

        root._call(cb);
    }

    function setGpuTempCurveWmi2(curve, cb) {
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

        root._call(cb);
    }

    function gpuHysteresisCurveWmi2(cb) {
        root._method = "GpuHysteresisCurveWmi2";

        root._call(cb);
    }

    function setGpuHysteresisCurveWmi2(curve, cb) {
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

        root._call(cb);
    }

    // Ec

    function ecDump(cb) {
        root._method = "EcDump";

        root._call(cb);
    }

    function ecDumpPretty(cb) {
        root._method = "EcDumpPretty";

        root._call(cb);
    }

    // Methods

    function methodList(cb) {
        root._method = "MethodList";

        root._call(cb);
    }

    function methodRead(method, op, cb) {
        root.method = "MethodRead";
        root._methodData = {
            "method": method,
            "op": op
        };

        root._call(cb);
    }

    function methodWrite(method, op, data, cb) {
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

        root._call(cb);
    }
}
