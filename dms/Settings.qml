import QtQuick
import Quickshell
import qs.Modules.Plugins

PluginSettings {
    id: root
    pluginId: "ecchan"

    property string socket: Quickshell.env("ECCHAN_SOCK") || "/run/ecchan.sock"

    ToggleSetting {
        settingKey: "startup"
        label: "Apply at Startup"
        description: "Apply selected profile at startup"
        defaultValue: true
    }

    StringSetting {
        settingKey: "socket"
        label: "Socket"
        description: "Path to ecchan server socket"
        placeholder: root.socket
        defaultValue: root.socket
    }
}
