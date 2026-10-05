//! Raccourcis clavier personnalisables (section `[raccourcis]` de settings.toml).
//!
//! Chaque action a une ou plusieurs touches (notation Qt : "Space", "Ctrl+O", "F6"…).
//! Une touche choisie par l'utilisateur l'emporte sur la même touche attribuée par
//! défaut à une autre action ; deux choix de l'utilisateur en conflit sont signalés.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// Actions et touches par défaut, dans l'ordre de priorité en cas de conflit.
/// Échap n'est pas personnalisable : il ferme menus, playlist et plein écran.
pub const DEFAULTS: &[(&str, &[&str])] = &[
    ("lecture_pause", &["Space"]),
    ("reculer", &["Left"]),
    ("avancer", &["Right"]),
    ("reculer_30s", &["Ctrl+Left"]),
    ("avancer_30s", &["Ctrl+Right"]),
    ("volume_plus", &["Up"]),
    ("volume_moins", &["Down"]),
    ("muet", &["M"]),
    ("plein_ecran", &["F", "Return"]),
    ("ouvrir", &["O"]),
    ("ouvrir_dossier", &["Ctrl+O"]),
    ("quitter", &["Ctrl+Q"]),
    ("capture", &["S"]),
    ("image_suivante", &["."]),
    ("image_precedente", &[","]),
    ("vitesse_plus", &["]"]),
    ("vitesse_moins", &["["]),
    ("vitesse_normale", &["Backspace"]),
    ("sous_titres_suivants", &["J"]),
    ("piste_audio_suivante", &["A"]),
    ("decalage_sous_titres_moins", &["Z"]),
    ("decalage_sous_titres_plus", &["X"]),
    ("fichier_suivant", &["PgDown"]),
    ("fichier_precedent", &["PgUp"]),
    ("chapitre_suivant", &["Ctrl+PgDown"]),
    ("chapitre_precedent", &["Ctrl+PgUp"]),
    ("playlist", &["F6"]),
    ("repetition", &["R"]),
    ("aleatoire", &["H"]),
    ("boucle_ab", &["L"]),
    ("signet", &["B"]),
    ("reglages_image", &["I"]),
    ("retirer_de_la_playlist", &["Delete"]),
];

/// Touches d'une action : une seule ("F6") ou une liste (["F", "Return"]) dans le fichier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Keys {
    One(String),
    Many(Vec<String>),
}

impl Keys {
    pub fn from_list(list: &[&str]) -> Self {
        match list {
            [one] => Keys::One(one.to_string()),
            many => Keys::Many(many.iter().map(|k| k.to_string()).collect()),
        }
    }

    /// Touches non vides ; une chaîne vide désactive le raccourci
    pub fn list(&self) -> Vec<String> {
        let all = match self {
            Keys::One(key) => vec![key.clone()],
            Keys::Many(keys) => keys.clone(),
        };
        all.into_iter().map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect()
    }
}

pub fn defaults() -> BTreeMap<String, Keys> {
    DEFAULTS.iter().map(|(action, keys)| (action.to_string(), Keys::from_list(keys))).collect()
}

/// Raccourcis du fichier complétés par les valeurs par défaut. Les actions inconnues
/// (faute de frappe…) sont conservées pour que l'avertissement reste visible.
pub fn merged(user: &BTreeMap<String, Keys>) -> BTreeMap<String, Keys> {
    let mut map = defaults();
    map.extend(user.iter().map(|(action, keys)| (action.clone(), keys.clone())));
    map
}

/// Raccourcis effectivement actifs, et avertissements à montrer à l'utilisateur
#[derive(Debug, Default, PartialEq)]
pub struct Resolved {
    pub keys: BTreeMap<String, Vec<String>>,
    pub warnings: Vec<String>,
}

/// Comparaison insensible à la casse et aux espaces : "ctrl + o" = "Ctrl+O"
fn normalized(key: &str) -> String {
    key.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase()
}

pub fn resolve(user: &BTreeMap<String, Keys>) -> Resolved {
    let mut warnings: Vec<String> = user
        .keys()
        .filter(|action| !DEFAULTS.iter().any(|(known, _)| known == action))
        .map(|action| format!("Raccourci inconnu ignoré : {action}"))
        .collect();

    let wanted: Vec<(&str, Vec<String>, bool)> = DEFAULTS
        .iter()
        .map(|(action, default)| {
            let default: Vec<String> = default.iter().map(|k| k.to_string()).collect();
            match user.get(*action) {
                Some(keys) => {
                    let list = keys.list();
                    let customized = list != default;
                    (*action, list, customized)
                }
                None => (*action, default, false),
            }
        })
        .collect();

    // Les choix de l'utilisateur d'abord, puis les valeurs par défaut sur les touches restantes
    let mut owner: HashMap<String, &str> = HashMap::new();
    let mut keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for pass_customized in [true, false] {
        for (action, list, customized) in &wanted {
            if *customized != pass_customized {
                continue;
            }
            let mut kept = Vec::new();
            for key in list {
                match owner.get(&normalized(key)) {
                    Some(other) if *customized => {
                        warnings.push(format!("Touche « {key} » déjà utilisée par {other} : ignorée pour {action}"))
                    }
                    Some(_) => {} // touche par défaut reprise par un choix de l'utilisateur
                    None => {
                        owner.insert(normalized(key), action);
                        kept.push(key.clone());
                    }
                }
            }
            keys.insert(action.to_string(), kept);
        }
    }
    Resolved { keys, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(entries: &[(&str, Keys)]) -> BTreeMap<String, Keys> {
        entries.iter().map(|(a, k)| (a.to_string(), k.clone())).collect()
    }

    #[test]
    fn defaults_have_no_conflicts() {
        let resolved = resolve(&BTreeMap::new());
        assert!(resolved.warnings.is_empty(), "{:?}", resolved.warnings);
        assert_eq!(resolved.keys["plein_ecran"], ["F", "Return"]);
        assert_eq!(resolved.keys.len(), DEFAULTS.len());
    }

    #[test]
    fn user_choice_wins_over_defaults() {
        // M passe au plein écran : « muet » perd M, sans avertissement
        let resolved = resolve(&user(&[("plein_ecran", Keys::One("m".into()))]));
        assert_eq!(resolved.keys["plein_ecran"], ["m"]);
        assert!(resolved.keys["muet"].is_empty());
        assert!(resolved.warnings.is_empty());
    }

    #[test]
    fn reports_conflicts_and_unknown_actions() {
        let resolved = resolve(&user(&[
            ("capture", Keys::One("F9".into())),
            ("signet", Keys::One("f9".into())),
            ("action_inventee", Keys::One("K".into())),
            ("muet", Keys::One("".into())), // désactivé
        ]));
        assert_eq!(resolved.keys["capture"], ["F9"]);
        assert!(resolved.keys["signet"].is_empty());
        assert!(resolved.keys["muet"].is_empty());
        assert_eq!(resolved.warnings.len(), 2, "{:?}", resolved.warnings);
        assert!(resolved.warnings.iter().any(|w| w.contains("action_inventee")));
        assert!(resolved.warnings.iter().any(|w| w.contains("capture") && w.contains("signet")));
    }

    #[test]
    fn merges_and_reads_lists() {
        let map = merged(&user(&[("playlist", Keys::Many(vec!["F6".into(), " P ".into()])), ("inconnu", Keys::One("K".into()))]));
        assert_eq!(map["playlist"].list(), ["F6", "P"]);
        assert!(map.contains_key("inconnu"));
        assert_eq!(map["ouvrir"], Keys::One("O".into()));
        assert_eq!(Keys::Many(vec!["".into()]).list(), Vec::<String>::new());
    }
}
