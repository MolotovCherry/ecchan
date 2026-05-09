#![expect(unused)]

use core::pin::Pin;

use cxx_qt_lib::QString;
use serde::{Serialize, de::DeserializeOwned};

use super::{JSEngineDeserializer, JSEngineSerializer, QJSValue};

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("ecchan-client/qjsengine.h");
        include!("ecchan-client/qjsvalue.h");
        type QJSValue = crate::cpp::QJSValue;
    }

    unsafe extern "C++Qt" {
        #[qobject]
        type QJSEngine;
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        #[doc(hidden)]
        #[rust_name = "qjsengine_new"]
        fn qjsengineNew() -> UniquePtr<QJSEngine>;

        #[doc(hidden)]
        #[rust_name = "qjsengine_new_array"]
        fn jsengineNewArray(engine: Pin<&mut QJSEngine>, length: u32) -> UniquePtr<QJSValue>;

        #[doc(hidden)]
        #[rust_name = "qjsengine_new_object"]
        fn jsengineNewObject(engine: Pin<&mut QJSEngine>) -> UniquePtr<QJSValue>;

        #[doc(hidden)]
        #[rust_name = "qjsengine_evaluate"]
        fn jsengineEvaluate(
            engine: Pin<&mut QJSEngine>,
            src: &QString,
            filename: &QString,
            line: i32,
        ) -> UniquePtr<QJSValue>;

        #[doc(hidden)]
        #[rust_name = "qjsengine_import_module"]
        fn jsengineImportModule(engine: Pin<&mut QJSEngine>, name: &QString)
        -> UniquePtr<QJSValue>;

        #[doc(hidden)]
        #[rust_name = "qjsengine_global_object"]
        fn jsengineGlobalObject(engine: Pin<&mut QJSEngine>) -> UniquePtr<QJSValue>;
    }
}

pub use qobject::QJSEngine;

impl QJSEngine {
    pub fn new() -> cxx::UniquePtr<Self> {
        qobject::qjsengine_new()
    }

    pub fn new_array(self: Pin<&mut Self>, length: u32) -> cxx::UniquePtr<QJSValue> {
        qobject::qjsengine_new_array(self, length)
    }

    pub fn new_object(self: Pin<&mut Self>) -> cxx::UniquePtr<QJSValue> {
        qobject::qjsengine_new_object(self)
    }

    pub fn evaluate(
        self: Pin<&mut Self>,
        src: &QString,
        filename: &QString,
        line: i32,
    ) -> cxx::UniquePtr<QJSValue> {
        qobject::qjsengine_evaluate(self, src, filename, line)
    }

    pub fn import_module(self: Pin<&mut Self>, name: &QString) -> cxx::UniquePtr<QJSValue> {
        qobject::qjsengine_import_module(self, name)
    }

    pub fn global_object(self: Pin<&mut Self>) -> cxx::UniquePtr<QJSValue> {
        qobject::qjsengine_global_object(self)
    }

    pub fn serialize<T: Serialize>(
        self: Pin<&mut Self>,
        value: &T,
    ) -> Result<cxx::UniquePtr<QJSValue>, serde_json::Error> {
        value.serialize(JSEngineSerializer::new(self))
    }

    pub fn deserialize<T: DeserializeOwned>(
        self: Pin<&mut Self>,
        value: &QJSValue,
    ) -> Result<T, serde_json::Error> {
        let de = JSEngineDeserializer::new(value);
        T::deserialize(de)
    }
}
