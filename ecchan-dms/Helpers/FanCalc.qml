import QtQml

import "../Services"

QtObject {
    id: root

    property var _c: Connections {
        target: EcchanClient

        property bool init: true

        function onInitStateChanged(state) {
            const finished = !state;

            if (finished && init) {
                init = false;
                fanTimer.start();
            }
        }
    }

    property Timer timer: Timer {
        id: fanTimer
        interval: 500
        repeat: true
        triggeredOnStart: true
        onTriggered: root.calc()
    }

    property string text: "Min"
    property int level: 0

    function calc() {
        const inRange = (num, min, max) => num >= min && num <= max;

        // use the rpm calculations to get an accurate realtime view
        //
        // these are the "source of truth" calculations for true fan speed percentage.
        const fan1Perc = Math.round(EcchanClient.fan1Rpm / 60);
        const fan2Perc = Math.round(EcchanClient.fan2Rpm / 60);
        const fan3Perc = Math.round(EcchanClient.fan3Rpm / 60);
        const fan4Perc = Math.round(EcchanClient.fan4Rpm / 60);

        // use highest percentage
        // qmlformat off
        let perc = Math.max(
            Math.max(fan3Perc, fan4Perc),
            Math.max(fan1Perc, fan2Perc)
        );
        // qmlformat on

        // once fan rpms hit the target percent, they fluctuate -1%-0% of the target,
        // leaving a slight bit of volatility. this could matter when a node (35) is just on the
        // boundary, e.g. 34-35. In this case, we prefer stability of the target percentage when at rest
        const targetFan = Math.max(EcchanClient.cpuRtFanSpeed, EcchanClient.gpuRtFanSpeed);
        if (perc - targetFan === -1) {
            perc = targetFan;
        }

        if (inRange(perc, 0, 35)) {
            level = 1;
            text = "Min";
        } else if (inRange(perc, 35, 45)) {
            level = 2;
            text = "Low";
        } else if (inRange(perc, 45, 60)) {
            level = 3;
            text = "Mid";
        } else if (inRange(perc, 60, 75)) {
            level = 4;
            text = "High";
        } else if (inRange(perc, 75, 150)) {
            level = 5;
            text = "Max";
        }
    }
}
