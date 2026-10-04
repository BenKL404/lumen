use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        // Modules Qt nécessaires (OpenGL pour le rendu mpv dans la scène QML)
        .qt_module("Quick")
        .qt_module("OpenGL")
        // Interface QML + objets Rust exposés à QML
        .qml_module(QmlModule {
            uri: "com.lumen.player",
            rust_files: &["src/bridge/utils.rs", "src/bridge/history.rs"],
            qml_files: &[
                "qml/Main.qml",
                "qml/ControlBar.qml",
                "qml/IconButton.qml",
                "qml/SeekBar.qml",
                "qml/TrackMenu.qml",
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
        })
        .build();

    // Liaison avec libmpv
    println!("cargo:rustc-link-lib=mpv");
    println!("cargo:rerun-if-changed=cpp/mpvitem.cpp");
    println!("cargo:rerun-if-changed=cpp/app.cpp");
    println!("cargo:rerun-if-changed=cpp/app.h");
}
