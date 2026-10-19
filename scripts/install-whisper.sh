#!/usr/bin/env bash
# Installe whisper-cli (whisper.cpp), utilisé par Lumen pour générer des sous-titres par
# reconnaissance vocale, dans ~/.local/bin (sans sudo). Le modèle est téléchargé par Lumen
# lui-même à la première génération.
#
#   scripts/install-whisper.sh          PREFIX=… pour installer ailleurs
#
# Il faut un compilateur C++, make et git. CMake est téléchargé s'il manque.
set -euo pipefail

WHISPER_VERSION=v1.9.5
CMAKE_VERSION=4.4.4
PREFIX=${PREFIX:-$HOME/.local}
WORK=${XDG_CACHE_HOME:-$HOME/.cache}/lumen/whisper-build
mkdir -p "$WORK" "$PREFIX/bin"

# CMake : celui du système, sinon une version officielle portable (somme SHA-256 vérifiée)
if command -v cmake >/dev/null; then
    CMAKE=cmake
else
    tarball=cmake-$CMAKE_VERSION-linux-x86_64.tar.gz
    base=https://github.com/Kitware/CMake/releases/download/v$CMAKE_VERSION
    if [[ ! -x "$WORK/cmake-$CMAKE_VERSION-linux-x86_64/bin/cmake" ]]; then
        echo "Téléchargement de CMake $CMAKE_VERSION…"
        curl -fsSL -o "$WORK/$tarball" "$base/$tarball"
        curl -fsSL -o "$WORK/cmake-SHA-256.txt" "$base/cmake-$CMAKE_VERSION-SHA-256.txt"
        (cd "$WORK" && grep " $tarball\$" cmake-SHA-256.txt | sha256sum -c -)
        tar -xzf "$WORK/$tarball" -C "$WORK"
    fi
    CMAKE=$WORK/cmake-$CMAKE_VERSION-linux-x86_64/bin/cmake
fi

echo "Téléchargement de whisper.cpp $WHISPER_VERSION…"
rm -rf "$WORK/whisper.cpp"
git clone -q --depth 1 --branch "$WHISPER_VERSION" https://github.com/ggml-org/whisper.cpp.git "$WORK/whisper.cpp"

echo "Compilation (quelques minutes)…"
"$CMAKE" -S "$WORK/whisper.cpp" -B "$WORK/whisper.cpp/build" -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=OFF -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_EXAMPLES=ON -DGGML_NATIVE=ON >/dev/null
"$CMAKE" --build "$WORK/whisper.cpp/build" --target whisper-cli -j"$(nproc)" >/dev/null

install -Dm755 "$WORK/whisper.cpp/build/bin/whisper-cli" "$PREFIX/bin/whisper-cli"
echo "whisper-cli installé : $PREFIX/bin/whisper-cli"
