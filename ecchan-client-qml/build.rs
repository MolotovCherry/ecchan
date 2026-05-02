use cxx_qt_build::{CxxQtBuilder, PluginType, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("com.cherry.ecchan").plugin_type(PluginType::Dynamic),
    )
    .files(["src/ec_socket.rs"])
    .build();
}
