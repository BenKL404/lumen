# Lumen — Fonctionnalités

Liste claire des fonctionnalités du lecteur vidéo Lumen : ce que fait déjà la v0.1, puis ce qui est prévu pour les versions suivantes.

---

## 🎨 Design : ultra moderne, pro, esprit PotPlayer

L'objectif : la richesse et l'efficacité de PotPlayer, avec la finition d'une application actuelle. Tout est pensé pour que l'image reste reine et que l'interface s'efface quand on regarde.

### Principes
- **La vidéo d'abord** : aucune bordure inutile, l'interface flotte au-dessus de l'image et disparaît pendant la lecture.
- **Dense mais lisible** : comme PotPlayer, beaucoup de fonctions accessibles en un clic, mais bien rangées et hiérarchisées.
- **Réactif** : chaque action a un retour visuel immédiat (survol, appui, OSD).
- **Cohérent** : un seul système de couleurs, d'espacements et d'arrondis pour toute l'application.

### Identité visuelle

| Élément | Choix |
|---|---|
| Fond | Noir bleuté profond `#0B0D10` |
| Surfaces | Verre fumé translucide avec flou d'arrière-plan |
| Texte principal | Blanc cassé `#ECE8E1` |
| Texte secondaire | Gris doux `#8B9099` |
| Couleur d'accent | Orange vif `#FF8C1A` |
| Arrondis | 16 px pour les panneaux, boutons ronds |
| Police | Sans-serif moderne (Inter ou équivalent), chiffres à chasse fixe pour les durées |
| Icônes | Jeu d'icônes vectorielles fines et homogènes (style Lucide) |

La couleur d'accent sera personnalisable, avec un thème clair en option.

### Fenêtre
- **Fenêtre sans bordure** avec barre de titre intégrée : nom du fichier, boutons réduire / agrandir / fermer.
- Barre de titre qui disparaît en même temps que les contrôles.
- **Toujours au premier plan** activable (épingle), comme dans PotPlayer.
- **Mode mini** : petite fenêtre compacte sans contrôles, redimensionnable, idéale pour regarder en travaillant.
- Redimensionnement intelligent qui respecte le format de la vidéo.

### Barre de contrôle
- Panneau flottant en verre fumé, détaché des bords, qui apparaît au mouvement de la souris.
- **Gauche** : lecture/pause (bouton ambre mis en avant), précédent, suivant, ±10 s, temps écoulé / durée totale.
- **Droite** : vitesse, pistes audio, sous-titres, volume, capture, playlist, paramètres, plein écran.
- Clic sur la durée totale pour afficher le **temps restant** à la place (comme PotPlayer).
- Barre de progression qui s'épaissit au survol, avec **miniature** et heure au-dessus du curseur.
- **Repères de chapitres** visibles directement sur la barre de progression.
- Barre de volume pouvant monter jusqu'à 130 %, avec indication visuelle au-delà de 100 %.

### Menus et panneaux
- **Menu clic droit complet**, signature de PotPlayer : ouvrir, lecture, vidéo, audio, sous-titres, playlist, captures, paramètres… avec sous-menus, icônes et raccourcis affichés à droite.
- **Playlist latérale** glissant depuis la droite : miniatures, durées, élément en cours surligné en ambre, glisser-déposer pour réorganiser.
- **Panneau d'informations** (touche Tab) : codec, résolution, débit, images par seconde, décodage matériel actif ou non, à la manière de l'écran d'infos de PotPlayer.
- **Paramètres** dans une fenêtre moderne à onglets (Général, Lecture, Vidéo, Audio, Sous-titres, Raccourcis, Apparence), avec recherche intégrée.

### OSD (messages à l'écran)
- Messages discrets en haut à droite : volume, vitesse, saut, capture, piste changée.
- **Jauge visuelle** pour le volume et la luminosité, en plus du texte.
- Message de reprise « Reprise à 42:15 — Recommencer » avec bouton cliquable.
- Apparition et disparition en fondu rapide.

### Animations
- Courtes et utiles (150 à 200 ms) : apparition des contrôles, ouverture des panneaux, survol des boutons.
- Effet de pression léger sur les boutons.
- Aucune animation décorative qui distrait de la vidéo.

### Écran d'accueil
- Quand aucune vidéo n'est ouverte : logo Lumen, invitation à glisser un fichier, et liste des **vidéos récentes** avec miniature et barre de progression indiquant où tu t'es arrêté.

### Personnalisation (esprit skins PotPlayer)
- Thèmes sombre, clair et « OLED noir pur ».
- Couleur d'accent au choix.
- Taille de l'interface réglable (compact, normal, grand).
- Skins complets définis dans des fichiers JSON, partageables entre utilisateurs.

---

## ✅ Ce que fait déjà la v0.1

### Lecture
- Lit tous les formats vidéo et audio courants (MKV, MP4, AVI, WebM, MOV, MP3, FLAC…).
- Utilise automatiquement la carte graphique pour décoder, ce qui économise le processeur et la batterie.
- Reste sur la dernière image à la fin de la vidéo au lieu de fermer le fichier.

### Ouverture de fichiers
- Bouton « Ouvrir » ou touche **O** pour choisir un fichier.
- Glisser-déposer d'une vidéo directement dans la fenêtre.

### Contrôles
- Clic sur la vidéo pour mettre en pause, double-clic pour le plein écran.
- Molette de la souris pour régler le volume.
- Barre de progression cliquable, avec l'heure affichée au survol.
- Boutons pour reculer ou avancer de 10 secondes.
- Bouton de vitesse (×1 → ×1,25 → ×1,5 → … → ×2).
- Curseur de volume, capture d'écran et plein écran.

### Interface
- La barre de contrôle et le curseur de la souris disparaissent pendant la lecture, puis réapparaissent dès que la souris bouge.
- Le titre de la vidéo s'affiche en haut de l'écran.
- Un petit message (OSD) confirme chaque action : volume, vitesse, saut, capture…

### Raccourcis clavier

| Touche | Action |
|---|---|
| Espace | Lecture / pause |
| ← / → | Reculer / avancer de 5 s |
| Ctrl + ← / → | Reculer / avancer de 30 s |
| ↑ / ↓ | Volume ±5 % |
| M | Muet |
| F ou Entrée | Plein écran |
| Échap | Quitter le plein écran |
| O | Ouvrir un fichier |
| S | Capture d'écran (enregistrée sur le Bureau) |
| . / , | Image suivante / précédente |
| ] / [ | Vitesse ±0,1 |
| Retour arrière | Vitesse normale |
| J | Changer de sous-titres |
| A | Changer de piste audio |
| Z / X | Décaler les sous-titres de −0,1 s / +0,1 s |
| Page préc. / Page suiv. | Fichier précédent / suivant de la playlist |
| F6 | Afficher / masquer la playlist |
| Ctrl + Page préc. / suiv. | Chapitre précédent / suivant |
| L | Boucle A-B : début, fin, désactivation |
| B | Ajouter un signet |
| R / H | Répétition / lecture aléatoire |
| Ctrl + O | Ouvrir un dossier |

---

## 🔜 Ce qui est prévu ensuite

### v0.2 — Pistes et mémoire
- Menus pour choisir précisément la piste audio et les sous-titres.
- Ajout de sous-titres externes (.srt, .ass) et réglage de leur décalage.
- **Rappel de la position après fermeture** : Lumen mémorise l'endroit où tu t'es arrêté dans chaque vidéo, même si tu fermes le lecteur ou éteins l'ordinateur.
  - À la réouverture du fichier, la lecture reprend automatiquement à cette position, avec un message « Reprise à 42:15 » et la possibilité de recommencer depuis le début.
  - La piste audio, les sous-titres choisis, leur décalage et la vitesse de lecture sont aussi restaurés.
  - La position n'est pas enregistrée pour les vidéos très courtes, ni quand la vidéo a été regardée jusqu'au bout.
  - Au démarrage, Lumen peut proposer de reprendre la dernière vidéo regardée.
  - L'historique peut être effacé et l'option désactivée dans les paramètres.
- Playlist, et passage automatique au fichier suivant du dossier.
- **Playlist automatique des vidéos du même nom** : à l'ouverture d'une vidéo, Lumen ajoute tout seul à la playlist les fichiers du même dossier qui portent le même nom de base (par exemple `Ma Série - Épisode 01.mkv`, `Ma Série - Épisode 02.mkv`, ou `Film - Partie 1.mp4`, `Film - Partie 2.mp4`).
  - Les fichiers sont classés dans l'ordre naturel (l'épisode 2 passe avant l'épisode 10).
  - La lecture enchaîne automatiquement sur le fichier suivant.
  - Les variantes courantes sont reconnues : `S01E02`, `E02`, `Ep 2`, `Partie 2`, `CD2`…
  - L'option peut être désactivée dans les paramètres.

### v0.3 — Le confort PotPlayer
- Miniatures de la vidéo au survol de la barre de progression.
- Boucle A-B pour répéter un passage.
- Signets et navigation par chapitres.
- Réglages d'image : luminosité, contraste, saturation, rotation, zoom.
- Menu clic droit complet.

### v0.4 — Intégration à Linux
- Contrôle par les touches multimédia du clavier et depuis le bureau (GNOME, KDE).
- L'écran ne se met plus en veille pendant un film.
- « Ouvrir avec Lumen » et lecteur par défaut du système.
- Raccourcis personnalisables et fichier de configuration.

### v1.0 — La puissance
- Filtres pour améliorer l'image (netteté, agrandissement haute qualité).
- Égaliseur audio.
- Lecture de vidéos en ligne et YouTube.
- Recherche automatique de sous-titres sur Internet.
- Système d'extensions pour ajouter des fonctions.
- Thèmes et apparence personnalisables.
- Installation facile via Flatpak, AppImage et AUR.
