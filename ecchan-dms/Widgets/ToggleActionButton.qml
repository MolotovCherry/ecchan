import QtQuick
import QtQuick.Layouts

import qs.Common
import qs.Widgets

Item {
    id: root

    required property bool checked

    required property string iconName
    property int iconSize: Theme.iconSize
    property color iconColor: checked ? Theme.primaryText : Theme.primary

    property int horizontalPadding: Theme.spacingL
    property color backgroundColor: checked ? Theme.primary : Theme.withAlpha(Theme.surfaceContainerHigh, Theme.popupTransparency)

    property bool iconFilled: false
    property real iconFill: iconFilled ? 1.0 : 0.0

    property int buttonHeight: 40
    property int buttonWidth: 80
    height: buttonHeight
    width: buttonWidth

    signal clicked

    ColumnLayout {
        id: layout

        anchors.fill: parent

        DankButton {
            id: actionBtn
            Layout.fillWidth: true
            iconName: root.iconName
            iconSize: root.iconSize
            backgroundColor: root.backgroundColor
            horizontalPadding: root.horizontalPadding
            buttonHeight: root.buttonHeight

            onClicked: root.clicked()

            DankIcon {
                anchors.centerIn: parent
                name: root.iconName
                size: root.iconSize
                color: root.iconColor
                filled: root.iconFilled
                fill: root.iconFill
            }
        }
    }
}
