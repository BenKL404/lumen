//! Lumen — lecteur vidéo pour Linux (Rust + CXX-Qt + QML + libmpv)

mod bridge;
mod single_instance;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    // Lumen déjà ouvert : lui confier le fichier et s'arrêter (option single_instance)
    let argument = std::env::args().nth(1);
    if single_instance::forward_to_running_instance(argument.as_deref()) {
        return;
    }

    use_desktop_portal();
    apply_ui_scale();

    // Doit être appelé avant la création de la fenêtre :
    // force le rendu OpenGL et enregistre le composant vidéo pour QML.
    bridge::video::ffi::lumen_init_video();

    bridge::app::ffi::lumen_init_app();
    let mut app = QGuiApplication::new();
    bridge::app::ffi::lumen_set_window_icon();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(mut engine) = engine.as_mut() {
        bridge::app::ffi::lumen_register_thumbnails(engine.as_mut());
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

    if let Some(engine) = engine.as_ref() {
        // `lumen fichier.mkv` : ouvre le fichier au démarrage
        if let Some(file) = &argument {
            bridge::app::ffi::lumen_open_file(engine, &QString::from(file));
        }
        // Outil de développement : LUMEN_SNAPSHOT=capture.png enregistre la fenêtre puis quitte
        if let Some(path) = std::env::var_os("LUMEN_SNAPSHOT") {
            let delay = std::env::var("LUMEN_SNAPSHOT_DELAY").ok().and_then(|d| d.parse().ok());
            bridge::app::ffi::lumen_snapshot(
                engine,
                &QString::from(&*path.to_string_lossy()),
                delay.unwrap_or(2500),
            );
        }
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}

/// Taille de l'interface choisie dans les préférences (QT_SCALE_FACTOR), sauf si
/// l'utilisateur a déjà réglé la variable lui-même.
fn apply_ui_scale() {
    if std::env::var_os("QT_SCALE_FACTOR").is_some() {
        return;
    }
    let scale = bridge::settings::SettingsFile::load(&bridge::settings::settings_path()).ui_scale;
    if scale != 100 {
        std::env::set_var("QT_SCALE_FACTOR", format!("{:.2}", scale as f64 / 100.0));
    }
}

/// Dialogues natifs du bureau (sélecteur de fichiers de COSMIC, GNOME, KDE…) via le
/// portail XDG, plutôt que le dialogue intégré de Qt. Ne remplace que les valeurs
/// sans effet en Qt 6 (variable absente, ou thème réservé à Qt 5 comme `qt5ct`) :
/// un thème Qt 6 choisi explicitement est respecté.
fn use_desktop_portal() {
    let current = std::env::var("QT_QPA_PLATFORMTHEME").unwrap_or_default();
    if current.is_empty() || current == "qt5ct" {
        std::env::set_var("QT_QPA_PLATFORMTHEME", "xdgdesktopportal");
    }
}
