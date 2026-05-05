#pragma once

#include <QtCore/QVariant>
#include <QtCore/QSharedPointer>
#include <QtQml/QQmlPropertyMap>

using namespace std;

namespace ecchan {
namespace qvariant {

inline QVariant qvariantConstructQQmlPropertyMap(unique_ptr<QQmlPropertyMap> value) noexcept
{
    QSharedPointer<QQmlPropertyMap> shared(value.release());
    return QVariant::fromValue(shared);
}

inline bool qvariantCanConvertQQmlPropertyMap(const QVariant& variant) noexcept
{
    return variant.canConvert<QSharedPointer<QQmlPropertyMap>>();
}

inline QQmlPropertyMap* qvariantValueOrDefaultQQmlPropertyMap(QVariant& variant) noexcept
{
    if (variant.canConvert<QSharedPointer<QQmlPropertyMap>>()) {
        return variant.value<QSharedPointer<QQmlPropertyMap>>().data();
    }
    return nullptr;
}

}
}
