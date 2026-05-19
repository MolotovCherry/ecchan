import QtQml

import "../Services"

QtObject {
    id: root

    property var _c: Connections {
        target: EcchanClient
        function onInitStateChanged(state) {
            const finished = !state;

            if (finished) {
                let extraArgs = [];
                switch (EcchanClient.fanCount) {
                    // qmlformat off
                    case 4:
                        extraArgs.push("fan4Rpm");
                    // fallthrough
                    case 3:
                        extraArgs.push("fan3Rpm");
                    // fallthrough
                    case 2:
                        extraArgs.push("fan2Rpm");
                        extraArgs.push("gpuRtFanSpeed");
                    // fallthrough
                    case 1:
                        extraArgs.push("fan1Rpm");
                        extraArgs.push("cpuRtFanSpeed");
                    // qmlformat on
                }

                Update.addRef(extraArgs);
                fanTimer.start();
            }
        }
    }

    Component.onDestruction: {
        Update.removeRef(["fan4Rpm", "fan3Rpm", "fan2Rpm", "fan1Rpm", "gpuRtFanSpeed", "cpuRtFanSpeed"]);
        fanTimer.stop();
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

        // use the rpm calculations to get a more realtime view
        // but for 1 and 2 we prefer rtFanSpeed over rpm calculations because it flucuates less
        // this could matter when it's just on the boundary, e.g. 34-35; it prevents flip flopping
        // back and forth
        const fan1Perc = Math.max(Math.round(EcchanClient.fan1Rpm / 60), EcchanClient.cpuRtFanSpeed);
        const fan2Perc = Math.max(Math.round(EcchanClient.fan2Rpm / 60), EcchanClient.gpuRtFanSpeed);
        const fan3Perc = Math.round(EcchanClient.fan3Rpm / 60);
        const fan4Perc = Math.round(EcchanClient.fan4Rpm / 60);

        // use highest percentage
        // qmlformat off
        const perc = Math.max(
            Math.max(fan3Perc, fan4Perc),
            Math.max(fan1Perc, fan2Perc)
        );
        // qmlformat on

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
