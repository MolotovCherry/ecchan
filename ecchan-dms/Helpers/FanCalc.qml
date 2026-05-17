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

    property string text: "Off"
    property int level: 0

    function calc() {
        //
        // <Isoff>
        //
        let isOff = true;

        switch (EcchanClient.fanCount) {
            // qmlformat off
            case 4:
                isOff = isOff && EcchanClient.fan4Rpm === 0;
            // fallthrough
            case 3:
                isOff = isOff && EcchanClient.fan3Rpm === 0;
            // fallthrough
            case 2:
                isOff = isOff && EcchanClient.fan2Rpm === 0;
            // fallthrough
            case 1:
                isOff = isOff && EcchanClient.fan1Rpm === 0;
            // qmlformat on
        }

        if (isOff) {
            text = "Off";
            level = 0;
            return;
        }

        //
        //  </IsOff>
        //

        const clamp = (val, min, max) => Math.min(Math.max(val, min), max);
        const inRange = (num, min, max) => num >= min && num <= max;

        // we could check the current fan %, but that requires a lot more programming
        // than just converting rpm into % ; we have a formula for this
        const fan1Perc = EcchanClient.fan1Rpm / 60;
        const fan2Perc = EcchanClient.fan2Rpm / 60;
        const fan3Perc = EcchanClient.fan3Rpm / 60;
        const fan4Perc = EcchanClient.fan4Rpm / 60;

        // average out percentages, both for main (2 main fans), and for all
        let fanRpmAll = 0;
        let fanRpmMain = 0;
        switch (EcchanClient.fanCount) {
            // qmlformat off
            case 4:
                fanRpmAll += fan4Perc;
                // fallthrough
            case 3:
                fanRpmAll += fan3Perc;
                // fallthrough
            case 2:
                fanRpmAll += fan2Perc;
                fanRpmMain += fan2Perc;
                // fallthrough
            case 1:
                fanRpmAll += fan1Perc;
                fanRpmMain += fan1Perc;
            // qmlformat on
        }

        fanRpmAll /= EcchanClient.fanCount;
        fanRpmMain /= clamp(EcchanClient.fanCount, 1, 2);

        // use highest percentage
        const perc = Math.max(fanRpmAll, fanRpmMain);

        if (inRange(perc, 0, 35)) {
            level = 1;
            text = "Min";
            return;
        } else if (inRange(perc, 35, 50)) {
            level = 2;
            text = "Low";
            return;
        } else if (inRange(perc, 50, 65)) {
            level = 3;
            text = "Mid";
            return;
        } else if (inRange(perc, 65, 75)) {
            level = 4;
            text = "High";
            return;
        } else if (inRange(perc, 75, 150)) {
            level = 5;
            text = "Max";
            return;
        }
    }
}
