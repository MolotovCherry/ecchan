pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import qs.Common
import qs.Widgets
import qs.Modules.Plugins
import qs.Services

import "./Services"
import "./Widgets"
import "./Helpers"

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

    property var profilesModel: []
    property int selectedProfile: 0
    property var profiles: []
    property var defaults: ({})
    property var profile: profiles[selectedProfile]
    property var firstInit: true

    Connections {
        target: EcchanClient

        property bool blocked: true

        function onInitStateChanged(state) {
            const finished = !state;

            if (finished) {
                // if they don't exist, we want to save the default fan curves on
                // the very very first startup so we can revert
                // this must be done before the first apply is done
                //
                // https://stackoverflow.com/a/32108184/9423933
                root.saveDefaults();

                EcchanClient.apply(root.profile.state);
                EcchanClient.queue(() => {
                    blocked = false;
                    root.profile.state = EcchanClient.serialize();
                    profileWriteTimer.restart();

                    if (root.firstInit) {
                        // should be started after we have the data
                        Update.addRef(["cpuRtTemp"]);

                        if (EcchanClient.hasDgpu) {
                            Update.addRef(["gpuRtTemp"]);
                        }

                        root.firstInit = false;
                    }
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

    function saveDefaults() {
        let isEmpty = true;
        for (var prop in defaults) {
            if (Object.prototype.hasOwnProperty.call(defaults, prop)) {
                isEmpty = false;
                break;
            }
        }

        if (isEmpty) {
            defaults = {
                cpuFanCurveWmi2: EcchanClient.cpuFanCurveWmi2,
                cpuTempCurveWmi2: EcchanClient.cpuTempCurveWmi2,
                cpuHysteresisCurveWmi2: EcchanClient.cpuHysteresisCurveWmi2,
                gpuFanCurveWmi2: EcchanClient.gpuFanCurveWmi2,
                gpuTempCurveWmi2: EcchanClient.gpuTempCurveWmi2,
                gpuHysteresisCurveWmi2: EcchanClient.gpuHysteresisCurveWmi2
            };
        }
    }

    Timer {
        id: profileWriteTimer
        interval: 500
        repeat: false
        triggeredOnStart: false
        onTriggered: {
            const state = EcchanClient.serialize();

            // we want to always keep our custom curves rather than overwriting them when serializing
            const curveKeys = [
                "cpuFanCurveWmi2", "cpuTempCurveWmi2", "cpuHysteresisCurveWmi2",
                "gpuFanCurveWmi2", "gpuTempCurveWmi2", "gpuHysteresisCurveWmi2"
            ];
            curveKeys.forEach(key => {
                if (root.profile.state && root.profile.state[key] != null) {
                    state[key] = root.profile.state[key];
                }
            });

            root.profile.state = state;
            root.profilesChanged();
        }
    }

    onPluginServiceChanged: {
        if (!pluginService) {
            return;
        }

        selectedProfile = _loadPluginData("selectedProfile", 0);
        selectedProfileChanged();

        defaults = _loadPluginData("defaults", {});
        defaultsChanged();

        profiles = _loadPluginData("profiles", [
            {
                name: "Default",
                state: {}
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

    onDefaultsChanged: {
        if (root.pluginService) {
            _savePluginData("defaults", defaults);
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

    horizontalBarPill: Component {
        Row {
            spacing: Theme.spacingS

            // cpu temp
            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXS

                DankIcon {
                    name: "memory"
                    size: root.iconSize
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }

                Item {
                    width: 20
                    height: parent.height
                    anchors.verticalCenter: parent.verticalCenter

                    StyledText {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: EcchanClient.cpuRtTemp + "°"
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }

            // gpu temp
            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXS

                DankIcon {
                    visible: EcchanClient.hasDgpu
                    name: "developer_board"
                    size: root.iconSize
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }

                Item {
                    width: 20
                    height: parent.height
                    anchors.verticalCenter: parent.verticalCenter

                    StyledText {
                        visible: EcchanClient.hasDgpu
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: EcchanClient.gpuRtTemp + "°"
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }

            // fan speed
            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXS

                DankIcon {
                    name: "mode_fan"
                    size: root.iconSize
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }

                Item {
                    width: 30
                    height: parent.height
                    anchors.verticalCenter: parent.verticalCenter

                    FanCalc {
                        id: fanCalc
                    }

                    StyledText {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.horizontalCenter: parent.horizontalCenter
                        font.pixelSize: Theme.fontSizeSmall
                        text: fanCalc.text
                        color: {
                            if (fanCalc.level <= 3) {
                                return Theme.surfaceText;
                            }

                            if (fanCalc.level === 4) {
                                return Theme.warning;
                            }

                            if (fanCalc.level === 5) {
                                return Theme.error;
                            }
                        }
                    }
                }
            }

            Rectangle {
                Layout.alignment: Qt.AlignHCenter
                implicitWidth: 1.1
                implicitHeight: root.iconSize
                color: Theme.outline
                opacity: 0.3
            }

            // selected profile
            StyledText {
                anchors.verticalCenter: parent.verticalCenter
                text: root.profile?.name ?? "Default"
                font.pixelSize: Theme.fontSizeSmall
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXS

                // shift mode
                DankIcon {
                    visible: EcchanClient.shiftModeSupported
                    name: {
                        switch (EcchanClient.shiftMode) {
                            // qmlformat off
                            case "Turbo":
                                return "rocket_launch";
                            case "Extreme Performance":
                                return "speed";
                            case "Balanced":
                                return "balance";
                            case "Super Battery":
                                return "psychiatry";
                            default:
                                return "";
                            // qmlformat on
                        }
                    }
                    size: root.iconSize
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }

                // super battery
                DankIcon {
                    visible: EcchanClient.superBattery
                    name: "battery_android_bolt"
                    size: root.iconSize
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }

                // fan mode
                DankIcon {
                    visible: EcchanClient.fanModeSupported
                    name: {
                        switch (EcchanClient.fanMode) {
                            // qmlformat off
                            case "Auto":
                                return "mode_fan";
                            case "Advanced":
                                return "tune";
                            case "Silent":
                                return "airwave";
                            default:
                                return "";
                            // qmlformat on
                        }
                    }
                    size: root.iconSize
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }

                // cooler boost
                DankIcon {
                    visible: EcchanClient.coolerBoostSupported
                    name: EcchanClient.coolerBoost ? "mode_cool" : "mode_cool_off"
                    size: root.iconSize
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }
    }

    verticalBarPill: Component {
        Column {
            spacing: Theme.spacingXS

            DankIcon {
                name: "memory"
                size: root.iconSize
                color: Theme.surfaceText
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }
    }

    // --

    signal areaClicked
    property int currentTab: 0

    popoutContent: Component {
        PopoutComponent {
            id: popout

            FocusScope {
                width: parent.width
                implicitHeight: root.popoutHeight - popout.headerHeight - popout.detailsHeight - Theme.spacingXL

                focus: true

                TapHandler {
                    target: null
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onTapped: (eventPoint, button) => root.areaClicked()
                }

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
                                    root.currentTab = 99;
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

                        DankIcon {
                            Layout.preferredWidth: 20
                            Layout.preferredHeight: 20

                            name: "circle"
                            filled: true
                            grade: 700
                            color: EcchanClient.connected ? Theme.primary : Theme.surfaceText
                            size: Theme.iconSize - 6
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

                                if (isSame) {
                                    root.selectedProfile = idx;
                                    return;
                                }

                                // clone it so we don't alter the real copy
                                const state = Object.assign({}, root.profiles[idx].state);

                                // when switching profiles DO NOT set custom curves UNLESS
                                // fanMode is at advanced
                                if (state.fanMode !== "Advanced") {
                                    const curveKeys = [
                                        "cpuFanCurveWmi2", "cpuTempCurveWmi2", "cpuHysteresisCurveWmi2",
                                        "gpuFanCurveWmi2", "gpuTempCurveWmi2", "gpuHysteresisCurveWmi2"
                                    ];
                                    curveKeys.forEach(key => {
                                        state[key] = root.defaults[key];
                                    });
                                }

                                EcchanClient.apply(state);

                                EcchanClient.queue(() => {
                                    // only visibly apply profile once api calls have finished
                                    root.selectedProfile = idx;
                                });
                            }

                            onValueAdded: (idx, name) => {
                                // explicit reassign so signals fire
                                const clone = Object.assign({}, root.profile);
                                clone.name = name;

                                root.profiles = [...root.profiles, clone];

                                root.selectedProfile = idx;
                            }

                            onValueEdited: (idx, name) => {
                                root.profiles[idx].name = name;
                                root.profilesChanged();
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
                                    icon: "analytics",
                                    supported: true
                                },
                                {
                                    text: "General",
                                    icon: "list_alt",
                                    supported: true
                                },
                                {
                                    text: "Performance",
                                    icon: "speed",
                                    supported: EcchanClient.shiftModeSupported
                                },
                                {
                                    text: "Fans",
                                    icon: "mode_fan",
                                    supported: EcchanClient.fanModeSupported || EcchanClient.wmiVer === 2
                                },
                                {
                                    text: "Battery",
                                    icon: "battery_android_full",
                                    supported: EcchanClient.batteryChargeModeSupported
                                }
                            ].filter(item => item.supported)

                            Row {
                                id: row

                                required property int index
                                required property string text
                                required property string icon

                                Rectangle {
                                    width: tabRowContent.implicitWidth + Theme.spacingS * 2
                                    height: Math.round(Theme.fontSizeSmall * 3.1)
                                    radius: Theme.cornerRadius
                                    color: root.currentTab === row.index ? Theme.primaryPressed : (tabMouseArea.containsMouse ? Theme.primaryHoverLight : "transparent")
                                    border.color: root.currentTab === row.index ? Theme.primary : "transparent"
                                    border.width: root.currentTab === row.index ? 1 : 0

                                    Row {
                                        id: tabRowContent
                                        anchors.centerIn: parent
                                        spacing: Theme.spacingXS

                                        DankIcon {
                                            name: row.icon
                                            size: Theme.iconSize - 2
                                            color: root.currentTab === row.index ? Theme.primary : Theme.surfaceText
                                            opacity: root.currentTab === row.index ? 1 : 0.7
                                            anchors.verticalCenter: parent.verticalCenter
                                        }

                                        StyledText {
                                            text: row.text
                                            font.pixelSize: Theme.fontSizeMedium
                                            font.weight: Font.Medium
                                            color: root.currentTab === row.index ? Theme.primary : Theme.surfaceText
                                            anchors.verticalCenter: parent.verticalCenter
                                        }
                                    }

                                    MouseArea {
                                        id: tabMouseArea
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: root.currentTab = row.index
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

                            visible: root.currentTab === 0
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
                                                Update.addRef(["cpuRtTemp"]);
                                                DgopService.addRef(["cpu"]);
                                            } else {
                                                Update.removeRef(["cpuRtTemp"]);
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
                                            if (page1.visible && EcchanClient.hasDgpu) {
                                                Update.addRef(["gpuRtTemp"]);
                                            } else {
                                                Update.removeRef(["gpuRtTemp"]);
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
                                                let fans = [];
                                                switch (EcchanClient.fanCount) {
                                                    case 4:
                                                        fans.push("fan4Rpm");
                                                        // fallthrough
                                                    case 3:
                                                        fans.push("fan3Rpm");
                                                        // fallthrough
                                                    case 2:
                                                        fans.push("fan2Rpm");
                                                        // fallthrough
                                                    case 1:
                                                        fans.push("fan1Rpm");
                                                }

                                                Update.addRef(fans);
                                            } else {
                                                Update.removeRef(["fan1Rpm", "fan2Rpm", "fan3Rpm", "fan4Rpm"]);
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
                                                        rpm: EcchanClient.fan1Rpm
                                                    },
                                                    {
                                                        rpm: EcchanClient.fan2Rpm
                                                    },
                                                    {
                                                        rpm: EcchanClient.fan3Rpm
                                                    },
                                                    {
                                                        rpm: EcchanClient.fan4Rpm
                                                    }
                                                ]

                                                ColumnLayout {
                                                    id: fanRow0
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
                                                            text: "Fan " + (fanRow0.index + 1)
                                                            font.pixelSize: Theme.fontSizeLarge
                                                            font.weight: Font.Medium
                                                            color: Theme.surfaceText
                                                        }

                                                        StyledText {
                                                            Layout.fillWidth: true
                                                            horizontalAlignment: Text.AlignRight
                                                            text: fanRow0.rpm + " rpm"
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

                            visible: root.currentTab === 1
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
                                                ToastService.showError("Got invalid value", "EcchanClient.methods returned wrong type: " + typeof (item.value));
                                                break;
                                        }
                                        // qmlformat on

                                        return {
                                            name: item.name,
                                            icon: null,
                                            description: "Custom model specific feature",
                                            supported: true,
                                            value: item.value,
                                            set: value => EcchanClient.methodWrite(item.method, value),
                                            type: "method",
                                            variation: variation,
                                            methodKey: item.method
                                        };
                                    })

                                    property var modelBase: [
                                        {
                                            name: "Webcam",
                                            icon: "camera_video",
                                            description: "Enable the integrated webcam (as if by a keyboard button)",
                                            supported: EcchanClient.webcamSupported,
                                            value: EcchanClient.webcam,
                                            set: state => EcchanClient.webcam = state,
                                            type: "toggle",
                                            variation: null,
                                            methodKey: null
                                        },
                                        {
                                            name: "Webcam Block",
                                            icon: "camera_video",
                                            description: "Block the integrated webcam (can't be enabled by a keyboard button)",
                                            supported: EcchanClient.webcamBlockSupported,
                                            value: EcchanClient.webcamBlock,
                                            set: state => EcchanClient.webcamBlock = state,
                                            type: "toggle",
                                            variation: null,
                                            methodKey: null
                                        },
                                        {
                                            name: "Swap Win/Fn",
                                            icon: null,
                                            description: "Swap the Fn / Windows key positions",
                                            supported: EcchanClient.fnWinSwapSupported,
                                            value: EcchanClient.fnKey,
                                            set: state => EcchanClient.fnKey = state,
                                            type: "swapKey",
                                            variation: null,
                                            methodKey: null
                                        },
                                        {
                                            name: "Mic Mute Light",
                                            icon: EcchanClient.micMuteLed ? "backlight_high" : "backlight_high_off",
                                            description: "Toggle the mic mute keyboard indicator light",
                                            supported: EcchanClient.micMuteLedSupported,
                                            value: EcchanClient.micMuteLed,
                                            set: state => EcchanClient.micMuteLed = state,
                                            type: "toggle",
                                            variation: null,
                                            methodKey: null
                                        },
                                        {
                                            name: "Mute Light",
                                            icon: EcchanClient.muteLed ? "backlight_high" : "backlight_high_off",
                                            description: "Toggle the audio mute keyboard indicator light",
                                            supported: EcchanClient.muteLedSupported,
                                            value: EcchanClient.muteLed,
                                            set: state => EcchanClient.muteLed = state,
                                            type: "toggle",
                                            variation: null,
                                            methodKey: null
                                        },
                                        // qmlformat off
                                        ...methods
                                        // qmlformat on
                                    ].filter(item => item.supported)

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
                                                                    x: x,
                                                                    y: y
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

                            visible: root.currentTab === 2
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
                                                name: "Turbo",
                                                id: "Turbo",
                                                icon: "rocket_launch",
                                                supported: EcchanClient.shiftModes.includes("Turbo"),
                                                setMode: () => {
                                                    if (EcchanClient.superBatterySupported && EcchanClient.superBattery) {
                                                        EcchanClient.superBattery = false;
                                                    }

                                                    EcchanClient.shiftMode = "Turbo";
                                                }
                                            },
                                            {
                                                name: "Extreme Performance",
                                                id: "Extreme Performance",
                                                icon: "speed",
                                                supported: EcchanClient.shiftModes.includes("Extreme Performance"),
                                                setMode: () => {
                                                    if (EcchanClient.superBatterySupported && EcchanClient.superBattery) {
                                                        EcchanClient.superBattery = false;
                                                    }

                                                    EcchanClient.shiftMode = "Extreme Performance";
                                                }
                                            },
                                            {
                                                name: "Balanced",
                                                id: "Balanced",
                                                icon: "balance",
                                                supported: EcchanClient.shiftModes.includes("Balanced"),
                                                setMode: () => {
                                                    if (EcchanClient.superBatterySupported && EcchanClient.superBattery) {
                                                        EcchanClient.superBattery = false;
                                                    }

                                                    EcchanClient.shiftMode = "Balanced";
                                                }
                                            },
                                            {
                                                name: "Eco",
                                                id: "Super Battery",
                                                icon: "psychiatry",
                                                supported: EcchanClient.shiftModes.includes("Super Battery"),
                                                setMode: () => EcchanClient.shiftMode = "Super Battery"
                                            },
                                        ].filter(item => item.supported)

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

                            visible: root.currentTab === 3
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

                                        visible: EcchanClient.fanModeSupported

                                        Layout.fillWidth: true
                                        Layout.alignment: Qt.AlignTop | Qt.AlignHCenter

                                        rowSpacing: Theme.spacingM
                                        columnSpacing: Theme.spacingM

                                        Repeater {
                                            model: [
                                                {
                                                    name: "Auto",
                                                    icon: "mode_fan",
                                                    selected: EcchanClient.fanMode === "Auto",
                                                    color: Theme.primary,
                                                    supported: EcchanClient.fanModes.includes("Auto"),
                                                    setMode: () => {
                                                        EcchanClient.fanMode = "Auto"
                                                        EcchanClient.cpuFanCurveWmi2 = root.defaults.cpuFanCurveWmi2;
                                                        EcchanClient.cpuTempCurveWmi2 = root.defaults.cpuTempCurveWmi2;
                                                        EcchanClient.cpuHysteresisCurveWmi2 = root.defaults.cpuHysteresisCurveWmi2;
                                                        EcchanClient.gpuFanCurveWmi2 = root.defaults.gpuFanCurveWmi2;
                                                        EcchanClient.gpuTempCurveWmi2 = root.defaults.gpuTempCurveWmi2;
                                                        EcchanClient.gpuHysteresisCurveWmi2 = root.defaults.gpuHysteresisCurveWmi2;
                                                    }
                                                },
                                                {
                                                    name: "Advanced",
                                                    icon: "tune",
                                                    selected: EcchanClient.fanMode === "Advanced",
                                                    color: Theme.primary,
                                                    supported: EcchanClient.fanModes.includes("Advanced"),
                                                    setMode: () => {
                                                        EcchanClient.fanMode = "Advanced";
                                                        EcchanClient.cpuFanCurveWmi2 = root.profile.state.cpuFanCurveWmi2;
                                                        EcchanClient.cpuTempCurveWmi2 = root.profile.state.cpuTempCurveWmi2;
                                                        EcchanClient.cpuHysteresisCurveWmi2 = root.profile.state.cpuHysteresisCurveWmi2;
                                                        EcchanClient.gpuFanCurveWmi2 = root.profile.state.gpuFanCurveWmi2;
                                                        EcchanClient.gpuTempCurveWmi2 = root.profile.state.gpuTempCurveWmi2;
                                                        EcchanClient.gpuHysteresisCurveWmi2 = root.profile.state.gpuHysteresisCurveWmi2;
                                                    }
                                                },
                                                {
                                                    name: "Silent",
                                                    icon: "airwave",
                                                    selected: EcchanClient.fanMode === "Silent",
                                                    color: Theme.primary,
                                                    supported: EcchanClient.fanModes.includes("Silent"),
                                                    setMode: () => {
                                                        EcchanClient.fanMode = "Silent";
                                                        EcchanClient.cpuFanCurveWmi2 = root.defaults.cpuFanCurveWmi2;
                                                        EcchanClient.cpuTempCurveWmi2 = root.defaults.cpuTempCurveWmi2;
                                                        EcchanClient.cpuHysteresisCurveWmi2 = root.defaults.cpuHysteresisCurveWmi2;
                                                        EcchanClient.gpuFanCurveWmi2 = root.defaults.gpuFanCurveWmi2;
                                                        EcchanClient.gpuTempCurveWmi2 = root.defaults.gpuTempCurveWmi2;
                                                        EcchanClient.gpuHysteresisCurveWmi2 = root.defaults.gpuHysteresisCurveWmi2;
                                                    }
                                                },
                                                {
                                                    name: "Cooler Boost",
                                                    icon: EcchanClient.coolerBoost ? "mode_cool" : "mode_cool_off",
                                                    selected: EcchanClient.coolerBoost,
                                                    color: Theme.secondary,
                                                    supported: EcchanClient.coolerBoostSupported,
                                                    setMode: () => EcchanClient.coolerBoost = !EcchanClient.coolerBoost
                                                },
                                            ].filter(item => item.supported)

                                            ColumnLayout {
                                                id: page4Column
                                                Layout.preferredHeight: page4Column.implicitHeight + Theme.spacingL
                                                Layout.alignment: Qt.AlignTop | Qt.AlignHCenter

                                                required property string name
                                                required property string icon
                                                required property color color
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
                                                    iconColor: checked ? Theme.primaryText : color
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

                                    DankColorfulTabBar {
                                        id: fanTab

                                        visible: EcchanClient.wmiVer === 2

                                        Layout.alignment: Qt.AlignTop
                                        Layout.fillWidth: true

                                        tabColor: item.color

                                        onTabClicked: index => {
                                            currentIndex = index;
                                            setNodes(getCurves());
                                        }

                                        function getCurves() {
                                            return root.profile.state[item.key];
                                        }

                                        readonly property var item: model[currentIndex] || modelConfig[0]
                                        property int n1: 0
                                        property int n2: 0
                                        property int n3: 0
                                        property int n4: 0
                                        property int n5: 0
                                        property int n6: 0
                                        property int n7: 0

                                        function setNodes(arr) {
                                            n1 = arr[0];
                                            n2 = arr[1];
                                            n3 = arr[2];
                                            n4 = arr[3];
                                            n5 = arr[4];
                                            n6 = arr[5];
                                            n7 = arr[6] || 0;
                                        }

                                        function getNodes() {
                                            return [n1, n2, n3, n4, n5, n6, n7];
                                        }

                                        Connections {
                                            target: root

                                            property bool init: true

                                            function onSelectedProfileChanged() {
                                                fanTab.setNodes(fanTab.getCurves());
                                            }

                                            function onProfilesChanged() {
                                                if (init) {
                                                    init = false;
                                                    fanTab.setNodes(fanTab.getCurves());
                                                }
                                            }
                                        }

                                        property var modelConfig: [
                                            {
                                                isAction: false,
                                                supported: EcchanClient.wmiVer == 2 && EcchanClient.fanMax > 0,
                                                icon: "memory",
                                                text: "Fan",
                                                key: "cpuFanCurveWmi2",
                                                unit: "%",
                                                min: 0,
                                                max: EcchanClient.fanMax,
                                                color: Theme.primary,
                                                sliders: 7,
                                                set: () => {
                                                    root.profile.state.cpuFanCurveWmi2 = fanTab.getNodes();
                                                    root.profilesChanged();

                                                    if (EcchanClient.fanMode === "Advanced") {
                                                        EcchanClient.cpuFanCurveWmi2 = fanTab.getNodes();
                                                    }
                                                },
                                                reset: () => {
                                                    const defaults = root.defaults.cpuFanCurveWmi2;
                                                    fanTab.setNodes(defaults);
                                                    EcchanClient.cpuFanCurveWmi2 = defaults;
                                                    root.profile.state.cpuFanCurveWmi2 = defaults;
                                                    root.profilesChanged();
                                                }
                                            },
                                            {
                                                isAction: false,
                                                supported: EcchanClient.wmiVer == 2,
                                                icon: "memory",
                                                text: "Temp",
                                                key: "cpuTempCurveWmi2",
                                                unit: "°C",
                                                min: 0,
                                                max: 100,
                                                color: Theme.primary,
                                                sliders: 7,
                                                set: () => {
                                                    root.profile.state.cpuTempCurveWmi2 = fanTab.getNodes();
                                                    root.profilesChanged();

                                                    if (EcchanClient.fanMode === "Advanced") {
                                                        EcchanClient.cpuTempCurveWmi2 = fanTab.getNodes();
                                                    }
                                                },
                                                reset: () => {
                                                    const defaults = root.defaults.cpuTempCurveWmi2;
                                                    fanTab.setNodes(defaults);
                                                    EcchanClient.cpuTempCurveWmi2 = defaults;
                                                    root.profile.state.cpuTempCurveWmi2 = defaults;
                                                    root.profilesChanged();
                                                }
                                            },
                                            {
                                                isAction: false,
                                                supported: EcchanClient.wmiVer == 2,
                                                icon: "memory",
                                                text: "Hysteresis",
                                                key: "cpuHysteresisCurveWmi2",
                                                unit: "°C",
                                                min: 0,
                                                max: 10,
                                                color: Theme.primary,
                                                sliders: 6,
                                                set: () => {
                                                    root.profile.state.cpuHysteresisCurveWmi2 = fanTab.getNodes();
                                                    root.profilesChanged();

                                                    if (EcchanClient.fanMode === "Advanced") {
                                                        EcchanClient.cpuHysteresisCurveWmi2 = fanTab.getNodes();
                                                    }
                                                },
                                                reset: () => {
                                                    const defaults = root.defaults.cpuHysteresisCurveWmi2;
                                                    fanTab.setNodes(defaults);
                                                    EcchanClient.cpuHysteresisCurveWmi2 = defaults;
                                                    root.profile.state.cpuHysteresisCurveWmi2 = defaults;
                                                    root.profilesChanged();
                                                }
                                            },
                                            {
                                                isAction: false,
                                                supported: EcchanClient.wmiVer == 2 && EcchanClient.hasDgpu && EcchanClient.fanMax > 0,
                                                icon: "developer_board",
                                                text: "DFan",
                                                key: "gpuFanCurveWmi2",
                                                unit: "%",
                                                min: 0,
                                                max: EcchanClient.fanMax,
                                                color: Theme.secondary,
                                                sliders: 7,
                                                set: () => {
                                                    root.profile.state.gpuFanCurveWmi2 = fanTab.getNodes();
                                                    root.profilesChanged();

                                                    if (EcchanClient.fanMode === "Advanced") {
                                                        EcchanClient.gpuFanCurveWmi2 = fanTab.getNodes();
                                                    }
                                                },
                                                reset: () => {
                                                    const defaults = root.defaults.gpuFanCurveWmi2;
                                                    fanTab.setNodes(defaults);
                                                    EcchanClient.gpuFanCurveWmi2 = defaults;
                                                    root.profile.state.gpuFanCurveWmi2 = defaults;
                                                    root.profilesChanged();
                                                }
                                            },
                                            {
                                                isAction: false,
                                                supported: EcchanClient.wmiVer == 2 && EcchanClient.hasDgpu,
                                                icon: "developer_board",
                                                text: "DTemp",
                                                key: "gpuTempCurveWmi2",
                                                unit: "°C",
                                                min: 0,
                                                max: 100,
                                                color: Theme.secondary,
                                                sliders: 7,
                                                set: () => {
                                                    root.profile.state.gpuTempCurveWmi2 = fanTab.getNodes();
                                                    root.profilesChanged();

                                                    if (EcchanClient.fanMode === "Advanced") {
                                                        EcchanClient.gpuTempCurveWmi2 = fanTab.getNodes();
                                                    }
                                                },
                                                reset: () => {
                                                    const defaults = root.defaults.gpuTempCurveWmi2;
                                                    fanTab.setNodes(defaults);
                                                    EcchanClient.gpuTempCurveWmi2 = defaults;
                                                    root.profile.state.gpuTempCurveWmi2 = defaults;
                                                    root.profilesChanged();
                                                }
                                            },
                                            {
                                                isAction: false,
                                                supported: EcchanClient.wmiVer == 2 && EcchanClient.hasDgpu,
                                                icon: "developer_board",
                                                text: "DHysteresis",
                                                key: "gpuHysteresisCurveWmi2",
                                                unit: "°C",
                                                min: 0,
                                                max: 10,
                                                color: Theme.secondary,
                                                sliders: 6,
                                                set: value => () => {
                                                    root.profile.state.gpuHysteresisCurveWmi2 = fanTab.getNodes();
                                                    root.profilesChanged();

                                                    if (EcchanClient.fanMode === "Advanced") {
                                                        EcchanClient.gpuHysteresisCurveWmi2 = fanTab.getNodes();
                                                    }
                                                },
                                                reset: () => {
                                                    const defaults = root.defaults.gpuHysteresisCurveWmi2;
                                                    fanTab.setNodes(defaults);
                                                    EcchanClient.gpuHysteresisCurveWmi2 = defaults;
                                                    root.profile.state.gpuHysteresisCurveWmi2 = defaults;
                                                    root.profilesChanged();
                                                }
                                            },
                                        ]

                                        model: modelConfig.filter(item => item.supported)
                                    }

                                    RowLayout {
                                        visible: EcchanClient.wmiVer === 2

                                        Layout.fillHeight: true
                                        Layout.fillWidth: true
                                        Layout.topMargin: Theme.spacingM

                                        Item {
                                            Layout.fillWidth: true
                                            Layout.fillHeight: true

                                            DankButton {
                                                width: 30
                                                height: 30
                                                anchors.left: parent.left
                                                anchors.bottom: parent.bottom

                                                backgroundColor: "transparent"

                                                onClicked: fanTab.item.reset()

                                                DankIcon {
                                                    anchors.centerIn: parent
                                                    name: "refresh"
                                                    color: Theme.surfaceText
                                                    size: Theme.iconSize
                                                    opacity: 0.6
                                                }
                                            }
                                        }

                                        RowLayout {
                                            Layout.fillHeight: true
                                            Layout.fillWidth: true

                                            spacing: Theme.spacingXL

                                            Repeater {
                                                model: [
                                                    {
                                                        value: fanTab.n1,
                                                        set: value => {
                                                            fanTab.n1 = value;
                                                            fanTab.item.set();
                                                        },
                                                        show: fanTab.item.sliders >= 1
                                                    },
                                                    {
                                                        value: fanTab.n2,
                                                        set: value => {
                                                            fanTab.n2 = value;
                                                            fanTab.item.set();
                                                        },
                                                        show: fanTab.item.sliders >= 2
                                                    },
                                                    {
                                                        value: fanTab.n3,
                                                        set: value => {
                                                            fanTab.n3 = value;
                                                            fanTab.item.set();
                                                        },
                                                        show: fanTab.item.sliders >= 3
                                                    },
                                                    {
                                                        value: fanTab.n4,
                                                        set: value => {
                                                            fanTab.n4 = value;
                                                            fanTab.item.set();
                                                        },
                                                        show: fanTab.item.sliders >= 4
                                                    },
                                                    {
                                                        value: fanTab.n5,
                                                        set: value => {
                                                            fanTab.n5 = value;
                                                            fanTab.item.set();
                                                        },
                                                        show: fanTab.item.sliders >= 5
                                                    },
                                                    {
                                                        value: fanTab.n6,
                                                        set: value => {
                                                            fanTab.n6 = value;
                                                            fanTab.item.set();
                                                        },
                                                        show: fanTab.item.sliders >= 6
                                                    },
                                                    {
                                                        value: fanTab.n7,
                                                        set: value => {
                                                            fanTab.n7 = value;
                                                            fanTab.item.set();
                                                        },
                                                        show: fanTab.item.sliders >= 7
                                                    }
                                                ]

                                                ColumnLayout {
                                                    id: sliderRoot
                                                    Layout.fillHeight: true

                                                    visible: show

                                                    required property int value
                                                    required property var set
                                                    required property bool show

                                                    Connections {
                                                        target: root
                                                        function onAreaClicked() {
                                                            textInput.focus = false;
                                                        }
                                                    }

                                                    TextInput {
                                                        id: textInput
                                                        Layout.alignment: Qt.AlignCenter

                                                        font.pixelSize: Theme.fontSizeSmall
                                                        font.family: Theme.fontFamily
                                                        color: Theme.surfaceText
                                                        selectionColor: Theme.primaryContainer
                                                        selectedTextColor: Theme.primary
                                                        horizontalAlignment: TextInput.AlignLeft
                                                        verticalAlignment: TextInput.AlignVCenter
                                                        selectByMouse: true
                                                        clip: true
                                                        activeFocusOnTab: true
                                                        onAccepted: {
                                                            const val = parseInt(text);
                                                            curveSlider.value = val;
                                                            sliderRoot.set(val);
                                                            focus = false;
                                                        }

                                                        text: activeFocus
                                                                  ? String(curveSlider.value)
                                                                  : curveSlider.value + fanTab.item.unit

                                                        onActiveFocusChanged: {
                                                            if (activeFocus) {
                                                                Qt.callLater(selectAll);
                                                            } else {
                                                                let val = parseInt(text);
                                                                if (!isNaN(val) && val >= fanTab.item.min && val <= fanTab.item.max) {
                                                                    accepted();
                                                                }
                                                            }
                                                        }

                                                        validator: IntValidator {
                                                            bottom: fanTab.item.min
                                                            top: fanTab.item.max
                                                        }

                                                        MouseArea {
                                                            anchors.fill: parent
                                                            hoverEnabled: true
                                                            cursorShape: Qt.IBeamCursor
                                                            acceptedButtons: Qt.NoButton
                                                        }
                                                    }

                                                    DankVerticalSlider {
                                                        id: curveSlider
                                                        Layout.fillHeight: true

                                                        unit: fanTab.item.unit
                                                        minimum: fanTab.item.min
                                                        maximum: fanTab.item.max
                                                        showValue: false

                                                        onSliderDragFinished: value => sliderRoot.set(value)

                                                        Binding on value {
                                                            value: sliderRoot.value
                                                            when: !curveSlider.pressed
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        Item {
                                            Layout.fillWidth: true
                                        }
                                    }
                                }
                            }
                        }

                        // Battery
                        ColumnLayout {
                            id: page5

                            visible: root.currentTab === 4 && EcchanClient.batteryChargeModeSupported
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            function modeToInt(mode) {
                                switch (mode) {
                                    // qmlformat off
                                    case "Mobility":
                                        return 100;
                                    case "Balanced":
                                        return 80;
                                    case "Healthy":
                                        return 60;
                                    default:
                                        return mode;
                                    // qmlformat on
                                }
                            }

                            property bool customChargeModeEnabled: false
                            property int customChargeModeValue: 100

                            function updateCustom() {
                                customChargeModeEnabled = root.profile?.customBatteryChargeModeEnabled || typeof (EcchanClient.batteryChargeMode) === "number";
                                customChargeModeValue = modeToInt(root.profile?.customBatteryChargeModeValue) || modeToInt(EcchanClient.batteryChargeMode);
                            }

                            Component.onCompleted: updateCustom()

                            Connections {
                                target: root

                                function onSelectedProfileChanged() {
                                    page5.updateCustom();
                                }
                            }

                            Connections {
                                target: EcchanClient

                                function onBatteryChargeModeChanged() {
                                    page5.updateCustom();
                                }
                            }

                            onCustomChargeModeEnabledChanged: {
                                if (root.pluginService) {
                                    root.profile.customBatteryChargeModeEnabled = customChargeModeEnabled;
                                    root.profilesChanged();
                                }
                            }

                            onCustomChargeModeValueChanged: {
                                if (root.pluginService) {
                                    root.profile.customBatteryChargeModeValue = customChargeModeValue;
                                    root.profilesChanged();
                                }
                            }

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
                                        id: modelRptr

                                        model: [
                                            {
                                                name: "Mobility",
                                                icon: "battery_android_full",
                                                selected: EcchanClient.batteryChargeMode === "Mobility",
                                                setMode: () => EcchanClient.batteryChargeMode = "Mobility"
                                            },
                                            {
                                                name: "Balanced",
                                                icon: "battery_android_5",
                                                selected: EcchanClient.batteryChargeMode === "Balanced",
                                                setMode: () => EcchanClient.batteryChargeMode = "Balanced"
                                            },
                                            {
                                                name: "Healthy",
                                                icon: "battery_android_4",
                                                selected: EcchanClient.batteryChargeMode === "Healthy",
                                                setMode: () => EcchanClient.batteryChargeMode = "Healthy"
                                            },
                                            {
                                                name: "Custom",
                                                icon: "battery_android_bolt",
                                                selected: page5.customChargeModeEnabled,
                                                setMode: () => EcchanClient.batteryChargeMode = page5.customChargeModeValue
                                            },
                                        ]

                                        ColumnLayout {
                                            id: page5Column

                                            Layout.alignment: Qt.AlignTop | Qt.AlignHCenter

                                            required property string name
                                            required property string icon
                                            required property bool selected
                                            required property var setMode

                                            spacing: Theme.spacingXS

                                            // toggles
                                            ToggleActionButton {
                                                id: customBtn
                                                iconName: icon
                                                checked: (name !== "Custom" && selected && !page5.customChargeModeEnabled) || (name === "Custom" && page5.customChargeModeEnabled)
                                                iconSize: Theme.iconSizeLarge + 16
                                                buttonHeight: 110
                                                buttonWidth: 140

                                                onClicked: {
                                                    setMode();

                                                    if (name === "Custom") {
                                                        page5.customChargeModeEnabled = true;
                                                    } else if (page5.customChargeModeEnabled) {
                                                        EcchanClient.queue(() => {
                                                            page5.customChargeModeEnabled = false;
                                                        });
                                                    }
                                                }

                                                RowLayout {
                                                    Layout.fillWidth: true

                                                    anchors.bottom: parent.bottom
                                                    anchors.horizontalCenter: parent.horizontalCenter
                                                    anchors.bottomMargin: Theme.spacingS

                                                    spacing: 2

                                                    StyledText {
                                                        text: name === "Custom" ? "Custom:" : name
                                                        font.pixelSize: Theme.fontSizeSmall
                                                        font.weight: Font.Bold
                                                        color: customBtn.checked ? Theme.primaryText : Theme.surfaceText

                                                        horizontalAlignment: Text.AlignCenter
                                                    }

                                                    Connections {
                                                        target: root
                                                        function onAreaClicked() {
                                                            batteryTextInput.focus = false;
                                                        }
                                                    }

                                                    TextInput {
                                                        id: batteryTextInput

                                                        property int previousValue: 100

                                                        enabled: page5.customChargeModeEnabled
                                                        visible: name === "Custom"

                                                        font.pixelSize: Theme.fontSizeSmall
                                                        font.family: Theme.fontFamily
                                                        font.weight: Font.Bold
                                                        color: page5.customChargeModeEnabled ? Theme.primaryText : Theme.surfaceText
                                                        selectionColor: Theme.primaryContainer
                                                        selectedTextColor: Theme.primary
                                                        horizontalAlignment: TextInput.AlignLeft
                                                        selectByMouse: true
                                                        clip: true
                                                        activeFocusOnTab: true
                                                        onAccepted: {
                                                            page5.customChargeModeValue = parseInt(text);
                                                            setMode();
                                                        }

                                                        text: String(page5.customChargeModeValue)

                                                        onActiveFocusChanged: {
                                                            if (activeFocus) {
                                                                previousValue = page5.customChargeModeValue;
                                                                selectAll();
                                                            } else {
                                                                let val = parseInt(text);
                                                                if (!isNaN(val) && val >= 10 && val <= 100) {
                                                                    accepted();
                                                                } else {
                                                                    page5.customChargeModeValue = previousValue;
                                                                    text = previousValue;
                                                                }
                                                            }
                                                        }

                                                        maximumLength: 3
                                                        validator: IntValidator {
                                                            bottom: 10
                                                            top: 100
                                                        }

                                                        MouseArea {
                                                            anchors.fill: parent
                                                            hoverEnabled: true
                                                            cursorShape: Qt.IBeamCursor
                                                            acceptedButtons: Qt.NoButton
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

                            visible: root.currentTab === 99
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            onVisibleChanged: {
                                if (visible) {
                                    Update.addRef(["ecDumpPretty"]);
                                } else {
                                    Update.removeRef(["ecDumpPretty"]);
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
