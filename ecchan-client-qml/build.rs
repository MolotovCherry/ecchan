use std::{env, path::PathBuf};

use cxx_qt_build::{CxxQtBuilder, PluginType, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("com.cherry.ecchan").plugin_type(PluginType::Dynamic),
    )
    .files(["src/qml.rs"])
    .build();

    // https://github.com/KDAB/cxx-qt/issues/1433
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let version_script = PathBuf::from(manifest_dir).join("qt-plugin.version");
    println!(
        "cargo::rustc-link-arg-cdylib=-Wl,--version-script={}",
        version_script.display()
    );
}
