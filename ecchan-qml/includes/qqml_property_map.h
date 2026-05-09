#pragma once

#include <QtCore/QVariant>
#include <QtQml/QQmlPropertyMap>

using namespace std;

namespace ecchan {
namespace qvariant {

QVariant qvariantConstructQQmlPropertyMap(const unique_ptr<QQmlPropertyMap>& value);

bool qvariantCanConvertQQmlPropertyMap(const QVariant& variant);

QQmlPropertyMap* qvariantValueOrDefaultQQmlPropertyMap(QVariant& variant);

}
}
