//! Pont vers le composant vidéo C++ (MpvItem).
//!
//! Le rendu libmpv dans Qt Quick nécessite d'hériter de
//! QQuickFramebufferObject, ce qui se fait proprement en C++.
//! Toute la logique applicative, elle, vit en Rust.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("mpvitem.h");

        /// Force le backend OpenGL et enregistre `MpvVideo` dans QML.
        fn lumen_init_video();
    }
}
