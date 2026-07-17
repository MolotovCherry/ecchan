pragma Singleton

import QtQml

QtObject {
    id: root

    property int currentTab: 0

    property var profilesModel: []
    property int selectedProfile: 0
    property var profiles: []
    property var defaults: ({})
    property var profile: profiles[selectedProfile]
    property string gpuPciBusId: ""

    // EcchanClient connections
    property bool blocked: true
    property bool init: true

    // fan tab index
    property int fanTabCurrentIndex: 0
}
