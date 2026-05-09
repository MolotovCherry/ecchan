#![expect(unused)]

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("ecchan-client/qjsvalue.h");
        type QJSValue = super::super::QJSValue;

        include!("ecchan-client/qjsvaluelist.h");
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        type QJSValueList;

        #[rust_name = "cxx_clear"]
        fn clear(self: Pin<&mut QJSValueList>);
        #[rust_name = "cxx_contains"]
        fn contains(self: &QJSValueList, _: &QJSValue) -> bool;
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        #[rust_name = "qjsvaluelist_new"]
        fn qjsvaluelistNew() -> UniquePtr<QJSValueList>;
        #[rust_name = "qjsvaluelist_clone"]
        fn qjsvaluelistClone(other: &QJSValueList) -> UniquePtr<QJSValueList>;
    }
}

pub use qobject::QJSValueList;

impl QJSValueList {
    pub fn new() -> cxx::UniquePtr<Self> {
        qobject::qjsvaluelist_new()
    }

    pub fn clone(&self) -> cxx::UniquePtr<Self> {
        qobject::qjsvaluelist_clone(self)
    }
}
