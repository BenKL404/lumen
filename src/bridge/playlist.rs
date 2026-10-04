//! Playlist automatique : à l'ouverture d'une vidéo, les fichiers du même dossier
//! qui portent le même nom de base (« Ma Série - Épisode 01 », « Film - Partie 2 »…)
//! sont ajoutés et classés dans l'ordre naturel. La barre d'outils du panneau permet
//! ensuite de la réorganiser, d'ajouter, de retirer et de trier des fichiers.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_f64 = cxx_qt_lib::QList<f64>;

        include!("probe.h");
        /// Durée en secondes (-1 si inconnue), lue dans l'en-tête du fichier.
        fn lumen_probe_duration(path: &QString) -> f64;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        // `items` : URL des fichiers ; `durations` : durée de chacun (-1 si inconnue) ;
        // `current` : index du fichier en cours (-1 s'il a été retiré de la playlist) ;
        // `nextIndex` / `previousIndex` : fichier à lire ensuite / avant (-1 si aucun)
        #[qobject]
        #[qml_element]
        #[qproperty(QStringList, items)]
        #[qproperty(QList_f64, durations)]
        #[qproperty(i32, current)]
        #[qproperty(i32, next_index)]
        #[qproperty(i32, previous_index)]
        type Playlist = super::PlaylistRust;

        /// À appeler à chaque ouverture : se place sur le fichier s'il est déjà dans
        /// la playlist, sinon reconstruit la playlist depuis son dossier.
        #[qinvokable]
        fn load(self: Pin<&mut Playlist>, url: &QString);

        /// Ajoute des fichiers en fin de playlist (sans doublon).
        #[qinvokable]
        fn add(self: Pin<&mut Playlist>, urls: &QStringList);

        /// Retire un fichier de la playlist (le fichier sur le disque n'est pas touché).
        #[qinvokable]
        fn remove(self: Pin<&mut Playlist>, index: i32);

        /// Déplace un fichier à une autre position.
        #[qinvokable]
        fn move_item(self: Pin<&mut Playlist>, from: i32, to: i32);

        /// Trie la playlist par nom, dans l'ordre naturel.
        #[qinvokable]
        fn sort(self: Pin<&mut Playlist>);
    }

    impl cxx_qt::Threading for Playlist {}
}

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::LazyLock;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString, QStringList};
use regex::Regex;

use super::utils::local_path;

#[derive(Default)]
pub struct PlaylistRust {
    model: Items,
    /// Durées déjà lues, par chemin ; NaN tant que la lecture est en cours
    durations_cache: HashMap<String, f64>,
    items: QStringList,
    durations: QList<f64>,
    current: i32,
    next_index: i32,
    previous_index: i32,
}

impl qobject::Playlist {
    pub fn load(mut self: Pin<&mut Self>, url: &QString) {
        let key = local_path(&url.to_string());
        let rebuild = || {
            if key.starts_with('/') {
                build(Path::new(&key)).iter().map(|p| p.to_string_lossy().into_owned()).collect()
            } else {
                Vec::new() // flux réseau : pas de dossier à parcourir
            }
        };
        self.as_mut().rust_mut().model.open(&key, rebuild);
        self.as_mut().sync();
    }

    pub fn add(mut self: Pin<&mut Self>, urls: &QStringList) {
        let keys: Vec<String> =
            QList::<QString>::from(urls).iter().map(|u| local_path(&u.to_string())).collect();
        self.as_mut().rust_mut().model.add(keys);
        self.as_mut().sync();
    }

    pub fn remove(mut self: Pin<&mut Self>, index: i32) {
        if let Ok(index) = usize::try_from(index) {
            self.as_mut().rust_mut().model.remove(index);
            self.as_mut().sync();
        }
    }

    pub fn move_item(mut self: Pin<&mut Self>, from: i32, to: i32) {
        if let (Ok(from), Ok(to)) = (usize::try_from(from), usize::try_from(to)) {
            self.as_mut().rust_mut().model.move_item(from, to);
            self.as_mut().sync();
        }
    }

    pub fn sort(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().model.sort();
        self.as_mut().sync();
    }

    /// Recopie le modèle dans les propriétés QML et lance la lecture des durées manquantes.
    fn sync(mut self: Pin<&mut Self>) {
        let model = &self.rust().model;
        let mut items = QList::<QString>::default();
        for key in &model.keys {
            items.append(QString::from(&item_url(key)));
        }
        let index = |i: Option<usize>| i.map_or(-1, |i| i as i32);
        let (current, next, previous) =
            (index(model.current_index()), index(model.next_index()), index(model.previous_index()));

        self.as_mut().set_items(QStringList::from(&items));
        self.as_mut().set_current(current);
        self.as_mut().set_next_index(next);
        self.as_mut().set_previous_index(previous);
        self.as_mut().refresh_durations();
        self.as_mut().probe_missing();
    }

    fn refresh_durations(mut self: Pin<&mut Self>) {
        let rust = self.rust();
        let mut durations = QList::<f64>::default();
        for key in &rust.model.keys {
            let d = rust.durations_cache.get(key).copied().filter(|d| d.is_finite()).unwrap_or(-1.0);
            durations.append(d);
        }
        self.as_mut().set_durations(durations);
    }

    /// Lit en arrière-plan la durée des fichiers locaux pas encore connus.
    fn probe_missing(mut self: Pin<&mut Self>) {
        let missing: Vec<String> = {
            let rust = self.rust();
            rust.model
                .keys
                .iter()
                .filter(|k| k.starts_with('/') && !rust.durations_cache.contains_key(*k))
                .cloned()
                .collect()
        };
        if missing.is_empty() {
            return;
        }
        for key in &missing {
            self.as_mut().rust_mut().durations_cache.insert(key.clone(), f64::NAN);
        }
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            for key in missing {
                let duration = qobject::lumen_probe_duration(&QString::from(&key));
                let queued = thread.queue(move |mut playlist| {
                    playlist.as_mut().rust_mut().durations_cache.insert(key, duration);
                    playlist.as_mut().refresh_durations();
                });
                if queued.is_err() {
                    break; // la playlist a été détruite (fermeture de Lumen)
                }
            }
        });
    }
}

fn item_url(key: &str) -> String {
    if key.starts_with('/') { file_url(key) } else { key.to_string() }
}

/// Contenu de la playlist et fichier en cours, indépendamment de Qt.
#[derive(Default, Debug)]
pub struct Items {
    /// Chemins locaux (ou URL pour un flux réseau), dans l'ordre de lecture
    pub keys: Vec<String>,
    /// Fichier en cours, suivi par son chemin pour survivre aux déplacements
    current: Option<String>,
    /// Ancienne position du fichier en cours s'il a été retiré :
    /// la lecture continue alors avec le fichier qui a pris sa place
    anchor: Option<usize>,
}

impl Items {
    /// Se place sur `key`, en reconstruisant la playlist s'il n'y figure pas.
    pub fn open(&mut self, key: &str, rebuild: impl FnOnce() -> Vec<String>) {
        if !self.keys.iter().any(|k| k == key) {
            self.keys = rebuild();
            if !self.keys.iter().any(|k| k == key) {
                self.keys.push(key.to_string());
            }
        }
        self.current = Some(key.to_string());
        self.anchor = None;
    }

    pub fn add(&mut self, keys: Vec<String>) {
        for key in keys {
            if !self.keys.contains(&key) {
                self.keys.push(key);
            }
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.keys.len() {
            return;
        }
        let key = self.keys.remove(index);
        if self.current.as_deref() == Some(key.as_str()) {
            self.current = None;
            self.anchor = Some(index);
        } else if let Some(anchor) = self.anchor.as_mut().filter(|a| **a > index) {
            *anchor -= 1;
        }
    }

    pub fn move_item(&mut self, from: usize, to: usize) {
        if from < self.keys.len() && to < self.keys.len() && from != to {
            let key = self.keys.remove(from);
            self.keys.insert(to, key);
            self.anchor = None;
        }
    }

    pub fn sort(&mut self) {
        let name = |k: &String| k.rsplit('/').next().unwrap_or(k).to_string();
        self.keys.sort_by(|a, b| natural_cmp(&name(a), &name(b)));
        self.anchor = None;
    }

    pub fn current_index(&self) -> Option<usize> {
        let current = self.current.as_ref()?;
        self.keys.iter().position(|k| k == current)
    }

    pub fn next_index(&self) -> Option<usize> {
        let len = self.keys.len();
        match self.current_index() {
            Some(c) => (c + 1 < len).then_some(c + 1),
            None => self.anchor.filter(|&a| a < len),
        }
    }

    pub fn previous_index(&self) -> Option<usize> {
        match self.current_index() {
            Some(c) => c.checked_sub(1),
            None => self.anchor.and_then(|a| a.checked_sub(1)).filter(|&p| p < self.keys.len()),
        }
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

    fn items(names: &[&str], current: &str) -> Items {
        let mut items = Items::default();
        items.open(current, || names.iter().map(|n| n.to_string()).collect());
        items
    }

    #[test]
    fn tracks_current_file() {
        let mut list = items(&["a", "b", "c"], "b");
        assert_eq!((list.previous_index(), list.current_index(), list.next_index()), (Some(0), Some(1), Some(2)));

        // Déplacer le fichier en cours : il reste le fichier en cours
        list.move_item(1, 2);
        assert_eq!(list.keys, ["a", "c", "b"]);
        assert_eq!((list.current_index(), list.next_index()), (Some(2), None));

        list.sort();
        assert_eq!(list.keys, ["a", "b", "c"]);
        assert_eq!(list.current_index(), Some(1));

        // Ouvrir un fichier déjà présent ne reconstruit pas la playlist
        list.open("c", || panic!("ne doit pas reconstruire"));
        assert_eq!(list.current_index(), Some(2));
    }

    #[test]
    fn continues_after_removing_current() {
        let mut list = items(&["a", "b", "c", "d"], "b");
        list.remove(1);
        assert_eq!(list.keys, ["a", "c", "d"]);
        assert_eq!(list.current_index(), None);
        // Le suivant est le fichier qui a pris sa place, le précédent ne change pas
        assert_eq!((list.previous_index(), list.next_index()), (Some(0), Some(1)));

        list.remove(0);
        assert_eq!((list.previous_index(), list.next_index()), (None, Some(0)));

        let mut last = items(&["a", "b"], "b");
        last.remove(1);
        assert_eq!((last.previous_index(), last.next_index()), (Some(0), None));
    }

    #[test]
    fn adds_without_duplicates() {
        let mut list = items(&["a"], "a");
        list.add(vec!["b".into(), "a".into(), "b".into()]);
        assert_eq!(list.keys, ["a", "b"]);

        // Flux réseau : la reconstruction ne donne rien, le flux est ajouté seul
        let mut stream = Items::default();
        stream.open("https://exemple.org/v.mp4", Vec::new);
        assert_eq!(stream.keys, ["https://exemple.org/v.mp4"]);
    }

    #[test]
    fn encodes_file_urls() {
        let path = "/films/Ma Série #1 (VF) 100%.mkv";
        let url = file_url(path);
        assert_eq!(url, "file:///films/Ma%20S%C3%A9rie%20%231%20%28VF%29%20100%25.mkv");
        assert_eq!(local_path(&url), path);
    }
}
