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
}
