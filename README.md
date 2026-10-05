<p align="center"><img src="assets/lumen.svg" width="128" alt="Logo de Lumen"></p>

# Lumen — lecteur vidéo pour Linux

Lecteur vidéo moderne et performant pour Linux, inspiré de PotPlayer.

**Stack :** Rust + CXX-Qt + QML (Qt 6) + libmpv

> Version 1.0 : toute la feuille de route est réalisée (voir [FONCTIONNALITES.md](FONCTIONNALITES.md)).

---

## Télécharger

Une **AppImage** (Linux x86_64) est jointe à chaque [version publiée](https://github.com/BenKL404/lumen-player/releases) :

```bash
chmod +x Lumen-1.0.0-x86_64.AppImage
./Lumen-1.0.0-x86_64.AppImage
```

Les fichiers Flatpak et AUR sont prêts dans [packaging/](packaging/README.md) mais pas encore
publiés sur Flathub ni sur l'AUR. Pour compiler depuis les sources, voir ci-dessous.

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

**Ubuntu / Pop!_OS 24.04 mis à jour depuis 22.04 :** si `apt` refuse `libmpv-dev` à cause de `libvdpau1`,
ajoute `--allow-downgrades libvdpau1=1.5-2build1 libjack-jackd2-dev` à la commande.

### Fedora

```bash
sudo dnf install qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qtsvg mpv-libs-devel \
  ffmpeg-free-devel pkgconf-pkg-config
```

### Arch Linux

```bash
sudo pacman -S qt6-base qt6-declarative qt6-svg mpv ffmpeg pkgconf
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
cargo test           # tests de la logique Rust
```

La première compilation est longue (CXX-Qt génère et compile beaucoup de code C++).
Les suivantes sont incrémentales et rapides.

---

## Installation

```bash
scripts/install.sh            # installe Lumen dans ~/.local (sans sudo)
scripts/install.sh --default  # … et en fait le lecteur vidéo par défaut
scripts/uninstall.sh          # désinstalle (paramètres et historique conservés)
```

Lumen apparaît alors dans le lanceur d'applications, avec son icône, et dans
clic droit › **Ouvrir avec** du gestionnaire de fichiers. `lumen fichier.mkv` ou
`lumen dossier/` fonctionnent aussi dans un terminal.

Sous COSMIC, la recherche de fenêtres peut afficher « unknown » et une icône générique
juste après la première installation : son moteur (`pop-launcher`) a chargé la liste des
applications avant. `pkill -x pop-launcher` (relancé automatiquement) ou une reconnexion règle ça.

---

## Vidéos en ligne (YouTube…)

**Ctrl+U** (ou menu › Ouvrir une vidéo en ligne…), **Ctrl+V** avec un lien copié, ou un lien
glissé depuis le navigateur. Lumen passe par [yt-dlp](https://github.com/yt-dlp/yt-dlp)
(qualité limitée à 1080p pour une lecture fluide).

La version de yt-dlp des dépôts Ubuntu est trop ancienne pour YouTube : installer la version
officielle dans `~/.local/bin` (Lumen l'utilise en priorité) et la mettre à jour de temps en temps :

```bash
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o ~/.local/bin/yt-dlp
chmod +x ~/.local/bin/yt-dlp
yt-dlp -U   # mise à jour
```

YouTube demande aussi un moteur JavaScript : Lumen trouve `deno` ou `node` (y compris via nvm).

---

## Raccourcis personnalisables

**F1** (ou menu Lumen › Raccourcis clavier…) affiche toutes les actions et leurs touches.
Son bouton **Modifier…** ouvre `~/.config/lumen/settings.toml` (section `[raccourcis]`) dans
l'éditeur de texte, et **Recharger** applique les changements sans relancer Lumen.

```toml
[raccourcis]
plein_ecran = ["F", "Return"]   # plusieurs touches possibles
capture = "F9"                  # notation Qt : Space, Ctrl+O, F6, PgDown…
muet = ""                       # chaîne vide : raccourci désactivé
```

Une touche choisie l'emporte sur la même touche attribuée par défaut à une autre action.
Deux choix en conflit, ou une action inconnue, sont signalés à l'écran au démarrage.
Échap n'est pas personnalisable (il ferme menus, playlist et plein écran).

---

## Thèmes et skins

Préférences (F5) › **Apparence** : thème sombre, clair ou OLED noir, couleur d'accent, taille de
l'interface. Un **skin** est un fichier JSON dans `~/.config/lumen/skins/` (un exemple, `nord.json`,
y est créé par le bouton « Ouvrir le dossier ») ; il part d'un thème et redéfinit des couleurs :

```json
{
  "name": "Nord",
  "base": "dark",
  "colors": { "background": "#2E3440", "chrome": "#3B4252", "text": "#ECEFF4", "accent": "#88C0D0" }
}
```

Couleurs redéfinissables : `background`, `chrome`, `menu`, `surface`, `surfaceHover`, `border`,
`text`, `muted`, `track`, `subtle`, `raised`, `strong`, `field`, `shade`, `divider`, `accent`
(format `#RRGGBB` ou `#AARRGGBB`). Un skin se partage en copiant simplement son fichier.

---

## Extensions

Lumen charge les scripts mpv (Lua `.lua` ou JavaScript `.js`) placés dans
`~/.config/lumen/scripts/`. Préférences (F5) › **Extensions** les liste avec un interrupteur
chacun ; une extension d'exemple, **Passer les génériques**, y est fournie (désactivée).

En-tête reconnu, et message affiché dans le style de Lumen :

```lua
-- Nom : Mon extension
-- Description : ce qu'elle fait, affiché dans les préférences

mp.register_event("file-loaded", function()
    mp.commandv("script-message", "lumen-osd", "Bonne séance !")
end)
```

API des scripts : https://mpv.io/manual/stable/#lua-scripting. Les raccourcis clavier sont gérés
par Lumen (F1) : un script ne reçoit pas les touches, il réagit aux événements et aux propriétés.

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
├── Cargo.toml, build.rs   Dépendances ; build.rs compile Rust + C++ + QML ensemble (cxx-qt-build)
├── src/
│   ├── main.rs            Point d'entrée : options d'environnement, fenêtre unique, chargement de l'interface
│   ├── single_instance.rs Transmet le fichier à un Lumen déjà ouvert (MPRIS)
│   └── bridge/            Logique en Rust exposée à QML : historique (SQLite), playlist, paramètres,
│                          raccourcis, son, shaders, thèmes, extensions, sous-titres en ligne, MPRIS…
├── cpp/                   Composant vidéo libmpv (mpvitem), miniatures, durée des fichiers
├── qml/                   Interface : Main.qml (fenêtre, thème, raccourcis, OSD) et ses panneaux
├── assets/                Logo, icônes, fichier .desktop, fiche AppStream, shaders
├── scripts/               Installation, désinstallation, AppImage, sources Flatpak
└── packaging/             Flatpak, AUR (voir packaging/README.md)
```

---

## 4. Architecture

```
┌──────────────────────────────────────────────┐
│  QML (interface)                             │
│  Main · barres · playlist · panneaux         │
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

Propriétés observées : `position`, `duration`, `paused`, `volume` (0–130), `speed`, `muted`,
`mediaTitle`, `hasMedia`, `tracks`, `subDelay`, `chapters`, `chapter`, `abLoopA`/`abLoopB`,
`eofReached`, `info`.

Méthodes : `loadFile(url)`, `togglePause()`, `stop()`, `seekAbsolute(s)`, `seekRelative(s)`,
`frameStep(forward)`, `screenshot()`, `getProperty(nom)` pour lire n'importe quelle propriété
mpv, et `command([...])` pour envoyer **n'importe quelle commande mpv** (ex. `["cycle", "sub"]`).

Signaux : `fileLoaded()`, `endOfFile()`, `loadFailed(raison)`, `scriptMessage(…)`, plus un signal `…Changed` par propriété.

---

## 5. Fonctionnalités

- **Lecture** : tous les formats de FFmpeg, décodage matériel, reprise là où on s'est arrêté
  (pistes, sous-titres, décalage), proposition de reprendre la dernière vidéo au démarrage
- **Playlist** : épisodes et parties du même nom ajoutés automatiquement, dossiers, aléatoire,
  répétition, glisser-déposer, recherche, durées
- **Pistes** : audio et sous-titres, sous-titres externes, recherche en ligne (OpenSubtitles),
  décalages audio et sous-titres
- **Navigation** : miniatures au survol, chapitres, signets, boucle A-B
- **Image et son** : luminosité, contraste…, netteté (AMD CAS), agrandissement (FSR),
  égaliseur 10 bandes, normalisation du volume
- **En ligne** : YouTube et la plupart des sites (yt-dlp)
- **Bureau** : MPRIS (touches multimédia, applet), pas de mise en veille pendant un film,
  « Ouvrir avec », fenêtre unique, sélecteur de fichiers natif
- **Interface** : style PotPlayer, playlist accolée, mode mini, fenêtre adaptée à la vidéo,
  panneau d'informations, thèmes et skins, préférences (F5), raccourcis personnalisables (F1),
  extensions (scripts mpv)

### Raccourcis clavier

Liste complète et modifiable dans Lumen : **F1** (voir aussi la section « Raccourcis
personnalisables »).

| Touche            | Action                        |
|-------------------|-------------------------------|
| Espace            | Lecture / pause               |
| ← / →             | Saut court                    |
| Ctrl + ← / →      | Saut long                     |
| ↑ / ↓             | Volume ±5 %                   |
| F ou Entrée       | Plein écran                   |
| O / Ctrl+O        | Ouvrir un fichier / un dossier|
| Ctrl+U / Ctrl+V   | Vidéo en ligne / coller un lien |
| F5 / F1           | Préférences / raccourcis      |
| Tab               | Informations sur le fichier   |

---

## 6. Idées pour la suite

- Miniatures dans la playlist
- Publication sur Flathub et l'AUR (fichiers prêts dans `packaging/`)
- Captures d'écran dans la fiche AppStream
- Traductions de l'interface

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

Lumen est un logiciel libre, distribué sous **GPL-3.0-or-later** (voir [LICENSE](LICENSE)).

- libmpv : GPL v2+ (ou LGPL selon sa compilation) — compatible avec la GPL v3 de Lumen
- Qt 6 : LGPL v3
- CXX-Qt : MIT / Apache 2.0
- Shaders AMD FidelityFX CAS et FSR : MIT (voir `assets/shaders/README.md`)
- Icônes : tracés du jeu Lucide (ISC)

Lumen n'est pas affilié à PotPlayer : il s'en inspire sans reprendre son nom, ses icônes ni ses skins.
