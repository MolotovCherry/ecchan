#include "ecchan-client/qqml_property_map.h"

using namespace std;

namespace rust
{
    namespace cxxqtlib1
    {
        QVariant qvariantConstructQQmlPropertyMap(const unique_ptr<QQmlPropertyMap>& value)
        {
            return QVariant::fromValue(value.get());
        }

        bool qvariantCanConvertQQmlPropertyMap(const QVariant& variant)
        {
            return variant.canConvert<QQmlPropertyMap*>();
        }

        QQmlPropertyMap* qvariantValueOrDefaultQQmlPropertyMap(QVariant& variant)
        {
            if (variant.canConvert<QQmlPropertyMap*>()) {
                return variant.value<QQmlPropertyMap*>();
            }
            return nullptr;
        }
    }
}
