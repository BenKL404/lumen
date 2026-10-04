//! Mémoire de lecture : position, vitesse et pistes de chaque vidéo, conservées
//! entre deux lancements dans une base SQLite (~/.local/share/lumen/history.db).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        // État mémorisé du dernier média passé à load()
        #[qobject]
        #[qml_element]
        #[qproperty(f64, position)]
        #[qproperty(f64, speed)]
        #[qproperty(i64, audio_id)]
        #[qproperty(i64, sub_id)]
        #[qproperty(f64, sub_delay)]
        #[qproperty(QString, sub_file)]
        type History = super::HistoryRust;

        /// Charge l'état mémorisé dans les propriétés. Faux s'il n'y a rien à reprendre.
        /// `audioId` / `subId` valent -1 si inconnus, `subId` vaut 0 si les sous-titres étaient désactivés.
        #[qinvokable]
        fn load(self: Pin<&mut History>, url: &QString) -> bool;

        /// Enregistre l'état de lecture, ou l'oublie si la vidéo a été vue jusqu'au bout.
        #[qinvokable]
        fn remember(
            self: &History,
            url: &QString,
            position: f64,
            duration: f64,
            speed: f64,
            audio_id: i64,
            sub_id: i64,
            sub_delay: f64,
            sub_file: &QString,
        );

        /// Efface la position enregistrée (bouton « Recommencer »).
        #[qinvokable]
        fn forget(self: &History, url: &QString);

        /// Efface tout l'historique de lecture.
        #[qinvokable]
        fn clear(self: &History);
    }
}

use std::path::{Path, PathBuf};
use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use rusqlite::{params, Connection, OptionalExtension};

use super::utils::local_path;

/// En dessous de cette durée, une vidéo n'est jamais mémorisée.
const MIN_DURATION: f64 = 120.0;
/// Avant cette position, on ne touche pas à ce qui est enregistré.
const MIN_POSITION: f64 = 10.0;
/// Au-delà de cette fraction de la durée, la vidéo est considérée comme vue.
const WATCHED_RATIO: f64 = 0.95;

pub struct HistoryRust {
    store: Option<HistoryStore>,
    position: f64,
    speed: f64,
    audio_id: i64,
    sub_id: i64,
    sub_delay: f64,
    sub_file: QString,
}

impl Default for HistoryRust {
    fn default() -> Self {
        let store = HistoryStore::open(&database_path())
            .map_err(|e| eprintln!("Lumen : historique de lecture indisponible ({e})"))
            .ok();
        let empty = Entry::default();
        Self {
            store,
            position: empty.position,
            speed: empty.speed,
            audio_id: -1,
            sub_id: -1,
            sub_delay: empty.sub_delay,
            sub_file: QString::default(),
        }
    }
}

impl qobject::History {
    pub fn load(mut self: Pin<&mut Self>, url: &QString) -> bool {
        let found = self
            .rust()
            .store
            .as_ref()
            .and_then(|store| store.get(&local_path(&url.to_string())).ok().flatten());
        let entry = found.clone().unwrap_or_default();

        self.as_mut().set_position(entry.position);
        self.as_mut().set_speed(entry.speed);
        self.as_mut().set_audio_id(entry.audio_id.unwrap_or(-1));
        self.as_mut().set_sub_id(entry.sub_id.unwrap_or(-1));
        self.as_mut().set_sub_delay(entry.sub_delay);
        self.as_mut().set_sub_file(QString::from(entry.sub_file.as_deref().unwrap_or("")));
        found.is_some()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn remember(
        &self,
        url: &QString,
        position: f64,
        duration: f64,
        speed: f64,
        audio_id: i64,
        sub_id: i64,
        sub_delay: f64,
        sub_file: &QString,
    ) {
        let Some(store) = &self.rust().store else { return };
        let key = local_path(&url.to_string());
        let sub_file = sub_file.to_string();
        let entry = Entry {
            position,
            speed,
            audio_id: (audio_id >= 0).then_some(audio_id),
            sub_id: (sub_id >= 0).then_some(sub_id),
            sub_delay,
            sub_file: (!sub_file.is_empty()).then_some(sub_file),
        };
        let result = match decide(position, duration) {
            Decision::Save => store.save(&key, &entry),
            Decision::Forget => store.forget(&key),
            Decision::Ignore => Ok(()),
        };
        if let Err(e) = result {
            eprintln!("Lumen : échec de l'enregistrement de la position ({e})");
        }
    }

    pub fn clear(&self) {
        if let Some(store) = &self.rust().store {
            if let Err(e) = store.clear() {
                eprintln!("Lumen : échec de l'effacement de l'historique ({e})");
            }
        }
    }

    pub fn forget(&self, url: &QString) {
        if let Some(store) = &self.rust().store {
            if let Err(e) = store.forget(&local_path(&url.to_string())) {
                eprintln!("Lumen : échec de l'effacement de la position ({e})");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub position: f64,
    pub speed: f64,
    pub audio_id: Option<i64>,
    /// `Some(0)` : sous-titres désactivés
    pub sub_id: Option<i64>,
    pub sub_delay: f64,
    /// Sous-titres externes ajoutés à la main (leurs ids changent d'une ouverture à l'autre)
    pub sub_file: Option<String>,
}

impl Default for Entry {
    fn default() -> Self {
        Self { position: 0.0, speed: 1.0, audio_id: None, sub_id: None, sub_delay: 0.0, sub_file: None }
    }
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
        // Migrations successives, repérées par PRAGMA user_version
        let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version < 1 {
            conn.execute_batch(
                "BEGIN;
                 ALTER TABLE positions ADD COLUMN audio_id INTEGER;
                 ALTER TABLE positions ADD COLUMN sub_id INTEGER;
                 ALTER TABLE positions ADD COLUMN sub_delay REAL NOT NULL DEFAULT 0;
                 ALTER TABLE positions ADD COLUMN sub_file TEXT;
                 PRAGMA user_version = 1;
                 COMMIT;",
            )?;
        }
        Ok(Self { conn })
    }

    pub fn get(&self, key: &str) -> rusqlite::Result<Option<Entry>> {
        self.conn
            .query_row(
                "SELECT position, speed, audio_id, sub_id, sub_delay, sub_file
                 FROM positions WHERE key = ?1",
                params![key],
                |row| {
                    Ok(Entry {
                        position: row.get(0)?,
                        speed: row.get(1)?,
                        audio_id: row.get(2)?,
                        sub_id: row.get(3)?,
                        sub_delay: row.get(4)?,
                        sub_file: row.get(5)?,
                    })
                },
            )
            .optional()
    }

    pub fn save(&self, key: &str, entry: &Entry) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO positions (key, position, speed, audio_id, sub_id, sub_delay, sub_file, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%s', 'now'))
             ON CONFLICT(key) DO UPDATE SET
                position = excluded.position,
                speed = excluded.speed,
                audio_id = excluded.audio_id,
                sub_id = excluded.sub_id,
                sub_delay = excluded.sub_delay,
                sub_file = excluded.sub_file,
                updated_at = excluded.updated_at",
            params![
                key,
                entry.position,
                entry.speed,
                entry.audio_id,
                entry.sub_id,
                entry.sub_delay,
                entry.sub_file
            ],
        )?;
        Ok(())
    }

    pub fn forget(&self, key: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM positions WHERE key = ?1", params![key])?;
        Ok(())
    }

    pub fn clear(&self) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM positions", [])?;
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
    fn stores_and_forgets_entries() {
        let store = HistoryStore::open_in_memory().unwrap();
        assert_eq!(store.get("a").unwrap(), None);

        store.save("a", &Entry { position: 120.0, ..Entry::default() }).unwrap();
        let full = Entry {
            position: 2535.0,
            speed: 1.5,
            audio_id: Some(2),
            sub_id: Some(0),
            sub_delay: -0.3,
            sub_file: Some("/films/Film.fr.srt".into()),
        };
        store.save("a", &full).unwrap();
        assert_eq!(store.get("a").unwrap(), Some(full));

        store.forget("a").unwrap();
        assert_eq!(store.get("a").unwrap(), None);

        store.save("a", &Entry { position: 120.0, ..Entry::default() }).unwrap();
        store.save("b", &Entry { position: 240.0, ..Entry::default() }).unwrap();
        store.clear().unwrap();
        assert_eq!((store.get("a").unwrap(), store.get("b").unwrap()), (None, None));
    }

    #[test]
    fn migrates_v0_database() {
        let dir = std::env::temp_dir().join(format!("lumen-test-{}", std::process::id()));
        let path = dir.join("history.db");
        let _ = std::fs::remove_file(&path);
        std::fs::create_dir_all(&dir).unwrap();

        // Base créée par la première version (sans colonnes de pistes)
        Connection::open(&path)
            .unwrap()
            .execute_batch(
                "CREATE TABLE positions (key TEXT PRIMARY KEY, position REAL NOT NULL,
                    speed REAL NOT NULL, updated_at INTEGER NOT NULL);
                 INSERT INTO positions VALUES ('a', 600.0, 1.25, 0);",
            )
            .unwrap();

        let store = HistoryStore::open(&path).unwrap();
        let entry = store.get("a").unwrap().unwrap();
        assert_eq!(entry, Entry { position: 600.0, speed: 1.25, ..Entry::default() });

        drop(store);
        HistoryStore::open(&path).unwrap(); // rouvrir ne relance pas la migration
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
