#![expect(unused)]

use std::pin::Pin;

use cxx::UniquePtr;
use cxx_qt_lib::QString;

use super::QJSValue;

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("ecchan-client/qjsvalueiterator.h");
        type QJSValueIterator;

        type QJSValue = super::QJSValue;
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        fn qjsvalueiterator_new(value: &QJSValue) -> UniquePtr<QJSValueIterator>;
        fn qjsvalueiterator_value(iterator: &QJSValueIterator) -> UniquePtr<QJSValue>;

        #[rust_name = "qjsvalueiterator_name"]
        fn name(self: &QJSValueIterator) -> QString;
        #[rust_name = "qjsvalueiterator_has_next"]
        fn hasNext(self: &QJSValueIterator) -> bool;
        #[rust_name = "qjsvalueiterator_next"]
        fn next(self: Pin<&mut QJSValueIterator>) -> bool;
    }
}

pub use qobject::QJSValueIterator;

impl QJSValueIterator {
    pub fn new(value: &QJSValue) -> UniquePtr<Self> {
        qobject::qjsvalueiterator_new(value)
    }

    pub fn value(&self) -> UniquePtr<QJSValue> {
        qobject::qjsvalueiterator_value(self)
    }

    pub fn has_next(&self) -> bool {
        self.qjsvalueiterator_has_next()
    }

    pub fn next(self: Pin<&mut Self>) -> bool {
        self.qjsvalueiterator_next()
    }

    pub fn name(&self) -> QString {
        self.qjsvalueiterator_name()
    }
}
