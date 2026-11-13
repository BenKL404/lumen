//! Une seule fenêtre Lumen : si Lumen tourne déjà, les fichiers demandés lui sont transmis
//! (méthodes MPRIS `OpenUri` et `Raise`, déjà publiées par chaque instance) et ce
//! nouveau processus s'arrête sans ouvrir de fenêtre. Plusieurs fichiers partent en un seul
//! `OpenUri`, séparés par des sauts de ligne (interdits dans une URI) : le premier se lit,
//! les autres rejoignent la playlist (voir `openFiles` dans Main.qml).

use std::path::Path;
use std::time::Duration;

use zbus::blocking::{connection, fdo::DBusProxy, Connection};
use zbus::names::BusName;

use crate::bridge::playlist::file_url;
use crate::bridge::settings::{settings_path, SettingsFile};

const SERVICE: &str = "org.mpris.MediaPlayer2.lumen";
const PATH: &str = "/org/mpris/MediaPlayer2";

/// Vrai si une instance déjà lancée a pris en charge `arguments` : il faut alors s'arrêter.
pub fn forward_to_running_instance(arguments: &[String]) -> bool {
    if !SettingsFile::load(&settings_path()).single_instance {
        return false;
    }
    match forward(arguments) {
        Ok(forwarded) => forwarded,
        Err(e) => {
            // En cas de doute, ouvrir une nouvelle fenêtre plutôt que rien
            eprintln!("Lumen : instance existante injoignable, nouvelle fenêtre ({e})");
            false
        }
    }
}

fn forward(arguments: &[String]) -> zbus::Result<bool> {
    // Délai court : une instance bloquée ne doit pas retarder l'ouverture
    let conn: Connection = connection::Builder::session()?.method_timeout(Duration::from_secs(3)).build()?;
    let running = DBusProxy::new(&conn)?.name_has_owner(BusName::try_from(SERVICE)?.into())?;
    if !running {
        return Ok(false);
    }
    if !arguments.is_empty() {
        let uris = arguments.iter().map(|a| to_uri(a)).collect::<Vec<_>>().join("\n");
        conn.call_method(Some(SERVICE), PATH, Some("org.mpris.MediaPlayer2.Player"), "OpenUri", &(uris,))?;
    }
    conn.call_method(Some(SERVICE), PATH, Some("org.mpris.MediaPlayer2"), "Raise", &())?;
    Ok(true)
}

/// URL complète : les chemins relatifs dépendent du dossier de ce processus, pas de l'instance
fn to_uri(argument: &str) -> String {
    if argument.contains("://") {
        return argument.to_string();
    }
    let path = std::fs::canonicalize(Path::new(argument)).unwrap_or_else(|_| Path::new(argument).to_path_buf());
    file_url(&path.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_absolute_uris() {
        assert_eq!(to_uri("https://exemple.org/v.mp4"), "https://exemple.org/v.mp4");
        assert_eq!(to_uri("file:///a/b.mkv"), "file:///a/b.mkv");
        let uri = to_uri("Cargo.toml"); // chemin relatif au dossier courant
        assert!(uri.starts_with("file:///") && uri.ends_with("/Cargo.toml"), "{uri}");
    }
}
