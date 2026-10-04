//! Playlist automatique : à l'ouverture d'une vidéo, les fichiers du même dossier
//! qui portent le même nom de base (« Ma Série - Épisode 01 », « Film - Partie 2 »…)
//! sont ajoutés et classés dans l'ordre naturel.

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
        // `items` : URL file:// des fichiers ; `current` : index du fichier en cours
        #[qobject]
        #[qml_element]
        #[qproperty(QStringList, items)]
        #[qproperty(i32, current)]
        type Playlist = super::PlaylistRust;

        /// À appeler à chaque ouverture : se place sur le fichier s'il est déjà dans
        /// la playlist, sinon reconstruit la playlist depuis son dossier.
        #[qinvokable]
        fn load(self: Pin<&mut Playlist>, url: &QString);
    }
}

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::LazyLock;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList};
use regex::Regex;

use super::utils::local_path;

#[derive(Default)]
pub struct PlaylistRust {
    items: QStringList,
    current: i32,
    /// Chemins locaux correspondant à `items`, pour les comparaisons
    paths: Vec<String>,
}

impl qobject::Playlist {
    pub fn load(mut self: Pin<&mut Self>, url: &QString) {
        let url = url.to_string();
        let path = local_path(&url);

        if let Some(index) = self.rust().paths.iter().position(|p| *p == path) {
            self.as_mut().set_current(index as i32);
            return;
        }

        let paths: Vec<String> = if path.starts_with('/') {
            build(Path::new(&path)).iter().map(|p| p.to_string_lossy().into_owned()).collect()
        } else {
            vec![path.clone()] // flux réseau : pas de dossier à parcourir
        };
        let mut items = QList::<QString>::default();
        for p in &paths {
            let item = if p.starts_with('/') { file_url(p) } else { url.clone() };
            items.append(QString::from(&item));
        }
        let current = paths.iter().position(|p| *p == path).unwrap_or(0) as i32;

        self.as_mut().rust_mut().paths = paths;
        self.as_mut().set_items(QStringList::from(&items));
        self.as_mut().set_current(current);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum MediaKind {
    Video,
    Audio,
}

const VIDEO_EXTENSIONS: &[&str] = &[
    "mkv", "mp4", "m4v", "avi", "webm", "mov", "wmv", "flv", "ts", "m2ts", "mts", "mpg", "mpeg", "ogv",
];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "opus", "wav", "m4a", "aac", "wma"];

fn media_kind(path: &Path) -> Option<MediaKind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        Some(MediaKind::Video)
    } else if AUDIO_EXTENSIONS.contains(&ext.as_str()) {
        Some(MediaKind::Audio)
    } else {
        None
    }
}

// « S01E02 », « s1.e2 »
static SEASON_EPISODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bs\d{1,2}[ ._-]?e\d{1,3}").unwrap());
// « Épisode 2 », « Ep.02 », « E02 », « Partie 2 », « Part 2 », « CD2 », « Disc 1 »…
static MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:épisode|episode|ep|e|partie|part|pt|cd|disc|disque|dvd|vol|volume|tome|chapitre|chapter)[ ._-]*\d{1,4}\b",
    )
    .unwrap()
});
// Dernier nombre isolé : « Naruto - 001 [1080p] » (1080p n'est pas isolé)
static LONE_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|[^\p{L}\p{N}])(\d{1,4})(?:[^\p{L}\p{N}]|$)").unwrap());
static SEPARATORS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s._-]+").unwrap());

/// Nom de base partagé par les fichiers d'une même série, ou None si le nom
/// ne contient pas de numéro d'épisode ou de partie.
pub fn series_key(stem: &str) -> Option<String> {
    let start = SEASON_EPISODE
        .find(stem)
        .or_else(|| MARKER.find(stem))
        .map(|m| m.start())
        .or_else(|| {
            LONE_NUMBER
                .captures_iter(stem)
                .filter_map(|c| c.get(1))
                .filter(|m| !is_year(m.as_str()))
                .last()
                .map(|m| m.start())
        })?;

    let prefix = SEPARATORS.replace_all(&stem[..start], " ").to_lowercase();
    Some(prefix.trim_end_matches([' ', '(', '[']).trim().to_string())
}

fn is_year(digits: &str) -> bool {
    digits.len() == 4 && matches!(digits.parse::<u32>(), Ok(1900..=2099))
}

/// Comparaison « naturelle » : « Épisode 2 » avant « Épisode 10 ».
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (a.peek().copied(), b.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let na = take_number(&mut a);
                let nb = take_number(&mut b);
                let ord = na.len().cmp(&nb.len()).then_with(|| na.cmp(&nb));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(x), Some(y)) => {
                let ord = x.to_lowercase().cmp(y.to_lowercase());
                if ord != Ordering::Equal {
                    return ord;
                }
                a.next();
                b.next();
            }
        }
    }
}

/// Lit une suite de chiffres, sans les zéros de tête (« 007 » -> « 7 »).
fn take_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut digits = String::new();
    while let Some(c) = chars.peek().copied().filter(char::is_ascii_digit) {
        digits.push(c);
        chars.next();
    }
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() { "0".into() } else { trimmed.into() }
}

/// Fichiers de la même série que `path` dans son dossier, dans l'ordre naturel.
/// Contient toujours `path`, même seul.
pub fn build(path: &Path) -> Vec<PathBuf> {
    let alone = || vec![path.to_path_buf()];
    let (Some(dir), Some(kind), Some(key)) = (
        path.parent(),
        media_kind(path),
        path.file_stem().and_then(|s| s.to_str()).and_then(series_key),
    ) else {
        return alone();
    };
    let Ok(entries) = std::fs::read_dir(dir) else { return alone() };

    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && media_kind(p) == Some(kind))
        .filter(|p| {
            p.file_stem().and_then(|s| s.to_str()).and_then(series_key).as_deref() == Some(key.as_str())
        })
        .collect();
    if !files.iter().any(|p| p == path) {
        files.push(path.to_path_buf());
    }
    files.sort_by(|a, b| {
        let name = |p: &PathBuf| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        natural_cmp(&name(a), &name(b))
    });
    files
}

/// URL file:// encodée, qui redonne le même chemin avec `local_path`.
pub fn file_url(path: &str) -> String {
    let mut url = String::from("file://");
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
            url.push(byte as char);
        } else {
            url.push_str(&format!("%{byte:02X}"));
        }
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_series_keys() {
        let same = |a: &str, b: &str| {
            let (ka, kb) = (series_key(a), series_key(b));
            assert!(ka.is_some() && ka == kb, "{a:?} -> {ka:?}, {b:?} -> {kb:?}");
        };
        same("Ma Série - Épisode 01", "Ma Série - Épisode 02");
        same("Ma Série - Episode 9", "Ma Série - épisode 10");
        same("Film - Partie 1", "Film - Partie 2");
        same("Film Part 1", "Film Part.2");
        same("Show.S01E02.1080p.WEB", "Show.S01E10.720p.HDTV");
        same("Show S01E09 - Pilot", "Show S02E01 - Return");
        same("Show - E02", "Show - E03");
        same("Show Ep 2", "Show Ep.10");
        same("Film CD1", "Film CD2");
        same("[Grp] Naruto - 001 [1080p]", "[Grp] Naruto - 002 [1080p]");
        same("Cours 01", "Cours 12");

        assert_ne!(series_key("Ma Série - Épisode 01"), series_key("Autre Série - Épisode 01"));
        assert_eq!(series_key("Un film"), None);
        assert_eq!(series_key("Un film 2019"), None); // une année n'est pas un épisode
        assert_eq!(series_key("Film 1080p x264"), None);
    }

    #[test]
    fn sorts_naturally() {
        let mut names = vec!["Ép 10.mkv", "Ép 2.mkv", "ép 1.mkv", "Ép 02b.mkv", "Ép 007.mkv"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, vec!["ép 1.mkv", "Ép 2.mkv", "Ép 02b.mkv", "Ép 007.mkv", "Ép 10.mkv"]);
        assert_eq!(natural_cmp("S01E09", "S01E10"), Ordering::Less);
        assert_eq!(natural_cmp("S02E01", "S01E10"), Ordering::Greater);
    }

    #[test]
    fn builds_playlist_from_folder() {
        let dir = std::env::temp_dir().join(format!("lumen-playlist-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for name in [
            "Série - Épisode 10.mkv",
            "Série - Épisode 2.mkv",
            "Série - Épisode 1.mp4",
            "Série - Épisode 3.srt", // pas une vidéo
            "Autre - Épisode 1.mkv", // autre série
            "Série - Bonus.mkv",     // pas de numéro
        ] {
            std::fs::write(dir.join(name), b"").unwrap();
        }

        let names: Vec<String> = build(&dir.join("Série - Épisode 2.mkv"))
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["Série - Épisode 1.mp4", "Série - Épisode 2.mkv", "Série - Épisode 10.mkv"]);

        let alone = build(&dir.join("Série - Bonus.mkv"));
        assert_eq!(alone, vec![dir.join("Série - Bonus.mkv")]);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn encodes_file_urls() {
        let path = "/films/Ma Série #1 (VF) 100%.mkv";
        let url = file_url(path);
        assert_eq!(url, "file:///films/Ma%20S%C3%A9rie%20%231%20%28VF%29%20100%25.mkv");
        assert_eq!(local_path(&url), path);
    }
}
