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
        // `repeatMode` : 0 désactivé, 1 le fichier, 2 la playlist
        #[qproperty(i32, repeat_mode, READ, NOTIFY)]
        #[qproperty(bool, shuffle, READ, NOTIFY)]
        // Faux : ouvrir un fichier ne lui ajoute pas les fichiers du même nom
        #[qproperty(bool, auto_build)]
        type Playlist = super::PlaylistRust;

        /// Règle la répétition (0 désactivée, 1 le fichier, 2 la playlist).
        #[qinvokable]
        fn set_repeat(self: Pin<&mut Playlist>, mode: i32);

        /// Active ou coupe la lecture aléatoire.
        #[qinvokable]
        fn set_shuffle_enabled(self: Pin<&mut Playlist>, on: bool);

        /// Remplace la playlist par les fichiers audio et vidéo d'un dossier.
        /// Renvoie l'URL du premier fichier à lire (vide si le dossier n'en contient pas).
        #[qinvokable]
        fn open_folder(self: Pin<&mut Playlist>, url: &QString) -> QString;

        /// Désactivé → répéter le fichier → répéter la playlist → désactivé
        #[qinvokable]
        fn cycle_repeat(self: Pin<&mut Playlist>);

        #[qinvokable]
        fn toggle_shuffle(self: Pin<&mut Playlist>);

        /// À appeler à chaque ouverture : se place sur le fichier s'il est déjà dans
        /// la playlist, sinon reconstruit la playlist depuis son dossier.
        #[qinvokable]
        fn load(self: Pin<&mut Playlist>, url: &QString);

        /// Ajoute des fichiers en fin de playlist (sans doublon).
        #[qinvokable]
        fn add(self: Pin<&mut Playlist>, urls: &QStringList);

        /// Ajoute en fin de playlist les fichiers audio et vidéo d'un dossier.
        #[qinvokable]
        fn add_folder(self: Pin<&mut Playlist>, url: &QString);

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
    repeat_mode: i32,
    shuffle: bool,
    auto_build: bool,
}

impl qobject::Playlist {
    pub fn load(mut self: Pin<&mut Self>, url: &QString) {
        let key = local_path(&url.to_string());
        let auto_build = self.rust().auto_build;
        let rebuild = || {
            if auto_build && key.starts_with('/') {
                build(Path::new(&key)).iter().map(|p| p.to_string_lossy().into_owned()).collect()
            } else {
                Vec::new() // flux réseau, ou playlist automatique désactivée : le fichier seul
            }
        };
        self.as_mut().rust_mut().model.open(&key, rebuild);
        self.as_mut().sync();
    }

    pub fn open_folder(mut self: Pin<&mut Self>, url: &QString) -> QString {
        let dir = local_path(&url.to_string());
        let keys = folder_files(Path::new(&dir)).iter().map(|p| p.to_string_lossy().into_owned()).collect();
        let first = self.as_mut().rust_mut().model.replace(keys);
        self.as_mut().sync();
        QString::from(&first.map(|k| item_url(&k)).unwrap_or_default())
    }

    pub fn set_repeat(mut self: Pin<&mut Self>, mode: i32) {
        let repeat = match mode {
            1 => Repeat::One,
            2 => Repeat::All,
            _ => Repeat::Off,
        };
        self.as_mut().rust_mut().model.repeat = repeat;
        self.as_mut().sync();
    }

    pub fn set_shuffle_enabled(mut self: Pin<&mut Self>, on: bool) {
        if self.rust().model.shuffle() != on {
            self.as_mut().rust_mut().model.set_shuffle(on);
            self.as_mut().sync();
        }
    }

    pub fn cycle_repeat(mut self: Pin<&mut Self>) {
        let repeat = self.rust().model.repeat.next();
        self.as_mut().rust_mut().model.repeat = repeat;
        self.as_mut().sync();
    }

    pub fn toggle_shuffle(mut self: Pin<&mut Self>) {
        let on = !self.rust().model.shuffle();
        self.as_mut().rust_mut().model.set_shuffle(on);
        self.as_mut().sync();
    }

    pub fn add(mut self: Pin<&mut Self>, urls: &QStringList) {
        let keys: Vec<String> =
            QList::<QString>::from(urls).iter().map(|u| local_path(&u.to_string())).collect();
        self.as_mut().rust_mut().model.add(keys);
        self.as_mut().sync();
    }

    pub fn add_folder(mut self: Pin<&mut Self>, url: &QString) {
        let dir = local_path(&url.to_string());
        let keys = folder_files(Path::new(&dir)).iter().map(|p| p.to_string_lossy().into_owned()).collect();
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
        let (repeat, shuffle) = (model.repeat as i32, model.shuffle());

        self.as_mut().set_items(QStringList::from(&items));
        self.as_mut().set_current(current);
        self.as_mut().set_next_index(next);
        self.as_mut().set_previous_index(previous);
        if self.rust().repeat_mode != repeat {
            self.as_mut().rust_mut().repeat_mode = repeat;
            self.as_mut().repeat_mode_changed();
        }
        if self.rust().shuffle != shuffle {
            self.as_mut().rust_mut().shuffle = shuffle;
            self.as_mut().shuffle_changed();
        }
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

/// Mode de répétition (valeurs exposées à QML).
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Repeat {
    #[default]
    Off = 0,
    /// Le fichier en cours, en boucle (géré par mpv : `loop-file`)
    One = 1,
    /// Toute la playlist : après le dernier fichier, retour au premier
    All = 2,
}

impl Repeat {
    pub fn next(self) -> Self {
        match self {
            Repeat::Off => Repeat::One,
            Repeat::One => Repeat::All,
            Repeat::All => Repeat::Off,
        }
    }
}

/// Contenu de la playlist et fichier en cours, indépendamment de Qt.
#[derive(Default, Debug)]
pub struct Items {
    /// Chemins locaux (ou URL pour un flux réseau), dans l'ordre d'affichage
    pub keys: Vec<String>,
    /// Fichier en cours, suivi par son chemin pour survivre aux déplacements
    current: Option<String>,
    /// Ancienne position (dans l'ordre de lecture) du fichier en cours s'il a été
    /// retiré : la lecture continue alors avec le fichier qui a pris sa place
    anchor: Option<usize>,
    pub repeat: Repeat,
    /// Ordre de lecture mélangé, quand la lecture aléatoire est active
    shuffle_order: Option<Vec<String>>,
    seed: u64,
}

impl Items {
    /// Se place sur `key`, en reconstruisant la playlist s'il n'y figure pas.
    pub fn open(&mut self, key: &str, rebuild: impl FnOnce() -> Vec<String>) {
        if !self.keys.iter().any(|k| k == key) {
            self.keys = rebuild();
            if !self.keys.iter().any(|k| k == key) {
                self.keys.push(key.to_string());
            }
            self.current = Some(key.to_string());
            self.reshuffle();
        }
        self.current = Some(key.to_string());
        self.anchor = None;
    }

    /// Remplace la playlist par `keys` (ouverture d'un dossier) et renvoie
    /// le premier fichier à lire, selon l'ordre de lecture.
    pub fn replace(&mut self, keys: Vec<String>) -> Option<String> {
        self.keys = keys;
        self.current = None;
        self.anchor = None;
        self.reshuffle();
        self.order().first().cloned()
    }

    pub fn add(&mut self, keys: Vec<String>) {
        let mut added = Vec::new();
        for key in keys {
            if !self.keys.contains(&key) {
                self.keys.push(key.clone());
                added.push(key);
            }
        }
        if self.shuffle_order.is_some() {
            self.shuffle_vec(&mut added);
            self.shuffle_order.get_or_insert_with(Vec::new).extend(added);
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.keys.len() {
            return;
        }
        let key = self.keys[index].clone();
        let Some(position) = self.order().iter().position(|k| *k == key) else { return };
        self.keys.remove(index);
        if let Some(order) = self.shuffle_order.as_mut() {
            order.remove(position);
        }
        if self.current.as_deref() == Some(key.as_str()) {
            self.current = None;
            self.anchor = Some(position);
        } else if let Some(anchor) = self.anchor.as_mut().filter(|a| **a > position) {
            *anchor -= 1;
        }
    }

    pub fn move_item(&mut self, from: usize, to: usize) {
        if from < self.keys.len() && to < self.keys.len() && from != to {
            let key = self.keys.remove(from);
            self.keys.insert(to, key);
            if self.shuffle_order.is_none() {
                self.anchor = None;
            }
        }
    }

    pub fn sort(&mut self) {
        let name = |k: &String| k.rsplit('/').next().unwrap_or(k).to_string();
        self.keys.sort_by(|a, b| natural_cmp(&name(a), &name(b)));
        if self.shuffle_order.is_none() {
            self.anchor = None;
        }
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle_order.is_some()
    }

    /// Active ou coupe la lecture aléatoire. À l'activation, le fichier en cours
    /// reste le premier de l'ordre mélangé : tous les autres passeront ensuite.
    pub fn set_shuffle(&mut self, on: bool) {
        self.anchor = None;
        if on {
            self.shuffle_order = Some(Vec::new());
            self.reshuffle();
        } else {
            self.shuffle_order = None;
        }
    }

    fn reshuffle(&mut self) {
        if self.shuffle_order.is_none() {
            return;
        }
        let current = self.current.clone().filter(|c| self.keys.contains(c));
        let mut rest: Vec<String> =
            self.keys.iter().filter(|k| Some(*k) != current.as_ref()).cloned().collect();
        self.shuffle_vec(&mut rest);
        self.shuffle_order = Some(current.into_iter().chain(rest).collect());
    }

    /// Mélange de Fisher-Yates (xorshift : pas besoin d'un hasard cryptographique).
    fn shuffle_vec(&mut self, items: &mut [String]) {
        for i in (1..items.len()).rev() {
            let j = (self.next_random() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }

    fn next_random(&mut self) -> u64 {
        if self.seed == 0 {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(1, |d| d.as_nanos() as u64);
            self.seed = nanos | 1;
        }
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }

    #[cfg(test)]
    fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Ordre de lecture : mélangé en aléatoire, sinon celui de la liste.
    fn order(&self) -> &[String] {
        self.shuffle_order.as_deref().unwrap_or(&self.keys)
    }

    pub fn current_index(&self) -> Option<usize> {
        let current = self.current.as_ref()?;
        self.keys.iter().position(|k| k == current)
    }

    /// Index (dans la liste) du fichier suivant ou précédent dans l'ordre de lecture.
    fn step(&self, forward: bool) -> Option<usize> {
        let order = self.order();
        let len = order.len();
        if len == 0 {
            return None;
        }
        let wrap = self.repeat == Repeat::All;
        let position = self.current.as_ref().and_then(|c| order.iter().position(|k| k == c));
        let target = match (position, self.anchor) {
            (Some(p), _) if forward => (p + 1 < len).then_some(p + 1).or(wrap.then_some(0)),
            (Some(p), _) => p.checked_sub(1).or(wrap.then_some(len - 1)),
            // Fichier en cours retiré : continuer avec celui qui a pris sa place
            (None, Some(a)) if forward => (a < len).then_some(a).or(wrap.then_some(0)),
            (None, Some(a)) => a.checked_sub(1).filter(|&p| p < len).or(wrap.then_some(len - 1)),
            (None, None) => None,
        }?;
        self.keys.iter().position(|k| *k == order[target])
    }

    pub fn next_index(&self) -> Option<usize> {
        self.step(true)
    }

    pub fn previous_index(&self) -> Option<usize> {
        self.step(false)
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

/// Fichiers audio et vidéo d'un dossier (sans les sous-dossiers), dans l'ordre naturel.
pub fn folder_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && media_kind(p).is_some())
        .collect();
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

    /// Fichiers visités en appuyant sur « suivant » depuis le fichier en cours
    fn play_through(list: &mut Items) -> Vec<String> {
        let mut visited = vec![list.keys[list.current_index().unwrap()].clone()];
        while let Some(next) = list.next_index() {
            let key = list.keys[next].clone();
            if visited.contains(&key) {
                break;
            }
            list.open(&key, || unreachable!());
            visited.push(key);
        }
        visited
    }

    #[test]
    fn repeats_playlist() {
        let mut list = items(&["a", "b", "c"], "c");
        assert_eq!(list.next_index(), None);
        list.repeat = Repeat::All;
        assert_eq!(list.next_index(), Some(0)); // retour au début
        list.open("a", || unreachable!());
        assert_eq!(list.previous_index(), Some(2)); // et à la fin dans l'autre sens

        assert_eq!(Repeat::Off.next(), Repeat::One);
        assert_eq!(Repeat::One.next(), Repeat::All);
        assert_eq!(Repeat::All.next(), Repeat::Off);
    }

    #[test]
    fn shuffles_every_file_once() {
        let names: Vec<String> = (1..=20).map(|i| format!("ép {i}")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut list = items(&refs, "ép 5").with_seed(42);
        list.set_shuffle(true);

        let visited = play_through(&mut list);
        assert_eq!(visited[0], "ép 5"); // le fichier en cours reste le premier
        let mut sorted = visited.clone();
        sorted.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(sorted, names); // chaque fichier exactement une fois
        assert_ne!(visited, names[4..].iter().chain(&names[..4]).cloned().collect::<Vec<_>>());

        // Fin de l'ordre mélangé : arrêt, ou retour au début en répétition
        assert_eq!(list.next_index(), None);
        list.repeat = Repeat::All;
        assert_eq!(list.next_index().map(|i| list.keys[i].as_str()), Some("ép 5"));

        // Couper l'aléatoire revient à l'ordre de la liste
        list.set_shuffle(false);
        let current = list.current_index().unwrap();
        assert_eq!(list.next_index(), Some((current + 1) % names.len()));
    }

    #[test]
    fn shuffle_survives_edits() {
        let mut list = items(&["a", "b", "c", "d"], "a").with_seed(7);
        list.set_shuffle(true);
        let next = list.next_index().unwrap();
        let next_key = list.keys[next].clone();

        // Retirer le fichier en cours : la lecture continue avec le suivant prévu
        list.remove(list.current_index().unwrap());
        assert_eq!(list.next_index().map(|i| list.keys[i].clone()), Some(next_key.clone()));

        // Trier la liste ne change pas l'ordre de lecture
        list.sort();
        assert_eq!(list.next_index().map(|i| list.keys[i].clone()), Some(next_key));

        // Un fichier ajouté est joué lui aussi
        list.add(vec!["e".into()]);
        list.open(&list.keys[list.next_index().unwrap()].clone(), || unreachable!());
        let rest = play_through(&mut list);
        assert_eq!(rest.len(), 4); // b, c, d, e dans un ordre quelconque
        assert!(rest.contains(&"e".to_string()));
    }

    #[test]
    fn opens_folders() {
        let dir = std::env::temp_dir().join(format!("lumen-folder-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sous-dossier")).unwrap();
        for name in ["clip 10.mp4", "clip 2.mkv", "musique.flac", "notes.txt", "film.srt"] {
            std::fs::write(dir.join(name), b"").unwrap();
        }
        let names: Vec<String> = folder_files(&dir)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["clip 2.mkv", "clip 10.mp4", "musique.flac"]);
        std::fs::remove_dir_all(&dir).unwrap();

        let mut list = Items::default();
        assert_eq!(list.replace(vec!["x".into(), "y".into()]), Some("x".into()));
        assert_eq!(list.current_index(), None);
        assert_eq!(list.replace(Vec::new()), None);
    }

    #[test]
    fn encodes_file_urls() {
        let path = "/films/Ma Série #1 (VF) 100%.mkv";
        let url = file_url(path);
        assert_eq!(url, "file:///films/Ma%20S%C3%A9rie%20%231%20%28VF%29%20100%25.mkv");
        assert_eq!(local_path(&url), path);
    }
}
