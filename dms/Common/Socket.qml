import QtQuick
import Quickshell.Io

Item {
    id: root

    property alias path: socket.path
    property alias parser: socket.parser
    property alias connected: socket.connected

    signal error(var error)

    Socket {
        id: socket

        // disconnect cause there was an error
        // qmllint disable signal-handler-parameters
        onError: error => {
            connected = false;
            root.error(error);
        }
    }

    function send(data) {
        const json = typeof data === "string" ? data : JSON.stringify(data);
        const message = json.endsWith("\n") ? json : json + "\n";
        socket.write(message);
        socket.flush();
    }
}
