# Shaders d'image

Shaders GLSL pour mpv, exécutés sur la carte graphique (compatibles avec le décodage matériel).
Lumen les intègre au programme et en écrit une copie réglée dans `~/.cache/lumen/shaders/`.

| Fichier | Rôle | Source |
|---|---|---|
| `CAS.glsl` | Netteté (image non redimensionnée) | AMD FidelityFX CAS, portage mpv par agyild : https://gist.github.com/agyild/bbb4e58298b2f86aa24da3032a0d2ee6 |
| `CAS-scaled.glsl` | Netteté (image agrandie) | même source |
| `FSR.glsl` | Agrandissement FSR 1.0 (EASU + RCAS) | AMD FidelityFX Super Resolution, portage mpv par agyild : https://gist.github.com/agyild/82219c545228d70c5604f865ce0b0ce5 |

Licence : MIT, © Advanced Micro Devices, Inc. (texte complet en tête de chaque fichier).
