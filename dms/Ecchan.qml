pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import qs.Common
import qs.Widgets
import qs.Modules.Plugins
import qs.Services

import "./EcchanClient"
import "./Widgets"

PluginComponent {
    id: root

    layerNamespacePlugin: "ecchan"

    onPluginDataChanged: {
        const socket = pluginData.socket;
        if (typeof (socket) === "string" && !EcchanClient.connected) {
            EcchanClient.path = socket;
            EcchanClient.connect();
        }
    }

    Component.onDestruction: {
        EcchanClient.disconnect();
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
            if (!EcchanClient.hasDgpu) {
                gpuUpdate.stop();
            }

            EcchanClient.updateGpuRtTemp();
        }
    }

    Timer {
        id: cpuUpdate
        interval: 1000
        repeat: true
        triggeredOnStart: true
        onTriggered: EcchanClient.updateCpuRtTemp()
    }

    Timer {
        id: fanUpdate
        interval: 1000
        repeat: true
        triggeredOnStart: true
        // qmlformat off
        onTriggered: {
            // qmllint disable unterminated-case
            switch (EcchanClient.fanCount) {
                case 4:
                    EcchanClient.updateFan4Rpm();
                case 3:
                    EcchanClient.updateFan3Rpm();
                case 2:
                    EcchanClient.updateFan2Rpm();
                case 1:
                default:
                    EcchanClient.updateFan1Rpm();
            }
        }
        // qmlformat on
    }

    property var profilesModel: []
    property int selectedProfile: 0
    property var profiles: []

    Connections {
        target: EcchanClient

        property bool blocked: true

        function onInitStateChanged(state) {
            const finished = !state;

            if (finished) {
                EcchanClient.apply(root.profiles[root.selectedProfile].state);
                EcchanClient.queue(() => {
                    blocked = false;
                    root.profiles[root.selectedProfile].state = EcchanClient.serialize();
                    profileWriteTimer.restart();
                });
            }
        }

        function onStateChanged(key) {
            if (blocked) {
                return;
            }

            if (!EcchanClient.profileProps.includes(key)) {
                return;
            }

            profileWriteTimer.restart();
        }
    }

    Timer {
        id: profileWriteTimer
        interval: 500
        repeat: false
        triggeredOnStart: false
        onTriggered: {
            const state = EcchanClient.serialize();
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
                "state": {}
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

                            onClicked: EcchanClient.connected ? EcchanClient.disconnect() : EcchanClient.connect()

                            DankIcon {
                                anchors.centerIn: parent
                                name: "circle"
                                filled: true
                                grade: 700
                                color: EcchanClient.connected ? Theme.primary : Theme.surfaceText
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

                                EcchanClient.apply(root.profiles[idx].state);
                            }

                            onValueAdded: (idx, name) => {
                                // explicit reassign so signals fire
                                root.profiles = [...root.profiles,
                                    {
                                        "name": name,
                                        "state": EcchanClient.serialize()
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

                                leftPadding: EcchanClient.hasDgpu && DgopService.dgopAvailable ? 0 : (width - 180) / 2

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

                                        value: DgopService.dgopAvailable ? (DgopService.cpuUsage / 100) : Math.min(1, EcchanClient.cpuRtTemp / 100)
                                        label: DgopService.dgopAvailable ? (DgopService.cpuUsage.toFixed(1) + "%") : (EcchanClient.cpuRtTemp + "°C")
                                        detail: DgopService.dgopAvailable ? (EcchanClient.cpuRtTemp > 0 ? (EcchanClient.cpuRtTemp + "°C") : "") : ""
                                        sublabel: "CPU"
                                        accentColor: {
                                            const dgop = DgopService.cpuUsage > 80 ? Theme.error : (DgopService.cpuUsage > 50 ? Theme.warning : Theme.primary);
                                            const cpu = EcchanClient.cpuRtTemp > 85 ? Theme.error : (EcchanClient.cpuRtTemp > 70 ? Theme.warning : Theme.primary);
                                            return DgopService.dgopAvailable ? dgop : cpu;
                                        }
                                        detailColor: EcchanClient.cpuRtTemp > 85 ? Theme.error : (EcchanClient.cpuRtTemp > 70 ? Theme.warning : Theme.surfaceVariantText)
                                    }
                                }

                                Item {
                                    id: gpuGauge

                                    implicitHeight: 180
                                    implicitWidth: 180

                                    visible: EcchanClient.hasDgpu

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

                                        value: Math.min(1, EcchanClient.gpuRtTemp / 100)
                                        label: EcchanClient.gpuRtTemp > 0 ? (EcchanClient.gpuRtTemp + "°C") : "--"
                                        sublabel: "GPU"
                                        accentColor: {
                                            const temp = EcchanClient.gpuRtTemp;
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
                                    height: EcchanClient.hasDgpu ? 180 * 2 : 180
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
                                        anchors.centerIn: EcchanClient.hasDgpu ? parent : undefined
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
                                                        "rpm": EcchanClient.fan1Rpm
                                                    },
                                                    {
                                                        "rpm": EcchanClient.fan2Rpm
                                                    },
                                                    {
                                                        "rpm": EcchanClient.fan3Rpm
                                                    },
                                                    {
                                                        "rpm": EcchanClient.fan4Rpm
                                                    }
                                                ]

                                                ColumnLayout {
                                                    id: fanRow
                                                    spacing: Theme.spacingL
                                                    visible: EcchanClient.fanCount > index

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

                                    property var methods: EcchanClient.methods.map(item => {
                                        // [{ value: <value>, "name": "name", "method": "method" }]
                                        let variation;
                                        // qmlformat off
                                        switch (typeof (item.value)) {
                                            case "boolean":
                                                variation = "bit";
                                                break;
                                            case "number":
                                                variation = "byte";
                                                break;
                                            case "object": // array
                                                variation = "range";
                                                break;
                                            default:
                                                ToastService.showError("Got invalid value", "EcchanClient.methods returned wrong type ?? " + typeof (item.value));
                                                break;
                                        }
                                        // qmlformat on

                                        return {
                                            "name": item.name,
                                            "icon": null,
                                            "description": null,
                                            "supported": true,
                                            "value": item.value,
                                            "set": value => EcchanClient.methodWrite(item.method, value),
                                            "type": "method",
                                            "variation": variation,
                                            "methodKey": item.method
                                        };
                                    }).filter(item => item != null)

                                    property var modelBase: [
                                        {
                                            "name": "Webcam",
                                            "icon": "camera_video",
                                            "description": "Enable the integrated webcam (as if by a keyboard button)",
                                            "supported": EcchanClient.webcamSupported,
                                            "value": EcchanClient.webcam,
                                            "set": state => EcchanClient.webcam = state,
                                            "type": "toggle",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        {
                                            "name": "Webcam Block",
                                            "icon": "camera_video",
                                            "description": "Block the integrated webcam (can't be enabled by a keyboard button)",
                                            "supported": EcchanClient.webcamBlockSupported,
                                            "value": EcchanClient.webcamBlock,
                                            "set": state => EcchanClient.webcamBlock = state,
                                            "type": "toggle",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        {
                                            "name": "Swap Win/Fn",
                                            "icon": null,
                                            "description": "Swap the Fn / Windows key positions",
                                            "supported": EcchanClient.fnWinSwapSupported,
                                            "value": EcchanClient.fnKey,
                                            "set": state => EcchanClient.fnKey = state,
                                            "type": "swapKey",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        {
                                            "name": "Mic Mute Light",
                                            "icon": EcchanClient.micMuteLed ? "backlight_high" : "backlight_high_off",
                                            "description": "Toggle the mic mute keyboard indicator light",
                                            "supported": EcchanClient.micMuteLedSupported,
                                            "value": EcchanClient.micMuteLed,
                                            "set": state => EcchanClient.micMuteLed = state,
                                            "type": "toggle",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        {
                                            "name": "Mute Light",
                                            "icon": EcchanClient.muteLed ? "backlight_high" : "backlight_high_off",
                                            "description": "Toggle the audio mute keyboard indicator light",
                                            "supported": EcchanClient.muteLedSupported,
                                            "value": EcchanClient.muteLed,
                                            "set": state => EcchanClient.muteLed = state,
                                            "type": "toggle",
                                            "variation": null,
                                            "methodKey": null
                                        },
                                        // qmlformat off
                                        ...methods
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
                                                            text: EcchanClient.winKey === "Left" ? "Win" : "Fn"
                                                            color: Theme.primaryText
                                                            anchors.centerIn: parent
                                                            font.weight: Font.Bold
                                                        }

                                                        MouseArea {
                                                            anchors.fill: parent
                                                            onClicked: set(EcchanClient.fnKey === "Left" ? "Right" : "Left")
                                                        }
                                                    }

                                                    Rectangle {
                                                        implicitHeight: 25
                                                        implicitWidth: 50
                                                        radius: height / 2
                                                        color: Theme.primary

                                                        StyledText {
                                                            text: EcchanClient.fnKey === "Right" ? "Fn" : "Win"
                                                            color: Theme.primaryText
                                                            anchors.centerIn: parent
                                                            font.weight: Font.Bold
                                                        }

                                                        MouseArea {
                                                            anchors.fill: parent
                                                            onClicked: set(EcchanClient.fnKey === "Left" ? "Right" : "Left")
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
                                                checked: value
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
                                                "supported": EcchanClient.shiftModes.includes("Turbo"),
                                                "setMode": () => {
                                                    EcchanClient.shiftMode = "Turbo";

                                                    if (EcchanClient.superBatterySupported && EcchanClient.superBattery) {
                                                        EcchanClient.superBattery = false;
                                                    }
                                                }
                                            },
                                            {
                                                "name": "Extreme Performance",
                                                "id": "Extreme Performance",
                                                "icon": "speed",
                                                "supported": EcchanClient.shiftModes.includes("Extreme Performance"),
                                                "setMode": () => {
                                                    EcchanClient.shiftMode = "Extreme Performance";

                                                    if (EcchanClient.superBatterySupported && EcchanClient.superBattery) {
                                                        EcchanClient.superBattery = false;
                                                    }
                                                }
                                            },
                                            {
                                                "name": "Balanced",
                                                "id": "Balanced",
                                                "icon": "balance",
                                                "supported": EcchanClient.shiftModes.includes("Balanced"),
                                                "setMode": () => {
                                                    EcchanClient.shiftMode = "Balanced";

                                                    if (EcchanClient.superBatterySupported && EcchanClient.superBattery) {
                                                        EcchanClient.superBattery = false;
                                                    }
                                                }
                                            },
                                            {
                                                "name": "Eco",
                                                "id": "Super Battery",
                                                "icon": "psychiatry",
                                                "supported": EcchanClient.shiftModes.includes("Super Battery"),
                                                "setMode": () => EcchanClient.shiftMode = "Super Battery"
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
                                                checked: EcchanClient.shiftMode === id
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
                                                visible: id === "Super Battery" && EcchanClient.superBatterySupported
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
                                                    enabled: EcchanClient.shiftMode === id
                                                    description: "Eco"
                                                    checked: EcchanClient.superBattery
                                                    onClicked: EcchanClient.superBattery = !checked
                                                    Layout.leftMargin: -5
                                                    scale: 0.6
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Fans
                        ColumnLayout {
                            id: page4

                            property int fanIndex: 0

                            visible: popout.currentTab === 3
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            StyledRect {
                                Layout.fillWidth: true
                                Layout.fillHeight: true

                                radius: Theme.cornerRadius
                                color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)

                                ColumnLayout {
                                    anchors.fill: parent
                                    anchors.margins: Theme.spacingM
                                    spacing: Theme.spacingM

                                    GridLayout {
                                        columns: 4

                                        Layout.fillWidth: true
                                        Layout.alignment: Qt.AlignTop | Qt.AlignHCenter

                                        rowSpacing: Theme.spacingM
                                        columnSpacing: Theme.spacingM

                                        Repeater {
                                            model: [
                                                {
                                                    "name": "Auto",
                                                    "icon": "mode_fan",
                                                    "selected": EcchanClient.fanMode === "Auto",
                                                    "supported": EcchanClient.fanModes.includes("Auto"),
                                                    "setMode": () => EcchanClient.fanMode = "Auto"
                                                },
                                                {
                                                    "name": "Advanced",
                                                    "icon": "tune",
                                                    "selected": EcchanClient.fanMode === "Advanced",
                                                    "supported": EcchanClient.fanModes.includes("Advanced"),
                                                    "setMode": () => EcchanClient.fanMode = "Advanced"
                                                },
                                                {
                                                    "name": "Silent",
                                                    "icon": "airwave",
                                                    "selected": EcchanClient.fanMode === "Silent",
                                                    "supported": EcchanClient.fanModes.includes("Silent"),
                                                    "setMode": () => EcchanClient.fanMode = "Silent"
                                                },
                                                {
                                                    "name": "Cooler Boost",
                                                    "icon": EcchanClient.coolerBoost ? "mode_cool" : "mode_cool_off",
                                                    "selected": EcchanClient.coolerBoost,
                                                    "supported": EcchanClient.coolerBoostSupported,
                                                    "setMode": () => EcchanClient.coolerBoost = !EcchanClient.coolerBoost
                                                },
                                            ].filter(item => item.supported)

                                            ColumnLayout {
                                                id: page4Column
                                                Layout.preferredHeight: page4Column.implicitHeight + Theme.spacingL
                                                Layout.alignment: Qt.AlignTop | Qt.AlignHCenter

                                                required property string name
                                                required property string icon
                                                required property bool selected
                                                required property bool supported
                                                required property var setMode

                                                spacing: Theme.spacingS

                                                // toggles
                                                ToggleActionButton {
                                                    id: actionBtn3

                                                    iconName: icon
                                                    checked: selected
                                                    iconSize: Theme.iconSizeLarge + 16
                                                    buttonHeight: 100
                                                    buttonWidth: 140

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
                                            }
                                        }
                                    }

                                    DankTabBar {
                                        id: fanTab

                                        Layout.alignment: Qt.AlignTop
                                        Layout.fillWidth: true

                                        currentIndex: page4.fanIndex

                                        property var item: model[page4.fanIndex]

                                        model: [
                                            {
                                                "isAction": false,
                                                "supported": EcchanClient.wmiVer == 2,
                                                "icon": "memory",
                                                "text": "CFan",
                                                "sliders": 7,
                                                "values": EcchanClient.cpuFanCurveWmi2
                                            },
                                            {
                                                "isAction": false,
                                                "supported": EcchanClient.wmiVer == 2,
                                                "icon": "memory",
                                                "text": "CTemp",
                                                "sliders": 7,
                                                "values": EcchanClient.cpuTempCurveWmi2
                                            },
                                            {
                                                "isAction": false,
                                                "supported": EcchanClient.wmiVer == 2,
                                                "icon": "memory",
                                                "text": "CHysteresis",
                                                "sliders": 6,
                                                "values": EcchanClient.cpuHysteresisCurveWmi2
                                            },
                                            {
                                                "isAction": false,
                                                "supported": EcchanClient.wmiVer == 2 && EcchanClient.hasDgpu,
                                                "icon": "developer_board",
                                                "text": "GFan",
                                                "sliders": 7,
                                                "values": EcchanClient.gpuFanCurveWmi2
                                            },
                                            {
                                                "isAction": false,
                                                "supported": EcchanClient.wmiVer == 2 && EcchanClient.hasDgpu,
                                                "icon": "developer_board",
                                                "text": "GTemp",
                                                "sliders": 7,
                                                "values": EcchanClient.gpuTempCurveWmi2
                                            },
                                            {
                                                "isAction": false,
                                                "supported": EcchanClient.wmiVer == 2 && EcchanClient.hasDgpu,
                                                "icon": "developer_board",
                                                "text": "GHysteresis",
                                                "sliders": 7,
                                                "values": EcchanClient.gpuHysteresisCurveWmi2
                                            },
                                        ].filter(item => item.supported)

                                        onTabClicked: index => page4.fanIndex = index
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

                                    text: EcchanClient.ecDumpPretty

                                    Timer {
                                        id: memTimer
                                        interval: 1000
                                        repeat: true
                                        triggeredOnStart: true
                                        onTriggered: EcchanClient.updateEcDumpPretty()
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
