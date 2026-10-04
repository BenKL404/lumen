//! Empêche la mise en veille de l'écran pendant la lecture d'une vidéo.
//!
//! Interface `org.freedesktop.ScreenSaver` en priorité (COSMIC, KDE, Xfce…), puis le
//! portail XDG `org.freedesktop.portal.Inhibit` (GNOME). Les appels D-Bus se font dans
//! un thread à part pour ne jamais bloquer l'interface. Si Lumen se ferme ou plante,
//! la connexion D-Bus se ferme et le bureau lève de lui-même l'inhibition.

#[cxx_qt::bridge]
pub mod qobject {
    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type ScreenGuard = super::ScreenGuardRust;

        /// Vrai : empêcher la mise en veille ; faux : la permettre à nouveau.
        #[qinvokable]
        fn set_active(self: &ScreenGuard, active: bool);
    }
}

use std::collections::HashMap;
use std::sync::mpsc::{self, Sender};

use cxx_qt::CxxQtType;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, Value};

const REASON: &str = "Lecture d'une vidéo";

pub struct ScreenGuardRust {
    requests: Sender<bool>,
}

impl Default for ScreenGuardRust {
    fn default() -> Self {
        let (requests, receiver) = mpsc::channel::<bool>();
        std::thread::spawn(move || {
            let mut connection: Option<Connection> = None;
            let mut held: Option<Inhibition> = None;
            for active in receiver {
                match (active, held.is_some()) {
                    (true, false) => {
                        if connection.is_none() {
                            connection = Connection::session()
                                .map_err(|e| eprintln!("Lumen : bus D-Bus de session indisponible ({e})"))
                                .ok();
                        }
                        if let Some(conn) = &connection {
                            held = inhibit(conn);
                        }
                    }
                    (false, true) => {
                        if let (Some(conn), Some(inhibition)) = (&connection, held.take()) {
                            release(conn, inhibition);
                        }
                    }
                    _ => {}
                }
            }
            // Lumen se ferme : lever l'inhibition proprement
            if let (Some(conn), Some(inhibition)) = (&connection, held) {
                release(conn, inhibition);
            }
        });
        Self { requests }
    }
}

impl qobject::ScreenGuard {
    pub fn set_active(&self, active: bool) {
        let _ = self.rust().requests.send(active);
    }
}

/// Inhibition en cours, selon l'interface qui l'a acceptée
enum Inhibition {
    ScreenSaver(u32),
    Portal(OwnedObjectPath),
}

fn inhibit(conn: &Connection) -> Option<Inhibition> {
    let screensaver = conn
        .call_method(
            Some("org.freedesktop.ScreenSaver"),
            "/org/freedesktop/ScreenSaver",
            Some("org.freedesktop.ScreenSaver"),
            "Inhibit",
            &("Lumen", REASON),
        )
        .and_then(|reply| reply.body().deserialize::<u32>());
    if let Ok(cookie) = screensaver {
        return Some(Inhibition::ScreenSaver(cookie));
    }

    // Drapeau 8 : empêcher la mise en veille pour inactivité
    let options: HashMap<&str, Value> = HashMap::from([("reason", Value::from(REASON))]);
    let portal = conn
        .call_method(
            Some("org.freedesktop.portal.Desktop"),
            "/org/freedesktop/portal/desktop",
            Some("org.freedesktop.portal.Inhibit"),
            "Inhibit",
            &("", 8u32, options),
        )
        .and_then(|reply| reply.body().deserialize::<OwnedObjectPath>());
    match portal {
        Ok(handle) => Some(Inhibition::Portal(handle)),
        Err(e) => {
            eprintln!("Lumen : impossible d'empêcher la mise en veille ({e})");
            None
        }
    }
}

fn release(conn: &Connection, inhibition: Inhibition) {
    let result = match inhibition {
        Inhibition::ScreenSaver(cookie) => conn
            .call_method(
                Some("org.freedesktop.ScreenSaver"),
                "/org/freedesktop/ScreenSaver",
                Some("org.freedesktop.ScreenSaver"),
                "UnInhibit",
                &(cookie,),
            )
            .map(drop),
        Inhibition::Portal(handle) => conn
            .call_method(
                Some("org.freedesktop.portal.Desktop"),
                handle.as_str(),
                Some("org.freedesktop.portal.Request"),
                "Close",
                &(),
            )
            .map(drop),
    };
    if let Err(e) = result {
        eprintln!("Lumen : impossible de rétablir la mise en veille ({e})");
    }
}
