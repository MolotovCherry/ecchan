#pragma once

#include <QtCore/QVariant>
#include <QtQml/QQmlPropertyMap>

using namespace std;

namespace rust
{
    namespace cxxqtlib1
    {
        QVariant qvariantConstructQQmlPropertyMap(const unique_ptr<QQmlPropertyMap>& value);

        bool qvariantCanConvertQQmlPropertyMap(const QVariant& variant);

        QQmlPropertyMap* qvariantValueOrDefaultQQmlPropertyMap(QVariant& variant);
    }
}
