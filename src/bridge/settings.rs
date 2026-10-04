//! Paramètres de Lumen, conservés d'une session à l'autre dans
//! ~/.config/lumen/settings.toml (lisible et modifiable à la main, Lumen fermé).

#[cxx_qt::bridge]
pub mod qobject {
    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, volume)]
        #[qproperty(bool, muted)]
        #[qproperty(i32, repeat_mode)]
        #[qproperty(bool, shuffle)]
        #[qproperty(i32, window_width)]
        #[qproperty(i32, window_height)]
        #[qproperty(bool, maximized)]
        #[qproperty(bool, playlist_open)]
        #[qproperty(bool, resume_playback)]
        #[qproperty(bool, auto_playlist)]
        type Settings = super::SettingsRust;

        /// Enregistre les paramètres sur le disque.
        #[qinvokable]
        fn save(self: &Settings);
    }
}

use std::path::{Path, PathBuf};

use cxx_qt::CxxQtType;
use serde::{Deserialize, Serialize};

/// Contenu du fichier. Toute clé absente prend sa valeur par défaut.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsFile {
    pub volume: f64,
    pub muted: bool,
    /// 0 : désactivée, 1 : le fichier, 2 : la playlist
    pub repeat_mode: i32,
    pub shuffle: bool,
    pub window_width: i32,
    pub window_height: i32,
    pub maximized: bool,
    pub playlist_open: bool,
    /// Reprendre chaque vidéo là où on s'est arrêté
    pub resume_playback: bool,
    /// Ajouter à la playlist les fichiers du même nom (épisodes, parties…)
    pub auto_playlist: bool,
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self {
            volume: 100.0,
            muted: false,
            repeat_mode: 0,
            shuffle: false,
            window_width: 1280,
            window_height: 760,
            maximized: false,
            playlist_open: false,
            resume_playback: true,
            auto_playlist: true,
        }
    }
}

impl SettingsFile {
    /// Ramène dans des limites raisonnables les valeurs modifiées à la main.
    fn sanitized(mut self) -> Self {
        let defaults = Self::default();
        self.volume = if self.volume.is_finite() { self.volume.clamp(0.0, 130.0) } else { defaults.volume };
        if !(0..=2).contains(&self.repeat_mode) {
            self.repeat_mode = defaults.repeat_mode;
        }
        // Mêmes minimums que la fenêtre (Main.qml), et pas de taille absurde
        self.window_width = self.window_width.clamp(640, 16384);
        self.window_height = self.window_height.clamp(400, 16384);
        self
    }

    /// Lit le fichier ; absent ou illisible, renvoie les valeurs par défaut.
    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else { return Self::default() };
        match toml::from_str::<Self>(&text) {
            Ok(settings) => settings.sanitized(),
            Err(e) => {
                eprintln!("Lumen : {} illisible, paramètres par défaut ({e})", path.display());
                Self::default()
            }
        }
    }

    /// Écrit dans un fichier temporaire puis le renomme : un arrêt brutal pendant
    /// l'écriture ne laisse jamais un fichier à moitié écrit.
    pub fn store(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let body = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        let text = format!("# Paramètres de Lumen — modifiables à la main, Lumen fermé\n\n{body}");
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, path)
    }
}

fn settings_path() -> PathBuf {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    config.join("lumen").join("settings.toml")
}

pub struct SettingsRust {
    volume: f64,
    muted: bool,
    repeat_mode: i32,
    shuffle: bool,
    window_width: i32,
    window_height: i32,
    maximized: bool,
    playlist_open: bool,
    resume_playback: bool,
    auto_playlist: bool,
}

impl From<SettingsFile> for SettingsRust {
    fn from(f: SettingsFile) -> Self {
        Self {
            volume: f.volume,
            muted: f.muted,
            repeat_mode: f.repeat_mode,
            shuffle: f.shuffle,
            window_width: f.window_width,
            window_height: f.window_height,
            maximized: f.maximized,
            playlist_open: f.playlist_open,
            resume_playback: f.resume_playback,
            auto_playlist: f.auto_playlist,
        }
    }
}

impl From<&SettingsRust> for SettingsFile {
    fn from(s: &SettingsRust) -> Self {
        Self {
            volume: s.volume,
            muted: s.muted,
            repeat_mode: s.repeat_mode,
            shuffle: s.shuffle,
            window_width: s.window_width,
            window_height: s.window_height,
            maximized: s.maximized,
            playlist_open: s.playlist_open,
            resume_playback: s.resume_playback,
            auto_playlist: s.auto_playlist,
        }
    }
}

impl Default for SettingsRust {
    fn default() -> Self {
        SettingsFile::load(&settings_path()).into()
    }
}

impl qobject::Settings {
    pub fn save(&self) {
        let file = SettingsFile::from(self.rust()).sanitized();
        if let Err(e) = file.store(&settings_path()) {
            eprintln!("Lumen : impossible d'enregistrer les paramètres ({e})");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lumen-settings-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("settings.toml")
    }

    #[test]
    fn round_trips() {
        let path = temp_path("roundtrip");
        let settings = SettingsFile {
            volume: 85.0,
            muted: true,
            repeat_mode: 2,
            shuffle: true,
            window_width: 1600,
            window_height: 900,
            maximized: true,
            playlist_open: true,
            resume_playback: false,
            auto_playlist: false,
        };
        settings.store(&path).unwrap(); // crée aussi le dossier
        assert_eq!(SettingsFile::load(&path), settings);
        assert!(!path.with_extension("toml.tmp").exists());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn tolerates_missing_and_bad_values() {
        let path = temp_path("partial");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        // Clés absentes : valeurs par défaut ; valeurs absurdes : corrigées
        std::fs::write(&path, "volume = 400.0\nrepeat_mode = 7\nwindow_width = 10\nshuffle = true\n").unwrap();
        let loaded = SettingsFile::load(&path);
        assert_eq!(loaded.volume, 130.0);
        assert_eq!(loaded.repeat_mode, 0);
        assert_eq!(loaded.window_width, 640);
        assert!(loaded.shuffle);
        assert!(loaded.resume_playback && loaded.auto_playlist);

        // Fichier illisible : paramètres par défaut, sans planter
        std::fs::write(&path, "ceci n'est pas du TOML [[[").unwrap();
        assert_eq!(SettingsFile::load(&path), SettingsFile::default());

        // Fichier absent
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
        assert_eq!(SettingsFile::load(&path), SettingsFile::default());
    }
}
