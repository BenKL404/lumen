//! Extensions : scripts mpv (Lua ou JavaScript) placés dans ~/.config/lumen/scripts/,
//! chargés au démarrage sauf ceux désactivés dans les préférences.
//!
//! Un script peut afficher un message dans le style de Lumen :
//!   mp.commandv("script-message", "lumen-osd", "Texte")

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

pub fn scripts_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lumen")
        .join("scripts")
}

fn is_script(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|e| e == "lua" || e == "js")
}

/// Commentaires de l'en-tête d'un script (« -- » en Lua, « // » en JavaScript)
fn header_comments(source: &str) -> Vec<&str> {
    source
        .lines()
        .take(15)
        .map(str::trim)
        .take_while(|l| l.is_empty() || l.starts_with("--") || l.starts_with("//"))
        .filter_map(|l| l.strip_prefix("--").or_else(|| l.strip_prefix("//")))
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

/// Valeur d'une ligne « Clé : valeur » de l'en-tête
fn header_field(source: &str, key: &str) -> Option<String> {
    header_comments(source).iter().find_map(|l| {
        let (name, value) = l.split_once(':')?;
        name.trim().eq_ignore_ascii_case(key).then(|| value.trim().to_string())
    })
}

/// Description d'un script : la ligne « -- Description : … » de son en-tête,
/// sinon son premier commentaire
pub fn description(source: &str) -> String {
    header_field(source, "description")
        .or_else(|| header_comments(source).first().map(|l| l.to_string()))
        .unwrap_or_default()
}

/// Nom affiché : la ligne « -- Nom : … » de l'en-tête, sinon le nom du fichier (« mon-script » -> « Mon script »)
pub fn display_name(source: &str, stem: &str) -> String {
    header_field(source, "nom").unwrap_or_else(|| {
        let name = stem.replace(['-', '_'], " ");
        let mut chars = name.chars();
        chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
    })
}

/// Scripts trouvés (JSON [{ file, name, description, enabled, path }]), triés par nom
pub fn list(dir: &Path, disabled: &[String]) -> Value {
    let mut scripts: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| is_script(p))
        .collect();
    scripts.sort();
    Value::Array(
        scripts
            .iter()
            .map(|path| {
                let file = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                let source = std::fs::read_to_string(path).unwrap_or_default();
                let name = display_name(&source, &path.file_stem().unwrap_or_default().to_string_lossy());
                json!({
                    "file": file,
                    "name": name,
                    "description": description(&source),
                    "enabled": !disabled.contains(&file),
                    "path": path.to_string_lossy(),
                })
            })
            .collect(),
    )
}

/// Chemins des scripts à charger
pub fn enabled_paths(dir: &Path, disabled: &[String]) -> Vec<String> {
    list(dir, disabled)
        .as_array()
        .into_iter()
        .flatten()
        .filter(|s| s["enabled"].as_bool() == Some(true))
        .filter_map(|s| s["path"].as_str().map(String::from))
        .collect()
}

pub const EXAMPLE_FILE: &str = "passer-les-generiques.lua";

/// Extension d'exemple : passe les chapitres de générique (désactivée par défaut)
pub const EXAMPLE_SCRIPT: &str = r#"-- Nom : Passer les génériques
-- Description : passe automatiquement les génériques (chapitres « Opening », « Intro », « Générique », « Credits »…)
--
-- Extension d'exemple de Lumen : un script mpv en Lua. Les scripts de ce dossier sont
-- chargés au démarrage de Lumen (Préférences › Extensions pour les activer ou non).
-- Documentation de l'API : https://mpv.io/manual/stable/#lua-scripting

local generiques = { "opening", "intro", "générique", "generique", "ending", "credits", "op", "ed" }

local function est_un_generique(titre)
    titre = titre:lower()
    for _, nom in ipairs(generiques) do
        if titre == nom or titre:find("^" .. nom .. "[%s%p%d]") then
            return true
        end
    end
    return false
end

mp.observe_property("chapter", "number", function(_, chapitre)
    if not chapitre or chapitre < 0 then return end
    local chapitres = mp.get_property_native("chapter-list") or {}
    local actuel = chapitres[chapitre + 1]
    if not actuel or not actuel.title or not est_un_generique(actuel.title) then return end

    -- Message dans le style de Lumen
    mp.commandv("script-message", "lumen-osd", "Générique passé")
    if chapitres[chapitre + 2] then
        mp.set_property_number("chapter", chapitre + 1)
    else
        -- Générique de fin : aller à la fin, Lumen enchaîne sur la suite de la playlist
        mp.commandv("seek", "100", "absolute-percent")
    end
end)
"#;

/// Crée le dossier et l'extension d'exemple s'il n'y a encore aucun script.
/// Renvoie vrai si l'exemple vient d'être créé (à désactiver par défaut).
pub fn ensure_example(dir: &Path) -> bool {
    let has_scripts = std::fs::read_dir(dir).is_ok_and(|mut e| e.any(|e| e.is_ok_and(|e| is_script(&e.path()))));
    if has_scripts || std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    std::fs::write(dir.join(EXAMPLE_FILE), EXAMPLE_SCRIPT).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_descriptions() {
        assert_eq!(description("-- Description : saute l'intro\n-- autre\nlocal x = 1"), "saute l'intro");
        assert_eq!(description("// Affiche l'heure\nmp.x()"), "Affiche l'heure");
        assert_eq!(description("\n-- \n-- Première ligne utile\n"), "Première ligne utile");
        assert_eq!(description("local x = 1\n-- trop tard"), "");
        assert!(description(EXAMPLE_SCRIPT).starts_with("passe automatiquement les génériques"));
        assert_eq!(display_name("-- Nom : Mon outil\n", "fichier"), "Mon outil");
        assert_eq!(display_name("", "sous_titres-auto"), "Sous titres auto");
    }

    #[test]
    fn lists_and_filters_scripts() {
        let dir = std::env::temp_dir().join(format!("lumen-scripts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        assert!(ensure_example(&dir)); // dossier vide : exemple créé
        assert!(!ensure_example(&dir)); // déjà un script : rien
        std::fs::write(dir.join("compteur.js"), "// Compte les fichiers ouverts\n").unwrap();
        std::fs::write(dir.join("notes.txt"), "pas un script").unwrap();

        let disabled = vec![EXAMPLE_FILE.to_string()];
        let scripts = list(&dir, &disabled);
        let files: Vec<&str> = scripts.as_array().unwrap().iter().map(|s| s["file"].as_str().unwrap()).collect();
        assert_eq!(files, ["compteur.js", EXAMPLE_FILE]);
        assert_eq!(scripts[0]["name"], "Compteur");
        assert_eq!(scripts[1]["name"], "Passer les génériques");
        assert_eq!(scripts[1]["enabled"], false);

        let paths = enabled_paths(&dir, &disabled);
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("compteur.js"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
