//! Recherche et téléchargement de sous-titres sur OpenSubtitles (API REST v1).
//!
//! Une clé d'API gratuite est nécessaire (compte OpenSubtitles › « API consumers »). La
//! recherche combine l'empreinte du fichier (correspondance exacte avec cette version de
//! la vidéo) et son nom nettoyé. Le réseau tourne dans un thread ; les résultats reviennent
//! au thread de Qt par des signaux.

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
        #[qproperty(QString, api_key)]
        // Une recherche ou un téléchargement est en cours
        #[qproperty(bool, busy)]
        type SubtitleSearch = super::SubtitleSearchRust;

        /// Titre à chercher, déduit du nom de fichier (« Film.2020.1080p.mkv » -> « Film 2020 »)
        #[qinvokable]
        fn suggested_query(self: &SubtitleSearch, url: &QString, title: &QString) -> QString;

        /// Lance une recherche. Résultat : `found` (JSON) ou `failed` (message).
        #[qinvokable]
        fn search(self: Pin<&mut SubtitleSearch>, url: &QString, query: &QString, languages: &QString);

        /// Télécharge un résultat et l'enregistre à côté de la vidéo. Résultat : `downloaded`
        /// (chemin du fichier) ou `failed`.
        #[qinvokable]
        fn download(self: Pin<&mut SubtitleSearch>, file_id: i64, language: &QString, url: &QString);

        /// Résultats : tableau JSON de { fileId, language, release, downloads, exact, hearingImpaired }
        #[qsignal]
        fn found(self: Pin<&mut SubtitleSearch>, results: QString);

        #[qsignal]
        fn downloaded(self: Pin<&mut SubtitleSearch>, path: QString);

        #[qsignal]
        fn failed(self: Pin<&mut SubtitleSearch>, message: QString);
    }

    impl cxx_qt::Threading for SubtitleSearch {}
}

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::LazyLock;
use std::time::Duration;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use regex::Regex;
use serde_json::{json, Value};

use super::utils::{file_name, is_online, local_path};

const API: &str = "https://api.opensubtitles.com/api/v1";
const USER_AGENT: &str = "Lumen v0.1";

#[derive(Default)]
pub struct SubtitleSearchRust {
    api_key: QString,
    busy: bool,
}

impl qobject::SubtitleSearch {
    pub fn suggested_query(&self, url: &QString, title: &QString) -> QString {
        let url = url.to_string();
        // Vidéo en ligne : le titre de la page ; fichier : son nom
        let source = if is_online(&url) { title.to_string() } else { file_name(&url) };
        QString::from(&clean_query(&source))
    }

    pub fn search(mut self: Pin<&mut Self>, url: &QString, query: &QString, languages: &QString) {
        let Some(key) = self.as_mut().begin() else { return };
        let path = local_path(&url.to_string());
        let hash = if is_online(&path) { None } else { movie_hash(Path::new(&path)).ok() };
        let (query, languages) = (query.to_string(), api_languages(&languages.to_string()));
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let result = search(&key, hash.as_deref(), &query, &languages);
            let _ = thread.queue(move |mut this| {
                this.as_mut().set_busy(false);
                match result {
                    Ok(results) => this.as_mut().found(QString::from(&results.to_string())),
                    Err(message) => this.as_mut().failed(QString::from(&message)),
                }
            });
        });
    }

    pub fn download(mut self: Pin<&mut Self>, file_id: i64, language: &QString, url: &QString) {
        let Some(key) = self.as_mut().begin() else { return };
        let target = subtitle_target(&url.to_string(), &language.to_string());
        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let result = download(&key, file_id, &target);
            let _ = thread.queue(move |mut this| {
                this.as_mut().set_busy(false);
                match result {
                    Ok(path) => this.as_mut().downloaded(QString::from(&path.to_string_lossy().into_owned())),
                    Err(message) => this.as_mut().failed(QString::from(&message)),
                }
            });
        });
    }

    /// Clé d'API, ou signal `failed` si elle manque ; marque l'objet occupé
    fn begin(mut self: Pin<&mut Self>) -> Option<String> {
        let key = self.rust().api_key.to_string().trim().to_string();
        if key.is_empty() {
            self.as_mut().failed(QString::from("Clé d'API OpenSubtitles manquante (Préférences › Sous-titres)"));
            return None;
        }
        self.as_mut().set_busy(true);
        Some(key)
    }
}

// ------------------------------------------------------------------- Réseau

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        // Les erreurs de l'API ont un message JSON utile : les lire plutôt qu'échouer tout de suite
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(20)))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

/// Message d'erreur lisible pour une réponse HTTP en échec
fn api_error(status: u16, body: &Value) -> String {
    let detail = body["message"].as_str().or_else(|| body["errors"][0].as_str()).unwrap_or("");
    match status {
        401 | 403 => "Clé d'API OpenSubtitles refusée (vérifie-la dans les préférences)".to_string(),
        406 | 429 => format!("Limite d'OpenSubtitles atteinte, réessaie plus tard. {detail}").trim().to_string(),
        _ => format!("OpenSubtitles : erreur {status} {detail}").trim().to_string(),
    }
}

fn search(key: &str, hash: Option<&str>, query: &str, languages: &str) -> Result<Value, String> {
    // Paramètres en minuscules et triés : l'API redirige sinon (301)
    let mut params: Vec<(&str, String)> = vec![("languages", languages.to_string())];
    if let Some(hash) = hash {
        params.push(("moviehash", hash.to_string()));
    }
    if !query.trim().is_empty() {
        params.push(("query", query.trim().to_lowercase()));
    }
    params.sort_by_key(|(name, _)| *name);
    let query_string: String = params
        .iter()
        .filter(|(_, value)| !value.is_empty())
        .map(|(name, value)| format!("{name}={}", form_encode(value)))
        .collect::<Vec<_>>()
        .join("&");

    let mut response = agent()
        .get(&format!("{API}/subtitles?{query_string}"))
        .header("Api-Key", key)
        .header("Accept", "application/json")
        .call()
        .map_err(|e| format!("OpenSubtitles injoignable ({e})"))?;
    let status = response.status().as_u16();
    let body: Value = response.body_mut().read_json().unwrap_or(Value::Null);
    if status != 200 {
        return Err(api_error(status, &body));
    }
    Ok(parse_results(&body))
}

fn download(key: &str, file_id: i64, target: &Path) -> Result<PathBuf, String> {
    let agent = agent();
    let mut response = agent
        .post(&format!("{API}/download"))
        .header("Api-Key", key)
        .header("Accept", "application/json")
        .send_json(json!({ "file_id": file_id }))
        .map_err(|e| format!("OpenSubtitles injoignable ({e})"))?;
    let status = response.status().as_u16();
    let body: Value = response.body_mut().read_json().unwrap_or(Value::Null);
    let Some(link) = body["link"].as_str().filter(|_| status == 200) else {
        return Err(api_error(status, &body));
    };

    let mut file = agent.get(link).call().map_err(|e| format!("Téléchargement impossible ({e})"))?;
    // Octets bruts : beaucoup de sous-titres ne sont pas en UTF-8 (mpv détecte l'encodage)
    let bytes = file
        .body_mut()
        .with_config()
        .limit(10 * 1024 * 1024)
        .read_to_vec()
        .map_err(|e| format!("Téléchargement interrompu ({e})"))?;
    write_subtitle(target, &bytes)
}

/// Écrit à côté de la vidéo, ou dans ~/.local/share/lumen/subtitles si c'est impossible
/// (dossier en lecture seule, vidéo en ligne)
fn write_subtitle(target: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let fallback = data_dir().join("subtitles").join(target.file_name().unwrap_or_default());
    for path in [target.to_path_buf(), fallback] {
        let path = unique_path(&path);
        if path.parent().is_some_and(|dir| std::fs::create_dir_all(dir).is_ok()) && std::fs::write(&path, bytes).is_ok() {
            return Ok(path);
        }
    }
    Err("Impossible d'enregistrer les sous-titres".to_string())
}

fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lumen")
}

/// « Film.fr.srt » existe déjà : « Film.fr.2.srt »
fn unique_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let ext = path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    (2..)
        .map(|n| path.with_file_name(format!("{stem}.{n}.{ext}")))
        .find(|p| !p.exists())
        .unwrap_or_else(|| path.to_path_buf())
}

fn form_encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b',' => (b as char).to_string(),
            b' ' => "+".to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

// ------------------------------------------------------------- Logique pure

/// Empreinte OpenSubtitles : taille du fichier + somme des mots de 64 bits des premiers et
/// derniers 64 Kio, en hexadécimal sur 16 chiffres.
pub fn movie_hash(path: &Path) -> std::io::Result<String> {
    const CHUNK: u64 = 64 * 1024;
    let mut file = std::fs::File::open(path)?;
    let size = file.metadata()?.len();
    let mut hash = size;
    let mut add_chunk = |file: &mut std::fs::File, from: u64| -> std::io::Result<()> {
        file.seek(SeekFrom::Start(from))?;
        let mut buffer = vec![0u8; CHUNK.min(size) as usize];
        file.read_exact(&mut buffer)?;
        for word in buffer.chunks_exact(8) {
            hash = hash.wrapping_add(u64::from_le_bytes(word.try_into().unwrap()));
        }
        Ok(())
    };
    add_chunk(&mut file, 0)?;
    add_chunk(&mut file, size.saturating_sub(CHUNK))?;
    Ok(format!("{hash:016x}"))
}

static NOISE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^(2160p|1080p|1080i|720p|480p|4k|uhd|hdr|hdr10|dv|x264|x265|h264|h265|hevc|avc|av1|10bit|webrip|web|webdl|web-dl|bluray|blu-ray|bdrip|brrip|dvdrip|hdtv|hdrip|remux|multi|vf|vff|vfq|vostfr|french|truefrench|subfrench|english|proper|repack|extended|aac|ac3|eac3|dts|ddp|ddp5|atmos)$",
    )
    .unwrap()
});
static EPISODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^s\d{1,2}e\d{1,3}$").unwrap());

/// Titre de recherche à partir d'un nom de fichier : séparateurs remplacés par des espaces,
/// arrêt aux mentions techniques (« 1080p », « WEBRip »…), épisode conservé.
pub fn clean_query(name: &str) -> String {
    let stem = match name.rsplit_once('.') {
        Some((stem, ext)) if ext.len() <= 4 && ext.chars().all(|c| c.is_ascii_alphanumeric()) => stem,
        _ => name,
    };
    let mut words = Vec::new();
    for word in stem.split(|c: char| c == '.' || c == '_' || c.is_whitespace() || c == '[' || c == ']' || c == '(' || c == ')') {
        if word.is_empty() || word == "-" {
            continue;
        }
        if NOISE.is_match(word) {
            break;
        }
        words.push(word);
        if EPISODE.is_match(word) {
            break; // « Série S01E02 » : la suite est le titre de l'épisode ou des mentions techniques
        }
    }
    words.join(" ")
}

/// Codes de langue pour l'API (ISO 639-1, « pt-br »…) à partir des préférences ;
/// français et anglais si rien n'est réglé
pub fn api_languages(preferred: &str) -> String {
    let codes: Vec<String> = preferred
        .split(',')
        .map(|c| c.trim().to_lowercase())
        .filter(|c| !c.is_empty())
        .map(|c| match c.as_str() {
            "fre" | "fra" => "fr".into(),
            "eng" => "en".into(),
            "spa" => "es".into(),
            "ger" | "deu" => "de".into(),
            "ita" => "it".into(),
            "por" => "pt-pt".into(),
            "dut" | "nld" => "nl".into(),
            "rus" => "ru".into(),
            "jpn" => "ja".into(),
            "kor" => "ko".into(),
            "chi" | "zho" => "zh-cn".into(),
            "ara" => "ar".into(),
            other => other.to_string(),
        })
        .collect();
    let mut unique: Vec<String> = Vec::new();
    for code in codes {
        if !unique.contains(&code) {
            unique.push(code);
        }
    }
    if unique.is_empty() { "fr,en".into() } else { unique.join(",") }
}

/// Résultats de recherche pour QML : correspondances exactes d'abord, puis les plus téléchargés
pub fn parse_results(body: &Value) -> Value {
    let mut results: Vec<Value> = body["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let attributes = &item["attributes"];
            let file = &attributes["files"][0];
            Some(json!({
                "fileId": file["file_id"].as_i64()?,
                "language": attributes["language"].as_str().unwrap_or(""),
                "release": attributes["release"].as_str().or_else(|| file["file_name"].as_str()).unwrap_or(""),
                "downloads": attributes["download_count"].as_i64().unwrap_or(0),
                "exact": attributes["moviehash_match"].as_bool().unwrap_or(false),
                "hearingImpaired": attributes["hearing_impaired"].as_bool().unwrap_or(false),
            }))
        })
        .collect();
    results.sort_by(|a, b| {
        b["exact"].as_bool().cmp(&a["exact"].as_bool()).then(b["downloads"].as_i64().cmp(&a["downloads"].as_i64()))
    });
    Value::Array(results)
}

/// Fichier de sous-titres à côté de la vidéo : « Film.mkv » -> « Film.fr.srt ».
/// Vidéo en ligne : dans le dossier de données de Lumen.
pub fn subtitle_target(url: &str, language: &str) -> PathBuf {
    let language: String = language.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
    let language = if language.is_empty() { "und".to_string() } else { language };
    let path = local_path(url);
    if is_online(&path) {
        let name: String = clean_query(&file_name(&path)).chars().filter(|c| c.is_alphanumeric() || *c == ' ').collect();
        let name = if name.trim().is_empty() { "video".to_string() } else { name.trim().to_string() };
        return data_dir().join("subtitles").join(format!("{name}.{language}.srt"));
    }
    let video = Path::new(&path);
    let stem = video.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "video".into());
    video.with_file_name(format!("{stem}.{language}.srt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_movie_hash() {
        let dir = std::env::temp_dir().join(format!("lumen-hash-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Fichier nul de 128 Kio : l'empreinte vaut la taille
        let zeros = dir.join("zeros.bin");
        std::fs::write(&zeros, vec![0u8; 131072]).unwrap();
        assert_eq!(movie_hash(&zeros).unwrap(), "0000000000020000");

        // Premier mot à 1 et dernier mot à 2 : taille + 1 + 2
        let mut bytes = vec![0u8; 200_000];
        bytes[0] = 1;
        let last_word = 200_000 - 8;
        bytes[last_word] = 2;
        let marked = dir.join("marked.bin");
        std::fs::write(&marked, bytes).unwrap();
        assert_eq!(movie_hash(&marked).unwrap(), format!("{:016x}", 200_000u64 + 1 + 2));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cleans_queries() {
        assert_eq!(clean_query("Soulm8te.2026.Multi.WEBRIP.mp4"), "Soulm8te 2026");
        assert_eq!(clean_query("The.Matrix.1999.1080p.BluRay.x264.mkv"), "The Matrix 1999");
        assert_eq!(clean_query("Ma Série - S01E02 - Le retour [1080p].mkv"), "Ma Série S01E02");
        assert_eq!(clean_query("Big Buck Bunny 60fps 4K - Official Blender Foundation Short Film"), "Big Buck Bunny 60fps");
        assert_eq!(clean_query("film_sans_extension"), "film sans extension");
    }

    #[test]
    fn maps_languages() {
        assert_eq!(api_languages(""), "fr,en");
        assert_eq!(api_languages("fre,eng,fr"), "fr,en");
        assert_eq!(api_languages("pt-BR, jpn"), "pt-br,ja");
    }

    #[test]
    fn parses_and_sorts_results() {
        let body = json!({ "data": [
            { "attributes": { "language": "fr", "release": "Film.2020.WEB", "download_count": 900,
                              "moviehash_match": false, "files": [{ "file_id": 11 }] } },
            { "attributes": { "language": "fr", "release": "Film.2020.1080p.BluRay", "download_count": 50,
                              "moviehash_match": true, "hearing_impaired": true, "files": [{ "file_id": 22 }] } },
            { "attributes": { "language": "en", "download_count": 5000, "files": [{ "file_id": 33, "file_name": "film.en.srt" }] } },
            { "attributes": { "language": "en", "files": [] } }
        ]});
        let results = parse_results(&body);
        let ids: Vec<i64> = results.as_array().unwrap().iter().map(|r| r["fileId"].as_i64().unwrap()).collect();
        assert_eq!(ids, [22, 33, 11]); // exacte d'abord, puis par téléchargements ; sans fichier : écarté
        assert_eq!(results[0]["hearingImpaired"], true);
        assert_eq!(results[1]["release"], "film.en.srt");
    }

    #[test]
    fn chooses_target_files() {
        assert_eq!(subtitle_target("file:///films/Mon%20Film.mkv", "fr"), PathBuf::from("/films/Mon Film.fr.srt"));
        assert_eq!(subtitle_target("/films/a.mp4", "pt-br"), PathBuf::from("/films/a.pt-br.srt"));
        let online = subtitle_target("https://www.youtube.com/watch?v=x", "en");
        assert!(online.ends_with("subtitles/watch v x.en.srt") || online.to_string_lossy().contains("/lumen/subtitles/"), "{online:?}");

        let dir = std::env::temp_dir().join(format!("lumen-unique-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let first = dir.join("a.fr.srt");
        std::fs::write(&first, b"x").unwrap();
        assert_eq!(unique_path(&first), dir.join("a.fr.2.srt"));
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(form_encode("le film d'été"), "le+film+d%27%C3%A9t%C3%A9");
    }
}
