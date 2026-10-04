#!/usr/bin/env bash
# Désinstalle Lumen de ~/.local (ou de PREFIX). Les paramètres (~/.config/lumen) et
# l'historique de lecture (~/.local/share/lumen) sont conservés.
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
SHARE="$PREFIX/share"

rm -f "$PREFIX/bin/lumen" "$SHARE/applications/lumen.desktop"
rm -f "$SHARE"/icons/hicolor/*/apps/lumen.png "$SHARE/icons/hicolor/scalable/apps/lumen.svg"

# Lignes « type=application » de la section [Default Applications] d'un fichier d'associations
default_entries() {
    awk '/^\[/ { in_defaults = ($0 == "[Default Applications]"); next } in_defaults && /=/' "$1"
}

# Retirer Lumen des associations (fichier standard et fichiers propres aux bureaux), puis
# rétablir l'application par défaut d'avant l'installation pour les types restés sans
for mimeapps in "${XDG_CONFIG_HOME:-$HOME/.config}"/*mimeapps.list; do
    [[ -f "$mimeapps" ]] || continue
    sed -i -e 's/lumen\.desktop;\?//g' -e '/^[^=[]*=$/d' "$mimeapps"

    backup="$mimeapps.avant-lumen"
    [[ -f "$backup" ]] || continue
    current=$(default_entries "$mimeapps" | cut -d= -f1)
    missing=$(default_entries "$backup" | grep -v 'lumen\.desktop' \
        | awk -F= -v current="$current" 'BEGIN { split(current, c, "\n"); for (i in c) have[c[i]] = 1 } !have[$1]')
    if [[ -n "$missing" ]]; then
        grep -q '^\[Default Applications\]' "$mimeapps" || printf '\n[Default Applications]\n' >> "$mimeapps"
        awk -v restore="$missing" '
            { print }
            $0 == "[Default Applications]" { print restore }
        ' "$mimeapps" > "$mimeapps.tmp" && mv "$mimeapps.tmp" "$mimeapps"
    fi
    rm -f "$backup"
done

update-desktop-database "$SHARE/applications" 2>/dev/null || true
gtk-update-icon-cache --force --ignore-theme-index "$SHARE/icons/hicolor" 2>/dev/null || true

echo "Lumen est désinstallé (paramètres et historique conservés)."
