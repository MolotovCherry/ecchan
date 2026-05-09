#![allow(dead_code)]

use std::pin::Pin;

use cxx::UniquePtr;
use cxx_qt_lib::QVariant;

pub use qobject::QQmlPropertyMap;

#[cxx_qt::bridge]
mod qobject {
    extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;

        include!("cxx-qt-lib/core/qhash/qhash_QString_QVariant.h");
        type QHash_QString_QVariant = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_QString_QVariant>;
    }

    #[auto_rust_name]
    unsafe extern "C++Qt" {
        include!("ecchan-client/qqml_property_map.h");
        #[qobject]
        pub type QQmlPropertyMap;

        #[qsignal]
        fn valueChanged(self: Pin<&mut QQmlPropertyMap>, key: &QString, value: &QVariant);
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");

        #[rust_name = "new"]
        fn make_unique() -> UniquePtr<QQmlPropertyMap>;
    }

    unsafe extern "C++" {
        fn clear(self: Pin<&mut QQmlPropertyMap>, key: &QString);
        fn contains(self: &QQmlPropertyMap, key: &QString) -> bool;
        fn count(self: &QQmlPropertyMap) -> i32;
        #[cfg(cxxqt_qt_version_at_least_6_1)]
        fn freeze(self: Pin<&mut QQmlPropertyMap>);
        #[cxx_name = "insert"]
        #[rust_name = "insert_values"]
        #[cfg(cxxqt_qt_version_at_least_6_1)]
        fn insertValues(self: Pin<&mut QQmlPropertyMap>, values: &QHash_QString_QVariant);
        fn insert(self: Pin<&mut QQmlPropertyMap>, key: &QString, value: &QVariant);
        #[rust_name = "is_empty"]
        fn isEmpty(self: &QQmlPropertyMap) -> bool;
        fn keys(self: &QQmlPropertyMap) -> QStringList;
        fn size(self: &QQmlPropertyMap) -> i32;
        fn value(self: &QQmlPropertyMap, key: &QString) -> QVariant;
    }

    #[namespace = "ecchan::qvariant"]
    unsafe extern "C++" {
        #[rust_name = "can_convert_QQmlPropertyMap"]
        fn qvariantCanConvertQQmlPropertyMap(variant: &QVariant) -> bool;
        #[rust_name = "construct_QQmlPropertyMap"]
        fn qvariantConstructQQmlPropertyMap(value: &UniquePtr<QQmlPropertyMap>) -> QVariant;
        #[rust_name = "value_or_default_QQmlPropertyMap"]
        fn qvariantValueOrDefaultQQmlPropertyMap(
            variant: Pin<&mut QVariant>,
        ) -> *mut QQmlPropertyMap;
    }
}

impl qobject::QQmlPropertyMap {
    pub fn new() -> UniquePtr<Self> {
        qobject::new()
    }

    pub fn get_mut(variant: Pin<&mut QVariant>) -> Option<&mut qobject::QQmlPropertyMap> {
        let raw = qobject::value_or_default_QQmlPropertyMap(variant);
        if raw.is_null() {
            None
        } else {
            Some(unsafe { &mut *raw })
        }
    }

    pub fn can_convert(variant: &QVariant) -> bool {
        qobject::can_convert_QQmlPropertyMap(variant)
    }

    /// Self MUST live at least as long as the QVariant
    pub unsafe fn as_qvariant(this: &UniquePtr<Self>) -> QVariant {
        // this is a shallow copy! you must keep the UniquePtr around!
        qobject::construct_QQmlPropertyMap(this)
    }
}
