<p align="center"><img src="assets/lumen.svg" width="128" alt="Logo de Lumen"></p>

# Lumen — lecteur vidéo pour Linux

Lecteur vidéo moderne et performant pour Linux, inspiré de PotPlayer.

**Stack :** Rust + CXX-Qt + QML (Qt 6) + libmpv

> Statut : squelette de départ (v0.1). Le code n'a pas encore été compilé de bout en bout ;
> attends-toi à quelques ajustements mineurs lors du premier build.

---

## 1. Prérequis

### Ubuntu / Debian

```bash
sudo apt install qt6-base-dev qt6-declarative-dev qmake6 libmpv-dev pkg-config libgl-dev \
  qml6-module-qtquick qml6-module-qtquick-window qml6-module-qtquick-dialogs \
  qml6-module-qtqml qml6-module-qtqml-workerscript qml6-module-qtqml-models \
  qml6-module-qtquick-controls qml6-module-qtquick-templates qml6-module-qtquick-layouts \
  qml6-module-qt-labs-folderlistmodel qt6-xdgdesktopportal-platformtheme
```

`qt6-xdgdesktopportal-platformtheme` donne les dialogues natifs du bureau (sélecteur de fichiers
de COSMIC, GNOME, KDE…). Sans lui, Qt affiche son propre dialogue.

Les modules `qml6-module-*` ne sont vérifiés qu'au lancement : s'il en manque un, la compilation
réussit mais la fenêtre ne s'ouvre pas (`module "…" is not installed`). `QtQuick.Dialogs` a besoin
des modules Controls, Templates, Layouts et FolderListModel.

**Pop!_OS 24.04 mis à jour depuis 22.04 :** si `apt` refuse `libmpv-dev` à cause de `libvdpau1`,
ajoute `--allow-downgrades libvdpau1=1.5-2build1 libjack-jackd2-dev` à la commande.

### Fedora

```bash
sudo dnf install qt6-qtbase-devel qt6-qtdeclarative-devel mpv-libs-devel pkgconf-pkg-config
```

### Arch Linux

```bash
sudo pacman -S qt6-base qt6-declarative mpv pkgconf
```

### Rust

Rust **1.85 ou plus récent** est requis (certaines dépendances utilisent l'édition 2024).
Installe-le via rustup plutôt que via le gestionnaire de paquets :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

CXX-Qt doit trouver `qmake` de Qt 6. Si la commande `qmake` pointe vers Qt 5 :

```bash
export QMAKE=/usr/bin/qmake6
```

---

## 2. Compiler et lancer

```bash
cargo run            # mode debug
cargo run --release  # mode optimisé
cargo test           # tests des utilitaires Rust
```

La première compilation est longue (CXX-Qt génère et compile beaucoup de code C++).
Les suivantes sont incrémentales et rapides.

---

## Données et paramètres

| Fichier | Contenu |
|---|---|
| `~/.config/lumen/settings.toml` | Paramètres (volume, répétition, aléatoire, fenêtre, options) — modifiable à la main, Lumen fermé |
| `~/.local/share/lumen/history.db` | Historique de lecture (positions, pistes, sous-titres) |

---

## 3. Structure du projet

```
lumen/
├── Cargo.toml             Dépendances Rust
├── build.rs               Compile Rust + C++ + QML ensemble (cxx-qt-build)
├── src/
│   ├── main.rs            Point d'entrée : initialise Qt et charge l'interface
│   └── bridge/
│       ├── mod.rs
│       ├── utils.rs       Objet Rust exposé à QML (formatage du temps, noms de fichiers)
│       └── video.rs       Pont Rust → C++ pour initialiser le moteur vidéo
├── cpp/
│   ├── mpvitem.h          Composant vidéo QML basé sur libmpv
│   └── mpvitem.cpp
└── qml/
    ├── Main.qml           Fenêtre, design tokens, glisser-déposer, raccourcis, OSD
    ├── ControlBar.qml     Barre de contrôle translucide auto-masquée
    ├── SeekBar.qml        Curseur générique (position / volume) avec info-bulle
    └── IconButton.qml     Bouton rond réutilisable
```

---

## 4. Architecture

```
┌──────────────────────────────────────────────┐
│  QML (interface)                             │
│  Main · ControlBar · SeekBar · IconButton    │
└───────────────┬──────────────────┬───────────┘
                │                  │
     ┌──────────▼─────────┐  ┌─────▼────────────────┐
     │  Rust (CXX-Qt)     │  │  C++ MpvItem          │
     │  logique, services │  │  rendu libmpv → FBO   │
     └────────────────────┘  └─────┬────────────────┘
                                   │
                          ┌────────▼────────┐
                          │ libmpv / FFmpeg │
                          │ VA-API · NVDEC  │
                          └─────────────────┘
```

### Pourquoi une couche C++ ?

Afficher mpv dans une scène QML exige d'hériter de `QQuickFramebufferObject`
et de redéfinir son renderer, ce que CXX-Qt ne permet pas encore proprement depuis Rust.
Cette couche (`cpp/mpvitem.*`) reste volontairement minimale : un simple « tuyau »
d'affichage et de commandes. **Toute la logique applicative doit être écrite en Rust.**

### Le composant `MpvVideo` (exposé à QML)

| Propriété    | Type   | Accès          |
|--------------|--------|----------------|
| `position`   | double | lecture        |
| `duration`   | double | lecture        |
| `paused`     | bool   | lecture/écriture |
| `volume`     | double | lecture/écriture (0–130) |
| `speed`      | double | lecture/écriture (0.25–4) |
| `mediaTitle` | string | lecture        |
| `hasMedia`   | bool   | lecture        |

Méthodes : `loadFile(url)`, `togglePause()`, `seekAbsolute(s)`, `seekRelative(s)`,
`frameStep(forward)`, `screenshot()`, et `command([...])` pour envoyer
**n'importe quelle commande mpv** (ex. `["cycle", "sub"]`).

Signaux : `fileLoaded()`, `endOfFile()`, plus un signal `…Changed` par propriété.

---

## 5. Fonctionnalités de la v0.1

- Lecture de tous les formats supportés par FFmpeg
- Décodage matériel automatique (`hwdec=auto-safe`), sans copie sous X11 et Wayland
- Ouverture par dialogue ou par glisser-déposer
- Barre de contrôle qui se masque automatiquement pendant la lecture
- Double-clic = lecture / pause, triple-clic = plein écran, molette = volume
- Info-bulle temporelle au survol de la barre de progression
- Affichage OSD des actions (volume, vitesse, saut…)

### Raccourcis clavier

| Touche            | Action                        |
|-------------------|-------------------------------|
| Espace            | Lecture / pause               |
| ← / →             | Reculer / avancer de 5 s      |
| Ctrl + ← / →      | Reculer / avancer de 30 s     |
| ↑ / ↓             | Volume ±5 %                   |
| M                 | Muet                          |
| F ou Entrée       | Plein écran                   |
| Échap             | Quitter le plein écran        |
| O                 | Ouvrir un fichier             |
| S                 | Capture d'écran (Bureau)      |
| . / ,             | Image suivante / précédente   |
| ] / [             | Vitesse ±0,1                  |
| Retour arrière    | Vitesse normale               |
| J                 | Piste de sous-titres suivante |
| A                 | Piste audio suivante          |
| Z / X             | Décalage sous-titres ∓0,1 s   |
| PgUp / PgDn       | Fichier précédent / suivant   |
| F6                | Afficher / masquer la playlist |
| Ctrl+PgUp / PgDn  | Chapitre précédent / suivant  |
| L                 | Boucle A-B (A, B, désactiver) |
| B                 | Ajouter un signet             |
| I                 | Réglages d'image              |
| R / H             | Répétition / aléatoire        |
| Ctrl+O            | Ouvrir un dossier             |

---

## 6. Feuille de route

### v0.2 — Pistes et mémoire
- Menus de sélection des pistes audio et sous-titres (propriété mpv `track-list`)
- Chargement de sous-titres externes et réglage du décalage
- Reprise de lecture à la dernière position (Rust + `rusqlite`)
- Playlist et lecture du fichier suivant dans le dossier

### v0.3 — Confort PotPlayer
- Miniatures au survol de la barre (service Rust séparé, `ffmpeg-next`, cache disque)
- Boucle A-B, signets, chapitres
- Réglages d'image (luminosité, contraste, saturation, rotation, zoom)
- Menu contextuel complet

### v0.4 — Intégration Linux
- MPRIS via `zbus` (touches multimédia, contrôles du bureau)
- Inhibition de la mise en veille (portail XDG)
- Fichier `.desktop` et associations MIME
- Raccourcis personnalisables et fichier de configuration (`serde` + `toml`)

### v1.0 — Puissance
- Shaders GLSL (upscaling, Anime4K…), égaliseur audio
- Streaming réseau et YouTube via `yt-dlp`
- Recherche de sous-titres en ligne (OpenSubtitles)
- Système d'extensions (scripts Lua de mpv ou plugins maison)
- Thèmes et skins en JSON
- Distribution : Flatpak, AppImage, AUR

---

## 7. Conseils et pièges connus

- **Locale :** libmpv exige `LC_NUMERIC=C`. C'est géré dans le constructeur de `MpvItem` ;
  ne le modifie pas ailleurs.
- **Backend graphique :** le rendu utilise OpenGL (`lumen_init_video()` le force avant la création
  de la fenêtre). Ne change pas ce réglage sans adapter le renderer.
- **Seek fluide :** la barre de progression n'envoie la position qu'au relâchement,
  pour ne pas saturer mpv de commandes pendant le glissement.
- **Cœur de logique en Rust :** si une fonctionnalité ne touche pas au rendu, elle va dans
  `src/`, pas dans `cpp/`.
- **Référence utile :** le code source de Haruna (KDE) utilise la même approche Qt Quick + libmpv.

---

## 8. Licences

- libmpv : GPL v2+ (ou LGPL selon sa compilation)
- Qt 6 : LGPL v3
- CXX-Qt : MIT / Apache 2.0

Si tu distribues Lumen lié à une libmpv GPL, le projet doit être publié sous licence compatible GPL.
N'utilise ni le nom, ni les icônes, ni les skins de PotPlayer.
