// Lecture rapide des métadonnées d'un fichier (sans le décoder), via libavformat.
#pragma once

#include <QtCore/QString>

// Durée en secondes, ou -1 si elle est inconnue. Appelable depuis n'importe quel thread.
double lumen_probe_duration(const QString &path);
