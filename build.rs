use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("devtray")
            .qml_file("qml/MainWindow.qml")
            .qml_file("qml/TaskCard.qml")
            .qml_file("qml/TaskDialog.qml")
            .qml_file("qml/LogViewer.qml")
            .qml_file("qml/ConfirmDialog.qml"),
    )
    .qt_module("Gui")
    .qt_module("Qml")
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .qt_module("Widgets")
    .file("src/bridge/task_bridge.rs")
    .build();
}
