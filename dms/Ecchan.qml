pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import qs.Common
import qs.Widgets
import qs.Modules.Plugins

import "./Services"

PluginComponent {
    id: root

    layerNamespacePlugin: "ecchan"

    Connections {
        target: root.pluginData
        function onPluginDataChanged() {
            const socketFile = root.pluginData.socketFile;

            if (typeof (socketFile) === "string") {
                EcSocket.init(root.pluginData.socketFile);
            }
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

    popoutContent: Component {
        PopoutComponent {
            id: popout

            property int currentTab: 0
            onCurrentTabChanged: {}

            FocusScope {
                width: parent.width
                implicitHeight: root.popoutHeight - popout.headerHeight - popout.detailsHeight - Theme.spacingXL

                focus: true

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    // Branding
                    Item {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(Theme.fontSizeMedium * 3.4)

                        Row {
                            Row {
                                spacing: Theme.spacingXS

                                DankIcon {
                                    name: "memory"
                                    size: Theme.iconSizeLarge - 6
                                    color: Theme.primary
                                    anchors.verticalCenter: parent.verticalCenter
                                }

                                StyledText {
                                    text: "Ecchan"
                                    font.pixelSize: Theme.fontSizeLarge
                                    font.weight: Font.Bold
                                    color: Theme.surfaceText
                                    anchors.verticalCenter: parent.verticalCenter
                                }
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
                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        Layout.topMargin: Theme.spacingXS

                        Item {
                            id: tab1

                            anchors.fill: parent

                            visible: popout.currentTab == 0

                            onVisibleChanged: {
                                if (!visible) {
                                    return;
                                }
                            }

                            RowLayout {
                                anchors.fill: parent

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true

                                    Rectangle {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true
                                        color: 'transparent'
                                    }
                                }

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    Rectangle {
                                        Layout.fillWidth: true
                                        Layout.fillHeight: true

                                        radius: Theme.cornerRadius
                                        color: Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)
                                        border.color: Theme.outlineLight
                                        border.width: 1
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
