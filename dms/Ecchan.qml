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
                    anchors.margins: 2
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

                            ColumnLayout {
                                Layout.fillHeight: true
                                Layout.preferredWidth: root.popoutWidth / 2

                                Item {
                                    id: cpuGauge

                                    implicitHeight: 180
                                    implicitWidth: 180
                                    Layout.alignment: Qt.AlignCenter

                                    Connections {
                                        target: page0
                                        function onVisibleChanged() {
                                            if (page0.visible && EcSocket.connected) {
                                                cpuUpdate.start();
                                            } else {
                                                cpuUpdate.stop();
                                            }
                                        }
                                    }

                                    property int temp: 0

                                    Timer {
                                        id: cpuUpdate
                                        interval: 1500
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

                                        value: Math.min(1, cpuGauge.temp / 100)
                                        label: cpuGauge.temp > 0 ? (cpuGauge.temp.toFixed(0) + "°C") : "--"
                                        sublabel: "CPU"
                                        accentColor: {
                                            const temp = cpuGauge.temp;
                                            if (temp > 85)
                                                return Theme.error;
                                            if (temp > 70)
                                                return Theme.warning;
                                            return vendorColor;
                                        }
                                    }
                                }

                                Item {
                                    Layout.fillHeight: true
                                    visible: gpuGauge.hasDGpu
                                }

                                Item {
                                    id: gpuGauge

                                    implicitHeight: 180
                                    implicitWidth: 180
                                    Layout.alignment: Qt.AlignCenter

                                    visible: hasDGpu

                                    Connections {
                                        target: page0
                                        function onVisibleChanged() {
                                            if (page0.visible && EcSocket.connected) {
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
                                        interval: 1500
                                        repeat: true
                                        triggeredOnStart: true
                                        onTriggered: {
                                            root.cachedCall('hasDGpu', state => gpuGauge.hasDGpu = state);
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
                            }

                            ColumnLayout {
                                Layout.fillHeight: true
                                Layout.preferredWidth: root.popoutWidth / 2

                                // Fans
                                Rectangle {
                                    Layout.fillWidth: true
                                    radius: Theme.cornerRadius
                                    color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)

                                    property int fanCount: 1

                                    DankIcon {
                                        name: "mode_fan"
                                        size: Theme.iconSizeSmall
                                        color: Theme.primary
                                    }

                                    StyledText {
                                        text: "Fans"
                                        font.pixelSize: Theme.fontSizeMedium
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

    popoutWidth: 650
    popoutHeight: 500
}
