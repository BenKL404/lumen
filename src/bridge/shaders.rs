//! Filtres d'image sur la carte graphique (shaders GLSL de mpv) : netteté AMD CAS et
//! agrandissement AMD FSR (voir assets/shaders/README.md). Les shaders sont intégrés au
//! programme ; une copie réglée est écrite dans ~/.cache/lumen/shaders pour mpv.

use std::path::{Path, PathBuf};

const CAS: &str = include_str!("../../assets/shaders/CAS.glsl");
const CAS_SCALED: &str = include_str!("../../assets/shaders/CAS-scaled.glsl");
const FSR: &str = include_str!("../../assets/shaders/FSR.glsl");

/// Agrandissement des vidéos plus petites que l'écran
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Upscaler {
    /// Algorithmes par défaut de mpv : rapides
    Standard = 0,
    /// Meilleurs algorithmes de mpv (ewa_lanczossharp…), plus gourmands
    HighQuality = 1,
    /// AMD FidelityFX Super Resolution
    Fsr = 2,
}

impl Upscaler {
    pub fn from_index(index: i32) -> Self {
        match index {
            1 => Upscaler::HighQuality,
            2 => Upscaler::Fsr,
            _ => Upscaler::Standard,
        }
    }
}

/// Options de mise à l'échelle de mpv pour un mode d'agrandissement (nom, valeur)
pub fn scaling_options(upscaler: Upscaler) -> Vec<(&'static str, &'static str)> {
    match upscaler {
        Upscaler::HighQuality => {
            vec![("scale", "ewa_lanczossharp"), ("cscale", "ewa_lanczossharp"), ("dscale", "mitchell"), ("deband", "yes")]
        }
        // FSR fait l'agrandissement de la luminance ; le reste reste aux réglages par défaut
        Upscaler::Standard | Upscaler::Fsr => {
            vec![("scale", "lanczos"), ("cscale", "lanczos"), ("dscale", "hermite"), ("deband", "no")]
        }
    }
}

/// Remplace la valeur d'un `#define NOM …` du shader
fn with_define(source: &str, name: &str, value: &str) -> String {
    let prefix = format!("#define {name} ");
    source
        .lines()
        .map(|line| {
            if line.starts_with(&prefix) {
                // Garde le commentaire explicatif en fin de ligne
                let comment = line.find("//").map(|i| &line[i..]).unwrap_or("");
                format!("{prefix}{value} {comment}").trim_end().to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Shaders à charger (nom de fichier, contenu) pour une netteté de 0 à 100 et un agrandissement
pub fn shader_sources(sharpness: i32, upscaler: Upscaler) -> Vec<(String, String)> {
    let mut shaders = Vec::new();
    if upscaler == Upscaler::Fsr {
        // FSR comprend sa propre accentuation (RCAS) une fois l'image agrandie
        shaders.push(("fsr.glsl".to_string(), FSR.to_string()));
    }
    let sharpness = sharpness.clamp(0, 100);
    if sharpness > 0 {
        let value = format!("{:.2}", sharpness as f64 / 100.0);
        // Image à sa taille : CAS ; image agrandie : CAS-scaled, sauf si FSR s'en charge déjà
        shaders.push((format!("cas-{sharpness}.glsl"), with_define(CAS, "SHARPENING", &value)));
        if upscaler != Upscaler::Fsr {
            shaders.push((format!("cas-scaled-{sharpness}.glsl"), with_define(CAS_SCALED, "SHARPENING", &value)));
        }
    }
    shaders
}

/// Écrit les shaders dans `dir` et renvoie leurs chemins, dans l'ordre d'application
pub fn write_shaders(dir: &Path, sharpness: i32, upscaler: Upscaler) -> std::io::Result<Vec<PathBuf>> {
    let sources = shader_sources(sharpness, upscaler);
    if !sources.is_empty() {
        std::fs::create_dir_all(dir)?;
    }
    sources
        .into_iter()
        .map(|(name, content)| {
            let path = dir.join(name);
            // Inutile de réécrire un shader identique (curseur déplacé puis ramené)
            if std::fs::read_to_string(&path).ok().as_deref() != Some(content.as_str()) {
                std::fs::write(&path, content)?;
            }
            Ok(path)
        })
        .collect()
}

pub fn cache_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lumen")
        .join("shaders")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_shaders() {
        assert!(shader_sources(0, Upscaler::Standard).is_empty());
        assert!(shader_sources(0, Upscaler::HighQuality).is_empty());

        let names = |s: i32, u: Upscaler| shader_sources(s, u).into_iter().map(|(n, _)| n).collect::<Vec<_>>();
        assert_eq!(names(0, Upscaler::Fsr), ["fsr.glsl"]);
        assert_eq!(names(40, Upscaler::Standard), ["cas-40.glsl", "cas-scaled-40.glsl"]);
        assert_eq!(names(40, Upscaler::Fsr), ["fsr.glsl", "cas-40.glsl"]);
        assert_eq!(names(250, Upscaler::Standard)[0], "cas-100.glsl");
    }

    #[test]
    fn sets_sharpening() {
        let (_, cas) = &shader_sources(35, Upscaler::Standard)[0];
        assert!(cas.contains("#define SHARPENING 0.35 // Adjusts"), "réglage absent");
        assert!(!cas.contains("#define SHARPENING 0.0 "));
        // Le reste du shader est intact (licence, passes)
        assert!(cas.starts_with("// LICENSE") && cas.contains("// Copyright (c) 2017-2019 Advanced Micro Devices"));
        assert_eq!(cas.matches("//!HOOK").count(), CAS.matches("//!HOOK").count());
    }

    #[test]
    fn writes_shaders() {
        let dir = std::env::temp_dir().join(format!("lumen-shaders-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let paths = write_shaders(&dir, 50, Upscaler::Fsr).unwrap();
        assert_eq!(paths, [dir.join("fsr.glsl"), dir.join("cas-50.glsl")]);
        assert!(std::fs::read_to_string(&paths[1]).unwrap().contains("#define SHARPENING 0.50"));
        assert!(write_shaders(&dir, 0, Upscaler::Standard).unwrap().is_empty());
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(Upscaler::from_index(2), Upscaler::Fsr);
        assert_eq!(Upscaler::from_index(9), Upscaler::Standard);
        assert_eq!(scaling_options(Upscaler::HighQuality)[0], ("scale", "ewa_lanczossharp"));
    }
}
