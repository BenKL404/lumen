//! Raccourcis clavier personnalisables (section `[raccourcis]` de settings.toml).
//!
//! Chaque action a une ou plusieurs touches (notation Qt : "Space", "Ctrl+O", "F6"…).
//! Une touche choisie par l'utilisateur l'emporte sur la même touche attribuée par
//! défaut à une autre action ; deux choix de l'utilisateur en conflit sont signalés.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// Actions (identifiant, nom affiché) et touches par défaut, dans l'ordre de priorité
/// en cas de conflit et d'affichage dans la fenêtre des raccourcis.
/// Échap n'est pas personnalisable : il ferme menus, playlist et plein écran.
pub const DEFAULTS: &[(&str, &str, &[&str])] = &[
    ("lecture_pause", "Lecture / pause", &["Space"]),
    ("reculer", "Reculer (saut court)", &["Left"]),
    ("avancer", "Avancer (saut court)", &["Right"]),
    ("reculer_30s", "Reculer (saut long)", &["Ctrl+Left"]),
    ("avancer_30s", "Avancer (saut long)", &["Ctrl+Right"]),
    ("volume_plus", "Augmenter le volume", &["Up"]),
    ("volume_moins", "Baisser le volume", &["Down"]),
    ("muet", "Muet", &["M"]),
    ("son", "Égaliseur et son", &["E"]),
    ("normaliser", "Normaliser le volume", &["N"]),
    ("decalage_audio_moins", "Son −0,1 s", &["Shift+Z"]),
    ("decalage_audio_plus", "Son +0,1 s", &["Shift+X"]),
    ("plein_ecran", "Plein écran", &["F", "Return"]),
    ("ouvrir", "Ouvrir un fichier", &["O"]),
    ("ouvrir_dossier", "Ouvrir un dossier", &["Ctrl+O"]),
    ("ouvrir_url", "Ouvrir une vidéo en ligne", &["Ctrl+U"]),
    ("coller_lien", "Coller un lien vidéo", &["Ctrl+V"]),
    ("quitter", "Quitter", &["Ctrl+Q"]),
    ("capture", "Capture d'écran", &["S"]),
    ("image_suivante", "Image suivante", &["."]),
    ("image_precedente", "Image précédente", &[","]),
    ("vitesse_plus", "Accélérer", &["]"]),
    ("vitesse_moins", "Ralentir", &["["]),
    ("vitesse_normale", "Vitesse normale", &["Backspace"]),
    ("sous_titres_suivants", "Sous-titres suivants", &["J"]),
    ("chercher_sous_titres", "Rechercher des sous-titres en ligne", &["Ctrl+J"]),
    ("piste_audio_suivante", "Piste audio suivante", &["A"]),
    ("decalage_sous_titres_moins", "Sous-titres −0,1 s", &["Z"]),
    ("decalage_sous_titres_plus", "Sous-titres +0,1 s", &["X"]),
    ("fichier_suivant", "Fichier suivant", &["PgDown"]),
    ("fichier_precedent", "Fichier précédent", &["PgUp"]),
    ("chapitre_suivant", "Chapitre suivant", &["Ctrl+PgDown"]),
    ("chapitre_precedent", "Chapitre précédent", &["Ctrl+PgUp"]),
    ("playlist", "Afficher la playlist", &["F6"]),
    ("repetition", "Répétition", &["R"]),
    ("aleatoire", "Lecture aléatoire", &["H"]),
    ("boucle_ab", "Boucle A-B", &["L"]),
    ("signet", "Ajouter un signet", &["B"]),
    ("reglages_image", "Réglages d'image", &["I"]),
    ("retirer_de_la_playlist", "Retirer de la playlist", &["Delete"]),
    ("raccourcis", "Raccourcis clavier", &["F1"]),
    ("preferences", "Préférences", &["F5"]),
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

/// Nom affiché d'une action
pub fn label(action: &str) -> &'static str {
    DEFAULTS.iter().find(|(id, _, _)| *id == action).map_or("", |(_, label, _)| label)
}

pub fn defaults() -> BTreeMap<String, Keys> {
    DEFAULTS.iter().map(|(action, _, keys)| (action.to_string(), Keys::from_list(keys))).collect()
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
        .filter(|action| !DEFAULTS.iter().any(|(known, _, _)| known == action))
        .map(|action| format!("Raccourci inconnu ignoré : {action}"))
        .collect();

    let wanted: Vec<(&str, Vec<String>, bool)> = DEFAULTS
        .iter()
        .map(|(action, _, default)| {
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
