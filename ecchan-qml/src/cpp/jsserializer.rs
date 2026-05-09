use core::pin::Pin;
use std::collections::HashMap;

use cxx::UniquePtr;
use cxx_qt_lib::QString;
use serde::ser::*;

use super::{QJSEngine, QJSValue};

pub struct JSEngineSerializer<'a> {
    engine: Pin<&'a mut QJSEngine>,
}

impl<'a> JSEngineSerializer<'a> {
    pub fn new(engine: Pin<&'a mut QJSEngine>) -> Self {
        Self { engine }
    }
}

impl<'a> Serializer for JSEngineSerializer<'a> {
    type Ok = UniquePtr<QJSValue>;
    type Error = serde_json::Error;

    type SerializeSeq = QJSSerializeSeq<'a>;
    type SerializeTuple = QJSSerializeSeq<'a>;
    type SerializeTupleStruct = QJSSerializeSeq<'a>;
    type SerializeTupleVariant = QJSSerializeTupleVariant<'a>;
    type SerializeMap = QJSSerializeMap<'a>;
    type SerializeStruct = QJSSerializeMap<'a>;
    type SerializeStructVariant = QJSSerializeStructVariant<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_int(v as i32))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_int(v as i32))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_int(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_int(v as i32)) // Assuming 32-bit int for simplicity
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_uint(v as u32))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_uint(v as u32))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_uint(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_f64(v as f64)) // Assuming 32-bit int for simplicity
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_f64(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_f64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let s: String = v.into();
        self.serialize_str(&s)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::from_str(v))
    }

    fn serialize_bytes(mut self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let vec: Vec<_> = v.iter().map(|&b| QJSValue::from_uint(b as u32)).collect();
        Ok(QJSValue::from_array(self.engine.as_mut(), &vec))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::null())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(QJSValue::null())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        mut self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut map: HashMap<String, UniquePtr<QJSValue>> = HashMap::new();

        let serializer = JSEngineSerializer::new(self.engine.as_mut());
        map.insert(variant.to_string(), value.serialize(serializer)?);
        Ok(QJSValue::from_map(self.engine.as_mut(), &map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let JSEngineSerializer { mut engine } = self;
        let array = engine.as_mut().new_array(len.unwrap_or(0) as u32);
        Ok(QJSSerializeSeq {
            array,
            index: 0,
            engine,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let JSEngineSerializer { mut engine } = self;
        let array = engine.as_mut().new_array(len as u32);
        Ok(QJSSerializeTupleVariant {
            name: variant.to_string(),
            array,
            index: 0,
            engine,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let JSEngineSerializer { mut engine } = self;
        let object = engine.as_mut().new_object();
        Ok(QJSSerializeMap {
            object,
            key: None,
            engine,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let JSEngineSerializer { mut engine } = self;
        let object = engine.as_mut().new_object();
        Ok(QJSSerializeStructVariant {
            name: variant.to_string(),
            object,
            engine,
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + std::fmt::Display,
    {
        let s = value.to_string();
        self.serialize_str(&s)
    }
}

pub struct QJSSerializeSeq<'a> {
    array: UniquePtr<QJSValue>,
    index: usize,
    engine: Pin<&'a mut QJSEngine>,
}

impl<'a> SerializeSeq for QJSSerializeSeq<'a> {
    type Ok = UniquePtr<QJSValue>;
    type Error = serde_json::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = {
            let serializer = JSEngineSerializer::new(self.engine.as_mut());
            value.serialize(serializer)?
        };
        self.array
            .as_mut()
            .unwrap()
            .set_element(self.index as u32, &value);
        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.array)
    }
}

impl<'a> SerializeTuple for QJSSerializeSeq<'a> {
    type Ok = UniquePtr<QJSValue>;
    type Error = serde_json::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = {
            let serializer = JSEngineSerializer::new(self.engine.as_mut());
            value.serialize(serializer)?
        };
        self.array
            .as_mut()
            .unwrap()
            .set_element(self.index as u32, &value);
        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.array)
    }
}

impl<'a> SerializeTupleStruct for QJSSerializeSeq<'a> {
    type Ok = UniquePtr<QJSValue>;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = {
            let serializer = JSEngineSerializer::new(self.engine.as_mut());
            value.serialize(serializer)?
        };
        self.array
            .as_mut()
            .unwrap()
            .set_element(self.index as u32, &value);
        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.array)
    }
}

pub struct QJSSerializeTupleVariant<'a> {
    name: String,
    array: UniquePtr<QJSValue>,
    index: usize,
    engine: Pin<&'a mut QJSEngine>,
}

impl<'a> SerializeTupleVariant for QJSSerializeTupleVariant<'a> {
    type Ok = UniquePtr<QJSValue>;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = {
            let serializer = JSEngineSerializer::new(self.engine.as_mut());
            value.serialize(serializer)?
        };
        self.array
            .as_mut()
            .unwrap()
            .set_element(self.index as u32, &value);
        self.index += 1;
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        let mut object = self.engine.as_mut().new_object();
        object
            .as_mut()
            .unwrap()
            .set_property(&QString::from(&self.name), &self.array);
        Ok(object)
    }
}

pub struct QJSSerializeMap<'a> {
    object: UniquePtr<QJSValue>,
    key: Option<QString>,
    engine: Pin<&'a mut QJSEngine>,
}

impl<'a> SerializeMap for QJSSerializeMap<'a> {
    type Ok = UniquePtr<QJSValue>;
    type Error = serde_json::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let serializer = JSEngineSerializer::new(self.engine.as_mut());
        self.key = Some(key.serialize(serializer)?.to_qstring());
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if let Some(ref key) = self.key {
            let value = {
                let serializer = JSEngineSerializer::new(self.engine.as_mut());
                value.serialize(serializer)?
            };
            self.object.as_mut().unwrap().set_property(key, &value);
        }
        self.key = None;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.object)
    }
}

impl<'a> SerializeStruct for QJSSerializeMap<'a> {
    type Ok = UniquePtr<QJSValue>;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = {
            let serializer = JSEngineSerializer::new(self.engine.as_mut());
            value.serialize(serializer)?
        };
        self.object
            .as_mut()
            .unwrap()
            .set_property(&QString::from(key), &value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.object)
    }
}

pub struct QJSSerializeStructVariant<'a> {
    name: String,
    object: UniquePtr<QJSValue>,
    engine: Pin<&'a mut QJSEngine>,
}

impl<'a> SerializeStructVariant for QJSSerializeStructVariant<'a> {
    type Ok = UniquePtr<QJSValue>;
    type Error = serde_json::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = {
            let serializer = JSEngineSerializer::new(self.engine.as_mut());
            value.serialize(serializer)?
        };
        self.object
            .as_mut()
            .unwrap()
            .set_property(&QString::from(key), &value);
        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        let mut variant = self.engine.as_mut().new_object();
        variant
            .as_mut()
            .unwrap()
            .set_property(&QString::from(&self.name), &self.object);
        Ok(variant)
    }
}
