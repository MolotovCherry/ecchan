pragma Singleton
pragma ComponentBehavior: Bound

import QtQuick
import qs.Services

import "../ecchan_client"

EcchanClient {
    id: root

    onConnectedChanged: {
        if (connected) {
            disableErrors = false;
        }

        if (!connected) {
            reconnectTimer.start();
        }
    }

    property bool disableErrors: false
    onError: error => {
        if (connected) {
            disableErrors = false;
            ToastService.showError("EcchanClient error", error);
        } else if (!connected && !disableErrors) {
            disableErrors = true;
            ToastService.showError("EcchanClient error", error);
        }
    }

    function disconnect() {
        connected = false;
    }

    function connect() {
        connected = true;
    }

    property var timer: Timer {
        id: reconnectTimer
        interval: 500
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            if (root.connected) {
                reconnectTimer.stop();
            } else {
                root.connected = true;
            }
        }
    }

    // qmlformat off
    property var profileProps: [
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
}
