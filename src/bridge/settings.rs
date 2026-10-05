//! Paramètres de Lumen, conservés d'une session à l'autre dans
//! ~/.config/lumen/settings.toml (lisible et modifiable à la main, Lumen fermé).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_i32 = cxx_qt_lib::QList<i32>;
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
        // Égaliseur (10 gains en dB) et normalisation du volume
        #[qproperty(QList_i32, equalizer)]
        #[qproperty(bool, normalize_volume)]
        // Préférences (fenêtre F5)
        #[qproperty(bool, show_osd)]
        #[qproperty(i32, seek_short)]
        #[qproperty(i32, seek_long)]
        #[qproperty(bool, hardware_decoding)]
        #[qproperty(QString, audio_languages)]
        #[qproperty(QString, subtitle_languages)]
        #[qproperty(i32, subtitle_scale)]
        // Filtres d'image : netteté 0…100, agrandissement 0 standard / 1 haute qualité / 2 FSR
        #[qproperty(i32, sharpness)]
        #[qproperty(i32, upscaler)]
        // Clé d'API OpenSubtitles (recherche de sous-titres en ligne)
        #[qproperty(QString, opensubtitles_api_key)]
        // Apparence : thème (« dark », « light », « oled », « skin:<nom> »), accent (#RRGGBB,
        // vide : celui du thème), taille de l'interface en % (au prochain lancement)
        #[qproperty(QString, theme)]
        #[qproperty(QString, accent)]
        #[qproperty(i32, ui_scale)]
        // Change à chaque rechargement des raccourcis : les liaisons QML les relisent
        #[qproperty(i32, shortcuts_version, READ, NOTIFY)]
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

        /// Identifiants des actions, dans l'ordre d'affichage.
        #[qinvokable]
        fn shortcut_actions(self: &Settings) -> QStringList;

        /// Nom affiché d'une action (« Plein écran »…).
        #[qinvokable]
        fn action_label(self: &Settings, action: &QString) -> QString;

        /// Relit la section [raccourcis] du fichier (modifiée à la main, Lumen ouvert).
        #[qinvokable]
        fn reload_shortcuts(self: Pin<&mut Settings>);

        /// Adresse file:// de settings.toml, pour l'ouvrir dans un éditeur.
        #[qinvokable]
        fn file_url(self: &Settings) -> QString;
    }
}

use std::path::{Path, PathBuf};
use std::pin::Pin;

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
    /// Égaliseur : gain des 10 bandes en dB, de -12 à 12 (voir audio.rs)
    pub equalizer: Vec<i32>,
    /// Normalisation dynamique du volume (dialogues plus audibles, explosions retenues)
    pub normalize_volume: bool,
    /// Messages d'action à l'écran (volume, saut, vitesse…)
    pub show_osd: bool,
    /// Durée des sauts, en secondes : flèches, et Ctrl+flèches
    pub seek_short: i32,
    pub seek_long: i32,
    /// Décodage matériel (VA-API, NVDEC…) : moins de processeur et de batterie
    pub hardware_decoding: bool,
    /// Langues préférées, codes séparés par des virgules (« fr,en ») ; vide : celles du fichier
    pub audio_languages: String,
    pub subtitle_languages: String,
    /// Taille des sous-titres en % (100 : taille normale)
    pub subtitle_scale: i32,
    /// Netteté (AMD CAS), de 0 (désactivée) à 100
    pub sharpness: i32,
    /// Agrandissement : 0 standard, 1 haute qualité, 2 AMD FSR (voir shaders.rs)
    pub upscaler: i32,
    /// Clé d'API OpenSubtitles (gratuite, compte opensubtitles.com › API consumers)
    pub opensubtitles_api_key: String,
    /// Thème : « dark », « light », « oled » ou « skin:<fichier> » (voir themes.rs)
    pub theme: String,
    /// Couleur d'accent « #RRGGBB » ; vide : celle du thème
    pub accent: String,
    /// Taille de l'interface en % (90, 100, 115, 130), appliquée au lancement
    pub ui_scale: i32,
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
            equalizer: vec![0; 10],
            normalize_volume: false,
            show_osd: true,
            seek_short: 5,
            seek_long: 30,
            hardware_decoding: true,
            audio_languages: String::new(),
            subtitle_languages: String::new(),
            subtitle_scale: 100,
            sharpness: 0,
            upscaler: 0,
            opensubtitles_api_key: String::new(),
            theme: "dark".into(),
            accent: String::new(),
            ui_scale: 100,
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
        self.equalizer = super::audio::sanitized(&self.equalizer);
        self.seek_short = self.seek_short.clamp(1, 60);
        self.seek_long = self.seek_long.clamp(5, 600);
        self.subtitle_scale = self.subtitle_scale.clamp(50, 300);
        self.sharpness = self.sharpness.clamp(0, 100);
        self.opensubtitles_api_key = self.opensubtitles_api_key.trim().to_string();
        self.ui_scale = self.ui_scale.clamp(75, 200);
        if !super::themes::is_color(self.accent.trim()) {
            self.accent = String::new();
        }
        if !(0..=2).contains(&self.upscaler) {
            self.upscaler = 0;
        }
        self.audio_languages = language_list(&self.audio_languages);
        self.subtitle_languages = language_list(&self.subtitle_languages);
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

/// Liste de langues propre : « FR, en ,, » -> « fr,en » (format attendu par mpv)
pub fn language_list(text: &str) -> String {
    text.split([',', ' ', ';'])
        .map(|code| code.trim().to_lowercase())
        .filter(|code| !code.is_empty() && code.chars().all(|c| c.is_ascii_alphabetic() || c == '-'))
        .collect::<Vec<_>>()
        .join(",")
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
    equalizer: QList<i32>,
    normalize_volume: bool,
    show_osd: bool,
    seek_short: i32,
    seek_long: i32,
    hardware_decoding: bool,
    audio_languages: QString,
    subtitle_languages: QString,
    subtitle_scale: i32,
    sharpness: i32,
    upscaler: i32,
    opensubtitles_api_key: QString,
    theme: QString,
    accent: QString,
    ui_scale: i32,
    shortcuts: BTreeMap<String, Keys>,
    resolved_shortcuts: Resolved,
    shortcuts_version: i32,
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
            equalizer: {
                let mut list = QList::<i32>::default();
                for gain in &f.equalizer {
                    list.append(*gain);
                }
                list
            },
            normalize_volume: f.normalize_volume,
            show_osd: f.show_osd,
            seek_short: f.seek_short,
            seek_long: f.seek_long,
            hardware_decoding: f.hardware_decoding,
            audio_languages: QString::from(&f.audio_languages),
            subtitle_languages: QString::from(&f.subtitle_languages),
            subtitle_scale: f.subtitle_scale,
            sharpness: f.sharpness,
            upscaler: f.upscaler,
            opensubtitles_api_key: QString::from(&f.opensubtitles_api_key),
            theme: QString::from(&f.theme),
            accent: QString::from(&f.accent),
            ui_scale: f.ui_scale,
            resolved_shortcuts: shortcuts::resolve(&f.shortcuts),
            shortcuts: f.shortcuts,
            shortcuts_version: 0,
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
            equalizer: s.equalizer.iter().copied().collect(),
            normalize_volume: s.normalize_volume,
            show_osd: s.show_osd,
            seek_short: s.seek_short,
            seek_long: s.seek_long,
            hardware_decoding: s.hardware_decoding,
            audio_languages: s.audio_languages.to_string(),
            subtitle_languages: s.subtitle_languages.to_string(),
            subtitle_scale: s.subtitle_scale,
            sharpness: s.sharpness,
            upscaler: s.upscaler,
            opensubtitles_api_key: s.opensubtitles_api_key.to_string(),
            theme: s.theme.to_string(),
            accent: s.accent.to_string(),
            ui_scale: s.ui_scale,
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

    pub fn shortcut_actions(&self) -> QStringList {
        let mut list = QList::<QString>::default();
        for (action, _, _) in shortcuts::DEFAULTS {
            list.append(QString::from(*action));
        }
        QStringList::from(&list)
    }

    pub fn action_label(&self, action: &QString) -> QString {
        QString::from(shortcuts::label(&action.to_string()))
    }

    pub fn reload_shortcuts(mut self: Pin<&mut Self>) {
        let map = SettingsFile::load(&settings_path()).shortcuts;
        let resolved = shortcuts::resolve(&map);
        let version = self.rust().shortcuts_version + 1;
        let mut rust = self.as_mut().rust_mut();
        rust.shortcuts = map;
        rust.resolved_shortcuts = resolved;
        rust.shortcuts_version = version;
        self.as_mut().shortcuts_version_changed();
    }

    pub fn file_url(&self) -> QString {
        QString::from(&super::playlist::file_url(&settings_path().to_string_lossy()))
    }

    pub fn save(&self) {
        let mut file = SettingsFile::from(self.rust()).sanitized();
        // Les raccourcis ne se modifient que dans le fichier : garder ceux du fichier,
        // pour ne pas écraser des changements faits pendant que Lumen est ouvert
        if settings_path().exists() {
            file.shortcuts = SettingsFile::load(&settings_path()).shortcuts;
        }
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
            equalizer: vec![3, 2, 0, 0, 0, 0, 0, 0, -1, -2],
            normalize_volume: true,
            show_osd: false,
            seek_short: 10,
            seek_long: 60,
            hardware_decoding: false,
            audio_languages: "fr,en".into(),
            subtitle_languages: "fr".into(),
            subtitle_scale: 120,
            sharpness: 40,
            upscaler: 2,
            opensubtitles_api_key: "abc123".into(),
            theme: "light".into(),
            accent: "#2D7FF9".into(),
            ui_scale: 115,
            shortcuts: shortcuts::merged(&BTreeMap::from([("capture".to_string(), Keys::One("F9".into()))])),
        };
        settings.store(&path).unwrap(); // crée aussi le dossier
        assert_eq!(SettingsFile::load(&path), settings);
        assert!(!path.with_extension("toml.tmp").exists());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn cleans_language_lists() {
        assert_eq!(language_list("FR, en ,, "), "fr,en");
        assert_eq!(language_list("fre;eng jpn"), "fre,eng,jpn");
        assert_eq!(language_list("pt-BR, <script>"), "pt-br");
        assert_eq!(language_list(""), "");
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
        assert!(loaded.show_osd && loaded.hardware_decoding);
        assert_eq!((loaded.seek_short, loaded.seek_long, loaded.subtitle_scale), (5, 30, 100));
        assert!(loaded.resume_playback && loaded.auto_playlist);

        // Fichier illisible : paramètres par défaut, sans planter
        std::fs::write(&path, "ceci n'est pas du TOML [[[").unwrap();
        assert_eq!(SettingsFile::load(&path), SettingsFile::default());

        // Fichier absent
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
        assert_eq!(SettingsFile::load(&path), SettingsFile::default());
    }
}
