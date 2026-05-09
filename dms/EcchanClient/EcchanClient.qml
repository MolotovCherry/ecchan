pragma Singleton
pragma ComponentBehavior: Bound

import QtQuick

import "../ecchan_client"

EcchanClient {
    id: root

    // whether to attempt immediate reconnection attempt after first fail
    property bool reconnect: true
    property int _reconnectAttempt: 0

    onConnectedChanged: {
        Method;
        if (connected) {
            _reconnectAttempt = 0;
        } else if (reconnect && _reconnectAttempt == 0) {
            _reconnectAttempt += 1;
            Qt.callLater(() => connect());
        }
    }

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

    function serialize() {
        const out = {};

        for (const prop of root.profileProps) {
            if (prop === "methods") {
                continue;
            }

            out[prop] = root[prop];
        }

        const methods = Object.assign({}, root.methods);
        delete methods.objectName;
        for (const method of Object.keys(methods)) {
            delete methods[method].objectName;
        }

        out.methods = methods;

        return out;
    }

    function apply(profile) {
        if (profile == null) {
            return;
        }

        for (const prop of root.profileProps) {
            if (prop === "methods") {
                continue;
            }

            const data = profile[prop];

            if (data == null) {
                continue;
            }

            root[prop] = data;
        }

        for (const [key, value] of Object.entries(profile.methods)) {
            if (!value.write) {
                continue;
            }

            root.methods[key].value = value.value;
        }
    }
}
