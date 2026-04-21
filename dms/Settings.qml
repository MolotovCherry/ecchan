import QtQuick
import Quickshell
import qs.Modules.Plugins

PluginSettings {
    id: root
    pluginId: "ecchan"

    property string socketFile: Quickshell.env("ECCHAN_SOCK") || "/run/ecchan.sock"

    StringSetting {
        settingKey: "socketFile"
        label: "Socket File"
        description: "Path to ecchan server socket file"
        placeholder: root.socketFile
        defaultValue: root.socketFile
    }
}
