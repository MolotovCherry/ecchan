pragma Singleton

import QtQml

import "../ecchan_client" as Plugin
import "../Services"

QtObject {
    id: root

    property var refCounts: ({})

    // Arg: [string]
    function addRef(name) {
        for (const prop of name) {
            if (typeof (prop) === "string") {
                const method = methods[prop];
                if (method == null) {
                    console.warn(`addRef() tried add ref for ${prop}, but it is not a method`);
                    return;
                }

                // incref
                refCounts[prop] = Math.max(0, (refCounts[prop] || 0) + 1);
            }
        }

        if (!updateMethods.running) {
            updateMethods.start();
        }
    }

    // Arg: [string]
    function removeRef(name) {
        for (const prop of name) {
            if (typeof (prop) === "string") {
                const method = methods[prop];
                if (method == null) {
                    console.warn(`removeRef() tried remove ref for ${prop}, but it is not a method`);
                    return;
                }

                // decref
                refCounts[prop] = Math.max(0, (refCounts[prop] || 0) - 1);
            }
        }
    }

    property var timer: Timer {
        id: updateMethods
        interval: 1000
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            if (!EcchanClient.connected) {
                return;
            }

            for (const [name, count] of Object.entries(root.refCounts)) {
                const method = root.methods[name];
                if (method == null || count < 1) {
                    continue;
                }

                EcchanClient.update(method);
            }
        }
    }

    property var methods: {
        "fanCount": Plugin.Method.FanCount,
        "fanMax": Plugin.Method.FanMax,
        "hasDGpu": Plugin.Method.HasDGpu,
        "wmiVer": Plugin.Method.WmiVer,
        "fwVersion": Plugin.Method.FwVersion,
        "fwDate": Plugin.Method.FwDate,
        "fwTime": Plugin.Method.FwTime,
        "shiftModes": Plugin.Method.ShiftModes,
        "shiftMode": Plugin.Method.ShiftMode,
        "shiftModeSupported": Plugin.Method.ShiftModeSupported,
        "batteryChargeMode": Plugin.Method.BatteryChargeMode,
        "batteryChargeModeSupported": Plugin.Method.BatteryChargeModeSupported,
        "superBattery": Plugin.Method.SuperBattery,
        "superBatterySupported": Plugin.Method.SuperBatterySupported,
        "fan1Rpm": Plugin.Method.Fan1Rpm,
        "fan2Rpm": Plugin.Method.Fan2Rpm,
        "fan3Rpm": Plugin.Method.Fan3Rpm,
        "fan4Rpm": Plugin.Method.Fan4Rpm,
        "fan1Supported": Plugin.Method.Fan1Supported,
        "fan2Supported": Plugin.Method.Fan2Supported,
        "fan3Supported": Plugin.Method.Fan3Supported,
        "fan4Supported": Plugin.Method.Fan4Supported,
        "fanModes": Plugin.Method.FanModes,
        "fanMode": Plugin.Method.FanMode,
        "fanModeSupported": Plugin.Method.FanModeSupported,
        "webcam": Plugin.Method.Webcam,
        "webcamBlock": Plugin.Method.WebcamBlock,
        "webcamSupported": Plugin.Method.WebcamSupported,
        "webcamBlockSupported": Plugin.Method.WebcamBlockSupported,
        "coolerBoost": Plugin.Method.CoolerBoost,
        "coolerBoostSupported": Plugin.Method.CoolerBoostSupported,
        "fnKey": Plugin.Method.FnKey,
        "winKey": Plugin.Method.WinKey,
        "fnWinSwapSupported": Plugin.Method.FnWinSwapSupported,
        "micMuteLed": Plugin.Method.MicMuteLed,
        "muteLed": Plugin.Method.MuteLed,
        "micMuteLedSupported": Plugin.Method.MicMuteLedSupported,
        "muteLedSupported": Plugin.Method.MuteLedSupported,
        "cpuRtFanSpeed": Plugin.Method.CpuRtFanSpeed,
        "cpuRtTemp": Plugin.Method.CpuRtTemp,
        "gpuRtFanSpeed": Plugin.Method.GpuRtFanSpeed,
        "gpuRtTemp": Plugin.Method.GpuRtTemp,
        "cpuFanCurveWmi2": Plugin.Method.CpuFanCurveWmi2,
        "cpuTempCurveWmi2": Plugin.Method.CpuTempCurveWmi2,
        "cpuHysteresisCurveWmi2": Plugin.Method.CpuHysteresisCurveWmi2,
        "gpuFanCurveWmi2": Plugin.Method.GpuFanCurveWmi2,
        "gpuTempCurveWmi2": Plugin.Method.GpuTempCurveWmi2,
        "gpuHysteresisCurveWmi2": Plugin.Method.GpuHysteresisCurveWmi2,
        "ecDumpRaw": Plugin.Method.EcDumpRaw,
        "ecDumpPretty": Plugin.Method.EcDumpPretty,
        "methods": Plugin.Method.Methods
    }
}
