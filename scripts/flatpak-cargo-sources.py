#!/usr/bin/env python3
"""Génère packaging/flatpak/cargo-sources.json depuis Cargo.lock.

Flathub construit sans accès au réseau : chaque crate est déclarée comme source (archive de
crates.io et sa somme SHA-256, déjà présente dans Cargo.lock), puis cargo est configuré pour
utiliser ce dossier « vendor ». Même résultat que flatpak-cargo-generator pour des
dépendances venant toutes de crates.io, sans dépendance Python.

    scripts/flatpak-cargo-sources.py      (à relancer après chaque changement de Cargo.lock)
"""
import json
import pathlib
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"


def main() -> int:
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    sources = []
    for package in lock["package"]:
        source = package.get("source")
        if source is None:
            continue  # Lumen lui-même
        if source != CRATES_IO:
            print(f"Source non prise en charge : {package['name']} ({source})", file=sys.stderr)
            return 1
        name, version, checksum = package["name"], package["version"], package["checksum"]
        dest = f"cargo/vendor/{name}-{version}"
        sources.append({
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": f"https://static.crates.io/crates/{name}/{name}-{version}.crate",
            "sha256": checksum,
            "dest": dest,
        })
        sources.append({
            "type": "inline",
            "contents": json.dumps({"package": checksum, "files": {}}),
            "dest": dest,
            "dest-filename": ".cargo-checksum.json",
        })
    sources.append({
        "type": "inline",
        "contents": '[source.vendored-sources]\ndirectory = "cargo/vendor"\n\n'
                    '[source.crates-io]\nreplace-with = "vendored-sources"\n',
        "dest": "cargo",
        "dest-filename": "config.toml",
    })
    output = ROOT / "packaging/flatpak/cargo-sources.json"
    output.write_text(json.dumps(sources, indent=2) + "\n")
    print(f"{output.relative_to(ROOT)} : {(len(sources) - 1) // 2} crates")
    return 0


if __name__ == "__main__":
    sys.exit(main())
