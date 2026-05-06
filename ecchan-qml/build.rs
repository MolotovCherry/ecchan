use std::{env, path::PathBuf};

use cxx_qt_build::{CxxQtBuilder, PluginType, QmlModule};

fn main() {
    let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR").unwrap().into();

    let builder = CxxQtBuilder::new_qml_module(
        QmlModule::new("ecchan_client").plugin_type(PluginType::Dynamic),
    )
    .files(["src/qml.rs", "src/qtlogging.rs", "src/qqml_property_map.rs"])
    .cpp_file("src/cpp/qtlogging.cpp")
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
