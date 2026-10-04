use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        // Modules Qt nécessaires (OpenGL pour le rendu mpv dans la scène QML)
        .qt_module("Quick")
        .qt_module("OpenGL")
        // Interface QML + objets Rust exposés à QML
        .qml_module(QmlModule {
            uri: "com.lumen.player",
            rust_files: &["src/bridge/utils.rs", "src/bridge/history.rs", "src/bridge/playlist.rs"],
            qml_files: &[
                "qml/Main.qml",
                "qml/ControlBar.qml",
                "qml/IconButton.qml",
                "qml/SeekBar.qml",
                "qml/TrackMenu.qml",
                "qml/PlaylistPanel.qml",
                "qml/TitleBar.qml",
                "qml/AppMenu.qml",
            ],
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
        })
        .build();

    // Liaison avec libmpv
    println!("cargo:rustc-link-lib=mpv");
    // Lecture des durées de la playlist (cpp/probe.cpp)
    println!("cargo:rustc-link-lib=avformat");
    println!("cargo:rustc-link-lib=avutil");
    println!("cargo:rerun-if-changed=cpp/mpvitem.cpp");
    println!("cargo:rerun-if-changed=cpp/app.cpp");
    println!("cargo:rerun-if-changed=cpp/app.h");
    println!("cargo:rerun-if-changed=cpp/probe.cpp");
    println!("cargo:rerun-if-changed=cpp/probe.h");
    println!("cargo:rerun-if-changed=cpp/mpvitem.h");
}
