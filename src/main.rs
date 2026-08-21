extern crate devtray;

use cxx_qt_lib::{QQmlApplicationEngine, QUrl};
use cxx_qt_lib_extras::QApplication;

fn main() {
    cxx_qt::init_crate!(devtray);

    let mut app = QApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/devtray/qml/MainWindow.qml"));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
