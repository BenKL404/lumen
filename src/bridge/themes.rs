//! Thèmes de l'interface : palettes sombre, claire, OLED et skins JSON de l'utilisateur
//! (~/.config/lumen/skins/*.json), plus la couleur d'accent choisie.
//!
//! Une palette est un ensemble de couleurs nommées (« jetons ») lues par Main.qml ; aucune
//! couleur n'est écrite en dur ailleurs dans l'interface.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{json, Value};

/// Jetons d'une palette, dans l'ordre où un skin peut les redéfinir
pub const TOKENS: &[&str] = &[
    "background", // fond de la fenêtre et de la zone vidéo
    "chrome",     // barres fixes (titre, contrôles, playlist)
    "menu",       // menus et fenêtres (préférences…)
    "surface",    // éléments flottants (OSD, plein écran)
    "surfaceHover",
    "border",
    "text",
    "muted",
    "track",  // fond des barres de progression et curseurs
    "subtle", // fond discret des boutons, lignes alternées
    "raised", // séparateurs, sélection
    "strong", // contours marqués (touches)
    "field",  // fond des champs de saisie
    "shade",  // fond de la barre latérale des préférences
    "divider", // séparation playlist accolée / fenêtre
    "accent",
];

const DEFAULT_ACCENT: &str = "#FF8C1A";

/// Palette de base d'un thème intégré (sans l'accent)
fn base(theme: &str) -> BTreeMap<&'static str, &'static str> {
    let colors: &[(&str, &str)] = match theme {
        "light" => &[
            ("background", "#ECEAE6"),
            ("chrome", "#F7F5F2"),
            ("menu", "#FFFFFF"),
            ("surface", "#EBFFFFFF"),
            ("surfaceHover", "#12000000"),
            ("border", "#1F000000"),
            ("text", "#1C1E22"),
            ("muted", "#6B7078"),
            ("track", "#26000000"),
            ("subtle", "#0D000000"),
            ("raised", "#14000000"),
            ("strong", "#30000000"),
            ("field", "#0A000000"),
            ("shade", "#0A000000"),
            ("divider", "#26000000"),
        ],
        "oled" => &[
            ("background", "#000000"),
            ("chrome", "#000000"),
            ("menu", "#0C0C0C"),
            ("surface", "#E6000000"),
            ("surfaceHover", "#17FFFFFF"),
            ("border", "#1AFFFFFF"),
            ("text", "#ECE8E1"),
            ("muted", "#8B9099"),
            ("track", "#29FFFFFF"),
            ("subtle", "#0DFFFFFF"),
            ("raised", "#17FFFFFF"),
            ("strong", "#26FFFFFF"),
            ("field", "#14FFFFFF"),
            ("shade", "#00000000"),
            ("divider", "#26FFFFFF"),
        ],
        // Sombre : le thème d'origine de Lumen
        _ => &[
            ("background", "#0B0D10"),
            ("chrome", "#13161B"),
            ("menu", "#1A1E24"),
            ("surface", "#DB121419"),
            ("surfaceHover", "#14FFFFFF"),
            ("border", "#14FFFFFF"),
            ("text", "#ECE8E1"),
            ("muted", "#8B9099"),
            ("track", "#29FFFFFF"),
            ("subtle", "#0DFFFFFF"),
            ("raised", "#17FFFFFF"),
            ("strong", "#24FFFFFF"),
            ("field", "#40000000"),
            ("shade", "#2E000000"),
            ("divider", "#FF000000"),
        ],
    };
    colors.iter().copied().collect()
}

/// Couleur valide pour Qt : « #RGB », « #RRGGBB » ou « #AARRGGBB »
pub fn is_color(value: &str) -> bool {
    value.strip_prefix('#').is_some_and(|hex| {
        matches!(hex.len(), 3 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit())
    })
}

/// Couleur du texte posé sur l'accent : sombre sur un accent clair, blanc sinon
pub fn on_accent(accent: &str) -> &'static str {
    let hex = accent.trim_start_matches('#');
    let hex = if hex.len() == 8 { &hex[2..] } else { hex };
    let channel = |i: usize| {
        let pair = if hex.len() == 3 { hex[i..i + 1].repeat(2) } else { hex[i * 2..i * 2 + 2].to_string() };
        u8::from_str_radix(&pair, 16).unwrap_or(0) as f64 / 255.0
    };
    let luminance = 0.2126 * channel(0) + 0.7152 * channel(1) + 0.0722 * channel(2);
    if luminance > 0.5 { "#1A1206" } else { "#FFFFFF" }
}

/// Skin de l'utilisateur : thème de base, puis couleurs redéfinies
#[derive(Debug, Deserialize)]
struct Skin {
    name: Option<String>,
    base: Option<String>,
    #[serde(default)]
    colors: BTreeMap<String, String>,
}

pub fn skins_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lumen")
        .join("skins")
}

fn read_skin(dir: &Path, id: &str) -> Option<Skin> {
    let text = std::fs::read_to_string(dir.join(format!("{id}.json"))).ok()?;
    serde_json::from_str(&text).ok()
}

/// Thèmes proposés : intégrés, puis skins valides du dossier (identifiant « skin:<fichier> »)
pub fn themes(dir: &Path) -> Value {
    let mut list = vec![
        json!({ "id": "dark", "label": "Sombre" }),
        json!({ "id": "light", "label": "Clair" }),
        json!({ "id": "oled", "label": "OLED noir" }),
    ];
    let mut skins: Vec<(String, String)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .filter_map(|p| {
            let id = p.file_stem()?.to_string_lossy().into_owned();
            let skin = read_skin(dir, &id)?;
            Some((id.clone(), skin.name.unwrap_or(id)))
        })
        .collect();
    skins.sort();
    list.extend(skins.into_iter().map(|(id, name)| json!({ "id": format!("skin:{id}"), "label": name })));
    Value::Array(list)
}

/// Palette complète (jetons -> couleurs, plus « onAccent ») pour un thème et un accent.
/// Un accent vide garde celui du thème (ou du skin).
pub fn palette(theme: &str, accent: &str, dir: &Path) -> Value {
    let (base_name, overrides) = match theme.strip_prefix("skin:") {
        Some(id) => match read_skin(dir, id) {
            Some(skin) => (skin.base.unwrap_or_else(|| "dark".into()), skin.colors),
            None => ("dark".to_string(), BTreeMap::new()),
        },
        None => (theme.to_string(), BTreeMap::new()),
    };
    let mut colors: BTreeMap<String, String> =
        base(&base_name).into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    colors.insert("accent".into(), DEFAULT_ACCENT.into());
    // Couleurs du skin : seulement des jetons connus et des couleurs valides
    for (token, value) in overrides {
        if TOKENS.contains(&token.as_str()) && is_color(&value) {
            colors.insert(token, value);
        }
    }
    if is_color(accent.trim()) {
        colors.insert("accent".into(), accent.trim().to_string());
    }
    let on = on_accent(&colors["accent"]);
    let mut map: serde_json::Map<String, Value> = colors.into_iter().map(|(k, v)| (k, Value::String(v))).collect();
    map.insert("onAccent".into(), Value::String(on.into()));
    Value::Object(map)
}

/// Skin d'exemple écrit dans le dossier des skins s'il est vide, pour montrer le format
pub const EXAMPLE_SKIN: &str = r##"{
  "name": "Nord (exemple)",
  "base": "dark",
  "colors": {
    "background": "#2E3440",
    "chrome": "#3B4252",
    "menu": "#434C5E",
    "text": "#ECEFF4",
    "muted": "#A3ABBA",
    "accent": "#88C0D0"
  }
}
"##;

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lumen-skins-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn builtin_palettes_are_complete() {
        let dir = temp("builtin");
        for theme in ["dark", "light", "oled", "inconnu"] {
            let p = palette(theme, "", &dir);
            for token in TOKENS {
                assert!(is_color(p[token].as_str().unwrap()), "{theme}: {token}");
            }
            assert_eq!(p["accent"], DEFAULT_ACCENT);
        }
        assert_eq!(palette("inconnu", "", &dir), palette("dark", "", &dir));
        assert_eq!(palette("light", "", &dir)["text"], "#1C1E22");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn applies_accent() {
        let dir = temp("accent");
        let blue = palette("dark", "#2D7FF9", &dir);
        assert_eq!(blue["accent"], "#2D7FF9");
        assert_eq!(blue["onAccent"], "#FFFFFF"); // accent foncé : texte blanc
        assert_eq!(palette("dark", "#FFD54A", &dir)["onAccent"], "#1A1206"); // accent clair : texte sombre
        assert_eq!(palette("dark", "pas une couleur", &dir)["accent"], DEFAULT_ACCENT);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn loads_skins() {
        let dir = temp("skins");
        std::fs::write(dir.join("nord.json"), EXAMPLE_SKIN).unwrap();
        std::fs::write(dir.join("casse.json"), "{ pas du json").unwrap();
        std::fs::write(
            dir.join("clair.json"),
            r##"{ "base": "light", "colors": { "accent": "#00AA55", "inconnu": "#FFFFFF", "text": "rouge" } }"##,
        )
        .unwrap();

        let ids: Vec<String> = themes(&dir).as_array().unwrap().iter().map(|t| t["id"].as_str().unwrap().to_string()).collect();
        assert_eq!(ids, ["dark", "light", "oled", "skin:clair", "skin:nord"]); // le skin illisible est ignoré

        let nord = palette("skin:nord", "", &dir);
        assert_eq!(nord["background"], "#2E3440");
        assert_eq!(nord["accent"], "#88C0D0");
        assert_eq!(nord["border"], palette("dark", "", &dir)["border"]); // non redéfini : celui de la base

        let clair = palette("skin:clair", "", &dir);
        assert_eq!(clair["accent"], "#00AA55");
        assert_eq!(clair["text"], "#1C1E22"); // « rouge » n'est pas une couleur : ignoré
        // L'accent choisi dans les préférences l'emporte sur celui du skin
        assert_eq!(palette("skin:nord", "#FF0000", &dir)["accent"], "#FF0000");
        // Skin supprimé : thème sombre
        assert_eq!(palette("skin:disparu", "", &dir), palette("dark", "", &dir));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
