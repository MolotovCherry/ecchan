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

    property var profileTimer: profileWriteTimer
    Component.onCompleted: {
        SocketHandler.addGlobal("profileSaver", (id, method, payload, isErr) => {
            if (isErr) {
                return;
            }

            if (root._blockUpdate) {
                root.profileTimer.stop();
                return;
            }

            // state is changed on every onDataReady firing; but that doesn't mean crucial properties changed!

            const updateFor = ["shiftMode", "batteryChargeMode", "superBattery", "fanMode", "webcam", "webcamBlock", "coolerBoost", "fnKey", "winKey", "micMuteLed", "muteLed", "cpuFanCurveWmi2", "cpuTempCurveWmi2", "cpuHysteresisCurveWmi2", "gpuFanCurveWmi2", "gpuTempCurveWmi2", "gpuHysteresisCurveWmi2", "methods"];

            const shouldSave = updateFor.includes(method) || method.startsWith("set");

            // avoid useless writes to disk ; acts as a debouncer
            if (shouldSave) {
                root.profileTimer.restart();
            }
        });
    }

    Component.onDestruction: {
        EcSocket.shutdown();
        SocketHandler.removeGlobal("profileSaver");
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
    property bool _blockUpdate: false

    Connections {
        target: EcSocket

        function onInitStarted() {
            root._blockUpdate = true;
        }

        function onInitFinished() {
            const state = EcSocket.state.deserialize(root.profiles[root.selectedProfile].state);
            EcSocket.applyState(state);
        }

        function onApplyStarted() {
        }

        function onApplyFinished() {
            root._blockUpdate = false;
            root.profiles[root.selectedProfile].state = EcSocket.state.serialize();
            root.profilesChanged();
        }
    }

    Timer {
        id: profileWriteTimer
        interval: 500
        repeat: false
        triggeredOnStart: false
        onTriggered: {
            const state = EcSocket.state.serialize();
            root.profiles[root.selectedProfile].state = state;
            root.profilesChanged();
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
                "state": EcSocket.state.serialize()
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

                                const state = EcSocket.state.deserialize(root.profiles[idx].state);
                                EcSocket.applyState(state);
                            }

                            onValueAdded: (idx, name) => {
                                // explicit reassign so signals fire
                                root.profiles = [...root.profiles,
                                    {
                                        "name": name,
                                        "state": EcSocket.state.serialize()
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
                                    text: "Fans",
                                    icon: "mode_fan"
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

                        // General
                        ColumnLayout {
                            id: page2

                            visible: popout.currentTab === 1
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            StyledRect {
                                Layout.fillWidth: true
                                Layout.fillHeight: true

                                radius: Theme.cornerRadius
                                color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)

                                GridLayout {
                                    columns: 4

                                    anchors.top: parent.top
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.margins: Theme.spacingM

                                    rowSpacing: 0
                                    columnSpacing: 0

                                    property var methodList: EcSocket.state.methodList.map(item => {
                                        // qmlformat off
                                        const ops = [
                                            { suffix: "Range", type: "range", op: "WriteRange" },
                                            { suffix: "",      type: "byte", op: "Write" },
                                            { suffix: "Bit",   type: "bit", op: "WriteBit" }
                                        ];
                                        // qmlformat on

                                        const found = ops.find(c => item.ops.includes("Read" + c.suffix) && item.ops.includes("Write" + c.suffix));

                                        // no support
                                        if (!found) {
                                            return;
                                        }

                                        const state = EcSocket.state.methods[item.method];
                                        if (state == null) {
                                            return;
                                        }

                                        return {
                                            "name": item.name,
                                            "icon": null,
                                            "description": null,
                                            "supported": true,
                                            "value": null,
                                            "set": value => EcSocket.methodWrite(item.method, found.op, value),
                                            "type": "method",
                                            "variation": found.type,
                                            "methodKey": item.method
                                        };
                                    }).filter(item => item != null)

                                    property var modelBase: [
                                        {
                                            "name": "Webcam",
                                            "icon": "camera_video",
                                            "description": "Enable the integrated webcam (as if by a keyboard button)",
                                            "supported": EcSocket.state.webcamSupported,
                                            "value": EcSocket.state.webcam,
                                            "set": state => EcSocket.setWebcam(state),
                                            "type": "toggle",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        {
                                            "name": "Webcam Block",
                                            "icon": "camera_video",
                                            "description": "Block the integrated webcam (can't be enabled by a keyboard button)",
                                            "supported": EcSocket.state.webcamBlockSupported,
                                            "value": EcSocket.state.webcamBlock,
                                            "set": state => EcSocket.setWebcamBlock(state),
                                            "type": "toggle",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        {
                                            "name": "Swap Win/Fn",
                                            "icon": null,
                                            "description": "Swap the Fn / Windows key positions",
                                            "supported": EcSocket.state.fnWinSwapSupported,
                                            "value": EcSocket.state.fnKey,
                                            "set": state => EcSocket.setFnKey(state),
                                            "type": "swapKey",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        {
                                            "name": "Mic Mute Light",
                                            "icon": "backlight_high",
                                            "description": "Toggle the mic mute keyboard indicator light",
                                            "supported": EcSocket.state.micMuteLedSupported,
                                            "value": EcSocket.state.micMuteLed,
                                            "set": state => EcSocket.setMicMuteLed(state),
                                            "type": "toggle",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        {
                                            "name": "Mute Light",
                                            "icon": "backlight_high",
                                            "description": "Toggle the audio mute keyboard indicator light",
                                            "supported": EcSocket.state.muteLedSupported,
                                            "value": EcSocket.state.muteLed,
                                            "set": state => EcSocket.setMuteLed(state),
                                            "type": "toggle",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        // qmlformat off
                                        ...methodList
                                        // qmlformat on
                                    ]

                                    property var filteredModel: modelBase.filter(item => item.supported)

                                    Repeater {
                                        model: parent.filteredModel

                                        ColumnLayout {
                                            id: page2Column
                                            Layout.preferredWidth: actionBtn.width
                                            Layout.preferredHeight: page2Column.implicitHeight + Theme.spacingL
                                            Layout.alignment: Qt.AlignTop | Qt.AlignHCenter

                                            required property string name
                                            required property var description
                                            required property bool supported
                                            required property var value
                                            required property var set
                                            required property string type
                                            required property string variation
                                            required property int index
                                            required property string methodKey
                                            required property var icon

                                            spacing: 0

                                            // toggles
                                            ToggleActionButton {
                                                id: actionBtn
                                                visible: type === "toggle"

                                                iconName: icon
                                                checked: value
                                                iconSize: Theme.iconSizeLarge
                                                buttonHeight: 70
                                                buttonWidth: 130

                                                onClicked: set(!value)
                                            }

                                            // swap key
                                            StyledRect {
                                                visible: type === "swapKey"

                                                radius: Theme.cornerRadius
                                                color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)

                                                Layout.preferredHeight: 70
                                                Layout.preferredWidth: 130

                                                RowLayout {

                                                    anchors.centerIn: parent

                                                    spacing: Theme.spacingXS

                                                    Item {
                                                        Layout.fillWidth: true
                                                    }

                                                    Rectangle {
                                                        implicitHeight: 25
                                                        implicitWidth: 50
                                                        radius: height / 2
                                                        color: Theme.primary

                                                        StyledText {
                                                            text: EcSocket.state.winKey === "Left" ? "Win" : "Fn"
                                                            color: Theme.primaryText
                                                            anchors.centerIn: parent
                                                            font.weight: Font.Bold
                                                        }

                                                        MouseArea {
                                                            anchors.fill: parent
                                                            onClicked: set(EcSocket.state.fnKey === "Left" ? "Right" : "Left")
                                                        }
                                                    }

                                                    Rectangle {
                                                        implicitHeight: 25
                                                        implicitWidth: 50
                                                        radius: height / 2
                                                        color: Theme.primary

                                                        StyledText {
                                                            text: EcSocket.state.fnKey === "Right" ? "Fn" : "Win"
                                                            color: Theme.primaryText
                                                            anchors.centerIn: parent
                                                            font.weight: Font.Bold
                                                        }

                                                        MouseArea {
                                                            anchors.fill: parent
                                                            onClicked: set(EcSocket.state.fnKey === "Left" ? "Right" : "Left")
                                                        }
                                                    }

                                                    Item {
                                                        Layout.fillWidth: true
                                                    }
                                                }
                                            }

                                            // custom methods
                                            // toggles
                                            ToggleActionButton {
                                                visible: type === "method" && variation === "bit"

                                                iconName: "switch_access"
                                                checked: EcSocket.state.methods[methodKey] ?? false
                                                iconSize: Theme.iconSizeLarge
                                                buttonHeight: 70
                                                buttonWidth: 130

                                                onClicked: set(!checked)
                                            }

                                            // name / description
                                            RowLayout {
                                                id: rowLayout
                                                Layout.alignment: Qt.AlignTop | Qt.AlignHCenter
                                                spacing: Theme.spacingXS
                                                Layout.fillWidth: true

                                                DankIcon {
                                                    id: cardInfoIcon
                                                    visible: description != null && description.length > 0
                                                    name: "info"
                                                    size: Theme.iconSize - 4
                                                    color: Theme.primary

                                                    Layout.alignment: Qt.AlignTop | Qt.AlignRight

                                                    Tooltip {
                                                        id: cardTooltip
                                                    }

                                                    HoverHandler {
                                                        onHoveredChanged: {
                                                            const cb = side => {
                                                                let x = 0;
                                                                let y = 0;
                                                                switch (side) {
                                                                case "right":
                                                                    y = cardInfoIcon.height + 10;
                                                                    x = -cardInfoIcon.width;
                                                                    break;
                                                                case "left":
                                                                    y = cardInfoIcon.height + 10;
                                                                    x = cardInfoIcon.width;
                                                                    break;
                                                                case "top":
                                                                    y = cardInfoIcon.height * 2;
                                                                    break;
                                                                case "bottom":
                                                                    break;
                                                                }

                                                                return {
                                                                    "x": x,
                                                                    "y": y
                                                                };
                                                            };

                                                            if (hovered) {
                                                                cardTooltip.show(description, cardInfoIcon, cb);
                                                            } else {
                                                                cardTooltip.hide();
                                                            }
                                                        }
                                                    }
                                                }

                                                StyledText {
                                                    id: text
                                                    Layout.maximumWidth: actionBtn.width - (cardInfoIcon.visible ? cardInfoIcon.width + rowLayout.spacing : 0)

                                                    text: name
                                                    font.pixelSize: Theme.fontSizeSmall
                                                    font.weight: Font.Medium
                                                    color: Theme.surfaceText

                                                    horizontalAlignment: Text.AlignLeft
                                                    wrapMode: Text.WordWrap
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Performance
                        ColumnLayout {
                            id: page3

                            visible: popout.currentTab === 2
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            StyledRect {
                                Layout.fillWidth: true
                                Layout.fillHeight: true

                                radius: Theme.cornerRadius
                                color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)

                                GridLayout {
                                    columns: 4

                                    anchors.top: parent.top
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.margins: Theme.spacingM
                                    anchors.centerIn: parent

                                    rowSpacing: Theme.spacingM
                                    columnSpacing: Theme.spacingM

                                    Repeater {
                                        model: [
                                            {
                                                "name": "Turbo",
                                                "id": "Turbo",
                                                "icon": "rocket_launch",
                                                "supported": EcSocket.state.shiftModes.includes("Turbo"),
                                                "setMode": () => {
                                                    EcSocket.setShiftMode("Turbo");

                                                    if (EcSocket.state.superBatterySupported && EcSocket.state.superBattery) {
                                                        EcSocket.setSuperBattery(false);
                                                    }
                                                }
                                            },
                                            {
                                                "name": "Extreme Performance",
                                                "id": "Extreme Performance",
                                                "icon": "speed",
                                                "supported": EcSocket.state.shiftModes.includes("Extreme Performance"),
                                                "setMode": () => {
                                                    EcSocket.setShiftMode("Extreme Performance");

                                                    if (EcSocket.state.superBatterySupported && EcSocket.state.superBattery) {
                                                        EcSocket.setSuperBattery(false);
                                                    }
                                                }
                                            },
                                            {
                                                "name": "Balanced",
                                                "id": "Balanced",
                                                "icon": "balance",
                                                "supported": EcSocket.state.shiftModes.includes("Balanced"),
                                                "setMode": () => {
                                                    EcSocket.setShiftMode("Balanced");

                                                    if (EcSocket.state.superBatterySupported && EcSocket.state.superBattery) {
                                                        EcSocket.setSuperBattery(false);
                                                    }
                                                }
                                            },
                                            {
                                                "name": "Eco",
                                                "id": "Super Battery",
                                                "icon": "psychiatry",
                                                "supported": EcSocket.state.shiftModes.includes("Super Battery"),
                                                "setMode": () => EcSocket.setShiftMode("Super Battery")
                                            },
                                        ]

                                        ColumnLayout {
                                            id: page3Column

                                            visible: supported

                                            Layout.alignment: Qt.AlignTop | Qt.AlignHCenter

                                            required property string name
                                            required property string id
                                            required property string icon
                                            required property bool supported
                                            required property var setMode

                                            spacing: Theme.spacingXS

                                            // toggles
                                            ToggleActionButton {
                                                iconName: icon
                                                checked: EcSocket.state.shiftMode === id
                                                iconSize: Theme.iconSizeLarge + 16
                                                buttonHeight: 110
                                                buttonWidth: 140
                                                iconFilled: id === "Turbo" ? true : false

                                                onClicked: setMode()

                                                StyledText {
                                                    Layout.maximumWidth: parent.width

                                                    anchors.bottom: parent.bottom
                                                    anchors.horizontalCenter: parent.horizontalCenter
                                                    anchors.bottomMargin: Theme.spacingS

                                                    text: name
                                                    font.pixelSize: Theme.fontSizeSmall
                                                    font.weight: Font.Bold
                                                    color: parent.checked ? Theme.primaryText : Theme.surfaceText

                                                    horizontalAlignment: Text.AlignCenter
                                                    wrapMode: Text.WordWrap
                                                }
                                            }

                                            RowLayout {
                                                id: superBatteryRow
                                                visible: id === "Super Battery" && EcSocket.state.superBatterySupported
                                                spacing: 0

                                                StyledText {
                                                    Layout.leftMargin: Theme.spacingM
                                                    text: "Super Battery"
                                                    font.pixelSize: Theme.fontSizeSmall
                                                    font.weight: Font.Bold
                                                    color: Theme.surfaceText

                                                    wrapMode: Text.WordWrap
                                                }

                                                DankToggle {
                                                    id: toggleItem
                                                    enabled: EcSocket.state.shiftMode === id
                                                    description: "Eco"
                                                    checked: EcSocket.state.superBattery
                                                    onClicked: EcSocket.setSuperBattery(!checked)
                                                    Layout.leftMargin: -5
                                                    scale: 0.6
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

                                    text: EcSocket.state.ecDumpPretty

                                    Timer {
                                        id: memTimer
                                        interval: 1000
                                        repeat: true
                                        triggeredOnStart: true
                                        onTriggered: EcSocket.ecDumpPretty()
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
