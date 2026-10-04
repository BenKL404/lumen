//! Pont vers les utilitaires d'application C++ (cpp/app.*).
//!
//! `cxx-qt-lib` n'expose pas `QQmlApplicationEngine::rootObjects()`.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;

        include!("app.h");

        /// Vrai si l'interface QML a bien été créée.
        fn lumen_qml_loaded(engine: &QQmlApplicationEngine) -> bool;
    }
}
