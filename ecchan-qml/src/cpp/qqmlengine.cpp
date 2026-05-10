#include "ecchan-client/qqmlengine.h"

QJSEngine* qqmlengine_upcast_qjsengine(QQmlEngine* engine) {
    return static_cast<QJSEngine*>(engine);
}

QQmlEngine* qjsengine_upcast_qqmlengine(QJSEngine* engine) {
    return static_cast<QQmlEngine*>(engine);
}
