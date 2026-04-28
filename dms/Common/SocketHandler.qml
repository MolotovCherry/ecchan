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

    function reset() {
        _callQueue = {};
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

    function _createChainer(id) {
        const chainer = {
            cb: callback => {
                _getId(id).cb.push(callback);
                return chainer;
            },
            cbErr: callbackErr => {
                _getId(id).cbErr.push(callbackErr);
                return chainer;
            }
        };

        return chainer;
    }

    function call(methodName, ...args) {
        const id = EcSocket[methodName](...args);
        return _createChainer(id);
    }

    function id(id) {
        return _createChainer(id);
    }
}
