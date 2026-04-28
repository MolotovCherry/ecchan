pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import qs.Common
import qs.Widgets
import qs.Modules.Plugins
import qs.Services

import "./Services"
import "./Widgets"
import "./Common"

PluginComponent {
    id: root

    layerNamespacePlugin: "ecchan"

    onPluginDataChanged: {
        const socket = pluginData.socket;
        if (typeof (socket) === "string" && !EcSocket.connected) {
            EcSocket.init(socket);
        }
    }

    Component.onDestruction: {
        EcSocket.shutdown();
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

    Timer {
        id: gpuUpdate
        interval: 1000
        repeat: true
        triggeredOnStart: true
        onTriggered: {
            if (!EcSocket.state.hasDGpu) {
                gpuUpdate.stop();
            }

            EcSocket.gpuRtTemp();
        }
    }

    Timer {
        id: cpuUpdate
        interval: 1000
        repeat: true
        triggeredOnStart: true
        onTriggered: EcSocket.cpuRtTemp()
    }

    Timer {
        id: fanUpdate
        interval: 1000
        repeat: true
        triggeredOnStart: true
        // qmlformat off
        onTriggered: {
            // qmllint disable unterminated-case
            switch (EcSocket.state.fanCount) {
                case 4:
                    EcSocket.fan4Rpm();
                case 3:
                    EcSocket.fan3Rpm();
                case 2:
                    EcSocket.fan2Rpm();
                case 1:
                default:
                    EcSocket.fan1Rpm();
            }
        }
        // qmlformat on
    }

    property var profilesModel: []
    property int selectedProfile: 0
    property var profiles: []
    property bool _blockProfileUpdate: true
    property bool _startup: true
    property SocketCbManager _cbQueue: SocketCbManager {}

    Connections {
        target: EcSocket

        function onInitStarted() {
            root._blockProfileUpdate = true;
        }

        function onInitFinished() {
            root._blockProfileUpdate = false;

            if (root._startup) {
                EcSocket.applyState(root.profiles[root.selectedProfile].state);
                root._startup = false;
            }
        }

        function onApplyStarted() {
            root._blockProfileUpdate = true;
        }

        function onApplyFinished() {
            root._blockProfileUpdate = false;
            root.profiles[root.selectedProfile].state = EcSocket.getSanitizedState();
            root.profilesChanged();
        }

        function onStateChanged() {
            if (!root._blockProfileUpdate) {
                root.profiles[root.selectedProfile].state = EcSocket.getSanitizedState();
                root.profilesChanged();
            }
        }
    }

    onPluginServiceChanged: {
        if (!pluginService) {
            return;
        }

        selectedProfile = _loadPluginData("selectedProfile", 0);
        selectedProfileChanged();

        profiles = _loadPluginData("profiles", [
            {
                "name": "Default",
                "state": EcSocket.getSanitizedState()
            }
        ]);
        profilesChanged();
    }

    onProfilesChanged: {
        profilesModel = profiles.map(item => item.name);

        if (root.pluginService) {
            _savePluginData("profiles", profiles);
        }
    }

    onSelectedProfileChanged: {
        if (root.pluginService) {
            _savePluginData("selectedProfile", selectedProfile);
        }
    }

    // Settings fns

    function _loadPluginData(key, defaultValue) {
        return pluginService.loadPluginData("ecchan", key, defaultValue);
    }

    function _savePluginData(key, value) {
        pluginService.savePluginData("ecchan", key, value);
    }

    function _getGlobalVar(key, defaultValue) {
        return pluginService.setGlobalVar("ecchan", key, defaultValue);
    }

    function _setGlobalVar(key, value) {
        pluginService.setGlobalVar("ecchan", key, value);
    }

    // --

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

                            MouseArea {
                                anchors.fill: parent
                                onClicked: {
                                    popout.currentTab = 99;
                                }
                            }
                        }

                        StyledText {
                            text: "Ecchan"
                            font.pixelSize: Theme.fontSizeLarge
                            font.weight: Font.Bold
                            color: Theme.surfaceText
                        }

                        Item {
                            implicitWidth: 1
                        }

                        Rectangle {
                            Layout.alignment: Qt.AlignHCenter
                            implicitWidth: 1.1
                            implicitHeight: 20
                            color: Theme.outline
                            opacity: 0.3
                        }

                        Item {
                            implicitWidth: 1
                        }

                        StyledText {
                            text: "Status"

                            font.pixelSize: Theme.fontSizeLarge
                            font.weight: Font.Bold
                            color: Theme.surfaceText
                        }

                        DankButton {
                            Layout.preferredWidth: 20
                            Layout.preferredHeight: 20
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

                        Item {
                            Layout.fillWidth: true
                        }

                        DankEditableDropdown {
                            currentIdx: root.selectedProfile
                            options: root.profilesModel
                            addNewTextEntry: "Add Profile"

                            onValueDeleted: (idx, name) => {
                                root.profiles.splice(idx, 1);
                                root.profilesChanged();
                            }

                            onValueChanged: (idx, name, isSame) => {
                                if (idx == -1) {
                                    valueAdded(0, "Default");
                                    return;
                                }

                                root.selectedProfile = idx;

                                if (isSame) {
                                    return;
                                }

                                const state = root.profiles[idx].state;
                                EcSocket.applyState(state);
                            }

                            onValueAdded: (idx, name) => {
                                // explicit reassign so signals fire
                                root.profiles = [...root.profiles,
                                    {
                                        "name": name,
                                        "state": EcSocket.getSanitizedState()
                                    }
                                ];

                                root.selectedProfile = idx;
                            }
                        }
                    }

                    // Top navigation buttons

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(Theme.fontSizeMedium * 3.7)

                        Item {
                            Layout.fillWidth: true
                        }

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
                                }
                            ]

                            Row {
                                id: row

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

                        Item {
                            Layout.fillWidth: true
                        }
                    }

                    // Content

                    RowLayout {
                        Layout.fillHeight: true
                        Layout.fillWidth: true

                        // Dashboard
                        RowLayout {
                            id: page1

                            visible: popout.currentTab === 0
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            Flow {
                                Layout.fillHeight: true
                                Layout.fillWidth: true
                                spacing: Theme.spacingXS

                                flow: Flow.TopToBottom

                                leftPadding: EcSocket.state.hasDGpu && DgopService.dgopAvailable ? 0 : (width - 180) / 2

                                Item {
                                    id: cpuGauge

                                    implicitHeight: 180
                                    implicitWidth: 180

                                    Connections {
                                        target: page1
                                        function onVisibleChanged() {
                                            if (page1.visible) {
                                                cpuUpdate.start();
                                                DgopService.addRef(["cpu"]);
                                            } else {
                                                cpuUpdate.stop();
                                                DgopService.removeRef(["cpu"]);
                                            }
                                        }
                                    }

                                    CircleGauge {
                                        width: parent.implicitHeight
                                        height: parent.implicitWidth

                                        readonly property color vendorColor: {
                                            return Theme.primary;
                                        }

                                        value: DgopService.dgopAvailable ? (DgopService.cpuUsage / 100) : Math.min(1, EcSocket.state.cpuRtTemp / 100)
                                        label: DgopService.dgopAvailable ? (DgopService.cpuUsage.toFixed(1) + "%") : (EcSocket.state.cpuRtTemp + "°C")
                                        detail: DgopService.dgopAvailable ? (EcSocket.state.cpuRtTemp > 0 ? (EcSocket.state.cpuRtTemp + "°C") : "") : ""
                                        sublabel: "CPU"
                                        accentColor: {
                                            const dgop = DgopService.cpuUsage > 80 ? Theme.error : (DgopService.cpuUsage > 50 ? Theme.warning : Theme.primary);
                                            const cpu = EcSocket.state.cpuRtTemp > 85 ? Theme.error : (EcSocket.state.cpuRtTemp > 70 ? Theme.warning : Theme.primary);
                                            return DgopService.dgopAvailable ? dgop : cpu;
                                        }
                                        detailColor: EcSocket.state.cpuRtTemp > 85 ? Theme.error : (EcSocket.state.cpuRtTemp > 70 ? Theme.warning : Theme.surfaceVariantText)
                                    }
                                }

                                Item {
                                    id: gpuGauge

                                    implicitHeight: 180
                                    implicitWidth: 180

                                    visible: EcSocket.state.hasDGpu

                                    Connections {
                                        target: page1
                                        function onVisibleChanged() {
                                            if (page1.visible) {
                                                gpuUpdate.start();
                                            } else {
                                                gpuUpdate.stop();
                                            }
                                        }
                                    }

                                    CircleGauge {
                                        width: parent.implicitHeight
                                        height: parent.implicitWidth

                                        readonly property color vendorColor: {
                                            return Theme.success;
                                        }

                                        value: Math.min(1, EcSocket.state.gpuRtTemp / 100)
                                        label: EcSocket.state.gpuRtTemp > 0 ? (EcSocket.state.gpuRtTemp + "°C") : "--"
                                        sublabel: "GPU"
                                        accentColor: {
                                            const temp = EcSocket.state.gpuRtTemp;
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
                                    height: EcSocket.state.hasDGpu ? 180 * 2 : 180
                                    Layout.fillWidth: true
                                    Layout.alignment: Qt.AlignCenter

                                    Connections {
                                        target: page1
                                        function onVisibleChanged() {
                                            if (page1.visible) {
                                                DgopService.addRef(["memory"]);
                                            } else {
                                                DgopService.removeRef(["memory"]);
                                            }
                                        }
                                    }

                                    CircleGauge {
                                        visible: DgopService.dgopAvailable
                                        anchors.centerIn: EcSocket.state.hasDGpu ? parent : undefined
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

                                    Connections {
                                        target: page1
                                        function onVisibleChanged() {
                                            if (page1.visible) {
                                                fanUpdate.start();
                                            } else {
                                                fanUpdate.stop();
                                            }
                                        }
                                    }

                                    StyledRect {
                                        id: fanRect
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
                                                spacing: Theme.spacingXS

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

                                            Repeater {
                                                id: fanRptr

                                                model: [
                                                    {
                                                        "rpm": EcSocket.state.fan1Rpm
                                                    },
                                                    {
                                                        "rpm": EcSocket.state.fan2Rpm
                                                    },
                                                    {
                                                        "rpm": EcSocket.state.fan3Rpm
                                                    },
                                                    {
                                                        "rpm": EcSocket.state.fan4Rpm
                                                    }
                                                ]

                                                ColumnLayout {
                                                    id: fanRow
                                                    spacing: Theme.spacingL
                                                    visible: EcSocket.state.fanCount > index

                                                    required property int index
                                                    required property string rpm

                                                    Rectangle {
                                                        Layout.alignment: Qt.AlignCenter
                                                        implicitWidth: parent.width
                                                        implicitHeight: 1.1
                                                        color: Theme.outline
                                                        opacity: 0.3
                                                    }

                                                    RowLayout {
                                                        Layout.fillWidth: true

                                                        StyledText {
                                                            text: "Fan " + (fanRow.index + 1)
                                                            font.pixelSize: Theme.fontSizeLarge
                                                            font.weight: Font.Medium
                                                            color: Theme.surfaceText
                                                        }

                                                        StyledText {
                                                            Layout.fillWidth: true
                                                            horizontalAlignment: Text.AlignRight
                                                            text: fanRow.rpm + " rpm"
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

                        // EcMem page
                        ColumnLayout {
                            id: page99

                            visible: popout.currentTab === 99
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            onVisibleChanged: {
                                if (visible) {
                                    memTimer.start();
                                } else {
                                    memTimer.stop();
                                }
                            }

                            StyledRect {
                                Layout.fillWidth: true
                                Layout.fillHeight: true

                                radius: Theme.cornerRadius
                                color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)

                                StyledText {
                                    id: styledMemText
                                    Layout.fillHeight: true
                                    Layout.fillWidth: true
                                    isMonospace: true
                                    font.pixelSize: 13

                                    anchors.centerIn: parent

                                    text: "|      | _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _A _B _C _D _E _F\n|------+------------------------------------------------\n| 0x0_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x1_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x2_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x3_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x4_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x5_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x6_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x7_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x8_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0x9_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xA_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xB_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xC_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xD_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xE_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n| 0xF_ | 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |................|\n"

                                    Timer {
                                        id: memTimer
                                        interval: 1000
                                        repeat: true
                                        triggeredOnStart: true
                                        onTriggered: {
                                            root._cbQueue.call("ecDumpPretty").cb(data => {
                                                styledMemText.text = data;
                                            });
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
