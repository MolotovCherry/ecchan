pragma Singleton

import Quickshell
import QtQuick

import qs.Services

import "../Services"

Singleton {
    id: root

    property var _callQueue: ({})
    property var _globalCb: []

    Connections {
        target: EcSocket

        function onDataReady(id, method, payload, isErr) {
            const queue = root._callQueue[id];

            if (queue == null) {
                return;
            }

            if (isErr) {
                for (const cbErr of queue.cbErr) {
                    try {
                        cbErr(payload);
                    } catch (e) {
                        console.error("Ecchan: cbErr failed", e);
                        ToastService.showError("cbErr failed", e);
                    }
                }
            } else {
                for (const cb of queue.cb) {
                    try {
                        cb(payload);
                    } catch (e) {
                        console.error("Ecchan: cb failed", e);
                        ToastService.showError("cb failed", e);
                    }
                }
            }

            for (const global of root._globalCb) {
                global.cb(id, method, payload, isErr);
            }

            delete root._callQueue[id];
        }
    }

    // ensure name is unique!
    // cb accepts (int id, string method, var payload, bool isErr)
    function addGlobal(name, cb) {
        _globalCb.push({
            "name": name,
            "cb": cb
        });
    }

    // remove all added globals by name;make sure your name was unique
    function removeGlobal(name) {
        _globalCb = _globalCb.filter(item => item.name !== name);
    }

    function reset() {
        _callQueue = {};
        // do not clear globalCb; it's not a processing queue
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
