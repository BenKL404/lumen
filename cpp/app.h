// Utilitaires d'application appelés depuis Rust.
#pragma once

#include <QtCore/QString>
#include <QtQml/QQmlApplicationEngine>

// Vrai si l'interface QML a bien été créée (au moins un objet racine).
bool lumen_qml_loaded(const QQmlApplicationEngine &engine);

// Ouvre un fichier (passé en argument de la ligne de commande) via openUrl() de Main.qml.
void lumen_open_file(const QQmlApplicationEngine &engine, const QString &path);

// Outil de développement : enregistre une image de la fenêtre après `delayMs`, puis quitte.
void lumen_snapshot(const QQmlApplicationEngine &engine, const QString &path, int delayMs);
