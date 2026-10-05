#!/usr/bin/env bash
# Construit dist/Lumen-<version>-x86_64.AppImage : un seul fichier exécutable qui embarque
# Lumen, Qt (avec ses modules QML et ses greffons Wayland/X11) et libmpv.
#
#   scripts/build-appimage.sh
#
# Outils téléchargés au premier lancement dans target/appimage-tools (linuxdeploy et son
# greffon Qt). L'AppImage fonctionne sur les distributions dont la glibc est au moins aussi
# récente que celle de la machine de construction : construire sur une distribution ancienne
# pour viser large.
set -euo pipefail

cd "$(dirname "$0")/.."
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
TOOLS=target/appimage-tools
APPDIR=target/appimage/AppDir
OUTPUT="dist/Lumen-$VERSION-x86_64.AppImage"

mkdir -p "$TOOLS" dist
for tool in linuxdeploy linuxdeploy-plugin-qt; do
    if [[ ! -x "$TOOLS/$tool-x86_64.AppImage" ]]; then
        echo "Téléchargement de $tool…"
        curl -sSLf -o "$TOOLS/$tool-x86_64.AppImage" \
            "https://github.com/linuxdeploy/$tool/releases/download/continuous/$tool-x86_64.AppImage"
        chmod +x "$TOOLS/$tool-x86_64.AppImage"
    fi
done
# Démarreur de l'AppImage, fourni à appimagetool (son propre téléchargement échoue parfois)
if [[ ! -f "$TOOLS/runtime-x86_64" ]]; then
    echo "Téléchargement du démarreur AppImage…"
    curl -sSLf -o "$TOOLS/runtime-x86_64" \
        "https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64"
fi
export LDAI_RUNTIME_FILE="$PWD/$TOOLS/runtime-x86_64"
# Pas besoin de FUSE : les outils s'extraient et s'exécutent
export APPIMAGE_EXTRACT_AND_RUN=1

echo "Compilation de Lumen $VERSION (mode optimisé)…"
cargo build --release

rm -rf "$APPDIR"

# Greffon Qt : modules QML repérés dans qml/, greffons Wayland et X11, thème du portail
# (sélecteur de fichiers natif), SVG (icônes)
export QMAKE=/usr/bin/qmake6
export QML_SOURCES_PATHS="$PWD/qml"
# « offscreen » : lancement sans affichage (tests, captures)
export EXTRA_PLATFORM_PLUGINS="libqwayland-egl.so;libqwayland-generic.so;libqoffscreen.so"
export EXTRA_QT_MODULES="svg;waylandclient"
export LDAI_OUTPUT="$OUTPUT"
export LINUXDEPLOY_OUTPUT_VERSION="$VERSION"

PATH="$PWD/$TOOLS:$PATH" "$TOOLS/linuxdeploy-x86_64.AppImage" \
    --appdir "$APPDIR" \
    --executable target/release/lumen \
    --desktop-file assets/lumen.desktop \
    --icon-file assets/icons/hicolor/256x256/apps/lumen.png \
    --icon-file assets/icons/hicolor/scalable/apps/lumen.svg \
    --plugin qt

# Greffons que linuxdeploy ne déduit pas des bibliothèques liées : intégration Wayland,
# thème du portail XDG (dialogues natifs), icônes et images SVG
QT_PLUGINS=$("$QMAKE" -query QT_INSTALL_PLUGINS)
for dir in wayland-shell-integration wayland-decoration-client wayland-graphics-integration-client \
           platformthemes iconengines imageformats; do
    if [[ -d "$QT_PLUGINS/$dir" ]]; then
        mkdir -p "$APPDIR/usr/plugins/$dir"
        cp -n "$QT_PLUGINS/$dir"/*.so "$APPDIR/usr/plugins/$dir/" 2>/dev/null || true
    fi
done
# Bibliothèques nécessaires à ces greffons ajoutés après coup
PATH="$PWD/$TOOLS:$PATH" "$TOOLS/linuxdeploy-x86_64.AppImage" --appdir "$APPDIR" \
    $(find "$APPDIR/usr/plugins" -name '*.so' -printf '--deploy-deps-only=%p ') \
    --output appimage

echo "AppImage prête : $OUTPUT"
