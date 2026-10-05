use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        // Modules Qt nécessaires (OpenGL pour le rendu mpv dans la scène QML)
        .qt_module("Quick")
        .qt_module("OpenGL")
        .qt_module("Qml")
        // Interface QML + objets Rust exposés à QML
        .qml_module(QmlModule {
            uri: "com.lumen.player",
            rust_files: &["src/bridge/utils.rs", "src/bridge/history.rs", "src/bridge/playlist.rs", "src/bridge/settings.rs", "src/bridge/screen_guard.rs", "src/bridge/mpris.rs"],
            qml_files: &[
                "qml/Main.qml",
                "qml/ControlBar.qml",
                "qml/IconButton.qml",
                "qml/SeekBar.qml",
                "qml/TrackMenu.qml",
                "qml/PlaylistPanel.qml",
                "qml/TitleBar.qml",
                "qml/AppMenu.qml",
                "qml/Icon.qml",
                "qml/ImagePanel.qml",
                "qml/ShortcutsPanel.qml",
                "qml/UrlPanel.qml",
            ],
            // Logo (écran d'accueil, barre de titre, icône de la fenêtre)
            qrc_files: &["assets/lumen.svg"],
            ..Default::default()
        })
        // Ponts Rust -> C++ (moteur vidéo, utilitaires d'application)
        .file("src/bridge/video.rs")
        .file("src/bridge/app.rs")
        // Composant vidéo C++ (rendu libmpv dans Qt Quick)
        .qobject_header("cpp/mpvitem.h")
        .cc_builder(|cc| {
            cc.include("cpp");
            cc.file("cpp/mpvitem.cpp");
            cc.file("cpp/app.cpp");
            cc.file("cpp/probe.cpp");
            cc.file("cpp/thumbnail.cpp");
        })
        .build();

    // Liaison avec libmpv
    println!("cargo:rustc-link-lib=mpv");
    // Lecture des durées de la playlist (cpp/probe.cpp)
    println!("cargo:rustc-link-lib=avformat");
    println!("cargo:rustc-link-lib=avutil");
    // Miniatures de la barre de progression (cpp/thumbnail.cpp)
    println!("cargo:rustc-link-lib=avcodec");
    println!("cargo:rustc-link-lib=swscale");
    println!("cargo:rerun-if-changed=cpp/mpvitem.cpp");
    println!("cargo:rerun-if-changed=cpp/app.cpp");
    println!("cargo:rerun-if-changed=cpp/app.h");
    println!("cargo:rerun-if-changed=cpp/probe.cpp");
    println!("cargo:rerun-if-changed=cpp/probe.h");
    println!("cargo:rerun-if-changed=cpp/thumbnail.cpp");
    println!("cargo:rerun-if-changed=cpp/thumbnail.h");
    println!("cargo:rerun-if-changed=cpp/mpvitem.h");
    println!("cargo:rerun-if-changed=assets/lumen.svg");
}
