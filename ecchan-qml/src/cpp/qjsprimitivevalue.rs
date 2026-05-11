use std::ffi::c_char;

use cxx::{ExternType, UniquePtr, kind};
use cxx_qt_lib::{QString, QVariant};

pub use qobject::QJSPrimitiveValue;

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("ecchan-client/qjsprimitivevalue.h");
        type QJSPrimitiveValue;

        include!("ecchan-client/qjsprimitivevalue.h");
        type QJSPrimitiveUndefined = super::QJSPrimitiveUndefined;

        include!("ecchan-client/qjsprimitivevalue.h");
        type QJSPrimitiveNull = super::QJSPrimitiveNull;
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");

        #[cxx_name = "construct"]
        fn qjsprimitiveundefined_construct() -> QJSPrimitiveUndefined;
        #[cxx_name = "construct"]
        fn qjsprimitivenull_construct() -> QJSPrimitiveNull;

        #[cxx_name = "make_unique"]
        fn qjsprimitivevalue_construct_undefined(
            value: QJSPrimitiveUndefined,
        ) -> UniquePtr<QJSPrimitiveValue>;

        #[cxx_name = "make_unique"]
        fn qjsprimitivevalue_construct_null(
            value: QJSPrimitiveNull,
        ) -> UniquePtr<QJSPrimitiveValue>;

        #[cxx_name = "make_unique"]
        fn qjsprimitivevalue_construct() -> UniquePtr<QJSPrimitiveValue>;

        #[cxx_name = "make_unique"]
        fn qjsprimitivevalue_construct_bool(value: bool) -> UniquePtr<QJSPrimitiveValue>;

        #[cxx_name = "make_unique"]
        fn qjsprimitivevalue_construct_qvariant(value: &QVariant) -> UniquePtr<QJSPrimitiveValue>;

        #[cxx_name = "make_unique"]
        fn qjsprimitivevalue_construct_qstring(value: &QString) -> UniquePtr<QJSPrimitiveValue>;

        #[cxx_name = "make_unique"]
        unsafe fn qjsprimitivevalue_construct_char(
            value: *const c_char,
        ) -> UniquePtr<QJSPrimitiveValue>;

        #[cxx_name = "make_unique"]
        fn qjsprimitivevalue_construct_double(value: f64) -> UniquePtr<QJSPrimitiveValue>;

        #[cxx_name = "make_unique"]
        fn qjsprimitivevalue_construct_int(value: i32) -> UniquePtr<QJSPrimitiveValue>;

        #[rust_name = "to_boolean"]
        fn toBoolean(self: &QJSPrimitiveValue) -> bool;

        #[rust_name = "to_double"]
        fn toDouble(self: &QJSPrimitiveValue) -> f64;

        #[rust_name = "to_integer"]
        fn toInteger(self: &QJSPrimitiveValue) -> i32;

        #[rust_name = "strictly_equals"]
        fn strictlyEquals(self: &QJSPrimitiveValue, other: &QJSPrimitiveValue) -> bool;
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {}

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {}
}

struct QJSPrimitiveUndefined;
struct QJSPrimitiveNull;

unsafe impl ExternType for QJSPrimitiveUndefined {
    type Id = cxx::type_id!("QJSPrimitiveUndefined");
    type Kind = kind::Trivial;
}

unsafe impl ExternType for QJSPrimitiveNull {
    type Id = cxx::type_id!("QJSPrimitiveNull");
    type Kind = kind::Trivial;
}

impl qobject::QJSPrimitiveValue {
    pub fn new() -> UniquePtr<Self> {
        qobject::qjsprimitivevalue_construct()
    }

    pub fn new_undefined() -> UniquePtr<Self> {
        qobject::qjsprimitivevalue_construct_undefined(qobject::qjsprimitiveundefined_construct())
    }

    pub fn new_null() -> UniquePtr<Self> {
        qobject::qjsprimitivevalue_construct_null(qobject::qjsprimitivenull_construct())
    }

    pub fn from_bool(value: bool) -> UniquePtr<Self> {
        qobject::qjsprimitivevalue_construct_bool(value)
    }

    pub fn from_qvariant(value: &QVariant) -> UniquePtr<Self> {
        qobject::qjsprimitivevalue_construct_qvariant(value)
    }

    pub fn from_qstring(value: &QString) -> UniquePtr<Self> {
        qobject::qjsprimitivevalue_construct_qstring(value)
    }

    pub unsafe fn from_char(value: *const c_char) -> UniquePtr<Self> {
        unsafe { qobject::qjsprimitivevalue_construct_char(value) }
    }

    pub fn from_double(value: f64) -> UniquePtr<Self> {
        qobject::qjsprimitivevalue_construct_double(value)
    }

    pub fn from_integer(value: i32) -> UniquePtr<Self> {
        qobject::qjsprimitivevalue_construct_int(value)
    }
}

impl PartialEq for qobject::QJSPrimitiveValue {
    fn eq(&self, other: &Self) -> bool {
        self.strictly_equals(other)
    }
}

impl Eq for qobject::QJSPrimitiveValue {}
