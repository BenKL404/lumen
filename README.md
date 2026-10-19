<p align="center"><img src="assets/lumen.svg" width="128" alt="Logo de Lumen"></p>

# Lumen — lecteur vidéo pour Linux

Lecteur vidéo moderne et performant pour Linux, inspiré de PotPlayer.

**Stack :** Rust + CXX-Qt + QML (Qt 6) + libmpv

> Version 1.0 : toute la feuille de route est réalisée (voir [FONCTIONNALITES.md](FONCTIONNALITES.md)).

En bref : lecture de tous les formats avec décodage matériel, reprise là où on s'est arrêté,
playlist automatique des épisodes, vidéos et **directs YouTube**, **chaînes IPTV** (M3U, Xtream
Codes), recherche de sous-titres en ligne, égaliseur et filtres d'image, thèmes et skins,
**raccourcis modifiables dans l'interface**, intégration au bureau Linux (MPRIS, « Ouvrir avec »).

<p align="center"><img src="assets/screenshots/lecture.png" width="720" alt="Lumen en cours de lecture"></p>

---

## Télécharger

Une **AppImage** (Linux x86_64) est jointe à chaque [version publiée](https://github.com/BenKL404/lumen/releases) :

```bash
chmod +x Lumen-*-x86_64.AppImage
./Lumen-*-x86_64.AppImage
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

**Directs** : le lien d'une vidéo en direct, ou celui d'une chaîne suivi de `/live`
(`https://www.youtube.com/@chaine/live`, toujours le direct en cours). Le compteur affiche le temps
écoulé depuis le début de la diffusion ; la reprise de lecture ne s'applique pas aux directs.
Liens directs vers un flux (`.mp4`, `.ts`, `.m3u8` HLS, `rtmp://`, `rtsp://`) acceptés aussi.
Si une vidéo refuse de démarrer, mettre d'abord yt-dlp à jour (`yt-dlp -U`).

**Sous-titres de YouTube** : ceux écrits par l'auteur et ceux **générés automatiquement** par
YouTube (traductions automatiques comprises, par exemple « French from English ») apparaissent dans
le menu des sous-titres, pour les langues choisies dans Préférences › Sous-titres (`fr,en` par
défaut) et la langue d'origine de la vidéo.

---

## Interface

- **Barre de contrôle** : l'essentiel seulement. À gauche lecture/pause, arrêt, précédent,
  suivant, aléatoire, répétition et le temps (clic : temps restant) ; à droite piste audio,
  sous-titres, réglages d'image et playlist. Vitesse, capture et plein écran passent par le
  clavier et les menus (plein écran : triple-clic sur la vidéo).
- **Souris** : double-clic lecture/pause, triple-clic plein écran, molette volume, clic droit menu.
  Un simple clic ne fait rien (pas de pause par accident).
- **Menu Lumen ⌄** et **clic droit** : mêmes sous-menus Lecture, Chapitres, Signets, Vitesse,
  Audio, Sous-titres, Vidéo. Dans une petite fenêtre, menus et panneaux défilent au lieu de déborder.
- **Préférences (F5)** : *tous* les réglages, classés par catégorie (Général, Lecture, Vidéo, Audio,
  Sous-titres, En ligne, Apparence, Extensions, Raccourcis) avec une recherche. On y retrouve aussi
  ce qui se règle en cours de lecture : volume, vitesse, égaliseur bande par bande, décalages audio
  et sous-titres, réglages d'image, liste IPTV.
- **Fenêtres modales** (préférences, raccourcis, vidéo en ligne, sous-titres en ligne) : tant
  qu'elles sont ouvertes, la vidéo ne réagit ni à la souris ni aux raccourcis ; Échap les ferme.
- **Playlist** (F6) : accolée à droite ; ligne en cours en orange, ligne choisie en gras,
  glisser-déposer, recherche, Suppr pour retirer.

---

## Sous-titres générés (reconnaissance vocale)

Lumen peut créer les sous-titres d'une vidéo de l'ordinateur en écoutant ses dialogues, avec
[whisper.cpp](https://github.com/ggml-org/whisper.cpp). Tout se fait en local, sans envoyer la
vidéo sur Internet.

1. Installer whisper.cpp une fois (compilation de quelques minutes, sans sudo) :
   ```bash
   scripts/install-whisper.sh      # installe ~/.local/bin/whisper-cli
   ```
2. Pendant la lecture : menu **Sous-titres › Générer les sous-titres** (clic droit ou menu Lumen),
   ou **Préférences › Sous-titres › Générer**.

Le modèle est téléchargé à la première génération dans `~/.local/share/lumen/whisper/`
(« Rapide » 148 Mo par défaut, « Équilibré » 488 Mo, « Précis » 1,5 Go : Préférences ›
Sous-titres ; le téléchargement reprend là où il s'est arrêté en cas de coupure). L'avancement s'affiche à l'écran ; le même menu permet d'annuler. Le fichier
`Vidéo.<langue>.srt` est enregistré à côté de la vidéo (sinon dans
`~/.local/share/lumen/subtitles/`) et activé tout de suite. La langue parlée est détectée
automatiquement, ou fixée dans les préférences.

Compter, sur un processeur récent, de quelques minutes à une demi-heure par heure de vidéo selon
le modèle. Réservé aux fichiers de l'ordinateur ; pour YouTube, ses propres sous-titres
automatiques sont proposés (voir plus haut).

---

## IPTV (chaînes en direct)

Lumen ouvre les listes de chaînes **M3U** et les comptes **Xtream Codes** :

- **Lien M3U** : Ctrl+U, coller le lien de la liste (`…/liste.m3u`, `…/get.php?…&type=m3u_plus`).
- **Fichier M3U** : l'ouvrir comme une vidéo (O, glisser-déposer, « Ouvrir avec »).
- **Compte Xtream Codes** : Ctrl+U › « Compte IPTV », avec le serveur, l'utilisateur et le
  mot de passe fournis par l'abonnement.

Les chaînes remplacent la playlist, avec leur nom et leur groupe (« Sport · … ») ; la recherche
de la playlist filtre sur ces noms. La dernière liste est retenue : menu › **Chaînes IPTV** la
recharge. Elle est enregistrée dans `settings.toml` (`iptv_list`), identifiants Xtream compris.
Un lien `.m3u8` qui désigne un seul flux (HLS) se lit directement.

---

## Raccourcis personnalisables

**F5 › Raccourcis** (ou F1 › **Modifier…**) : chaque action avec ses touches.

- **+** : appuyer sur la nouvelle touche (Échap annule) ; elle s'ajoute aux touches existantes.
- **Clic sur une touche** : la retirer.
- **↺** : revenir aux touches d'origine de l'action ; **Tout réinitialiser** pour toutes.

Une touche déjà prise par une autre action lui est retirée (un message l'indique). Les
changements s'appliquent aussitôt et sont enregistrés dans `~/.config/lumen/settings.toml`
(section `[raccourcis]`), que l'on peut aussi modifier à la main puis appliquer avec
**Recharger** :

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
| `~/.config/lumen/settings.toml` | Paramètres (volume, répétition, aléatoire, fenêtre, options), dernière liste IPTV (`iptv_list`, identifiants Xtream compris), raccourcis (`[raccourcis]`) — modifiable à la main, Lumen fermé |
| `~/.config/lumen/skins/` | Skins (fichiers JSON) |
| `~/.config/lumen/scripts/` | Extensions (scripts mpv `.lua`, `.js`) |
| `~/.local/share/lumen/history.db` | Historique de lecture (positions, pistes, sous-titres, signets) |
| `~/.local/share/lumen/whisper/` | Modèles de reconnaissance vocale (sous-titres générés) |
| `~/.local/share/lumen/subtitles/` | Sous-titres téléchargés quand le dossier de la vidéo n'est pas modifiable |

---

## 3. Structure du projet

```
lumen/
├── Cargo.toml, build.rs   Dépendances ; build.rs compile Rust + C++ + QML ensemble (cxx-qt-build)
├── src/
│   ├── main.rs            Point d'entrée : options d'environnement, fenêtre unique, chargement de l'interface
│   ├── single_instance.rs Transmet le fichier à un Lumen déjà ouvert (MPRIS)
│   └── bridge/            Logique en Rust exposée à QML : historique (SQLite), playlist, paramètres,
│                          raccourcis, son, shaders, thèmes, extensions, sous-titres en ligne,
│                          listes IPTV (iptv.rs), MPRIS, mise en veille…
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
  sous-titres générés par reconnaissance vocale (whisper.cpp), sous-titres automatiques de YouTube,
  décalages audio et sous-titres
- **Navigation** : miniatures au survol, chapitres, signets, boucle A-B
- **Image et son** : luminosité, contraste…, netteté (AMD CAS), agrandissement (FSR),
  égaliseur 10 bandes, normalisation du volume
- **En ligne** : YouTube et la plupart des sites (yt-dlp), directs YouTube, flux directs
  (HLS, `.ts`, RTMP, RTSP), chaînes IPTV (listes M3U, comptes Xtream Codes)
- **Bureau** : MPRIS (touches multimédia, applet), pas de mise en veille pendant un film,
  « Ouvrir avec », fenêtre unique, sélecteur de fichiers natif
- **Interface** : inspirée de PotPlayer, barre de contrôle épurée, menus à sous-menus, playlist
  accolée, mode mini, fenêtre adaptée à la vidéo, panneau d'informations, thèmes et skins,
  préférences complètes (F5), raccourcis modifiables dans l'interface, fenêtres modales qui
  bloquent la vidéo, extensions (scripts mpv)

### Raccourcis clavier

Liste complète : **F1** ; modification : **Préférences › Raccourcis** (voir « Raccourcis
personnalisables »). Liste détaillée dans [FONCTIONNALITES.md](FONCTIONNALITES.md).

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
| F6                | Playlist                      |
| E / I             | Son / réglages d'image        |
| S                 | Capture d'écran               |
| Ctrl+M            | Mode mini                     |

---

## 6. Idées pour la suite

- Miniatures dans la playlist
- Publication sur Flathub (demande d'ajout à compléter) et sur l'AUR (fichiers prêts dans `packaging/`)
- Guide des programmes (EPG) et logos des chaînes IPTV
- Option « toutes les vidéos du dossier » pour la playlist automatique
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
