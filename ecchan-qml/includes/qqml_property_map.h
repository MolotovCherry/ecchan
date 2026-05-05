#pragma once

#include <QtCore/QVariant>
#include <QtQml/QQmlPropertyMap>

using namespace std;

namespace ecchan {
namespace qvariant {

inline QVariant qvariantConstructQQmlPropertyMap(const unique_ptr<QQmlPropertyMap>& value) noexcept
{
    return QVariant::fromValue(value.get());
}

inline bool qvariantCanConvertQQmlPropertyMap(const QVariant& variant) noexcept
{
    return variant.canConvert<QQmlPropertyMap*>();
}

inline QQmlPropertyMap* qvariantValueOrDefaultQQmlPropertyMap(QVariant& variant) noexcept
{
    if (variant.canConvert<QQmlPropertyMap*>()) {
        return variant.value<QQmlPropertyMap*>();
    }
    return nullptr;
}

}
}
