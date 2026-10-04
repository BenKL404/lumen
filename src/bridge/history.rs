//! Mémoire de lecture : position et vitesse de chaque vidéo, conservées
//! entre deux lancements dans une base SQLite (~/.local/share/lumen/history.db).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type History = super::HistoryRust;

        /// Position où reprendre la vidéo, ou 0 s'il n'y a rien à reprendre.
        #[qinvokable]
        #[cxx_name = "savedPosition"]
        fn saved_position(self: &History, url: &QString) -> f64;

        /// Vitesse enregistrée pour la vidéo, ou 0 si aucune.
        #[qinvokable]
        #[cxx_name = "savedSpeed"]
        fn saved_speed(self: &History, url: &QString) -> f64;

        /// Enregistre l'état de lecture, ou l'oublie si la vidéo a été vue jusqu'au bout.
        #[qinvokable]
        fn remember(self: &History, url: &QString, position: f64, duration: f64, speed: f64);

        /// Efface la position enregistrée (bouton « Recommencer »).
        #[qinvokable]
        fn forget(self: &History, url: &QString);
    }
}

use std::path::{Path, PathBuf};

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use rusqlite::{params, Connection, OptionalExtension};

use super::utils::percent_decode;

/// En dessous de cette durée, une vidéo n'est jamais mémorisée.
const MIN_DURATION: f64 = 120.0;
/// Avant cette position, on ne touche pas à ce qui est enregistré.
const MIN_POSITION: f64 = 10.0;
/// Au-delà de cette fraction de la durée, la vidéo est considérée comme vue.
const WATCHED_RATIO: f64 = 0.95;

pub struct HistoryRust {
    store: Option<HistoryStore>,
}

impl Default for HistoryRust {
    fn default() -> Self {
        let store = HistoryStore::open(&database_path())
            .map_err(|e| eprintln!("Lumen : historique de lecture indisponible ({e})"))
            .ok();
        Self { store }
    }
}

impl qobject::History {
    pub fn saved_position(&self, url: &QString) -> f64 {
        self.entry(url).map_or(0.0, |e| e.position)
    }

    pub fn saved_speed(&self, url: &QString) -> f64 {
        self.entry(url).map_or(0.0, |e| e.speed)
    }

    pub fn remember(&self, url: &QString, position: f64, duration: f64, speed: f64) {
        let Some(store) = &self.rust().store else { return };
        let key = media_key(&url.to_string());
        let result = match decide(position, duration) {
            Decision::Save => store.save(&key, Entry { position, speed }),
            Decision::Forget => store.forget(&key),
            Decision::Ignore => Ok(()),
        };
        if let Err(e) = result {
            eprintln!("Lumen : échec de l'enregistrement de la position ({e})");
        }
    }

    pub fn forget(&self, url: &QString) {
        if let Some(store) = &self.rust().store {
            if let Err(e) = store.forget(&media_key(&url.to_string())) {
                eprintln!("Lumen : échec de l'effacement de la position ({e})");
            }
        }
    }

    fn entry(&self, url: &QString) -> Option<Entry> {
        let store = self.rust().store.as_ref()?;
        store.get(&media_key(&url.to_string())).ok().flatten()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entry {
    pub position: f64,
    pub speed: f64,
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Save,
    Forget,
    Ignore,
}

/// Que faire de l'état de lecture courant.
pub fn decide(position: f64, duration: f64) -> Decision {
    if !position.is_finite() || !duration.is_finite() || duration < MIN_DURATION {
        // Durée inconnue (fichier en cours de chargement) ou vidéo trop courte
        Decision::Ignore
    } else if position >= duration * WATCHED_RATIO {
        Decision::Forget
    } else if position < MIN_POSITION {
        // Début de lecture, ou seek de reprise pas encore effectué :
        // ne pas écraser une position enregistrée.
        Decision::Ignore
    } else {
        Decision::Save
    }
}

/// Clé stable d'un média : chemin local décodé, ou l'URL telle quelle.
/// Le dialogue et le glisser-déposer n'encodent pas toujours les URL de la même façon.
pub fn media_key(url: &str) -> String {
    match url.strip_prefix("file://") {
        Some(path) => percent_decode(path),
        None => url.to_string(),
    }
}

fn database_path() -> PathBuf {
    let data = std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    data.join("lumen").join("history.db")
}

pub struct HistoryStore {
    conn: Connection,
}

impl HistoryStore {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> rusqlite::Result<Self> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS positions (
                key        TEXT PRIMARY KEY,
                position   REAL NOT NULL,
                speed      REAL NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn get(&self, key: &str) -> rusqlite::Result<Option<Entry>> {
        self.conn
            .query_row(
                "SELECT position, speed FROM positions WHERE key = ?1",
                params![key],
                |row| Ok(Entry { position: row.get(0)?, speed: row.get(1)? }),
            )
            .optional()
    }

    pub fn save(&self, key: &str, entry: Entry) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO positions (key, position, speed, updated_at)
             VALUES (?1, ?2, ?3, strftime('%s', 'now'))
             ON CONFLICT(key) DO UPDATE SET
                position = excluded.position,
                speed = excluded.speed,
                updated_at = excluded.updated_at",
            params![key, entry.position, entry.speed],
        )?;
        Ok(())
    }

    pub fn forget(&self, key: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM positions WHERE key = ?1", params![key])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decides_what_to_remember() {
        assert_eq!(decide(600.0, 3600.0), Decision::Save);
        assert_eq!(decide(3500.0, 3600.0), Decision::Forget); // vue jusqu'au bout
        assert_eq!(decide(5.0, 3600.0), Decision::Ignore); // tout début
        assert_eq!(decide(50.0, 90.0), Decision::Ignore); // vidéo trop courte
        assert_eq!(decide(50.0, 0.0), Decision::Ignore); // durée encore inconnue
        assert_eq!(decide(f64::NAN, 3600.0), Decision::Ignore);
    }

    #[test]
    fn builds_stable_keys() {
        assert_eq!(media_key("file:///films/Mon%20Film.mkv"), "/films/Mon Film.mkv");
        assert_eq!(media_key("file:///films/Mon Film.mkv"), "/films/Mon Film.mkv");
        assert_eq!(media_key("https://exemple.org/v.mp4"), "https://exemple.org/v.mp4");
    }

    #[test]
    fn stores_and_forgets_positions() {
        let store = HistoryStore::open_in_memory().unwrap();
        assert_eq!(store.get("a").unwrap(), None);

        store.save("a", Entry { position: 120.0, speed: 1.0 }).unwrap();
        store.save("a", Entry { position: 2535.0, speed: 1.5 }).unwrap();
        assert_eq!(store.get("a").unwrap(), Some(Entry { position: 2535.0, speed: 1.5 }));

        store.forget("a").unwrap();
        assert_eq!(store.get("a").unwrap(), None);
    }
}
