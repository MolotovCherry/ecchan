#![allow(dead_code)]

use std::pin::Pin;

use cxx::UniquePtr;
use cxx_qt_lib::QVariant;

pub use ffi::QQmlPropertyMap;

#[doc(hidden)]
#[cxx_qt::bridge]
mod ffi {
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
        fn qvariantConstructQQmlPropertyMap(value: UniquePtr<QQmlPropertyMap>) -> QVariant;
        #[rust_name = "value_or_default_QQmlPropertyMap"]
        fn qvariantValueOrDefaultQQmlPropertyMap(
            variant: Pin<&mut QVariant>,
        ) -> *mut QQmlPropertyMap;
    }
}

impl ffi::QQmlPropertyMap {
    pub fn new() -> UniquePtr<Self> {
        let mut n = ffi::new();
        n.pin_mut().clear(&"objectName".into());
        n
    }
}

pub trait QVariantConvertQQmlPropertyMap
where
    Self: Sized,
{
    fn into_qvariant(self) -> QVariant;
    fn can_convert(variant: &QVariant) -> bool;
    fn as_mut<'a>(variant: Pin<&'a mut QVariant>) -> Option<&'a mut ffi::QQmlPropertyMap>;
}

impl QVariantConvertQQmlPropertyMap for UniquePtr<ffi::QQmlPropertyMap> {
    fn into_qvariant(self) -> QVariant {
        // this function takes ownership of the UniquePtr!
        ffi::construct_QQmlPropertyMap(self)
    }

    fn can_convert(variant: &QVariant) -> bool {
        ffi::can_convert_QQmlPropertyMap(variant)
    }

    fn as_mut<'a>(variant: Pin<&'a mut QVariant>) -> Option<&'a mut ffi::QQmlPropertyMap> {
        let raw = ffi::value_or_default_QQmlPropertyMap(variant);
        if raw.is_null() {
            None
        } else {
            Some(unsafe { &mut *raw })
        }
    }
}
