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
