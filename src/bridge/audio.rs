//! Traitement du son : égaliseur 10 bandes et normalisation du volume, sous forme de
//! chaîne de filtres FFmpeg pour la propriété `af` de mpv.

/// Fréquences centrales des 10 bandes de l'égaliseur, en Hz
pub const BANDS: [u32; 10] = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
/// Gain maximal d'une bande, en dB (dans les deux sens)
pub const MAX_GAIN: i32 = 12;

/// Préréglages : nom affiché et gains des 10 bandes
pub const PRESETS: &[(&str, [i32; 10])] = &[
    ("Plat", [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ("Voix", [-3, -2, -1, 0, 2, 4, 4, 3, 1, 0]),
    ("Basses", [6, 5, 4, 2, 0, 0, 0, 0, 0, 0]),
    ("Aigus", [0, 0, 0, 0, 0, 1, 2, 4, 5, 6]),
    ("Cinéma", [4, 3, 1, 0, 0, 2, 3, 2, 2, 3]),
];

/// Gains ramenés à 10 bandes dans [-12, 12] (valeurs manquantes : 0)
pub fn sanitized(gains: &[i32]) -> Vec<i32> {
    (0..BANDS.len()).map(|i| gains.get(i).copied().unwrap_or(0).clamp(-MAX_GAIN, MAX_GAIN)).collect()
}

/// Valeur de la propriété `af` de mpv ; vide si aucun traitement n'est actif.
pub fn filter_chain(gains: &[i32], normalize: bool) -> String {
    let mut filters: Vec<String> = sanitized(gains)
        .iter()
        .zip(BANDS)
        .filter(|(gain, _)| **gain != 0)
        // Filtre en cloche d'une octave de large centré sur la bande
        .map(|(gain, freq)| format!("equalizer=f={freq}:t=o:w=1:g={gain}"))
        .collect();
    if normalize {
        // Normalisation dynamique : remonte les passages faibles (dialogues), retient les forts.
        // f : fenêtre de 250 ms environ ; g : lissage ; p : niveau visé ; m : gain maximal
        filters.push("dynaudnorm=f=250:g=15:p=0.7:m=8".to_string());
    }
    if filters.is_empty() {
        String::new()
    } else {
        format!("lavfi=[{}]", filters.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_filter_chains() {
        assert_eq!(filter_chain(&[0; 10], false), "");
        assert_eq!(filter_chain(&[], false), "");
        assert_eq!(
            filter_chain(&[6, 0, 0, 0, 0, 0, 0, 0, 0, -3], false),
            "lavfi=[equalizer=f=31:t=o:w=1:g=6,equalizer=f=16000:t=o:w=1:g=-3]"
        );
        assert_eq!(filter_chain(&[0; 10], true), "lavfi=[dynaudnorm=f=250:g=15:p=0.7:m=8]");
        assert!(filter_chain(&[0, 0, 0, 0, 2, 0, 0, 0, 0, 0], true).ends_with("g=2,dynaudnorm=f=250:g=15:p=0.7:m=8]"));
    }

    #[test]
    fn sanitizes_gains() {
        assert_eq!(sanitized(&[40, -40]), [12, -12, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(sanitized(&[1; 14]).len(), 10);
        assert!(PRESETS.iter().all(|(_, gains)| sanitized(gains) == gains));
    }
}
