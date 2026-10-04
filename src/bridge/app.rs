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
        include!("thumbnail.h");

        /// Enregistre le fournisseur d'images des miniatures (avant de charger le QML).
        fn lumen_register_thumbnails(engine: Pin<&mut QQmlApplicationEngine>);

        /// Nom et identifiant de l'application ; avant de créer QGuiApplication.
        fn lumen_init_app();

        /// Icône des fenêtres ; après la création de QGuiApplication.
        fn lumen_set_window_icon();

        /// Vrai si l'interface QML a bien été créée.
        fn lumen_qml_loaded(engine: &QQmlApplicationEngine) -> bool;

        /// Ouvre un fichier via `openUrl()` de Main.qml.
        fn lumen_open_file(engine: &QQmlApplicationEngine, path: &QString);

        /// Enregistre une image de la fenêtre après `delay_ms`, puis quitte.
        fn lumen_snapshot(engine: &QQmlApplicationEngine, path: &QString, delay_ms: i32);
    }
}
