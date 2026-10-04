#!/usr/bin/env bash
# Installe Lumen pour l'utilisateur courant, dans ~/.local (sans sudo) :
# binaire, icônes, entrée du lanceur d'applications et associations de fichiers.
#
#   scripts/install.sh            installe Lumen (proposé dans « Ouvrir avec »)
#   scripts/install.sh --default  … et en fait le lecteur vidéo par défaut
#
# PREFIX=/autre/chemin scripts/install.sh pour installer ailleurs.
set -euo pipefail

cd "$(dirname "$0")/.."
PREFIX="${PREFIX:-$HOME/.local}"
SHARE="$PREFIX/share"

echo "Compilation de Lumen (mode optimisé)…"
cargo build --release

echo "Installation dans $PREFIX…"
install -Dm755 target/release/lumen "$PREFIX/bin/lumen"

install -Dm644 assets/icons/hicolor/scalable/apps/lumen.svg "$SHARE/icons/hicolor/scalable/apps/lumen.svg"
for size in 16 24 32 48 64 128 256 512; do
    install -Dm644 "assets/icons/hicolor/${size}x${size}/apps/lumen.png" \
        "$SHARE/icons/hicolor/${size}x${size}/apps/lumen.png"
done

# Chemins complets dans Exec/TryExec : fonctionne même si ~/.local/bin n'est pas dans le PATH.
# Icon aussi : certains bureaux (COSMIC) ne cherchent pas les icônes dans ~/.local/share/icons
# faute de fichier index.theme ; un chemin complet est valable partout.
tmp="$(mktemp)"
sed -e "s|^Exec=lumen|Exec=$PREFIX/bin/lumen|" -e "s|^TryExec=lumen|TryExec=$PREFIX/bin/lumen|" \
    -e "s|^Icon=lumen|Icon=$SHARE/icons/hicolor/scalable/apps/lumen.svg|" \
    assets/lumen.desktop > "$tmp"
install -Dm644 "$tmp" "$SHARE/applications/lumen.desktop"
rm -f "$tmp"

# Mise à jour des caches (associations de fichiers, icônes) ; facultatif selon le bureau
update-desktop-database "$SHARE/applications" 2>/dev/null || true
gtk-update-icon-cache --force --ignore-theme-index "$SHARE/icons/hicolor" 2>/dev/null || true

# Fait de Lumen l'application par défaut des types `$@` dans le fichier d'associations $1
# (section [Default Applications], en remplaçant l'éventuelle ligne existante).
set_default() {
    local file=$1 mime
    shift
    mkdir -p "$(dirname "$file")"
    [[ -f "$file" ]] || : > "$file"
    # Copie des associations d'origine, une seule fois : la désinstallation les rétablit
    [[ -f "$file.avant-lumen" ]] || cp "$file" "$file.avant-lumen"
    grep -q '^\[Default Applications\]' "$file" || printf '\n[Default Applications]\n' >> "$file"
    for mime in "$@"; do
        awk -v entry="$mime=lumen.desktop" -v prefix="$mime=" '
            /^\[/ {
                if (in_defaults && !done) { print entry; done = 1 }
                in_defaults = ($0 == "[Default Applications]")
                print; next
            }
            in_defaults && index($0, prefix) == 1 { if (!done) { print entry; done = 1 }; next }
            { print }
            END { if (!done) print entry }
        ' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
    done
}

if [[ "${1:-}" == "--default" ]]; then
    # Vidéos seulement : les fichiers audio restent au lecteur de musique habituel
    mapfile -t mimes < <(grep '^MimeType=' assets/lumen.desktop | cut -d= -f2 | tr ';' '\n' | grep '^video/')
    config="${XDG_CONFIG_HOME:-$HOME/.config}"
    set_default "$config/mimeapps.list" "${mimes[@]}"
    # Les bureaux lisent d'abord leur propre fichier (cosmic-mimeapps.list, gnome-mimeapps.list…)
    IFS=: read -ra desktops <<< "${XDG_CURRENT_DESKTOP:-}"
    for desktop in "${desktops[@]}"; do
        [[ -n "$desktop" ]] && set_default "$config/${desktop,,}-mimeapps.list" "${mimes[@]}"
    done
    echo "Lumen est maintenant le lecteur vidéo par défaut."
fi

echo "Lumen est installé : lanceur d'applications, et clic droit › Ouvrir avec dans le gestionnaire de fichiers."
