//! Utilitaires exposés à QML, implémentés en Rust.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Utils = super::UtilsRust;

        /// Formate une durée en secondes : 75.3 -> "1:15", 3725 -> "1:02:05"
        #[qinvokable]
        #[cxx_name = "formatTime"]
        fn format_time(self: &Utils, seconds: f64) -> QString;

        /// Durée au format long, toujours avec les heures : 91.0 -> "00:01:31"
        #[qinvokable]
        #[cxx_name = "formatClock"]
        fn format_clock(self: &Utils, seconds: f64) -> QString;

        /// Extension en majuscules pour le badge de format : ".../film.mkv" -> "MKV"
        #[qinvokable]
        #[cxx_name = "fileExtension"]
        fn file_extension(self: &Utils, url: &QString) -> QString;

        /// Ligne d'infos techniques : "H.264 1920×1080 · AAC 48 kHz" ou "MP3 · 320 kbps · 44,1 kHz"
        #[qinvokable]
        #[cxx_name = "mediaInfo"]
        fn media_info(
            self: &Utils,
            video_codec: &QString,
            width: i64,
            height: i64,
            audio_codec: &QString,
            sample_rate: i64,
            audio_bitrate: f64,
        ) -> QString;

        /// Extrait un nom lisible depuis une URL : "file:///films/Mon%20Film.mkv" -> "Mon Film.mkv"
        #[qinvokable]
        #[cxx_name = "fileName"]
        fn file_name(self: &Utils, url: &QString) -> QString;

        /// Chemin local d'une URL : "file:///films/Mon%20Film.mkv" -> "/films/Mon Film.mkv"
        #[qinvokable]
        #[cxx_name = "localPath"]
        fn local_path(self: &Utils, url: &QString) -> QString;

        /// Adresse de la miniature de `url` à `seconds` (vide pour un flux réseau)
        #[qinvokable]
        #[cxx_name = "thumbnailUrl"]
        fn thumbnail_url(self: &Utils, url: &QString, seconds: f64) -> QString;

        /// yt-dlp à utiliser pour les vidéos en ligne (vide : celui du PATH)
        #[qinvokable]
        #[cxx_name = "ytdlPath"]
        fn ytdl_path(self: &Utils) -> QString;

        /// Moteur JavaScript pour yt-dlp, au format « node:/chemin » (vide si aucun)
        #[qinvokable]
        #[cxx_name = "jsRuntime"]
        fn js_runtime(self: &Utils) -> QString;

        /// Vrai pour une adresse en ligne (http, https…) plutôt qu'un fichier local
        #[qinvokable]
        #[cxx_name = "isOnline"]
        fn is_online(self: &Utils, url: &QString) -> bool;

        /// Vrai si l'URL désigne un dossier local
        #[qinvokable]
        #[cxx_name = "isFolder"]
        fn is_folder(self: &Utils, url: &QString) -> bool;

        /// Vrai si l'URL désigne un fichier de sous-titres (.srt, .ass…)
        #[qinvokable]
        #[cxx_name = "isSubtitle"]
        fn is_subtitle(self: &Utils, url: &QString) -> bool;

        /// Nom lisible d'une piste : "Français · AC3 5.1"
        #[qinvokable]
        #[cxx_name = "trackLabel"]
        fn track_label(
            self: &Utils,
            id: i64,
            title: &QString,
            lang: &QString,
            codec: &QString,
            channels: i64,
        ) -> QString;
    }
}

use cxx_qt_lib::QString;

#[derive(Default)]
pub struct UtilsRust;

impl qobject::Utils {
    pub fn format_time(&self, seconds: f64) -> QString {
        QString::from(&format_time(seconds))
    }

    pub fn format_clock(&self, seconds: f64) -> QString {
        QString::from(&format_clock(seconds))
    }

    pub fn file_extension(&self, url: &QString) -> QString {
        QString::from(&file_extension(&url.to_string()))
    }

    pub fn media_info(
        &self,
        video_codec: &QString,
        width: i64,
        height: i64,
        audio_codec: &QString,
        sample_rate: i64,
        audio_bitrate: f64,
    ) -> QString {
        QString::from(&media_info(
            &video_codec.to_string(),
            width,
            height,
            &audio_codec.to_string(),
            sample_rate,
            audio_bitrate,
        ))
    }

    pub fn file_name(&self, url: &QString) -> QString {
        QString::from(&file_name(&url.to_string()))
    }

    pub fn local_path(&self, url: &QString) -> QString {
        QString::from(&local_path(&url.to_string()))
    }

    pub fn thumbnail_url(&self, url: &QString, seconds: f64) -> QString {
        QString::from(&thumbnail_url(&url.to_string(), seconds))
    }

    pub fn ytdl_path(&self) -> QString {
        let found = find_ytdl(&home(), &std::env::var("PATH").unwrap_or_default());
        QString::from(&found.map(|p| p.to_string_lossy().into_owned()).unwrap_or_default())
    }

    pub fn js_runtime(&self) -> QString {
        let found = find_js_runtime(&home(), &std::env::var("PATH").unwrap_or_default());
        QString::from(&found.unwrap_or_default())
    }

    pub fn is_online(&self, url: &QString) -> bool {
        is_online(&url.to_string())
    }

    pub fn is_folder(&self, url: &QString) -> bool {
        std::path::Path::new(&local_path(&url.to_string())).is_dir()
    }

    pub fn is_subtitle(&self, url: &QString) -> bool {
        is_subtitle(&url.to_string())
    }

    pub fn track_label(
        &self,
        id: i64,
        title: &QString,
        lang: &QString,
        codec: &QString,
        channels: i64,
    ) -> QString {
        QString::from(&track_label(
            id,
            &title.to_string(),
            &lang.to_string(),
            &codec.to_string(),
            channels,
        ))
    }
}

pub fn format_time(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "0:00".into();
    }
    let total = seconds.floor() as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub fn format_clock(seconds: f64) -> String {
    let total = if seconds.is_finite() && seconds > 0.0 { seconds.floor() as u64 } else { 0 };
    format!("{:02}:{:02}:{:02}", total / 3600, (total % 3600) / 60, total % 60)
}

pub fn file_extension(url: &str) -> String {
    let name = file_name(url);
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && ext.len() <= 5 => ext.to_ascii_uppercase(),
        _ => String::new(),
    }
}

/// Nom usuel d'un codec à partir du nom FFmpeg utilisé par mpv.
fn codec_name(codec: &str) -> String {
    match codec.to_ascii_lowercase().as_str() {
        "h264" => "H.264".into(),
        "hevc" | "h265" => "HEVC".into(),
        "mpeg4" => "MPEG-4".into(),
        "mpeg2video" => "MPEG-2".into(),
        "eac3" => "E-AC3".into(),
        "truehd" => "TrueHD".into(),
        "vorbis" => "Vorbis".into(),
        "opus" => "Opus".into(),
        c if c.starts_with("pcm_") => "PCM".into(),
        c if c.starts_with("dts") => "DTS".into(),
        c => c.to_ascii_uppercase(),
    }
}

/// "44,1 kHz", "48 kHz"
fn sample_rate_name(rate: i64) -> Option<String> {
    (rate > 0).then(|| {
        let khz = rate as f64 / 1000.0;
        if khz.fract() == 0.0 {
            format!("{khz:.0} kHz")
        } else {
            format!("{khz:.1} kHz").replace('.', ",")
        }
    })
}

pub fn media_info(
    video_codec: &str,
    width: i64,
    height: i64,
    audio_codec: &str,
    sample_rate: i64,
    audio_bitrate: f64,
) -> String {
    let mut parts = Vec::new();
    let has_video = !video_codec.is_empty() && width > 0 && height > 0;
    if has_video {
        parts.push(format!("{} {width}×{height}", codec_name(video_codec)));
    }
    if !audio_codec.is_empty() {
        parts.push(codec_name(audio_codec));
        // Le débit audio n'est parlant que pour un fichier audio
        if !has_video && audio_bitrate.is_finite() && audio_bitrate > 0.0 {
            parts.push(format!("{:.0} kbps", audio_bitrate / 1000.0));
        }
        if let Some(rate) = sample_rate_name(sample_rate) {
            if has_video {
                let last = parts.pop().unwrap_or_default();
                parts.push(format!("{last} {rate}"));
            } else {
                parts.push(rate);
            }
        }
    }
    parts.join(" · ")
}

pub fn file_name(url: &str) -> String {
    let last = url.trim_end_matches('/').rsplit('/').next().unwrap_or(url);
    percent_decode(last)
}

/// Chemin local décodé pour une URL file://, ou l'URL telle quelle.
pub fn local_path(url: &str) -> String {
    match url.strip_prefix("file://") {
        Some(path) => percent_decode(path),
        None => url.to_string(),
    }
}

/// « image://thumbnail/<secondes>/<chemin UTF-8 en hexadécimal> » (voir cpp/thumbnail.cpp).
/// L'hexadécimal évite tout souci d'encodage d'URL avec les accents, espaces, # ou %.
pub fn thumbnail_url(url: &str, seconds: f64) -> String {
    let path = local_path(url);
    if !path.starts_with('/') || !seconds.is_finite() {
        return String::new();
    }
    let hex: String = path.bytes().map(|b| format!("{b:02x}")).collect();
    format!("image://thumbnail/{:.1}/{hex}", seconds.max(0.0))
}

use std::path::{Path, PathBuf};

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

fn in_path(name: &str, path_env: &str) -> Option<PathBuf> {
    path_env.split(':').filter(|d| !d.is_empty()).map(|d| Path::new(d).join(name)).find(|p| is_executable(p))
}

/// yt-dlp à utiliser : celui de ~/.local/bin d'abord (installé à la main, donc récent ;
/// celui des dépôts est souvent trop vieux pour YouTube), sinon le premier du PATH.
pub fn find_ytdl(home: &Path, path_env: &str) -> Option<PathBuf> {
    let local = home.join(".local/bin/yt-dlp");
    if is_executable(&local) {
        return Some(local);
    }
    in_path("yt-dlp", path_env)
}

/// Moteur JavaScript dont yt-dlp a besoin pour YouTube : deno ou node dans le PATH, sinon
/// le node le plus récent de nvm (absent du PATH des applications lancées depuis le bureau).
pub fn find_js_runtime(home: &Path, path_env: &str) -> Option<String> {
    for name in ["deno", "node"] {
        if let Some(path) = in_path(name, path_env) {
            return Some(format!("{name}:{}", path.display()));
        }
    }
    let versions = std::fs::read_dir(home.join(".nvm/versions/node")).ok()?;
    let mut nodes: Vec<PathBuf> = versions
        .filter_map(Result::ok)
        .map(|e| e.path().join("bin/node"))
        .filter(|p| is_executable(p))
        .collect();
    // Version la plus récente : tri naturel (v24 après v9)
    nodes.sort_by(|a, b| crate::bridge::playlist::natural_cmp(&a.to_string_lossy(), &b.to_string_lossy()));
    nodes.pop().map(|p| format!("node:{}", p.display()))
}

pub fn is_online(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    ["http://", "https://", "rtmp://", "rtsp://", "ytdl://"].iter().any(|scheme| lower.starts_with(scheme))
}

const SUBTITLE_EXTENSIONS: &[&str] = &["srt", "ass", "ssa", "vtt", "sub", "sup", "idx", "smi"];

pub fn is_subtitle(url: &str) -> bool {
    url.rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .is_some_and(|ext| SUBTITLE_EXTENSIONS.contains(&ext.as_str()))
}

/// Nom de langue à partir d'un code ISO 639-1 ou 639-2 (formats utilisés par mpv).
pub fn language_name(code: &str) -> Option<&'static str> {
    let name = match code.to_ascii_lowercase().as_str() {
        "fr" | "fre" | "fra" => "Français",
        "en" | "eng" => "Anglais",
        "es" | "spa" => "Espagnol",
        "de" | "ger" | "deu" => "Allemand",
        "it" | "ita" => "Italien",
        "pt" | "por" => "Portugais",
        "nl" | "dut" | "nld" => "Néerlandais",
        "ru" | "rus" => "Russe",
        "pl" | "pol" => "Polonais",
        "tr" | "tur" => "Turc",
        "ar" | "ara" => "Arabe",
        "he" | "heb" => "Hébreu",
        "hi" | "hin" => "Hindi",
        "ja" | "jpn" => "Japonais",
        "ko" | "kor" => "Coréen",
        "zh" | "chi" | "zho" => "Chinois",
        "sv" | "swe" => "Suédois",
        "da" | "dan" => "Danois",
        "no" | "nor" | "nob" => "Norvégien",
        "fi" | "fin" => "Finnois",
        _ => return None,
    };
    Some(name)
}

fn channels_name(channels: i64) -> Option<String> {
    match channels {
        n if n <= 0 => None,
        1 => Some("Mono".into()),
        2 => Some("Stéréo".into()),
        6 => Some("5.1".into()),
        8 => Some("7.1".into()),
        n => Some(format!("{n} canaux")),
    }
}

pub fn track_label(id: i64, title: &str, lang: &str, codec: &str, channels: i64) -> String {
    let title = title.trim();
    let lang = lang.trim();
    let mut parts: Vec<String> = Vec::new();

    let language = language_name(lang).map(str::to_string).or_else(|| {
        (!lang.is_empty() && lang != "und").then(|| lang.to_ascii_uppercase())
    });
    if let Some(language) = &language {
        parts.push(language.clone());
    }
    // Le titre n'apporte rien s'il répète la langue
    if !title.is_empty() && language.as_deref() != Some(title) {
        parts.push(title.to_string());
    }
    if parts.is_empty() {
        parts.push(format!("Piste {id}"));
    }

    let technical: Vec<String> = [
        (!codec.is_empty()).then(|| codec.to_ascii_uppercase()),
        channels_name(channels),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !technical.is_empty() {
        parts.push(technical.join(" "));
    }
    parts.join(" · ")
}

pub(crate) fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_time() {
        assert_eq!(format_time(0.0), "0:00");
        assert_eq!(format_time(75.3), "1:15");
        assert_eq!(format_time(3725.0), "1:02:05");
        assert_eq!(format_time(f64::NAN), "0:00");
    }

    #[test]
    fn extracts_file_name() {
        assert_eq!(file_name("file:///films/Mon%20Film.mkv"), "Mon Film.mkv");
        assert_eq!(file_name("file:///a/%C3%A9t%C3%A9.mp4"), "été.mp4");
    }

    #[test]
    fn formats_clock() {
        assert_eq!(format_clock(91.4), "00:01:31");
        assert_eq!(format_clock(3725.0), "01:02:05");
        assert_eq!(format_clock(-3.0), "00:00:00");
        assert_eq!(format_clock(f64::NAN), "00:00:00");
    }

    #[test]
    fn extracts_extensions() {
        assert_eq!(file_extension("file:///films/Mon%20Film.mkv"), "MKV");
        assert_eq!(file_extension("/a/b/chanson.Mp3"), "MP3");
        assert_eq!(file_extension("/a/.cache"), "");
        assert_eq!(file_extension("/a/sans-extension"), "");
    }

    #[test]
    fn describes_media() {
        assert_eq!(media_info("h264", 1920, 1080, "aac", 48000, 192000.0), "H.264 1920×1080 · AAC 48 kHz");
        assert_eq!(media_info("hevc", 3840, 2160, "eac3", 0, 0.0), "HEVC 3840×2160 · E-AC3");
        assert_eq!(media_info("", 0, 0, "mp3", 44100, 320000.0), "MP3 · 320 kbps · 44,1 kHz");
        // Pochette d'album : mpv la voit comme une piste vidéo sans dimensions utiles
        assert_eq!(media_info("mjpeg", 0, 0, "flac", 96000, 0.0), "FLAC · 96 kHz");
        assert_eq!(media_info("", 0, 0, "", 0, 0.0), "");
    }

    #[test]
    fn converts_local_paths() {
        assert_eq!(local_path("file:///films/Mon%20Film.mkv"), "/films/Mon Film.mkv");
        assert_eq!(local_path("file:///films/Mon Film.mkv"), "/films/Mon Film.mkv");
        assert_eq!(local_path("https://exemple.org/v.mp4"), "https://exemple.org/v.mp4");
    }

    #[test]
    fn builds_thumbnail_urls() {
        assert_eq!(thumbnail_url("file:///a/%C3%A9%20%23.mkv", 12.34), "image://thumbnail/12.3/2f612fc3a920232e6d6b76");
        assert_eq!(thumbnail_url("file:///a/b.mkv", -5.0), "image://thumbnail/0.0/2f612f622e6d6b76");
        assert_eq!(thumbnail_url("https://exemple.org/v.mp4", 10.0), "");
    }

    #[test]
    fn finds_online_tools() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("lumen-tools-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let executable = |path: &Path| {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"").unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        };
        let home = root.join("home");
        let system = root.join("usr/bin");
        let path_env = format!("/inexistant:{}", system.display());

        // yt-dlp : seulement celui du système, puis celui de ~/.local/bin en priorité
        executable(&system.join("yt-dlp"));
        assert_eq!(find_ytdl(&home, &path_env), Some(system.join("yt-dlp")));
        executable(&home.join(".local/bin/yt-dlp"));
        assert_eq!(find_ytdl(&home, &path_env), Some(home.join(".local/bin/yt-dlp")));

        // Moteur JavaScript : aucun, puis le node le plus récent de nvm, puis celui du PATH
        assert_eq!(find_js_runtime(&home, &path_env), None);
        executable(&home.join(".nvm/versions/node/v9.0.0/bin/node"));
        executable(&home.join(".nvm/versions/node/v24.21.0/bin/node"));
        let nvm = find_js_runtime(&home, &path_env).unwrap();
        assert!(nvm.starts_with("node:") && nvm.ends_with("v24.21.0/bin/node"), "{nvm}");
        executable(&system.join("node"));
        assert_eq!(find_js_runtime(&home, &path_env), Some(format!("node:{}", system.join("node").display())));

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn detects_online_urls() {
        assert!(is_online("https://www.youtube.com/watch?v=aqz-KE-bpKQ"));
        assert!(is_online("  HTTP://exemple.org/v.mp4"));
        assert!(!is_online("file:///films/a.mkv"));
        assert!(!is_online("/films/a.mkv"));
    }

    #[test]
    fn detects_subtitles() {
        assert!(is_subtitle("file:///films/Film.FR.srt"));
        assert!(is_subtitle("/films/film.ASS"));
        assert!(!is_subtitle("file:///films/Film.mkv"));
        assert!(!is_subtitle("sans-extension"));
    }

    #[test]
    fn labels_tracks() {
        assert_eq!(track_label(1, "", "fre", "ac3", 6), "Français · AC3 5.1");
        assert_eq!(track_label(2, "Commentaires", "eng", "aac", 2), "Anglais · Commentaires · AAC Stéréo");
        assert_eq!(track_label(3, "Français", "fr", "subrip", 0), "Français · SUBRIP");
        assert_eq!(track_label(4, "", "und", "", 0), "Piste 4");
        assert_eq!(track_label(5, "", "xyz", "", 0), "XYZ");
    }
}
