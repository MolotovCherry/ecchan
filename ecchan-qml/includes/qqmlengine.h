#include <QtQml/QQmlEngine>
#include <QtQml/QJSEngine>

QJSEngine* qqmlengine_upcast_qjsengine(QQmlEngine* engine);
QQmlEngine* qjsengine_upcast_qqmlengine(QJSEngine* engine);
