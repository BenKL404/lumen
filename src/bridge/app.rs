//! Pont vers les utilitaires d'application C++ (cpp/app.*).
//!
//! `cxx-qt-lib` n'expose pas `QQmlApplicationEngine::rootObjects()`.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qqmlapplicationengine.h");
        type QQmlApplicationEngine = cxx_qt_lib::QQmlApplicationEngine;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("app.h");

        /// Vrai si l'interface QML a bien été créée.
        fn lumen_qml_loaded(engine: &QQmlApplicationEngine) -> bool;

        /// Ouvre un fichier via `openUrl()` de Main.qml.
        fn lumen_open_file(engine: &QQmlApplicationEngine, path: &QString);

        /// Enregistre une image de la fenêtre après `delay_ms`, puis quitte.
        fn lumen_snapshot(engine: &QQmlApplicationEngine, path: &QString, delay_ms: i32);
    }
}
