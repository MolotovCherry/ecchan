use std::{env, path::PathBuf};

use cxx_qt_build::{CxxQtBuilder, PluginType, QmlModule};

fn main() {
    let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();

    let builder = CxxQtBuilder::new_qml_module(
        QmlModule::new("ecchan_client").plugin_type(PluginType::Dynamic),
    )
    .files([
        "src/cpp/ecchan.rs",
        "src/cpp/qjsengine.rs",
        "src/cpp/qtlogging.rs",
        "src/cpp/qjsvalueiterator.rs",
        "src/cpp/qjsvaluelist.rs",
        "src/cpp/qqml_property_map.rs",
        "src/cpp/qjsvalue.rs",
        "src/cpp/qqmlengine.rs",
    ])
    .cpp_file("src/cpp/qqml_property_map.cpp")
    .cpp_file("src/cpp/qjsengine.cpp")
    .cpp_file("src/cpp/qjsvalueiterator.cpp")
    .cpp_file("src/cpp/qjsvaluelist.cpp")
    .cpp_file("src/cpp/qtlogging.cpp")
    .cpp_file("src/cpp/qjsvalue.cpp")
    .cpp_file("src/cpp/qqmlengine.cpp")
    .include_dir(manifest_dir.join("includes/"));

    let builder = unsafe {
        builder.cc_builder(|cc| {
            cc.warnings(false);
        })
    };

    builder.build();

    // https://github.com/KDAB/cxx-qt/issues/1433
    let version_script = manifest_dir.join("qt-plugin.version");
    println!(
        "cargo::rustc-link-arg-cdylib=-Wl,--version-script={}",
        version_script.display()
    );
}
