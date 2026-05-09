use cxx_qt_lib::QString;
use serde::{
    Deserializer,
    de::{
        self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
    },
};

use super::{QJSValue, QJSValueIterator};

pub struct JSEngineDeserializer<'a> {
    value: &'a QJSValue,
}

impl<'a> JSEngineDeserializer<'a> {
    pub fn new(value: &'a QJSValue) -> Self {
        Self { value }
    }
}

impl<'de, 'a> de::Deserializer<'de> for JSEngineDeserializer<'a> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_bool() {
            visitor.visit_bool(self.value.to_bool())
        } else if self.value.is_number() {
            visitor.visit_f64(self.value.to_f64())
        } else if self.value.is_string() {
            let s = self.value.to_qstring();
            visitor.visit_string(s.to_string())
        } else if self.value.is_array() {
            self.deserialize_seq(visitor)
        } else if self.value.is_object() {
            self.deserialize_map(visitor)
        } else if self.value.is_null() {
            visitor.visit_unit()
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("unsupported type"),
                &visitor,
            ))
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_bool() {
            visitor.visit_bool(self.value.to_bool())
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a bool"),
                &visitor,
            ))
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_i8(self.value.to_int() as i8)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not an i8"),
                &visitor,
            ))
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_i16(self.value.to_int() as i16)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not an i16"),
                &visitor,
            ))
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_i32(self.value.to_int())
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not an i32"),
                &visitor,
            ))
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_i64(self.value.to_int() as i64)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not an i64"),
                &visitor,
            ))
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_u8(self.value.to_uint() as u8)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a u8"),
                &visitor,
            ))
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_u16(self.value.to_uint() as u16)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a u16"),
                &visitor,
            ))
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_u32(self.value.to_uint())
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a u32"),
                &visitor,
            ))
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_u64(self.value.to_uint() as u64)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a u64"),
                &visitor,
            ))
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_f32(self.value.to_f64() as f32)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a f32"),
                &visitor,
            ))
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_number() {
            visitor.visit_f64(self.value.to_f64())
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a f64"),
                &visitor,
            ))
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_string() {
            let s = self.value.to_qstring();
            let s = s.to_string();
            let mut chars = s.chars();
            if let Some(c) = chars.next()
                && chars.next().is_none()
            {
                return visitor.visit_char(c);
            }
            Err(de::Error::invalid_type(
                Unexpected::Str(&s.to_string()),
                &visitor,
            ))
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a char"),
                &visitor,
            ))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_string() {
            visitor.visit_str(&self.value.to_qstring().to_string())
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a string"),
                &visitor,
            ))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_string() {
            visitor.visit_string(self.value.to_qstring().to_string())
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a string"),
                &visitor,
            ))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_array() {
            let len = self.value.get_property(&QString::from("length")).to_uint() as usize;
            let mut vec = Vec::with_capacity(len);
            for i in 0..len {
                let elem = self.value.get_element(i as u32);
                if elem.is_number() {
                    vec.push(elem.to_uint() as u8);
                } else {
                    return Err(de::Error::invalid_type(
                        Unexpected::Other("not a byte array"),
                        &visitor,
                    ));
                }
            }
            visitor.visit_bytes(&vec)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not bytes"),
                &visitor,
            ))
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_array() {
            let len = self.value.get_property(&QString::from("length")).to_uint() as usize;
            let mut vec = Vec::with_capacity(len);
            for i in 0..len {
                let elem = self.value.get_element(i as u32);
                if elem.is_number() {
                    vec.push(elem.to_uint() as u8);
                } else {
                    return Err(de::Error::invalid_type(
                        Unexpected::Other("not a byte array"),
                        &visitor,
                    ));
                }
            }
            visitor.visit_byte_buf(vec)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not bytes"),
                &visitor,
            ))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_null() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_null() {
            visitor.visit_unit()
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a unit"),
                &visitor,
            ))
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_array() {
            let len = self.value.get_property(&QString::from("length")).to_uint() as usize;
            let seq = QJSSeqAccess {
                array: self.value,
                index: 0,
                len,
            };
            visitor.visit_seq(seq)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a sequence"),
                &visitor,
            ))
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_object() {
            let keys = QJSValueIterator::new(self.value);
            let map = QJSMapAccess {
                object: self.value,
                keys,
            };
            visitor.visit_map(map)
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not a map"),
                &visitor,
            ))
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.value.is_string() {
            visitor.visit_enum(QJSVariantAccess {
                variant: self.value.to_qstring().to_string(),
                value: self.value.clone(),
            })
        } else if self.value.is_object() {
            let mut keys = QJSValueIterator::new(self.value);
            if keys.has_next() {
                keys.as_mut().unwrap().next();
                let key = keys.name();
                let value = self.value.get_property(&key);
                visitor.visit_enum(QJSVariantAccess {
                    variant: key.to_string(),
                    value,
                })
            } else {
                Err(de::Error::invalid_type(
                    Unexpected::Other("not a single key object"),
                    &visitor,
                ))
            }
        } else {
            Err(de::Error::invalid_type(
                Unexpected::Other("not an enum"),
                &visitor,
            ))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct QJSSeqAccess<'a> {
    array: &'a QJSValue,
    index: usize,
    len: usize,
}

impl<'de, 'a> SeqAccess<'de> for QJSSeqAccess<'a> {
    type Error = serde_json::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.index >= self.len {
            return Ok(None);
        }
        let elem = self.array.get_element(self.index as u32);
        self.index += 1;
        let de = JSEngineDeserializer::new(&elem);
        seed.deserialize(de).map(Some)
    }
}

struct QJSMapAccess<'a> {
    object: &'a QJSValue,
    keys: cxx::UniquePtr<QJSValueIterator>,
}

impl<'de, 'a> MapAccess<'de> for QJSMapAccess<'a> {
    type Error = serde_json::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if !self.keys.has_next() {
            return Ok(None);
        }
        self.keys.as_mut().unwrap().next();
        let key = self.keys.name();
        let key_str = key.to_string();
        let value = QJSValue::from_str(&key_str);
        let de = JSEngineDeserializer::new(&value);
        seed.deserialize(de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let key = self.keys.name();
        let value = self.object.get_property(&key);
        let de = JSEngineDeserializer::new(&value);
        seed.deserialize(de)
    }
}

struct QJSVariantAccess {
    variant: String,
    value: cxx::UniquePtr<QJSValue>,
}

impl<'de> EnumAccess<'de> for QJSVariantAccess {
    type Error = serde_json::Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let binding = QJSValue::from_str(&self.variant);
        let de = JSEngineDeserializer::new(&binding);
        seed.deserialize(de).map(|v| (v, self))
    }
}

impl<'de> VariantAccess<'de> for QJSVariantAccess {
    type Error = serde_json::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let de = JSEngineDeserializer::new(&self.value);
        seed.deserialize(de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let de = JSEngineDeserializer::new(&self.value);
        de.deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let de = JSEngineDeserializer::new(&self.value);
        de.deserialize_map(visitor)
    }
}
