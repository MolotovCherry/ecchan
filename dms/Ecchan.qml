pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import qs.Common
import qs.Widgets
import qs.Modules.Plugins
import qs.Services

import "./Services"
import "./Widgets"

PluginComponent {
    id: root

    layerNamespacePlugin: "ecchan"

    Connections {
        target: root.pluginData
        function onPluginDataChanged() {
            const socket = root.pluginData.socket;
            if (typeof (socket) === "string") {
                EcSocket.init(socket);
            }
        }
    }

    Component.onDestruction: {
        EcSocket.shutdown();
    }

    property var socketCache: ({})

    function call(name, cb, cbErr) {
        const fn = EcSocket?.[name];

        if (typeof fn !== "function") {
            const msg = `No function: ${name}`;
            cbErr?.(msg);
            ToastService.showError(msg);
            return;
        }

        const wrappedCb = value => {
            socketCache[name] = value;
            cb?.(value);
        };

        return fn.call(EcSocket, wrappedCb, cbErr);
    }

    function cachedCall(name, cb, cbErr) {
        const value = socketCache?.[name];
        if (value !== undefined) {
            cb?.(value);
        } else {
            call(name, cb, cbErr);
        }
    }

    horizontalBarPill: Component {
        Row {
            spacing: Theme.spacingS

            DankIcon {
                name: "memory"
                size: Theme.iconSizeSmall + 2
                color: Theme.surfaceText
                anchors.verticalCenter: parent.verticalCenter
            }
        }
    }

    verticalBarPill: Component {
        Column {
            spacing: Theme.spacingXS

            DankIcon {
                name: "memory"
                size: Theme.iconSizeSmall + 2
                color: Theme.surfaceText
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }
    }

    popoutContent: Component {
        PopoutComponent {
            id: popout

            property int currentTab: 0

            FocusScope {
                width: parent.width
                implicitHeight: root.popoutHeight - popout.headerHeight - popout.detailsHeight - Theme.spacingXL

                focus: true

                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingXS
                    spacing: Theme.spacingXS

                    // Branding

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacingXS

                        DankIcon {
                            id: memIcon
                            name: "memory"
                            size: Theme.iconSizeLarge - 6
                            color: Theme.primary
                        }

                        StyledText {
                            text: "Ecchan"
                            font.pixelSize: Theme.fontSizeLarge
                            font.weight: Font.Bold
                            color: Theme.surfaceText
                        }

                        Item {
                            Layout.fillWidth: true
                        }

                        DankButton {
                            Layout.preferredWidth: 30
                            Layout.preferredHeight: 30
                            hovered: false
                            pressed: false
                            backgroundColor: "transparent"
                            enableRipple: false

                            onClicked: EcSocket.connected ? EcSocket.shutdown() : EcSocket.reconnect()

                            DankIcon {
                                anchors.centerIn: parent
                                name: "circle"
                                filled: true
                                grade: 700
                                color: EcSocket.connected ? Theme.primary : Theme.surfaceText
                                size: Theme.iconSize - 6
                            }
                        }
                    }

                    // Top navigation buttons

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(Theme.fontSizeMedium * 3.7)

                        Repeater {
                            id: btns

                            model: [
                                {
                                    text: "Dashboard",
                                    icon: "analytics"
                                },
                                {
                                    text: "General",
                                    icon: "list_alt"
                                },
                                {
                                    text: "Performance",
                                    icon: "speed"
                                },
                                {
                                    text: "Curves",
                                    icon: "diagonal_line"
                                },
                                {
                                    text: "Battery",
                                    icon: "battery_android_full"
                                },
                                {
                                    text: "Methods",
                                    icon: "switch_access"
                                }
                            ]

                            Row {
                                id: row

                                Layout.fillWidth: true

                                required property int index
                                required property string text
                                required property string icon

                                Rectangle {
                                    width: tabRowContent.implicitWidth + Theme.spacingS * 2
                                    height: Math.round(Theme.fontSizeSmall * 3.1)
                                    radius: Theme.cornerRadius
                                    color: popout.currentTab === row.index ? Theme.primaryPressed : (tabMouseArea.containsMouse ? Theme.primaryHoverLight : "transparent")
                                    border.color: popout.currentTab === row.index ? Theme.primary : "transparent"
                                    border.width: popout.currentTab === row.index ? 1 : 0

                                    Row {
                                        id: tabRowContent
                                        anchors.centerIn: parent
                                        spacing: Theme.spacingXS

                                        DankIcon {
                                            name: row.icon
                                            size: Theme.iconSize - 2
                                            color: popout.currentTab === row.index ? Theme.primary : Theme.surfaceText
                                            opacity: popout.currentTab === row.index ? 1 : 0.7
                                            anchors.verticalCenter: parent.verticalCenter
                                        }

                                        StyledText {
                                            text: row.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            font.weight: Font.Medium
                                            color: popout.currentTab === row.index ? Theme.primary : Theme.surfaceText
                                            anchors.verticalCenter: parent.verticalCenter
                                        }
                                    }

                                    MouseArea {
                                        id: tabMouseArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: popout.currentTab = row.index
                                    }

                                    Behavior on color {
                                        ColorAnimation {
                                            duration: Theme.shortDuration
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Content

                    RowLayout {
                        Layout.fillHeight: true
                        Layout.fillWidth: true

                        RowLayout {
                            id: page0

                            visible: popout.currentTab == 0
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            Flow {
                                Layout.fillHeight: true
                                Layout.fillWidth: true
                                spacing: Theme.spacingXS

                                flow: Flow.TopToBottom

                                leftPadding: gpuGauge.hasDGpu ? 0 : (width - 180) / 2

                                Item {
                                    id: cpuGauge

                                    implicitHeight: 180
                                    implicitWidth: 180

                                    Connections {
                                        target: page0
                                        function onVisibleChanged() {
                                            if (page0.visible) {
                                                cpuUpdate.start();
                                            } else {
                                                cpuUpdate.stop();
                                            }
                                        }
                                    }

                                    property int temp: 0

                                    Timer {
                                        id: cpuUpdate
                                        interval: 1000
                                        repeat: true
                                        triggeredOnStart: true
                                        onTriggered: {
                                            root.call('cpuRtTemp', temp => cpuGauge.temp = temp);
                                        }
                                    }

                                    CircleGauge {
                                        width: parent.implicitHeight
                                        height: parent.implicitWidth

                                        readonly property color vendorColor: {
                                            return Theme.primary;
                                        }

                                        value: DgopService.cpuUsage / 100
                                        label: DgopService.cpuUsage.toFixed(1) + "%"
                                        detail: cpuGauge.temp > 0 ? (cpuGauge.temp.toFixed(0) + "°C") : ""
                                        sublabel: "CPU"
                                        accentColor: DgopService.cpuUsage > 80 ? Theme.error : (DgopService.cpuUsage > 50 ? Theme.warning : Theme.primary)
                                        detailColor: cpuGauge.temp > 85 ? Theme.error : (cpuGauge.temp > 70 ? Theme.warning : Theme.surfaceVariantText)
                                    }
                                }

                                Item {
                                    id: gpuGauge

                                    implicitHeight: 180
                                    implicitWidth: 180

                                    visible: hasDGpu

                                    Connections {
                                        target: page0
                                        function onVisibleChanged() {
                                            if (page0.visible) {
                                                gpuUpdate.start();
                                            } else {
                                                gpuUpdate.stop();
                                            }
                                        }
                                    }

                                    property bool hasDGpu: false
                                    property int temp: 0

                                    Timer {
                                        id: gpuUpdate
                                        interval: 1000
                                        repeat: true
                                        triggeredOnStart: true
                                        onTriggered: {
                                            root.cachedCall('hasDGpu', state => gpuGauge.hasDGpu = state);

                                            if (!gpuGauge.hasDGpu) {
                                                gpuUpdate.stop();
                                            }

                                            root.call('gpuRtTemp', temp => gpuGauge.temp = temp);
                                        }
                                    }

                                    CircleGauge {
                                        width: parent.implicitHeight
                                        height: parent.implicitWidth

                                        readonly property color vendorColor: {
                                            return Theme.success;
                                        }

                                        value: Math.min(1, gpuGauge.temp / 100)
                                        label: gpuGauge.temp > 0 ? (gpuGauge.temp.toFixed(0) + "°C") : "--"
                                        sublabel: "GPU"
                                        accentColor: {
                                            const temp = gpuGauge.temp;
                                            if (temp > 85)
                                                return Theme.error;
                                            if (temp > 70)
                                                return Theme.warning;
                                            return vendorColor;
                                        }
                                    }
                                }

                                Item {
                                    width: 180
                                    height: gpuGauge.hasDGpu ? 180 * 2 : 180
                                    Layout.fillWidth: true
                                    Layout.alignment: Qt.AlignCenter

                                    CircleGauge {
                                        anchors.centerIn: gpuGauge.hasDGpu ? parent : undefined
                                        width: 180
                                        height: 180
                                        value: DgopService.memoryUsage / 100
                                        label: compactMem(DgopService.usedMemoryKB)
                                        sublabel: "Memory"
                                        detail: DgopService.totalSwapKB > 0 ? ("+" + compactMem(DgopService.usedSwapKB)) : ""
                                        accentColor: DgopService.memoryUsage > 90 ? Theme.error : (DgopService.memoryUsage > 70 ? Theme.warning : Theme.secondary)

                                        function compactMem(kb) {
                                            if (kb < 1024 * 1024) {
                                                const mb = kb / 1024;
                                                return mb.toFixed(1) + " MB";
                                            }
                                            const gb = kb / (1024 * 1024);
                                            return gb.toFixed(1) + " GB";
                                        }
                                    }
                                }
                            }

                            ColumnLayout {
                                Layout.fillHeight: true
                                // do not expand past this
                                Layout.maximumWidth: (root.popoutWidth / 2) - 80
                                Layout.preferredWidth: (root.popoutWidth / 2) - 80
                                // stay stuck on right
                                Layout.alignment: Qt.AlignRight

                                // Fans
                                Item {
                                    id: fanSection

                                    Layout.fillHeight: true
                                    Layout.fillWidth: true

                                    property int fanCount: 1
                                    property int fan1Rpm: 0
                                    property int fan2Rpm: 0
                                    property int fan3Rpm: 0
                                    property int fan4Rpm: 0

                                    Connections {
                                        target: page0
                                        function onVisibleChanged() {
                                            if (page0.visible) {
                                                fanUpdate.start();
                                            } else {
                                                fanUpdate.stop();
                                            }
                                        }
                                    }

                                    Timer {
                                        id: fanUpdate
                                        interval: 1000
                                        repeat: true
                                        triggeredOnStart: true
                                        onTriggered: {
                                            root.cachedCall('fanCount', count => fanSection.fanCount = count);

                                            if (fanSection.fanCount >= 1) {
                                                root.call('fan1Rpm', rpm => fanSection.fan1Rpm = rpm);
                                            }
                                            if (fanSection.fanCount >= 2) {
                                                root.call('fan2Rpm', rpm => fanSection.fan2Rpm = rpm);
                                            }
                                            if (fanSection.fanCount >= 3) {
                                                root.call('fan3Rpm', rpm => fanSection.fan3Rpm = rpm);
                                            }
                                            if (fanSection.fanCount >= 4) {
                                                root.call('fan4Rpm', rpm => fanSection.fan4Rpm = rpm);
                                            }
                                        }
                                    }

                                    StyledRect {
                                        anchors.left: parent.left
                                        anchors.right: parent.right

                                        implicitHeight: fanCol.implicitHeight + Theme.spacingM * 2

                                        radius: Theme.cornerRadius
                                        color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)

                                        ColumnLayout {
                                            id: fanCol
                                            anchors.fill: parent
                                            anchors.margins: Theme.spacingM
                                            spacing: Theme.spacingL

                                            Row {
                                                spacing: Theme.spacingS

                                                DankIcon {
                                                    id: modeFanIcon
                                                    name: "mode_fan"
                                                    size: Theme.iconSize
                                                    color: Theme.primary
                                                }

                                                StyledText {
                                                    anchors.verticalCenter: parent.verticalCenter
                                                    text: "Fans"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }
                                            }

                                            Rectangle {
                                                Layout.alignment: Qt.AlignCenter
                                                implicitWidth: parent.width
                                                implicitHeight: 1
                                                color: Theme.outline
                                                opacity: 0.3
                                            }

                                            RowLayout {
                                                StyledText {
                                                    text: "Fan 1"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }

                                                Item {
                                                    Layout.fillWidth: true
                                                }

                                                StyledText {
                                                    text: fanSection.fan1Rpm + " rpm"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }
                                            }

                                            Rectangle {
                                                visible: fanSection.fanCount >= 2
                                                Layout.alignment: Qt.AlignCenter
                                                implicitWidth: parent.width
                                                implicitHeight: 1
                                                color: Theme.outline
                                                opacity: 0.3
                                            }

                                            RowLayout {
                                                visible: fanSection.fanCount >= 2

                                                StyledText {
                                                    text: "Fan 2"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }

                                                Item {
                                                    Layout.fillWidth: true
                                                }

                                                StyledText {
                                                    text: fanSection.fan2Rpm + " rpm"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }
                                            }

                                            Rectangle {
                                                visible: fanSection.fanCount >= 3
                                                Layout.alignment: Qt.AlignCenter
                                                implicitWidth: parent.width
                                                implicitHeight: 1
                                                color: Theme.outline
                                                opacity: 0.3
                                            }

                                            RowLayout {
                                                visible: fanSection.fanCount >= 3

                                                StyledText {
                                                    text: "Fan 3"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }

                                                Item {
                                                    Layout.fillWidth: true
                                                }

                                                StyledText {
                                                    text: fanSection.fan3Rpm + " rpm"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }
                                            }

                                            Rectangle {
                                                visible: fanSection.fanCount >= 4
                                                Layout.alignment: Qt.AlignCenter
                                                implicitWidth: parent.width
                                                implicitHeight: 1
                                                color: Theme.outline
                                                opacity: 0.3
                                            }

                                            RowLayout {
                                                visible: fanSection.fanCount >= 4

                                                StyledText {
                                                    text: "Fan 4"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }

                                                Item {
                                                    Layout.fillWidth: true
                                                }

                                                StyledText {
                                                    text: fanSection.fan4Rpm + " rpm"
                                                    font.pixelSize: Theme.fontSizeLarge
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    popoutWidth: 650
    popoutHeight: 500
}
