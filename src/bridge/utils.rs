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
