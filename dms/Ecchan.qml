pragma ComponentBehavior: Bound

import QtQuick

import qs.Common
import qs.Widgets
import qs.Modules.Plugins

import "./Services"

PluginComponent {
    id: root

    Component.onCompleted: {
        // EcSocket.fanCount(data => {
        //     ToastService.showInfo(data, JSON.stringify(data));
        // });

    }

    Connections {
        target: EcSocket
        function onConnectedChanged() {
            if (!EcSocket.connected) {
                console.warn("trying reconnect in connection changed", EcSocket.connected);
                //EcSocket.reconnect();
            }
        }
    }

    horizontalBarPill: Component {
        Row {
            spacing: Theme.spacingS

            DankIcon {
                name: "widgets"
                size: Theme.iconSize
                color: Theme.primary
                anchors.verticalCenter: parent.verticalCenter
            }

            StyledText {
                text: root.pluginData.socketFile
                font.pixelSize: Theme.fontSizeMedium
                color: Theme.surfaceText
                anchors.verticalCenter: parent.verticalCenter
            }
        }
    }

    verticalBarPill: Component {
        Column {
            spacing: Theme.spacingXS

            DankIcon {
                name: "widgets"
                size: Theme.iconSize
                color: Theme.primary
                anchors.horizontalCenter: parent.horizontalCenter
            }

            StyledText {
                text: root.pluginData.socketFile
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.surfaceText
                anchors.horizontalCenter: parent.horizontalCenter
            }
        }
    }
}
