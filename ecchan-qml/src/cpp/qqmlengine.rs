use std::pin::Pin;

use cxx_qt::{QObject, casting::Upcast};
use cxx_qt_lib::QQmlEngine;

use crate::cpp::QJSEngine;

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qobject.h");
        type QObject = cxx_qt::QObject;

        include!("cxx-qt-lib/qqmlengine.h");
        type QQmlEngine = cxx_qt_lib::QQmlEngine;

        include!("ecchan-client/qjsengine.h");
        type QJSEngine = crate::cpp::QJSEngine;

        include!("ecchan-client/qqmlengine.h");
        unsafe fn qqmlengine_upcast_qjsengine(ptr: *mut QQmlEngine) -> *mut QJSEngine;
        unsafe fn qjsengine_upcast_qqmlengine(ptr: *mut QJSEngine) -> *mut QQmlEngine;
    }

    unsafe extern "C++" {
        #[rust_name = "qml_engine"]
        unsafe fn qmlEngine(object: *const QObject) -> *mut QQmlEngine;
    }
}

unsafe impl Upcast<QJSEngine> for qobject::QQmlEngine {
    unsafe fn upcast_ptr(this: *const Self) -> *const QJSEngine {
        unsafe { qobject::qqmlengine_upcast_qjsengine(this.cast_mut()) }
    }

    unsafe fn from_base_ptr(base: *const QJSEngine) -> *const Self {
        unsafe { qobject::qjsengine_upcast_qqmlengine(base.cast_mut()) }
    }
}

pub trait QQmlEngineExt {
    /// Get a reference to a QObject's parent QQmlEngine
    #[allow(clippy::self_named_constructors)]
    #[allow(clippy::mut_from_ref)]
    fn qml_engine<T>(obj: &T) -> Option<Pin<&mut QQmlEngine>>
    where
        T: Upcast<QObject>,
    {
        let t: &QObject = obj.upcast();
        let ptr = unsafe { qobject::qml_engine(t) };
        if ptr.is_null() {
            None
        } else {
            let ptr = unsafe { &mut *ptr };
            Some(unsafe { Pin::new_unchecked(ptr) })
        }
    }

    /// Get a reference to a QObject's parent QJSEngine
    #[allow(clippy::self_named_constructors)]
    #[allow(clippy::mut_from_ref)]
    fn js_engine<T>(obj: &T) -> Option<Pin<&mut QJSEngine>>
    where
        T: Upcast<QObject>,
    {
        let t: &QObject = obj.upcast();
        let ptr = unsafe { qobject::qml_engine(t) };
        if ptr.is_null() {
            None
        } else {
            let ptr = unsafe { &mut *ptr };
            let js: &mut QJSEngine = ptr.upcast_mut();

            Some(unsafe { Pin::new_unchecked(js) })
        }
    }
}

impl QQmlEngineExt for qobject::QQmlEngine {}
