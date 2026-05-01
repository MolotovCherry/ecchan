import QtQuick
import Quickshell.Io

Item {
    id: root

    property alias path: socket.path
    property alias parser: socket.parser
    property bool connected: false

    property int _reconnectAttempt: 0

    signal connectionStateChanged

    onConnectedChanged: {
        socket.connected = connected;
    }

    Socket {
        id: socket

        // disconnect cause there was an error
        // qmllint disable signal-handler-parameters
        onError: error => {
            Qt.callLater(() => connected = false);
        }

        onConnectionStateChanged: {
            root.connectionStateChanged();
            if (connected) {
                root._reconnectAttempt = 0;
                return;
            } else if (root._reconnectAttempt === 0) {
                // try one more time to reconnect
                Qt.callLater(() => connected = true);
                root._reconnectAttempt += 1;
            }
        }
    }

    function send(data) {
        const json = typeof data === "string" ? data : JSON.stringify(data);
        const message = json.endsWith("\n") ? json : json + "\n";
        socket.write(message);
        socket.flush();
    }
}
