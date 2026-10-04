// Miniatures pour l'aperçu au survol de la barre de progression.
#pragma once

#include <QtGui/QImage>
#include <QtQml/QQmlApplicationEngine>

// Image de la vidéo `path` vers `seconds` (image clé la plus proche, avant), large de
// `width` pixels. Image nulle si le fichier n'a pas de vidéo. Appelable depuis n'importe quel thread.
QImage lumen_decode_thumbnail(const QString &path, double seconds, int width);

// Enregistre le fournisseur d'images « image://thumbnail/<secondes>/<chemin en hexadécimal> ».
// À appeler avant de charger le QML.
void lumen_register_thumbnails(QQmlApplicationEngine &engine);
