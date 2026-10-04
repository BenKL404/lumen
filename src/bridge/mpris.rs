//! Contrôle par le bureau (MPRIS) : touches multimédia du clavier, applet du panneau,
//! écran de verrouillage, casques Bluetooth…
//!
//! Lumen publie `org.mpris.MediaPlayer2.lumen` sur le bus de session. L'état de lecture
//! arrive de QML par `update()` ; les commandes du bureau repartent vers QML par le
//! signal `command`, via la file d'événements de Qt (le serveur D-Bus a son propre thread).

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Mpris = super::MprisRust;

        /// Publie l'état de lecture. `status` : 0 arrêté, 1 en lecture, 2 en pause ;
        /// `repeat` : 0, 1 (fichier) ou 2 (playlist) ; durées en secondes ; volume en %.
        #[qinvokable]
        fn update(
            self: &Mpris,
            status: i32,
            title: &QString,
            url: &QString,
            length: f64,
            position: f64,
            volume: f64,
            rate: f64,
            repeat: i32,
            shuffle: bool,
            can_next: bool,
            can_previous: bool,
        );

        /// Commande reçue du bureau : `name` (play-pause, play, pause, stop, next,
        /// previous, seek, position, volume, rate, repeat, shuffle, raise, quit, open),
        /// avec une valeur numérique ou un texte selon la commande.
        #[qsignal]
        fn command(self: Pin<&mut Mpris>, name: QString, number: f64, text: QString);
    }

    impl cxx_qt::Threading for Mpris {}
}

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use cxx_qt::{CxxQtThread, CxxQtType, Threading};
use cxx_qt_lib::QString;
use zbus::blocking::Connection;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedValue, Value};

const PATH: &str = "/org/mpris/MediaPlayer2";

/// État publié sur D-Bus
#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    pub status: i32,
    pub title: String,
    pub url: String,
    pub length: f64,
    pub position: f64,
    pub volume: f64,
    pub rate: f64,
    pub repeat: i32,
    pub shuffle: bool,
    pub can_next: bool,
    pub can_previous: bool,
}

impl State {
    fn playback_status(&self) -> &'static str {
        match self.status {
            1 => "Playing",
            2 => "Paused",
            _ => "Stopped",
        }
    }

    fn loop_status(&self) -> &'static str {
        match self.repeat {
            1 => "Track",
            2 => "Playlist",
            _ => "None",
        }
    }

    /// Identifiant de piste stable pour un fichier donné (exigé par MPRIS)
    fn track_id(&self) -> String {
        let hash = self.url.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
        format!("/org/lumen/track/t{hash:016x}")
    }

    fn metadata(&self) -> HashMap<String, OwnedValue> {
        let mut map = HashMap::new();
        let mut insert = |key: &str, value: Value| {
            if let Ok(value) = OwnedValue::try_from(value) {
                map.insert(key.to_string(), value);
            }
        };
        if let Ok(path) = ObjectPath::try_from(self.track_id()) {
            insert("mpris:trackid", Value::from(path));
        }
        if self.status != 0 {
            insert("xesam:title", Value::from(self.title.clone()));
            insert("xesam:url", Value::from(self.url.clone()));
            if self.length > 0.0 {
                insert("mpris:length", Value::from(micros(self.length)));
            }
        }
        map
    }
}

fn micros(seconds: f64) -> i64 {
    (seconds.max(0.0) * 1_000_000.0) as i64
}

/// Transmet une commande du bureau au thread de Qt
#[derive(Clone)]
struct Commander(CxxQtThread<qobject::Mpris>);

impl Commander {
    fn send(&self, name: &'static str, number: f64, text: String) {
        let _ = self.0.queue(move |mut mpris| {
            mpris.as_mut().command(QString::from(name), number, QString::from(&text));
        });
    }
}

struct Root {
    commands: Commander,
}

#[zbus::interface(name = "org.mpris.MediaPlayer2")]
impl Root {
    fn raise(&self) {
        self.commands.send("raise", 0.0, String::new());
    }

    fn quit(&self) {
        self.commands.send("quit", 0.0, String::new());
    }

    #[zbus(property)]
    fn can_quit(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_raise(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn has_track_list(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn identity(&self) -> String {
        "Lumen".into()
    }

    #[zbus(property)]
    fn desktop_entry(&self) -> String {
        "lumen".into()
    }

    #[zbus(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        vec!["file".into()]
    }

    #[zbus(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        ["video/x-matroska", "video/mp4", "video/webm", "video/x-msvideo", "video/quicktime", "audio/mpeg", "audio/flac"]
            .map(String::from)
            .to_vec()
    }
}

struct Player {
    state: Arc<Mutex<State>>,
    commands: Commander,
}

impl Player {
    fn state(&self) -> State {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn send(&self, name: &'static str, number: f64) {
        self.commands.send(name, number, String::new());
    }
}

#[zbus::interface(name = "org.mpris.MediaPlayer2.Player")]
impl Player {
    fn next(&self) {
        self.send("next", 0.0);
    }

    fn previous(&self) {
        self.send("previous", 0.0);
    }

    fn pause(&self) {
        self.send("pause", 0.0);
    }

    fn play_pause(&self) {
        self.send("play-pause", 0.0);
    }

    fn stop(&self) {
        self.send("stop", 0.0);
    }

    fn play(&self) {
        self.send("play", 0.0);
    }

    /// Décalage relatif, en microsecondes
    fn seek(&self, offset: i64) {
        self.send("seek", offset as f64 / 1_000_000.0);
    }

    /// Position absolue, ignorée si elle vise une autre piste (spécification MPRIS)
    fn set_position(&self, track_id: ObjectPath<'_>, position: i64) {
        if track_id.as_str() == self.state().track_id() {
            self.send("position", position as f64 / 1_000_000.0);
        }
    }

    fn open_uri(&self, uri: String) {
        self.commands.send("open", 0.0, uri);
    }

    #[zbus(signal)]
    async fn seeked(emitter: &SignalEmitter<'_>, position: i64) -> zbus::Result<()>;

    #[zbus(property)]
    fn playback_status(&self) -> String {
        self.state().playback_status().into()
    }

    #[zbus(property)]
    fn loop_status(&self) -> String {
        self.state().loop_status().into()
    }

    #[zbus(property)]
    fn set_loop_status(&mut self, value: String) {
        let repeat = match value.as_str() {
            "Track" => 1.0,
            "Playlist" => 2.0,
            _ => 0.0,
        };
        self.send("repeat", repeat);
    }

    #[zbus(property)]
    fn rate(&self) -> f64 {
        self.state().rate
    }

    #[zbus(property)]
    fn set_rate(&mut self, value: f64) {
        if value > 0.0 {
            self.send("rate", value);
        }
    }

    #[zbus(property)]
    fn shuffle(&self) -> bool {
        self.state().shuffle
    }

    #[zbus(property)]
    fn set_shuffle(&mut self, value: bool) {
        self.send("shuffle", if value { 1.0 } else { 0.0 });
    }

    #[zbus(property)]
    fn metadata(&self) -> HashMap<String, OwnedValue> {
        self.state().metadata()
    }

    /// 0.0 à 1.3 : le volume de Lumen monte jusqu'à 130 %
    #[zbus(property)]
    fn volume(&self) -> f64 {
        self.state().volume / 100.0
    }

    #[zbus(property)]
    fn set_volume(&mut self, value: f64) {
        self.send("volume", (value * 100.0).clamp(0.0, 130.0));
    }

    /// Interrogée par le bureau quand il en a besoin (pas de signal de changement)
    #[zbus(property(emits_changed_signal = "false"))]
    fn position(&self) -> i64 {
        micros(self.state().position)
    }

    #[zbus(property)]
    fn minimum_rate(&self) -> f64 {
        0.25
    }

    #[zbus(property)]
    fn maximum_rate(&self) -> f64 {
        4.0
    }

    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        self.state().can_next
    }

    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        self.state().can_previous
    }

    #[zbus(property)]
    fn can_play(&self) -> bool {
        self.state().status != 0
    }

    #[zbus(property)]
    fn can_pause(&self) -> bool {
        self.state().status != 0
    }

    #[zbus(property)]
    fn can_seek(&self) -> bool {
        self.state().status != 0 && self.state().length > 0.0
    }

    #[zbus(property)]
    fn can_control(&self) -> bool {
        true
    }
}

pub struct MprisRust {
    state: Arc<Mutex<State>>,
    /// Connexion établie en arrière-plan ; None tant qu'elle n'est pas prête (ou en cas d'échec)
    connection: Arc<Mutex<Option<Connection>>>,
    /// Pour détecter les sauts de position (signal Seeked)
    last_update: Mutex<Option<(Instant, f64)>>,
}

impl Default for MprisRust {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::default())),
            connection: Arc::new(Mutex::new(None)),
            last_update: Mutex::new(None),
        }
    }
}

impl qobject::Mpris {
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &self,
        status: i32,
        title: &QString,
        url: &QString,
        length: f64,
        position: f64,
        volume: f64,
        rate: f64,
        repeat: i32,
        shuffle: bool,
        can_next: bool,
        can_previous: bool,
    ) {
        self.ensure_connection();
        let new = State {
            status,
            title: title.to_string(),
            url: url.to_string(),
            length,
            position,
            volume,
            rate,
            repeat,
            shuffle,
            can_next,
            can_previous,
        };
        let old = match self.rust().state.lock() {
            Ok(mut state) => std::mem::replace(&mut *state, new.clone()),
            Err(_) => return,
        };

        // Saut de position (seek) : position éloignée de celle attendue depuis le dernier envoi
        let seeked = {
            let mut last = self.rust().last_update.lock().ok();
            let now = Instant::now();
            let jumped = last.as_ref().and_then(|l| **l).is_some_and(|(at, pos)| {
                let expected = if old.status == 1 { pos + now.duration_since(at).as_secs_f64() * old.rate } else { pos };
                old.url == new.url && (new.position - expected).abs() > 1.5
            });
            if let Some(last) = last.as_mut() {
                **last = Some((now, new.position));
            }
            jumped
        };

        let Ok(guard) = self.rust().connection.lock() else { return };
        let Some(conn) = guard.as_ref() else { return };
        let Ok(player) = conn.object_server().interface::<_, Player>(PATH) else { return };
        let emitter = player.signal_emitter();
        let iface = player.get();
        zbus::block_on(async {
            if old.status != new.status {
                let _ = iface.playback_status_changed(emitter).await;
                let _ = iface.can_play_changed(emitter).await;
                let _ = iface.can_pause_changed(emitter).await;
                let _ = iface.can_seek_changed(emitter).await;
            }
            if (old.title, &old.url, old.length) != (new.title.clone(), &new.url, new.length) || old.status != new.status {
                let _ = iface.metadata_changed(emitter).await;
            }
            if old.volume != new.volume {
                let _ = iface.volume_changed(emitter).await;
            }
            if old.rate != new.rate {
                let _ = iface.rate_changed(emitter).await;
            }
            if old.repeat != new.repeat {
                let _ = iface.loop_status_changed(emitter).await;
            }
            if old.shuffle != new.shuffle {
                let _ = iface.shuffle_changed(emitter).await;
            }
            if old.can_next != new.can_next {
                let _ = iface.can_go_next_changed(emitter).await;
            }
            if old.can_previous != new.can_previous {
                let _ = iface.can_go_previous_changed(emitter).await;
            }
            if seeked {
                let _ = Player::seeked(emitter, micros(new.position)).await;
            }
        });
    }

    /// Au premier appel, publie le service D-Bus en arrière-plan (sans bloquer l'interface).
    fn ensure_connection(&self) {
        static STARTED: std::sync::Once = std::sync::Once::new();
        let commands = Commander(self.qt_thread());
        let state = self.rust().state.clone();
        let slot = self.rust().connection.clone();
        STARTED.call_once(move || {
            std::thread::spawn(move || match connect(commands, state) {
                Ok(conn) => {
                    if let Ok(mut slot) = slot.lock() {
                        *slot = Some(conn);
                    }
                }
                Err(e) => eprintln!("Lumen : contrôle par le bureau (MPRIS) indisponible ({e})"),
            });
        });
    }
}

/// Publie les deux interfaces MPRIS, sous `lumen` ou, si un autre Lumen tourne déjà,
/// sous `lumen.instance<pid>` (convention MPRIS).
fn connect(commands: Commander, state: Arc<Mutex<State>>) -> zbus::Result<Connection> {
    let names = [
        "org.mpris.MediaPlayer2.lumen".to_string(),
        format!("org.mpris.MediaPlayer2.lumen.instance{}", std::process::id()),
    ];
    let mut last_error = None;
    for name in names {
        let built = zbus::blocking::connection::Builder::session()?
            .name(name)?
            .serve_at(PATH, Root { commands: commands.clone() })?
            .serve_at(PATH, Player { state: state.clone(), commands: commands.clone() })?
            .build();
        match built {
            Ok(conn) => return Ok(conn),
            Err(e) => last_error = Some(e),
        }
    }
    Err(last_error.unwrap_or(zbus::Error::Unsupported))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_state() {
        let mut state = State { status: 1, title: "Film".into(), url: "file:///f.mkv".into(), length: 90.5, ..State::default() };
        assert_eq!(state.playback_status(), "Playing");
        assert_eq!(state.loop_status(), "None");
        state.repeat = 2;
        assert_eq!(state.loop_status(), "Playlist");

        let metadata = state.metadata();
        assert_eq!(i64::try_from(metadata["mpris:length"].clone()).unwrap(), 90_500_000);
        assert!(metadata.contains_key("xesam:title"));

        // Identifiant stable pour un même fichier, différent pour un autre, et chemin D-Bus valide
        let id = state.track_id();
        assert_eq!(id, state.track_id());
        assert!(ObjectPath::try_from(id.clone()).is_ok());
        state.url = "file:///autre.mkv".into();
        assert_ne!(id, state.track_id());

        // Arrêté : seulement l'identifiant de piste
        state.status = 0;
        assert_eq!(state.playback_status(), "Stopped");
        assert_eq!(state.metadata().len(), 1);
    }
}
