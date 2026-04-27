import QtQuick
import qs.Services

import "../Services"

Item {
    id: root

    property var _callQueue: ({})

    Connections {
        target: EcSocket

        function onDataReady(id, payload, isErr) {
            const queue = root._callQueue[id];

            if (queue == null) {
                return;
            }

            if (isErr) {
                for (const cbErr of queue.cbErr) {
                    try {
                        cbErr(payload);
                    } catch (e) {
                        console.error("cbErr failed", e);
                        ToastService.showError("cbErr failed", e);
                    }
                }
            } else {
                for (const cb of queue.cb) {
                    try {
                        cb(payload);
                    } catch (e) {
                        console.error("cb failed", e);
                        ToastService.showError("cb failed", e);
                    }
                }
            }

            delete root._callQueue[id];
        }
    }

    function _getId(id) {
        if (_callQueue[id] == null) {
            _callQueue[id] = {
                "cb": [],
                "cbErr": []
            };
        }

        return _callQueue[id];
    }

    function register(id, cb, cbErr) {
        const data = _getId(id);
        data.cb.push(cb);
        data.cbErr.push(cbErr);
    }

    function registerCb(id, cb) {
        const data = _getId(id);
        data.cb.push(cb);
    }

    function registerCbErr(id, cbErr) {
        const data = _getId(id);
        data.cbErr.push(cbErr);
    }
}
