//! Lumen — lecteur vidéo pour Linux (Rust + CXX-Qt + QML + libmpv)

mod bridge;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() {
    // Doit être appelé avant la création de la fenêtre :
    // force le rendu OpenGL et enregistre le composant vidéo pour QML.
    bridge::video::ffi::lumen_init_video();

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/com/lumen/player/qml/Main.qml"));
    }

    // Sans objet racine, Qt resterait ouvert sans fenêtre : on quitte avec une erreur claire.
    // Les détails (module manquant, erreur de syntaxe…) sont affichés juste au-dessus par Qt.
    if !engine.as_ref().is_some_and(bridge::app::ffi::lumen_qml_loaded) {
        eprintln!(
            "Lumen : impossible de charger l'interface QML. \
             Vérifie que les modules qml6-module-* du README sont installés."
        );
        std::process::exit(1);
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
