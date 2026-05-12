pragma Singleton
pragma ComponentBehavior: Bound

import QtQuick
import qs.Services

import "../ecchan_client"

EcchanClient {
    id: root

    // whether to attempt immediate reconnection attempt after first fail
    property bool reconnect: true
    property int _reconnectAttempt: 0

    onConnectedChanged: {
        if (connected) {
            _reconnectAttempt = 0;
        } else if (reconnect && _reconnectAttempt == 0) {
            _reconnectAttempt += 1;
            Qt.callLater(() => connect());
        }
    }

    onError: error => ToastService.showError("EcchanClient error", error)

    function disconnect() {
        // do not reconnect since this was intentional
        _reconnectAttempt += 1;
        connected = false;
    }

    function connect() {
        connected = true;
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
