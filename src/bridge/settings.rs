//! Paramètres de Lumen, conservés d'une session à l'autre dans
//! ~/.config/lumen/settings.toml (lisible et modifiable à la main, Lumen fermé).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

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
        #[qproperty(bool, single_instance)]
        // Réglages d'image : -100…100, zoom en % (25…400)
        #[qproperty(i32, brightness)]
        #[qproperty(i32, contrast)]
        #[qproperty(i32, saturation)]
        #[qproperty(i32, gamma)]
        #[qproperty(i32, hue)]
        #[qproperty(i32, zoom)]
        type Settings = super::SettingsRust;

        /// Enregistre les paramètres sur le disque.
        #[qinvokable]
        fn save(self: &Settings);

        /// Touches actives d'une action (voir shortcuts.rs), au format de QKeySequence.
        #[qinvokable]
        fn keys(self: &Settings, action: &QString) -> QStringList;

        /// Problèmes de la section [raccourcis] (conflits, actions inconnues), un par ligne.
        #[qinvokable]
        fn shortcut_warnings(self: &Settings) -> QString;
    }
}

use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList};
use serde::{Deserialize, Serialize};

use super::shortcuts::{self, Keys, Resolved};

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
    /// Ouvrir les fichiers dans la fenêtre Lumen déjà ouverte plutôt qu'une nouvelle
    pub single_instance: bool,
    /// Réglages d'image, de -100 à 100 (0 : neutre)
    pub brightness: i32,
    pub contrast: i32,
    pub saturation: i32,
    pub gamma: i32,
    pub hue: i32,
    /// Zoom de l'image en % (100 : taille normale)
    pub zoom: i32,
    /// Raccourcis clavier ; en dernier : TOML exige les tables après les valeurs simples
    #[serde(rename = "raccourcis")]
    pub shortcuts: BTreeMap<String, Keys>,
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
            single_instance: true,
            brightness: 0,
            contrast: 0,
            saturation: 0,
            gamma: 0,
            hue: 0,
            zoom: 100,
            shortcuts: shortcuts::defaults(),
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
        for value in [&mut self.brightness, &mut self.contrast, &mut self.saturation, &mut self.gamma, &mut self.hue] {
            *value = (*value).clamp(-100, 100);
        }
        self.zoom = self.zoom.clamp(25, 400);
        // Liste complète dans le fichier : toutes les actions sont visibles et modifiables
        self.shortcuts = shortcuts::merged(&self.shortcuts);
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

pub fn settings_path() -> PathBuf {
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
    single_instance: bool,
    brightness: i32,
    contrast: i32,
    saturation: i32,
    gamma: i32,
    hue: i32,
    zoom: i32,
    shortcuts: BTreeMap<String, Keys>,
    resolved_shortcuts: Resolved,
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
            single_instance: f.single_instance,
            brightness: f.brightness,
            contrast: f.contrast,
            saturation: f.saturation,
            gamma: f.gamma,
            hue: f.hue,
            zoom: f.zoom,
            resolved_shortcuts: shortcuts::resolve(&f.shortcuts),
            shortcuts: f.shortcuts,
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
            single_instance: s.single_instance,
            brightness: s.brightness,
            contrast: s.contrast,
            saturation: s.saturation,
            gamma: s.gamma,
            hue: s.hue,
            zoom: s.zoom,
            shortcuts: s.shortcuts.clone(),
        }
    }
}

impl Default for SettingsRust {
    fn default() -> Self {
        SettingsFile::load(&settings_path()).into()
    }
}

impl qobject::Settings {
    pub fn keys(&self, action: &QString) -> QStringList {
        let mut list = QList::<QString>::default();
        for key in self.rust().resolved_shortcuts.keys.get(&action.to_string()).into_iter().flatten() {
            list.append(QString::from(key));
        }
        QStringList::from(&list)
    }

    pub fn shortcut_warnings(&self) -> QString {
        QString::from(&self.rust().resolved_shortcuts.warnings.join("\n"))
    }

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
            single_instance: false,
            brightness: 10,
            contrast: -5,
            saturation: 20,
            gamma: 0,
            hue: -3,
            zoom: 150,
            shortcuts: shortcuts::merged(&BTreeMap::from([("capture".to_string(), Keys::One("F9".into()))])),
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
        std::fs::write(
            &path,
            "volume = 400.0\nrepeat_mode = 7\nwindow_width = 10\nshuffle = true\nbrightness = 250\nzoom = 5\n",
        )
        .unwrap();
        let loaded = SettingsFile::load(&path);
        assert_eq!(loaded.volume, 130.0);
        assert_eq!(loaded.repeat_mode, 0);
        assert_eq!(loaded.window_width, 640);
        assert!(loaded.shuffle);
        assert_eq!((loaded.brightness, loaded.zoom), (100, 25));
        assert!(loaded.resume_playback && loaded.auto_playlist);

        // Fichier illisible : paramètres par défaut, sans planter
        std::fs::write(&path, "ceci n'est pas du TOML [[[").unwrap();
        assert_eq!(SettingsFile::load(&path), SettingsFile::default());

        // Fichier absent
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
        assert_eq!(SettingsFile::load(&path), SettingsFile::default());
    }
}
