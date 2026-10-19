# Lumen — Fonctionnalités

Liste claire des fonctionnalités du lecteur vidéo Lumen : le design, les fonctions de base, puis la feuille de route, entièrement réalisée en version 1.0.

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

La couleur d'accent est personnalisable, et des thèmes clair et OLED existent (voir Personnalisation). Aucune couleur n'est écrite en dur dans l'interface : tout passe par les jetons du thème.

### Fenêtre
- **Fenêtre sans bordure** avec barre de titre intégrée : nom du fichier, boutons réduire / agrandir / fermer.
- En plein écran et en mode mini, la barre de titre disparaît en même temps que les contrôles.
- **Toujours au premier plan** activable (épingle), comme dans PotPlayer.
- **Mode mini** : petite fenêtre compacte sans contrôles, redimensionnable, idéale pour regarder en travaillant.
- Redimensionnement intelligent qui respecte le format de la vidéo (option « Adapter la fenêtre à la vidéo »).
- Taille minimale 640 × 400 : menus et panneaux restent dans la fenêtre et font défiler leur contenu quand la place manque.

### Barre de contrôle
- Sous la vidéo en mode fenêtré ; en plein écran et en mode mini, panneau flottant en verre fumé qui apparaît au mouvement de la souris.
- **Gauche** : lecture/pause (ouvre un fichier quand rien n'est chargé), arrêt, précédent, suivant, aléatoire, répétition, temps écoulé / durée totale.
- **Droite** : piste audio, sous-titres, réglages d'image, playlist (vitesse et capture : clic droit et clavier ; plein écran : triple-clic, F ou Entrée).
- Clic sur la durée totale pour afficher le **temps restant** à la place (comme PotPlayer).
- Barre de progression qui s'épaissit au survol, avec **miniature** et heure au-dessus du curseur.
- **Repères de chapitres** visibles directement sur la barre de progression.
- Barre de volume pouvant monter jusqu'à 130 %, avec indication visuelle au-delà de 100 %.
- Peu de boutons, l'essentiel seulement : le reste passe par les menus et le clavier. Dans une fenêtre étroite, les boutons secondaires se masquent par paliers (aléatoire et répétition d'abord, puis pistes et réglages d'image, puis arrêt).

### Menus et panneaux
- **Menu Lumen ⌄** (barre de titre) : ouvrir (fichier, dossier, vidéo en ligne, chaînes IPTV, sous-titres), puis les sous-menus **Lecture, Chapitres, Signets, Vitesse, Audio, Sous-titres, Vidéo**, l'affichage (playlist, plein écran, mode mini, premier plan), les préférences et les raccourcis.
- **Menu clic droit complet**, signature de PotPlayer : les mêmes sous-menus, avec icônes et raccourcis affichés à droite.
- **Playlist latérale** accolée à droite de la fenêtre : durées, glisser-déposer pour réorganiser, recherche. La ligne en cours est en orange avec un repère à gauche ; la ligne choisie est en gras ; aucun fond, le texte seul fait la différence. Pour une liste IPTV, chaque ligne porte le nom et le groupe de la chaîne.
- **Panneaux posés au-dessus de la barre** : pistes audio et sous-titres, son (égaliseur 10 bandes, normalisation, décalage), réglages d'image.
- **Panneau d'informations** (touche Tab) : codec, résolution, débit, images par seconde, décodage matériel actif ou non, à la manière de l'écran d'infos de PotPlayer.
- **Préférences** (F5) : catégories Général, Lecture, Vidéo, Audio, Sous-titres, En ligne, Apparence, Extensions, Raccourcis, avec recherche intégrée. **Tous les réglages de l'application y figurent**, y compris ceux qu'on change en cours de lecture (volume, vitesse, égaliseur bande par bande, décalages audio et sous-titres, réglages d'image, liste IPTV).
- **Raccourcis modifiables dans l'interface** (Préférences › Raccourcis) : + pour capturer une touche, clic pour en retirer une, ↺ pour revenir à l'origine. Une touche reprise à une autre action lui est retirée, sans conflit possible.
- **Fenêtres modales** (préférences, raccourcis, vidéo en ligne, recherche de sous-titres) : un voile assombrit le reste et la vidéo ne réagit plus (clics, molette, raccourcis) tant qu'elles sont ouvertes ; Échap ou un clic à côté les ferme.
- **Choix sobres** : les options exclusives (agrandissement, répétition, thème…) forment un sélecteur segmenté discret, l'option active en gras sur un fond un peu plus clair, sans couleur d'accent.

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
- Quand aucune vidéo n'est ouverte : **écran vide**, le fond seul, sans logo ni texte. On ouvre une vidéo par glisser-déposer, touche O, clic droit ou menu Lumen.

### Personnalisation (esprit skins PotPlayer)
- Thèmes sombre, clair et « OLED noir pur ».
- Couleur d'accent au choix.
- Taille de l'interface réglable (compact, normal, grand).
- Skins complets définis dans des fichiers JSON, partageables entre utilisateurs.

---

## ✅ Fonctionnalités de base

### Lecture
- Lit tous les formats vidéo et audio courants (MKV, MP4, AVI, WebM, MOV, MP3, FLAC…).
- Utilise automatiquement la carte graphique pour décoder, ce qui économise le processeur et la batterie.
- Reste sur la dernière image à la fin de la vidéo au lieu de fermer le fichier.

### Ouverture de fichiers
- Touche **O** (fichier) ou **Ctrl+O** (dossier entier en playlist), menu Lumen ou clic droit.
- Le bouton lecture ouvre le sélecteur de fichiers quand aucune vidéo n'est chargée.
- Glisser-déposer d'une vidéo, d'un dossier, d'un lien ou d'une liste `.m3u` directement dans la fenêtre.
- « Ouvrir avec Lumen » depuis le gestionnaire de fichiers, ou `lumen fichier|dossier|lien` dans un terminal.

### Contrôles
- Double-clic sur la vidéo pour lecture / pause, triple-clic pour le plein écran (un simple clic ne fait rien, pour éviter les pauses par accident).
- Molette de la souris pour régler le volume.
- Barre de progression cliquable, avec l'heure et une miniature au survol.
- Sauts courts et longs au clavier (5 s et 30 s par défaut, réglables dans les préférences).
- Vitesse, capture d'écran et plein écran au clavier, dans les menus et dans les préférences (vitesse).
- Curseur de volume sous la barre de progression.

### Interface
- La barre de contrôle et le curseur de la souris disparaissent pendant la lecture, puis réapparaissent dès que la souris bouge.
- Le titre de la vidéo s'affiche en haut de l'écran.
- Un petit message (OSD) confirme chaque action : volume, vitesse, saut, capture…

### Raccourcis clavier

| Touche | Action |
|---|---|
| Espace | Lecture / pause |
| ← / → | Reculer / avancer (saut court, 5 s par défaut) |
| Ctrl + ← / → | Reculer / avancer (saut long, 30 s par défaut) |
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
| I | Réglages d'image (luminosité, contraste, saturation, gamma, teinte, zoom) |
| F1 | Liste des raccourcis (« Modifier… » ouvre Préférences › Raccourcis) |
| F5 | Préférences (comme PotPlayer) |
| Ctrl + J | Rechercher des sous-titres en ligne (OpenSubtitles) |
| Tab | Informations sur le fichier (codec, résolution, débit, décodage…) |
| Ctrl + M | Mode mini |
| Ctrl + U | Ouvrir une vidéo en ligne (YouTube…) |
| E | Son : égaliseur, normalisation, décalage audio |
| N | Normaliser le volume (dialogues / explosions) |
| Maj + Z / Maj + X | Décaler le son de −0,1 s / +0,1 s |
| Ctrl + V | Coller un lien vidéo |
| R / H | Répétition / lecture aléatoire |
| Ctrl + O | Ouvrir un dossier |
| Suppr | Retirer de la playlist |
| Ctrl + Q | Quitter |

Toutes ces touches se changent dans **Préférences › Raccourcis** (Échap reste réservé à la fermeture des menus, de la playlist et du plein écran).

---

## ✅ Feuille de route — réalisée (Lumen 1.0)

### ✅ v0.2 — Pistes et mémoire
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

### ✅ v0.3 — Le confort PotPlayer
- Miniatures de la vidéo au survol de la barre de progression.
- Boucle A-B pour répéter un passage.
- Signets et navigation par chapitres.
- Réglages d'image : luminosité, contraste, saturation, rotation, zoom.
- Menu clic droit complet.

### ✅ v0.4 — Intégration à Linux
- Contrôle par les touches multimédia du clavier et depuis le bureau (GNOME, KDE).
- L'écran ne se met plus en veille pendant un film.
- « Ouvrir avec Lumen » et lecteur par défaut du système.
- Raccourcis personnalisables (dans l'interface ou dans le fichier de configuration).

### ✅ v1.0 — La puissance
- Filtres pour améliorer l'image (netteté, agrandissement haute qualité).
- Égaliseur audio.
- Lecture de vidéos en ligne et YouTube, y compris les **directs** (lien de la vidéo ou `youtube.com/@chaine/live`).
- Sous-titres de YouTube, y compris ceux **générés automatiquement** et leurs traductions, dans les langues préférées.
- Chaînes IPTV : listes M3U (lien ou fichier) et comptes Xtream Codes, chaînes nommées dans la playlist.
- Recherche automatique de sous-titres sur Internet.
- Système d'extensions pour ajouter des fonctions.
- Thèmes et apparence personnalisables.
- Installation facile : AppImage publiée, paquets Flatpak et AUR prêts (`packaging/`).
