//! Listes de chaînes IPTV : fichiers ou liens M3U (« #EXTINF ») et comptes Xtream Codes,
//! dont l'adresse de liste se construit à partir du serveur, de l'utilisateur et du mot de passe.
//! Une liste HLS (`.m3u8` d'un seul flux, balises « #EXT-X- ») n'est pas une liste de chaînes :
//! elle se lit directement.

/// Une chaîne de la liste
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub url: String,
    pub name: String,
    pub group: String,
}

/// Contenu d'une liste M3U
#[derive(Debug, PartialEq)]
pub enum List {
    /// Liste de chaînes (au moins une)
    Channels(Vec<Channel>),
    /// Flux HLS : à donner tel quel au lecteur
    Stream,
}

/// Vrai si l'adresse ressemble à une liste M3U (extension, ou lien Xtream `get.php?…type=m3u`).
pub fn is_list_url(url: &str) -> bool {
    let lower = url.trim().to_ascii_lowercase();
    let path = lower.split(['?', '#']).next().unwrap_or("");
    path.ends_with(".m3u") || path.ends_with(".m3u8") || (path.ends_with("/get.php") && lower.contains("type=m3u"))
}

/// Analyse une liste M3U. `base` sert à résoudre les adresses relatives.
/// `None` : pas une liste de lecture (aucune entrée).
pub fn parse(text: &str, base: &str) -> Option<List> {
    let text = text.trim_start_matches('\u{feff}');
    if text.lines().any(|l| {
        let l = l.trim_start();
        l.starts_with("#EXT-X-TARGETDURATION") || l.starts_with("#EXT-X-STREAM-INF") || l.starts_with("#EXT-X-MEDIA-SEQUENCE")
    }) {
        return Some(List::Stream);
    }

    // Sans en-tête #EXTM3U, n'accepter que des adresses (pas une page d'erreur HTML)
    let header = text.lines().map(str::trim).find(|l| !l.is_empty()).is_some_and(|l| l.starts_with("#EXTM3U"));
    let mut channels: Vec<Channel> = Vec::new();
    let (mut name, mut group) = (String::new(), String::new());
    for line in text.lines().map(str::trim) {
        if let Some(info) = line.strip_prefix("#EXTINF:") {
            let (attributes, title) = split_extinf(info);
            name = title.to_string();
            group = attribute(attributes, "group-title").unwrap_or_default();
            if name.is_empty() {
                name = attribute(attributes, "tvg-name").unwrap_or_default();
            }
        } else if let Some(g) = line.strip_prefix("#EXTGRP:") {
            group = g.trim().to_string();
        } else if !line.is_empty() && !line.starts_with('#') {
            if !header && !(line.contains("://") || line.starts_with('/')) || line.contains('<') {
                return None;
            }
            let url = resolve(line, base);
            if !channels.iter().any(|c| c.url == url) {
                channels.push(Channel { url, name: std::mem::take(&mut name), group: std::mem::take(&mut group) });
            }
            name.clear();
            group.clear();
        }
    }
    (!channels.is_empty()).then_some(List::Channels(channels))
}

/// Adresse de la liste M3U d'un compte Xtream Codes.
pub fn xtream_url(server: &str, user: &str, password: &str) -> String {
    let server = server.trim().trim_end_matches('/');
    let server = if server.contains("://") { server.to_string() } else { format!("http://{server}") };
    format!(
        "{server}/get.php?username={}&password={}&type=m3u_plus&output=ts",
        encode(user.trim()),
        encode(password.trim())
    )
}

/// Nom affiché d'une chaîne : « Groupe · Nom », ou le nom seul
pub fn title(channel: &Channel) -> String {
    match (channel.group.is_empty(), channel.name.is_empty()) {
        (_, true) => String::new(),
        (true, false) => channel.name.clone(),
        (false, false) => format!("{} · {}", channel.group, channel.name),
    }
}

/// Sépare les attributs (`tvg-id="…" group-title="…"`) du titre, après la première
/// virgule hors guillemets.
fn split_extinf(info: &str) -> (&str, &str) {
    let mut quoted = false;
    for (i, c) in info.char_indices() {
        match c {
            '"' => quoted = !quoted,
            ',' if !quoted => return (&info[..i], info[i + 1..].trim()),
            _ => {}
        }
    }
    (info, "")
}

fn attribute(attributes: &str, key: &str) -> Option<String> {
    let start = attributes.find(&format!("{key}=\""))? + key.len() + 2;
    let end = attributes[start..].find('"')?;
    Some(attributes[start..start + end].trim().to_string()).filter(|v| !v.is_empty())
}

fn resolve(url: &str, base: &str) -> String {
    if url.contains("://") || url.starts_with('/') || base.is_empty() {
        return url.to_string();
    }
    match base.rfind('/') {
        Some(i) => format!("{}/{url}", &base[..i]),
        None => url.to_string(),
    }
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_list_urls() {
        assert!(is_list_url("http://exemple.tv/liste.m3u"));
        assert!(is_list_url("https://exemple.tv/live.M3U8?token=1"));
        assert!(is_list_url("http://srv:8080/get.php?username=a&password=b&type=m3u_plus"));
        assert!(is_list_url("/home/moi/chaines.m3u"));
        assert!(!is_list_url("http://exemple.tv/film.ts"));
        assert!(!is_list_url("https://www.youtube.com/watch?v=x"));
    }

    #[test]
    fn parses_channels_with_attributes() {
        let text = "\u{feff}#EXTM3U\n\
            #EXTINF:-1 tvg-id=\"a\" tvg-name=\"Info, 24h\" group-title=\"Actualités\",Info 24h\n\
            http://srv/live/1.ts\n\
            #EXTINF:-1,Sport\n\
            #EXTVLCOPT:http-user-agent=X\n\
            http://srv/live/2.ts\n\
            #EXTINF:-1 tvg-name=\"Cinéma\",\n\
            relatif/3.m3u8\n\
            http://srv/live/1.ts\n";
        let Some(List::Channels(channels)) = parse(text, "http://srv/listes/tout.m3u") else { panic!() };
        assert_eq!(channels.len(), 3, "doublon ignoré");
        assert_eq!(channels[0], Channel { url: "http://srv/live/1.ts".into(), name: "Info 24h".into(), group: "Actualités".into() });
        assert_eq!(title(&channels[0]), "Actualités · Info 24h");
        assert_eq!(channels[1].name, "Sport");
        assert_eq!(title(&channels[1]), "Sport");
        assert_eq!(channels[2].name, "Cinéma", "tvg-name quand le titre est vide");
        assert_eq!(channels[2].url, "http://srv/listes/relatif/3.m3u8");
    }

    #[test]
    fn hls_playlist_is_a_stream() {
        let hls = "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:4\n#EXTINF:4.0,\nlive0.ts\n";
        assert_eq!(parse(hls, "http://srv/live.m3u8"), Some(List::Stream));
        let master = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=800000\nbas.m3u8\n";
        assert_eq!(parse(master, ""), Some(List::Stream));
    }

    #[test]
    fn empty_list_is_rejected() {
        assert_eq!(parse("#EXTM3U\n", ""), None);
        assert_eq!(parse("<html>Erreur</html>\n", ""), None);
        assert_eq!(parse("Accès refusé\n", ""), None);
        let Some(List::Channels(plain)) = parse("http://a/1.ts\nhttp://a/2.ts\n", "") else { panic!() };
        assert_eq!(plain.len(), 2, "liste simple sans en-tête");
    }

    #[test]
    fn builds_xtream_url() {
        assert_eq!(
            xtream_url(" srv.tv:8080/ ", "moi", "p@ss word"),
            "http://srv.tv:8080/get.php?username=moi&password=p%40ss%20word&type=m3u_plus&output=ts"
        );
        assert!(xtream_url("https://srv.tv", "a", "b").starts_with("https://srv.tv/get.php?"));
    }
}
