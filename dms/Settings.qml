import QtQuick
import qs.Modules.Plugins

PluginSettings {
    id: root
    pluginId: "ecchan"

    StringSetting {
        settingKey: "socketFile"
        label: "Socket File"
        description: "Path to ecchan server socket file"
        placeholder: "/run/ecchan.sock"
        defaultValue: "/run/ecchan.sock"
    }
}
