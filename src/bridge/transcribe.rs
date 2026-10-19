//! Sous-titres générés par reconnaissance vocale, sur l'ordinateur (whisper.cpp) :
//! téléchargement du modèle à la première utilisation, extraction du son avec ffmpeg,
//! reconnaissance avec `whisper-cli`, puis fichier `.srt` enregistré à côté de la vidéo.
//! Tout se passe hors du fil de l'interface ; l'avancement remonte par `progress` et `stage`.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        // `busy` : génération en cours ; `progress` : de 0 à 100 pour l'étape en cours ;
        // `stage` : étape affichée (« Téléchargement du modèle », « Reconnaissance »…)
        #[qobject]
        #[qml_element]
        #[qproperty(bool, busy)]
        #[qproperty(i32, progress)]
        #[qproperty(QString, stage)]
        type Transcriber = super::TranscriberRust;

        /// Vrai si `whisper-cli` est installé (PATH ou ~/.local/bin).
        #[qinvokable]
        fn available(self: &Transcriber) -> bool;

        /// Vrai si le modèle est déjà téléchargé.
        #[qinvokable]
        fn has_model(self: &Transcriber, model: &QString) -> bool;

        /// Lance la génération pour une vidéo locale. `language` : code (« fr ») ou vide pour
        /// une détection automatique. Résultat : `finished` (chemin du .srt) ou `failed`.
        #[qinvokable]
        fn start(self: Pin<&mut Transcriber>, url: &QString, language: &QString, model: &QString);

        /// Interrompt la génération en cours.
        #[qinvokable]
        fn cancel(self: Pin<&mut Transcriber>);

        #[qsignal]
        fn finished(self: Pin<&mut Transcriber>, path: QString);

        #[qsignal]
        fn failed(self: Pin<&mut Transcriber>, message: QString);
    }

    impl cxx_qt::Threading for Transcriber {}
}

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use super::online_subtitles::{subtitle_target, write_subtitle};
use super::utils::{is_online, local_path};

/// Modèles proposés (taille du fichier) : `base` rapide, `small` bon compromis, `medium` précis
pub const MODELS: &[(&str, &str)] = &[("base", "148 Mo"), ("small", "488 Mo"), ("medium", "1,5 Go")];

#[derive(Default)]
pub struct TranscriberRust {
    busy: bool,
    progress: i32,
    stage: QString,
    cancel: Arc<AtomicBool>,
}

/// Étape et avancement, envoyés au fil de l'interface
type Report<'a> = &'a (dyn Fn(&str, i32) + Sync);

impl qobject::Transcriber {
    pub fn available(&self) -> bool {
        whisper_cli().is_some()
    }

    pub fn has_model(&self, model: &QString) -> bool {
        model_path(&model_name(&model.to_string())).exists()
    }

    pub fn start(mut self: Pin<&mut Self>, url: &QString, language: &QString, model: &QString) {
        if self.rust().busy {
            return;
        }
        let (url, language, model) = (url.to_string(), language.to_string(), model_name(&model.to_string()));
        let cancel = Arc::new(AtomicBool::new(false));
        self.as_mut().rust_mut().cancel = cancel.clone();
        self.as_mut().set_busy(true);
        self.as_mut().set_progress(0);
        self.as_mut().set_stage(QString::from("Préparation"));

        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let progress_thread = thread.clone();
            let report = move |stage: &str, percent: i32| {
                let stage = QString::from(stage);
                let _ = progress_thread.queue(move |mut this| {
                    this.as_mut().set_stage(stage);
                    this.as_mut().set_progress(percent);
                });
            };
            let result = transcribe(&url, &language, &model, &cancel, &report);
            let _ = thread.queue(move |mut this| {
                this.as_mut().set_busy(false);
                match result {
                    Ok(path) => this.as_mut().finished(QString::from(&path.to_string_lossy().into_owned())),
                    Err(message) => this.as_mut().failed(QString::from(&message)),
                }
            });
        });
    }

    pub fn cancel(self: Pin<&mut Self>) {
        self.rust().cancel.store(true, Ordering::SeqCst);
    }
}

fn model_name(model: &str) -> String {
    if MODELS.iter().any(|(name, _)| *name == model) { model.to_string() } else { "base".to_string() }
}

fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lumen")
}

/// Fichiers temporaires (son extrait, sortie de whisper-cli) : $XDG_CACHE_HOME/lumen/whisper
fn cache_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lumen")
        .join("whisper")
}

fn model_path(model: &str) -> PathBuf {
    data_dir().join("whisper").join(format!("ggml-{model}.bin"))
}

/// `whisper-cli` dans le PATH, sinon dans ~/.local/bin (scripts/install-whisper.sh)
fn whisper_cli() -> Option<PathBuf> {
    let path = std::env::var("PATH").unwrap_or_default();
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    std::env::split_paths(&path)
        .chain([home.join(".local/bin")])
        .map(|dir| dir.join("whisper-cli"))
        .find(|p| p.is_file())
}

fn transcribe(url: &str, language: &str, model: &str, cancel: &AtomicBool, report: Report) -> Result<PathBuf, String> {
    let video = local_path(url);
    if is_online(&video) || !Path::new(&video).is_file() {
        return Err("La génération de sous-titres ne fonctionne qu'avec un fichier sur l'ordinateur".into());
    }
    let whisper = whisper_cli().ok_or("whisper-cli introuvable : lancer scripts/install-whisper.sh")?;
    let model_file = model_path(model);
    if !model_file.exists() {
        download_model(model, &model_file, cancel, report)?;
    }

    let work = cache_dir();
    std::fs::create_dir_all(&work).map_err(|e| format!("Dossier temporaire impossible ({e})"))?;
    let wav = work.join(format!("audio-{}.wav", std::process::id()));
    let out = work.join(format!("sous-titres-{}", std::process::id()));
    let cleanup = || {
        let _ = std::fs::remove_file(&wav);
        let _ = std::fs::remove_file(out.with_extension("srt"));
    };

    // Son en mono 16 kHz, le format attendu par Whisper
    report("Extraction du son", 0);
    let ffmpeg = Command::new("ffmpeg")
        .args(["-nostdin", "-y", "-v", "error", "-i"])
        .arg(&video)
        .args(["-vn", "-ac", "1", "-ar", "16000", "-c:a", "pcm_s16le"])
        .arg(&wav)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("ffmpeg introuvable ({e})"))?;
    if !wait(ffmpeg, cancel)? {
        cleanup();
        return Err("Le son de cette vidéo n'a pas pu être extrait".into());
    }

    report("Reconnaissance de la parole", 0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get().min(8));
    let mut child = Command::new(&whisper)
        .arg("-m")
        .arg(&model_file)
        .arg("-f")
        .arg(&wav)
        .args(["-l", if language.is_empty() { "auto" } else { language }])
        .args(["-t", &threads.to_string(), "-osrt", "-pp", "-of"])
        .arg(&out)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("whisper-cli ne démarre pas ({e})"))?;

    // Avancement (« progress = 42% ») et langue détectée, lus sur la sortie d'erreur
    let stderr = child.stderr.take().ok_or("whisper-cli : sortie illisible")?;
    let detected = std::sync::Mutex::new(String::new());
    // Lecture et attente en parallèle : l'annulation reste possible pendant la reconnaissance
    let ok = std::thread::scope(|scope| {
        scope.spawn(|| {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Some(percent) = parse_progress(&line) {
                    report("Reconnaissance de la parole", percent);
                }
                if let Some(lang) = parse_language(&line) {
                    *detected.lock().unwrap() = lang;
                }
            }
        });
        wait(child, cancel)
    });
    let ok = match ok {
        Ok(ok) => ok,
        Err(message) => {
            cleanup();
            return Err(message);
        }
    };
    let srt = out.with_extension("srt");
    let bytes = std::fs::read(&srt).unwrap_or_default();
    cleanup();
    if !ok {
        return Err("La reconnaissance a échoué".into());
    }
    if bytes.iter().all(|b| b.is_ascii_whitespace()) {
        return Err("Aucune parole reconnue dans cette vidéo".into());
    }

    let detected = detected.into_inner().unwrap_or_default();
    let lang = if language.is_empty() { detected } else { language.to_string() };
    write_subtitle(&subtitle_target(url, &lang), &bytes)
}

/// Attend la fin du programme ; Faux s'il a échoué. Annulation : le programme est arrêté.
fn wait(mut child: Child, cancel: &AtomicBool) -> Result<bool, String> {
    loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Génération annulée".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status.success()),
            Ok(None) => std::thread::sleep(Duration::from_millis(200)),
            Err(e) => return Err(format!("Programme interrompu ({e})")),
        }
    }
}

/// Télécharge le modèle dans `<modèle>.part`, puis le renomme. Une coupure (connexion lente ou
/// instable) n'oblige pas à recommencer : le téléchargement reprend là où il s'était arrêté
/// (en-tête Range), y compris lors d'une génération suivante.
fn download_model(model: &str, target: &Path, cancel: &AtomicBool, report: Report) -> Result<(), String> {
    const ATTEMPTS: u32 = 30;
    let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{model}.bin");
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_body(Some(Duration::from_secs(60)))
        .http_status_as_error(false)
        .user_agent(concat!("Lumen/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    std::fs::create_dir_all(target.parent().unwrap_or(Path::new("."))).map_err(|e| format!("Dossier du modèle impossible ({e})"))?;
    let partial = target.with_extension("part");
    let mut last_error = String::new();
    let mut last = -1;
    report("Téléchargement du modèle", 0);

    for attempt in 0..ATTEMPTS {
        if cancel.load(Ordering::SeqCst) {
            return Err("Génération annulée".into());
        }
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(2));
        }
        let done = std::fs::metadata(&partial).map_or(0, |m| m.len());
        let mut response = match agent.get(&url).header("Range", format!("bytes={done}-")).call() {
            Ok(response) => response,
            Err(e) => {
                last_error = e.to_string();
                continue;
            }
        };
        let status = response.status().as_u16();
        let length: u64 = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let (mut written, total, append) = match status {
            206 => (done, done + length, true),
            200 => (0, length, false),
            416 if done > 0 => break, // déjà complet
            _ => {
                last_error = format!("réponse {status}");
                continue;
            }
        };
        let file = std::fs::OpenOptions::new().create(true).write(true).append(append).truncate(!append).open(&partial);
        let mut file = file.map_err(|e| format!("Modèle impossible à enregistrer ({e})"))?;
        let mut reader = response.body_mut().with_config().limit(4 * 1024 * 1024 * 1024).reader();
        let mut buffer = vec![0u8; 1 << 16];
        let finished = loop {
            if cancel.load(Ordering::SeqCst) {
                return Err("Génération annulée".into()); // la partie téléchargée est gardée
            }
            match reader.read(&mut buffer) {
                Ok(0) => break total == 0 || written >= total,
                Ok(n) => {
                    file.write_all(&buffer[..n]).map_err(|e| format!("Disque plein ? ({e})"))?;
                    written += n as u64;
                    let percent = if total > 0 { (written * 100 / total) as i32 } else { 0 };
                    if percent != last {
                        last = percent;
                        report("Téléchargement du modèle", percent);
                    }
                }
                Err(e) => {
                    last_error = e.to_string();
                    break false;
                }
            }
        };
        if finished {
            return std::fs::rename(&partial, target).map_err(|e| format!("Modèle impossible à enregistrer ({e})"));
        }
    }
    if std::fs::metadata(&partial).is_ok_and(|m| m.len() > 0) && last_error.is_empty() {
        return std::fs::rename(&partial, target).map_err(|e| format!("Modèle impossible à enregistrer ({e})"));
    }
    Err(format!("Téléchargement du modèle interrompu ({last_error}) ; il reprendra à la prochaine génération"))
}

/// « whisper_print_progress_callback: progress =  42% » -> 42
fn parse_progress(line: &str) -> Option<i32> {
    let rest = line.split("progress =").nth(1)?;
    rest.trim().trim_end_matches('%').trim().parse().ok().map(|p: i32| p.clamp(0, 100))
}

/// « whisper_full_with_state: auto-detected language: fr (p = 0.97) » -> « fr »
fn parse_language(line: &str) -> Option<String> {
    let rest = line.split("auto-detected language:").nth(1)?;
    let code: String = rest.trim().chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    (!code.is_empty()).then_some(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_whisper_output() {
        assert_eq!(parse_progress("whisper_print_progress_callback: progress =  42%"), Some(42));
        assert_eq!(parse_progress("progress = 100%"), Some(100));
        assert_eq!(parse_progress("autre ligne"), None);
        assert_eq!(parse_language("whisper_full_with_state: auto-detected language: fr (p = 0.97)"), Some("fr".into()));
        assert_eq!(parse_language("rien"), None);
    }

    #[test]
    fn unknown_model_falls_back_to_base() {
        assert_eq!(model_name("base"), "base");
        assert_eq!(model_name("géant"), "base");
        assert!(model_path("small").ends_with("whisper/ggml-small.bin"));
    }
}
