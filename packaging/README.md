# Distribution de Lumen

Trois formats, en plus de l'installation locale (`scripts/install.sh`).

| Format | Fichiers | Produire | Vérifié |
|---|---|---|---|
| **AppImage** | `scripts/build-appimage.sh` | `scripts/build-appimage.sh` → `dist/Lumen-<version>-x86_64.AppImage` | Construite et lancée (interface, lecture, icônes, chapitres) |
| **Flatpak** | `flatpak/io.github.benkl404.Lumen.yml`, `flatpak/cargo-sources.json`, `../assets/io.github.benkl404.Lumen.metainfo.xml` | voir ci-dessous | Tous les modules compilent (LuaJIT, libass, uchardet, libplacebo, mpv, yt-dlp, Lumen) ; Lumen lancé dans le bac à sable avec le runtime KDE 6.11 |
| **AUR** | `aur/PKGBUILD`, `aur/.SRCINFO` | `makepkg -si` dans `aur/` (sur Arch) | Syntaxe vérifiée ; non construit (pas d'Arch disponible) |

## AppImage

```bash
scripts/build-appimage.sh
```

Télécharge `linuxdeploy` et son greffon Qt dans `target/appimage-tools/`, compile Lumen et
rassemble Qt (modules QML, greffons Wayland/X11, SVG, portail), libmpv et FFmpeg. L'AppImage
fonctionne sur les distributions dont la glibc est au moins aussi récente que celle de la
machine de construction : pour viser large, construire sur une distribution plus ancienne.

## Flatpak (Flathub)

Le manifeste construit libmpv et ses dépendances sur le runtime KDE (Qt 6) ; les crates Rust
sont listées dans `cargo-sources.json` (Flathub construit sans réseau). Après tout changement
de `Cargo.lock` :

```bash
scripts/flatpak-cargo-sources.py
```

Construction et installation locales (outil de construction installé depuis Flathub) :

```bash
flatpak install --user flathub org.flatpak.Builder org.kde.Sdk//6.11 org.freedesktop.Sdk.Extension.rust-stable//25.08
flatpak run org.flatpak.Builder --user --install-deps-from=flathub --force-clean --install \
    build-dir packaging/flatpak/io.github.benkl404.Lumen.yml
flatpak run io.github.benkl404.Lumen
```

Pour Flathub : demande d'ajout sur https://github.com/flathub/flathub (branche `new-pr`) avec
le manifeste, `cargo-sources.json` et une source `git` (tag et commit publiés) à la place de
`type: dir`. L'identifiant `io.github.benkl404.Lumen` doit correspondre au dépôt
`github.com/BenKL404/lumen` (vérifié par le linter de Flathub). Les captures d'écran de la fiche
AppStream sont dans `assets/screenshots/`, servies depuis le tag de la version.

Vérification avant envoi (`flatpak-builder-lint`, fourni avec `org.flatpak.Builder`) :

```bash
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest packaging/flatpak/io.github.benkl404.Lumen.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder appstream assets/io.github.benkl404.Lumen.metainfo.xml
```

Permissions demandées (`finish-args`) : Wayland/X11, carte graphique, son, réseau (vidéos en
ligne, sous-titres), mise en veille (`org.freedesktop.ScreenSaver`) et MPRIS. Fichiers : dossiers
Vidéos, Téléchargements, Musique et Bureau (captures), disques externes (`/media`, `/run/media`,
`/mnt`). Flathub refuse `--filesystem=host` sans dérogation : ailleurs, une vidéo ouverte passe par
le portail de documents et s'ouvre seule (pas de playlist automatique des fichiers voisins ;
sous-titres téléchargés enregistrés dans le dossier de données de Lumen).

## AUR

```bash
cd packaging/aur
makepkg -si              # construire et installer
makepkg --printsrcinfo > .SRCINFO   # après toute modification du PKGBUILD
```

Publication : pousser `PKGBUILD` et `.SRCINFO` dans le dépôt AUR `lumen-player-git`
(compte AUR et clé SSH nécessaires).
