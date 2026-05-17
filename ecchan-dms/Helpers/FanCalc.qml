import QtQml

import "../Services"

QtObject {
    id: root

    Component.onCompleted: {
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
            // fallthrough
            case 1:
                extraArgs.push("fan1Rpm");
            // qmlformat on
        }

        Update.addRef(extraArgs);
        fanTimer.start();
    }

    Component.onDestruction: {
        Update.removeRef(["fan4Rpm", "fan3Rpm", "fan2Rpm", "fan1Rpm"]);
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

        // we could check the current fan %, but that requires a lot more programming
        // than just converting rpm into % ; we have a formula for this
        const fan1Perc = EcchanClient.fan1Rpm / 60;
        const fan2Perc = EcchanClient.fan2Rpm / 60;
        const fan3Perc = EcchanClient.fan3Rpm / 60;
        const fan4Perc = EcchanClient.fan4Rpm / 60;

        // use highest percentage
        // qmlformat off
        const perc = Math.round(
            Math.max(
                Math.max(fan3Perc, fan4Perc),
                Math.max(fan1Perc, fan2Perc)
            )
        );
        // qmlformat on

        if (inRange(perc, 0, 35)) {
            level = 1;
            text = "Min";
        } else if (inRange(perc, 35, 45)) {
            level = 2;
            text = "Low";
        } else if (inRange(perc, 45, 55)) {
            level = 3;
            text = "Mid";
        } else if (inRange(perc, 55, 70)) {
            level = 4;
            text = "High";
        } else if (inRange(perc, 70, 150)) {
            level = 5;
            text = "Max";
        }
    }
}
